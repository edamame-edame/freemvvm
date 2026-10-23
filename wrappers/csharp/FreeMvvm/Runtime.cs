using System;
using System.Text;

namespace FreeMvvm;

public sealed class RuntimeOptions
{
    public string? NativeArtifactPath { get; init; }
}

public enum ShutdownMode : uint
{
    Async = 0,
    Sync = 1,
}

public enum ShutdownDisposition
{
    Completed,
    AcceptedPending,
}

public enum ShutdownRequestState : uint
{
    Pending = 0,
    Completed = 1,
    HostUnavailable = 2,
}

public readonly record struct ShutdownRequestSnapshot(uint RawState, uint Reason)
{
    public ShutdownRequestState? KnownState => Enum.IsDefined(typeof(ShutdownRequestState), RawState)
        ? (ShutdownRequestState)RawState
        : null;
}

public sealed class UiRuntime : IDisposable
{
    private readonly TokenOwner _owner;

    private UiRuntime(TokenOwner owner) => _owner = owner;

    public static UiResult<UiRuntime> TryCreate(RuntimeOptions? options = null)
    {
        INativeApi api;
        try
        {
            api = NativeRuntimeApi.Open(options?.NativeArtifactPath);
        }
        catch (Exception exception) when (
            exception is DllNotFoundException or BadImageFormatException or EntryPointNotFoundException or
                NativeApiException or InvalidOperationException or ArgumentException)
        {
            uint status = exception is NativeApiException nativeError
                ? nativeError.RawStatus
                : UiStatus.Unsupported;
            byte[] message = Encoding.UTF8.GetBytes(exception.Message);
            return UiResult<UiRuntime>.Failure(new UiErrorSnapshot(status, messageUtf8: message));
        }

        return TryCreate(api);
    }

    public static UiRuntime Create(RuntimeOptions? options = null) => TryCreate(options).GetValueOrThrow();

    public UiResult<bool> IsUiThread()
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<bool>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        var error = new PreparedError(_owner.Api);
        uint status;
        ulong errorToken;
        byte result;
        try
        {
            status = _owner.Api.RuntimeIsUiThread(lease.Token, out result);
            errorToken = 0;
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status != UiStatus.Ok)
        {
            return UiResult<bool>.Failure(error.Capture(status, errorToken));
        }

        return result <= 1
            ? UiResult<bool>.Success(result == 1)
            : UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
    }

    public UiResult<ShutdownDisposition> Shutdown()
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<ShutdownDisposition>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        var error = new PreparedError(_owner.Api);
        uint status;
        ulong errorToken;
        try
        {
            status = _owner.Api.RuntimeShutdown(lease.Token, out errorToken);
        }
        catch
        {
            return UiResult<ShutdownDisposition>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status == UiStatus.Ok)
        {
            return UiResult<ShutdownDisposition>.Success(ShutdownDisposition.Completed);
        }

        if (status == UiStatus.Pending)
        {
            return UiResult<ShutdownDisposition>.Success(ShutdownDisposition.AcceptedPending);
        }

        return UiResult<ShutdownDisposition>.Failure(error.Capture(status, errorToken));
    }

    public UiResult<ShutdownReceipt> RequestShutdown(ShutdownMode mode = ShutdownMode.Async)
    {
        if (mode is not ShutdownMode.Async and not ShutdownMode.Sync)
        {
            return UiResult<ShutdownReceipt>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        var receiptOwner = new TokenOwner(_owner.Api);
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<ShutdownReceipt>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        var error = new PreparedError(_owner.Api);
        uint status;
        ulong request;
        ulong errorToken;
        try
        {
            status = _owner.Api.RuntimeRequestShutdown(lease.Token, (uint)mode, out request, out errorToken);
        }
        catch
        {
            return UiResult<ShutdownReceipt>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status is not UiStatus.Ok and not UiStatus.Pending)
        {
            if (request != 0)
            {
                receiptOwner.Adopt(request);
                receiptOwner.Dispose();
            }

            return UiResult<ShutdownReceipt>.Failure(error.Capture(status, errorToken));
        }

        if (request == 0)
        {
            return UiResult<ShutdownReceipt>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        receiptOwner.Adopt(request);
        return UiResult<ShutdownReceipt>.Success(new ShutdownReceipt(receiptOwner));
    }

    public UiResult<UiCloseDisposition> TryClose() => _owner.TryClose();

    public void Dispose() => _owner.Dispose();

    internal static UiResult<UiRuntime> TryCreate(INativeApi api)
    {
        var runtimeOwner = new TokenOwner(api);
        var error = new PreparedError(api);
        uint status;
        ulong runtime;
        ulong errorToken;
        try
        {
            status = api.RuntimeCreate(out runtime, out errorToken);
        }
        catch
        {
            return UiResult<UiRuntime>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status != UiStatus.Ok)
        {
            if (runtime != 0)
            {
                runtimeOwner.Adopt(runtime);
                runtimeOwner.Dispose();
            }

            return UiResult<UiRuntime>.Failure(error.Capture(status, errorToken));
        }

        if (runtime == 0)
        {
            return UiResult<UiRuntime>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        runtimeOwner.Adopt(runtime);
        return UiResult<UiRuntime>.Success(new UiRuntime(runtimeOwner));
    }
}

public sealed class ShutdownReceipt : IDisposable
{
    private readonly TokenOwner _owner;

    internal ShutdownReceipt(TokenOwner owner) => _owner = owner;

    public UiResult<ShutdownRequestSnapshot> Query()
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<ShutdownRequestSnapshot>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        var error = new PreparedError(_owner.Api);
        uint status;
        uint state;
        uint reason;
        ulong errorToken;
        try
        {
            status = _owner.Api.ShutdownRequestQuery(lease.Token, out state, out reason, out errorToken);
        }
        catch
        {
            return UiResult<ShutdownRequestSnapshot>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        return status == UiStatus.Ok
            ? UiResult<ShutdownRequestSnapshot>.Success(new ShutdownRequestSnapshot(state, reason))
            : UiResult<ShutdownRequestSnapshot>.Failure(error.Capture(status, errorToken));
    }

    public UiResult<UiCloseDisposition> TryClose() => _owner.TryClose();

    public void Dispose() => _owner.Dispose();
}
