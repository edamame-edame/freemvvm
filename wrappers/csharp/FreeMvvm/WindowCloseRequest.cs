using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace FreeMvvm;

public enum UiWindowCloseDecision : uint
{
    Accept = 1,
    Decline = 2,
}

public sealed class UiWindowCloseRequest
{
    private readonly INativeApi _api;
    private readonly UiWindowCloseRequestScope _scope;
    private readonly ulong _borrowedToken;
    private readonly Func<UiResult<TokenOwner>> _retainRuntimeAnchor;

    internal UiWindowCloseRequest(
        INativeApi api,
        UiWindowCloseRequestScope scope,
        ulong borrowedToken,
        Func<UiResult<TokenOwner>> retainRuntimeAnchor)
    {
        _api = api;
        _scope = scope;
        _borrowedToken = borrowedToken;
        _retainRuntimeAnchor = retainRuntimeAnchor;
    }

    public UiResult<bool> TryResolve(UiWindowCloseDecision decision)
    {
        if (!_scope.IsActive)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.Closed));
        }

        if (!Enum.IsDefined(decision))
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        PreparedError preparedError;
        try
        {
            preparedError = new PreparedError(_api);
        }
        catch (OutOfMemoryException)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        try
        {
            uint status = _api.WindowCloseRequestResolve(_borrowedToken, (uint)decision, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<UiDeferredWindowCloseRequest> TryDefer()
    {
        if (!_scope.IsActive)
        {
            return UiResult<UiDeferredWindowCloseRequest>.Failure(new UiErrorSnapshot(UiStatus.Closed));
        }

        TokenOwner requestOwner;
        PreparedError preparedError;
        try
        {
            requestOwner = new TokenOwner(_api);
            preparedError = new PreparedError(_api);
        }
        catch (OutOfMemoryException)
        {
            return UiResult<UiDeferredWindowCloseRequest>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        UiResult<TokenOwner> anchorResult = _retainRuntimeAnchor();
        if (anchorResult.IsFailure)
        {
            requestOwner.Dispose();
            return UiResult<UiDeferredWindowCloseRequest>.Failure(anchorResult.Error!);
        }

        TokenOwner runtimeAnchor = anchorResult.Value;
        ulong requestToken = 0;
        try
        {
            uint status = _api.WindowCloseRequestDefer(_borrowedToken, out requestToken, out ulong error);
            if (requestToken != 0)
            {
                requestOwner.Adopt(requestToken);
            }

            if (status != UiStatus.Ok || requestToken == 0)
            {
                requestOwner.Dispose();
                runtimeAnchor.Dispose();
                return UiResult<UiDeferredWindowCloseRequest>.Failure(status == UiStatus.Ok
                    ? new UiErrorSnapshot(UiStatus.InternalError)
                    : preparedError.Capture(status, error));
            }

            return UiResult<UiDeferredWindowCloseRequest>.Success(
                new UiDeferredWindowCloseRequest(requestOwner, runtimeAnchor));
        }
        catch
        {
            requestOwner.Dispose();
            runtimeAnchor.Dispose();
            return UiResult<UiDeferredWindowCloseRequest>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }
}

public sealed class UiDeferredWindowCloseRequest : IDisposable
{
    private readonly TokenOwner _owner;
    private readonly TokenOwner _runtimeAnchor;

    internal UiDeferredWindowCloseRequest(TokenOwner owner, TokenOwner runtimeAnchor)
    {
        _owner = owner;
        _runtimeAnchor = runtimeAnchor;
    }

    public UiResult<bool> TryResolve(UiWindowCloseDecision decision)
    {
        if (!Enum.IsDefined(decision))
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        UiResult<CallLease> leaseResult = _owner.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<bool>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        PreparedError preparedError;
        try
        {
            preparedError = new PreparedError(_owner.Api);
        }
        catch (OutOfMemoryException)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        try
        {
            uint status = _owner.Api.WindowCloseRequestResolve(lease.Token, (uint)decision, out ulong error);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(preparedError.Capture(status, error));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<UiCloseDisposition> TryClose() => _owner.TryClose();

    public void Dispose()
    {
        _owner.Dispose();
        _runtimeAnchor.Dispose();
    }
}

internal sealed class UiWindowCloseRequestScope
{
    private int _active = 1;

    internal bool IsActive => System.Threading.Volatile.Read(ref _active) == 1;

    internal void Invalidate() => System.Threading.Interlocked.Exchange(ref _active, 0);
}

internal sealed class UiWindowCloseCallbackRoot(
    INativeApi api,
    TokenOwner callbackRuntimeAnchor,
    CallbackHandlerState callbackHandlers,
    Action<UiWindowCloseRequest> callback)
{
    private int _destroyed;

    internal void Invoke(ulong borrowedToken)
    {
        var scope = new UiWindowCloseRequestScope();
        try
        {
            callback(new UiWindowCloseRequest(api, scope, borrowedToken, RetainRuntimeAnchor));
        }
        catch (Exception exception)
        {
            callbackHandlers.Report(exception, UiCallbackFailureKind.WindowCloseRequest);
        }
        finally
        {
            scope.Invalidate();
        }
    }

    internal void Destroy()
    {
        if (System.Threading.Interlocked.Exchange(ref _destroyed, 1) == 0)
        {
            callbackRuntimeAnchor.Dispose();
        }
    }

    private UiResult<TokenOwner> RetainRuntimeAnchor()
    {
        UiResult<CallLease> leaseResult = callbackRuntimeAnchor.AcquireLease();
        if (leaseResult.IsFailure)
        {
            return UiResult<TokenOwner>.Failure(leaseResult.Error!);
        }

        using CallLease lease = leaseResult.Value;
        return UiCommand.RetainOwner(api, lease.Token);
    }
}

internal static unsafe class WindowCloseCallbackEntryPoints
{
    internal static IntPtr CallbackPointer =>
        (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, ulong, void>)&Invoke;

    internal static IntPtr DestroyPointer => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, void>)&Destroy;

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Invoke(IntPtr userdata, ulong borrowedToken)
    {
        try
        {
            GCHandle root = GCHandle.FromIntPtr(userdata);
            if (root.Target is UiWindowCloseCallbackRoot callbackRoot)
            {
                callbackRoot.Invoke(borrowedToken);
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
            if (root.Target is UiWindowCloseCallbackRoot callbackRoot)
            {
                callbackRoot.Destroy();
            }

            if (root.IsAllocated)
            {
                root.Free();
            }
        }
        catch
        {
            // Destroy must not throw through the C ABI.
        }
    }
}
