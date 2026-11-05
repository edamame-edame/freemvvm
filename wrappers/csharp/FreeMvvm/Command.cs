using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

namespace FreeMvvm;

public enum UiCommandState : long
{
    Idle = 0,
    Running = 1,
    Cancelling = 2,
    Succeeded = 3,
    Failed = 4,
    Cancelled = 5,
}

public sealed class UiCommand : IDisposable
{
    private readonly TokenOwner _owner;
    private readonly TokenOwner _runtimeAnchor;
    private readonly CallbackHandlerState _callbackHandlers;

    private delegate uint PropertyGetter(ulong command, out ulong property);

    private UiCommand(TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers)
    {
        _owner = owner;
        _runtimeAnchor = runtimeAnchor;
        _callbackHandlers = callbackHandlers;
    }

    internal TokenOwner Owner => _owner;

    internal static UiCommand FromOwned(
        TokenOwner owner, TokenOwner runtimeAnchor, CallbackHandlerState callbackHandlers) =>
        new(owner, runtimeAnchor, callbackHandlers);

    public static UiResult<UiCommand> TryCreateAsync(
        UiRuntime runtime,
        UiProperty<bool>? eligibility,
        Action<UiCommandWorker> start)
    {
        if (runtime is null || start is null)
        {
            return UiResult<UiCommand>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        INativeApi api = runtime.Owner.Api;
        if (eligibility is not null && !ReferenceEquals(api, eligibility.Owner.Api))
        {
            return UiResult<UiCommand>.Failure(new UiErrorSnapshot(UiStatus.WrongRuntime));
        }

        UiResult<CallLease> runtimeLeaseResult = runtime.Owner.AcquireLease();
        if (runtimeLeaseResult.IsFailure)
        {
            return UiResult<UiCommand>.Failure(runtimeLeaseResult.Error!);
        }

        using CallLease runtimeLease = runtimeLeaseResult.Value;
        CallLease? eligibilityLease = null;
        if (eligibility is not null)
        {
            UiResult<CallLease> eligibilityLeaseResult = eligibility.Owner.AcquireLease();
            if (eligibilityLeaseResult.IsFailure)
            {
                return UiResult<UiCommand>.Failure(eligibilityLeaseResult.Error!);
            }

            eligibilityLease = eligibilityLeaseResult.Value;
        }

        using (eligibilityLease)
        {
            UiResult<TokenOwner> wrapperAnchorResult = RetainOwner(api, runtimeLease.Token);
            if (wrapperAnchorResult.IsFailure)
            {
                return UiResult<UiCommand>.Failure(wrapperAnchorResult.Error!);
            }

            TokenOwner wrapperAnchor = wrapperAnchorResult.Value;
            UiResult<TokenOwner> callbackAnchorResult = RetainOwner(api, runtimeLease.Token);
            if (callbackAnchorResult.IsFailure)
            {
                wrapperAnchor.Dispose();
                return UiResult<UiCommand>.Failure(callbackAnchorResult.Error!);
            }

            var callbackRoot = new CommandStartRoot(
                api,
                callbackAnchorResult.Value,
                runtime.CallbackHandlers,
                start);
            GCHandle callbackRootHandle = default;
            ulong commandToken = 0;
            ulong errorToken = 0;
            uint status;
            try
            {
                callbackRootHandle = GCHandle.Alloc(callbackRoot);
                status = api.CommandCreateAsync(
                    runtimeLease.Token,
                    eligibilityLease?.Token ?? 0,
                    CommandStartEntry.CallbackPointer,
                    GCHandle.ToIntPtr(callbackRootHandle),
                    CommandStartEntry.DestroyPointer,
                    out commandToken,
                    out errorToken);
            }
            catch
            {
                if (callbackRootHandle.IsAllocated)
                {
                    callbackRootHandle.Free();
                }

                callbackRoot.Dispose();
                wrapperAnchor.Dispose();
                return UiResult<UiCommand>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
            }

            var preparedError = new PreparedError(api);
            if (status != UiStatus.Ok || commandToken == 0)
            {
                if (status != UiStatus.Ok)
                {
                    if (callbackRootHandle.IsAllocated)
                    {
                        callbackRootHandle.Free();
                    }

                    callbackRoot.Dispose();
                }
                wrapperAnchor.Dispose();
                if (commandToken != 0)
                {
                    try
                    {
                        api.HandleRelease(commandToken);
                    }
                    catch
                    {
                        // Keep the primary creation outcome stable across the C ABI boundary.
                    }
                }

                return UiResult<UiCommand>.Failure(
                    preparedError.Capture(status == UiStatus.Ok ? UiStatus.InternalError : status, errorToken));
            }

            var commandOwner = new TokenOwner(api);
            try
            {
                commandOwner.Adopt(commandToken);
            }
            catch
            {
                try
                {
                    api.CommandClose(commandToken, out _);
                    api.HandleRelease(commandToken);
                }
                catch
                {
                    // The ABI owns the callback root after successful creation.
                }

                wrapperAnchor.Dispose();
                return UiResult<UiCommand>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
            }

            return UiResult<UiCommand>.Success(
                new UiCommand(commandOwner, wrapperAnchor, runtime.CallbackHandlers));
        }
    }

    public UiResult<ulong> TryExecute()
    {
        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<ulong>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var preparedError = new PreparedError(_owner.Api);
        try
        {
            uint status = _owner.Api.CommandExecute(lease.Token, out ulong operationId, out ulong error);
            if (status != UiStatus.Ok)
            {
                return UiResult<ulong>.Failure(preparedError.Capture(status, error));
            }

            return operationId == 0
                ? UiResult<ulong>.Failure(new UiErrorSnapshot(UiStatus.InternalError))
                : UiResult<ulong>.Success(operationId);
        }
        catch
        {
            return UiResult<ulong>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TryCancel(ulong operationId)
    {
        if (operationId == 0)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var preparedError = new PreparedError(_owner.Api);
        try
        {
            uint status = _owner.Api.CommandCancel(lease.Token, operationId, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<UiProperty<long>> TryGetStateProperty() => GetProperty<long>(
        _owner.Api.CommandStateProperty,
        _ => UiProperty<long>.FromOwned(_, _callbackHandlers));

    public UiResult<UiProperty<bool>> TryGetCanExecuteProperty() => GetProperty<bool>(
        _owner.Api.CommandCanExecuteProperty,
        _ => UiProperty<bool>.FromOwned(_, _callbackHandlers));

    public UiResult<UiErrorSnapshot?> TryGetFailureSnapshot()
    {
        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<UiErrorSnapshot?>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        try
        {
            uint status = _owner.Api.CommandFailureSnapshot(lease.Token, out ulong snapshotToken);
            if (status != UiStatus.Ok)
            {
                if (snapshotToken != 0)
                {
                    _owner.Api.HandleRelease(snapshotToken);
                }

                return UiResult<UiErrorSnapshot?>.Failure(new UiErrorSnapshot(status));
            }

            if (snapshotToken == 0)
            {
                return UiResult<UiErrorSnapshot?>.Success(null);
            }

            return UiResult<UiErrorSnapshot?>.Success(
                new PreparedError(_owner.Api).CaptureOwnedSnapshot(snapshotToken));
        }
        catch
        {
            return UiResult<UiErrorSnapshot?>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TryClose()
    {
        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        var preparedError = new PreparedError(_owner.Api);
        try
        {
            uint status = _owner.Api.CommandClose(lease.Token, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public void Dispose()
    {
        _owner.Dispose();
        _runtimeAnchor.Dispose();
    }

    private UiResult<UiProperty<TValue>> GetProperty<TValue>(
        PropertyGetter getProperty,
        Func<TokenOwner, UiProperty<TValue>> createProperty)
    {
        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<UiProperty<TValue>>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        ulong propertyToken = 0;
        TokenOwner? propertyOwner = null;
        try
        {
            uint status = getProperty(lease.Token, out propertyToken);
            if (status != UiStatus.Ok || propertyToken == 0)
            {
                if (propertyToken != 0)
                {
                    _owner.Api.HandleRelease(propertyToken);
                }

                return UiResult<UiProperty<TValue>>.Failure(
                    new UiErrorSnapshot(status == UiStatus.Ok ? UiStatus.InternalError : status));
            }

            propertyOwner = new TokenOwner(_owner.Api);
            propertyOwner.Adopt(propertyToken);
            return UiResult<UiProperty<TValue>>.Success(createProperty(propertyOwner));
        }
        catch
        {
            if (propertyOwner is not null)
            {
                propertyOwner.Dispose();
            }
            else if (propertyToken != 0)
            {
                try
                {
                    _owner.Api.HandleRelease(propertyToken);
                }
                catch
                {
                    // Preserve a stable wrapper error if property ownership cannot be adopted.
                }
            }

            return UiResult<UiProperty<TValue>>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    internal static UiResult<TokenOwner> RetainOwner(INativeApi api, ulong borrowedToken)
    {
        ulong ownedToken = 0;
        try
        {
            uint status = api.HandleRetain(borrowedToken, out ownedToken);
            if (status != UiStatus.Ok || ownedToken == 0)
            {
                if (ownedToken != 0)
                {
                    api.HandleRelease(ownedToken);
                }

                return UiResult<TokenOwner>.Failure(
                    new UiErrorSnapshot(status == UiStatus.Ok ? UiStatus.InternalError : status));
            }

            var owner = new TokenOwner(api);
            try
            {
                owner.Adopt(ownedToken);
            }
            catch
            {
                api.HandleRelease(ownedToken);
                return UiResult<TokenOwner>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
            }

            return UiResult<TokenOwner>.Success(owner);
        }
        catch
        {
            if (ownedToken != 0)
            {
                try
                {
                    api.HandleRelease(ownedToken);
                }
                catch
                {
                    // Retain failure remains the primary wrapper outcome.
                }
            }

            return UiResult<TokenOwner>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }
}

public sealed class UiCommandWorker : IDisposable
{
    private readonly INativeApi _api;
    private readonly TokenOwner _command;
    private readonly TokenOwner _cancelToken;
    private readonly TokenOwner _runtime;
    private int _reportState;
    private int _disposed;

    internal UiCommandWorker(
        INativeApi api,
        ulong operationId,
        TokenOwner command,
        TokenOwner cancelToken,
        TokenOwner runtime)
    {
        _api = api;
        OperationId = operationId;
        _command = command;
        _cancelToken = cancelToken;
        _runtime = runtime;
    }

    public ulong OperationId { get; }

    public UiResult<bool> IsCancellationRequested()
    {
        UiResult<CallLease> leaseResult = _cancelToken.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        try
        {
            uint status = _api.CommandCancelTokenRequested(lease.Token, out byte requested);
            return status == UiStatus.Ok && requested <= 1
                ? UiResult<bool>.Success(requested == 1)
                : UiResult<bool>.Failure(new UiErrorSnapshot(status == UiStatus.Ok ? UiStatus.InternalError : status));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TryReportSuccess() => TryPost(CommandReportKind.Success, 0, 0, null);

    public UiResult<bool> TryReportFailure(uint domain, uint code, string message)
    {
        if (message is null)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        byte[] messageUtf8;
        try
        {
            messageUtf8 = new UTF8Encoding(false, true).GetBytes(message);
        }
        catch (EncoderFallbackException)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        return TryPost(CommandReportKind.Failure, domain, code, messageUtf8);
    }

    public UiResult<bool> TryReportCancelled() => TryPost(CommandReportKind.Cancelled, 0, 0, null);

    public void Dispose()
    {
        if (Interlocked.Exchange(ref _disposed, 1) != 0)
        {
            return;
        }

        _command.Dispose();
        _cancelToken.Dispose();
        _runtime.Dispose();
    }

    private UiResult<bool> TryPost(CommandReportKind kind, uint domain, uint code, byte[]? message)
    {
        if (Volatile.Read(ref _disposed) != 0)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.Closed));
        }

        if (Interlocked.CompareExchange(ref _reportState, 1, 0) != 0)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.Closed));
        }

        UiResult<CallLease> commandLeaseResult = _command.AcquireLease();
        if (commandLeaseResult.IsFailure)
        {
            Volatile.Write(ref _reportState, 0);
            return UiResult<bool>.Failure(commandLeaseResult.Error!);
        }

        using CallLease commandLease = commandLeaseResult.Value;
        UiResult<CallLease> runtimeLeaseResult = _runtime.AcquireLease();
        if (runtimeLeaseResult.IsFailure)
        {
            Volatile.Write(ref _reportState, 0);
            return UiResult<bool>.Failure(runtimeLeaseResult.Error!);
        }

        using CallLease runtimeLease = runtimeLeaseResult.Value;
        UiResult<TokenOwner> reportCommandResult = UiCommand.RetainOwner(_api, commandLease.Token);
        if (reportCommandResult.IsFailure)
        {
            Volatile.Write(ref _reportState, 0);
            return UiResult<bool>.Failure(reportCommandResult.Error!);
        }

        var reportRoot = new CommandReportRoot(
            _api,
            reportCommandResult.Value,
            OperationId,
            kind,
            domain,
            code,
            message);
        GCHandle reportHandle = default;
        ulong errorToken = 0;
        uint status;
        try
        {
            reportHandle = GCHandle.Alloc(reportRoot);
            status = _api.RuntimePost(
                runtimeLease.Token,
                CommandReportEntry.CallbackPointer,
                GCHandle.ToIntPtr(reportHandle),
                CommandReportEntry.DestroyPointer,
                out errorToken);
        }
        catch
        {
            if (reportHandle.IsAllocated)
            {
                reportHandle.Free();
            }

            reportRoot.Dispose();
            Volatile.Write(ref _reportState, 0);
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status != UiStatus.Ok)
        {
            if (reportHandle.IsAllocated)
            {
                reportHandle.Free();
            }

            reportRoot.Dispose();
            Volatile.Write(ref _reportState, 0);
            return UiResult<bool>.Failure(new PreparedError(_api).Capture(status, errorToken));
        }

        return UiResult<bool>.Success(true);
    }
}

internal enum CommandReportKind
{
    Success,
    Failure,
    Cancelled,
}

internal sealed class CommandStartRoot(
    INativeApi api,
    TokenOwner runtimeAnchor,
    CallbackHandlerState handlers,
    Action<UiCommandWorker> start) : IDisposable
{
    internal uint Start(ulong borrowedCommand, ulong operationId, ulong borrowedCancelToken)
    {
        UiResult<TokenOwner> commandResult = UiCommand.RetainOwner(api, borrowedCommand);
        if (commandResult.IsFailure)
        {
            return commandResult.Error!.RawStatus;
        }

        UiResult<TokenOwner> cancelResult = UiCommand.RetainOwner(api, borrowedCancelToken);
        if (cancelResult.IsFailure)
        {
            commandResult.Value.Dispose();
            return cancelResult.Error!.RawStatus;
        }

        UiResult<CallLease> runtimeLeaseResult = runtimeAnchor.AcquireLease();
        if (runtimeLeaseResult.IsFailure)
        {
            commandResult.Value.Dispose();
            cancelResult.Value.Dispose();
            return runtimeLeaseResult.Error!.RawStatus;
        }

        using CallLease runtimeLease = runtimeLeaseResult.Value;
        UiResult<TokenOwner> workerRuntimeResult = UiCommand.RetainOwner(api, runtimeLease.Token);
        if (workerRuntimeResult.IsFailure)
        {
            commandResult.Value.Dispose();
            cancelResult.Value.Dispose();
            return workerRuntimeResult.Error!.RawStatus;
        }

        var worker = new UiCommandWorker(
            api,
            operationId,
            commandResult.Value,
            cancelResult.Value,
            workerRuntimeResult.Value);
        try
        {
            start(worker);
            return UiStatus.Ok;
        }
        catch (Exception exception)
        {
            worker.Dispose();
            handlers.Report(exception, UiCallbackFailureKind.CommandStart);
            return UiStatus.InternalError;
        }
    }

    public void Dispose() => runtimeAnchor.Dispose();
}

internal sealed class CommandReportRoot(
    INativeApi api,
    TokenOwner command,
    ulong operationId,
    CommandReportKind kind,
    uint domain,
    uint code,
    byte[]? message) : IDisposable
{
    internal void Invoke()
    {
        UiResult<CallLease> leaseResult = command.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return;
        }

        using CallLease lease = leaseResult.Value;
        try
        {
            switch (kind)
            {
                case CommandReportKind.Success:
                    api.CommandCompleteSuccess(lease.Token, operationId);
                    break;
                case CommandReportKind.Failure:
                    byte[] bytes = message ?? [];
                    unsafe
                    {
                        fixed (byte* buffer = bytes)
                        {
                            api.CommandCompleteFailureUtf8(
                                lease.Token,
                                operationId,
                                domain,
                                code,
                                buffer,
                                (ulong)bytes.Length);
                        }
                    }

                    break;
                case CommandReportKind.Cancelled:
                    api.CommandCompleteCancelled(lease.Token, operationId);
                    break;
            }
        }
        catch
        {
            // A stale or closed operation is intentionally ignored at the managed boundary.
        }
    }

    public void Dispose() => command.Dispose();
}

internal static unsafe class CommandStartEntry
{
    internal static IntPtr CallbackPointer => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, ulong, ulong, ulong, uint>)&Invoke;

    internal static IntPtr DestroyPointer => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, void>)&Destroy;

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static uint Invoke(IntPtr userdata, ulong command, ulong operationId, ulong cancelToken)
    {
        try
        {
            GCHandle root = GCHandle.FromIntPtr(userdata);
            return root.Target is CommandStartRoot startRoot
                ? startRoot.Start(command, operationId, cancelToken)
                : UiStatus.InternalError;
        }
        catch
        {
            return UiStatus.InternalError;
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Destroy(IntPtr userdata)
    {
        try
        {
            GCHandle root = GCHandle.FromIntPtr(userdata);
            if (root.IsAllocated)
            {
                if (root.Target is CommandStartRoot startRoot)
                {
                    startRoot.Dispose();
                }

                root.Free();
            }
        }
        catch
        {
            // Destroy must not throw through the C ABI.
        }
    }
}

internal static unsafe class CommandReportEntry
{
    internal static IntPtr CallbackPointer => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, void>)&Invoke;

    internal static IntPtr DestroyPointer => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, void>)&Destroy;

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Invoke(IntPtr userdata)
    {
        try
        {
            GCHandle root = GCHandle.FromIntPtr(userdata);
            if (root.Target is CommandReportRoot reportRoot)
            {
                reportRoot.Invoke();
            }
        }
        catch (Exception exception)
        {
            CallbackHandlerState.ReportUnexpected(exception);
        }
    }

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Destroy(IntPtr userdata)
    {
        try
        {
            GCHandle root = GCHandle.FromIntPtr(userdata);
            if (root.IsAllocated)
            {
                if (root.Target is CommandReportRoot reportRoot)
                {
                    reportRoot.Dispose();
                }

                root.Free();
            }
        }
        catch
        {
            // Destroy must not throw through the C ABI.
        }
    }
}
