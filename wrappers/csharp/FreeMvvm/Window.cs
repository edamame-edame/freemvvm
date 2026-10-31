using System;

namespace FreeMvvm;

public readonly struct RgbColor
{
    public RgbColor(float red, float green, float blue)
    {
        Red = red;
        Green = green;
        Blue = blue;
    }

    public float Red { get; }

    public float Green { get; }

    public float Blue { get; }
}

public sealed class UiWindow : IDisposable
{
    private readonly TokenOwner _owner;

    private UiWindow(TokenOwner owner) => _owner = owner;

    public static UiResult<UiWindow> TryCreate(UiRuntime runtime)
    {
        if (runtime is null)
        {
            return UiResult<UiWindow>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        TokenOwner windowOwner;
        PreparedError error;
        try
        {
            windowOwner = new TokenOwner(runtime.Owner.Api);
            error = new PreparedError(runtime.Owner.Api);
        }
        catch (OutOfMemoryException)
        {
            return UiResult<UiWindow>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        UiResult<CallLease> acquired = runtime.Owner.AcquireLease();
        if (acquired.IsFailure)
        {
            windowOwner.Dispose();
            return UiResult<UiWindow>.Failure(acquired.Error!);
        }

        uint status;
        ulong window;
        ulong errorToken;
        try
        {
            using CallLease lease = acquired.Value;
            status = runtime.Owner.Api.WindowCreate(lease.Token, out window, out errorToken);
        }
        catch
        {
            windowOwner.Dispose();
            return UiResult<UiWindow>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status != UiStatus.Ok)
        {
            if (window != 0)
            {
                windowOwner.Adopt(window);
            }

            windowOwner.Dispose();
            return UiResult<UiWindow>.Failure(error.Capture(status, errorToken));
        }

        if (window == 0)
        {
            windowOwner.Dispose();
            return UiResult<UiWindow>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        windowOwner.Adopt(window);
        return UiResult<UiWindow>.Success(new UiWindow(windowOwner));
    }

    public static UiWindow Create(UiRuntime runtime) => TryCreate(runtime).GetValueOrThrow();

    public UiResult<RgbColor> TryGetBackgroundRgb()
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<RgbColor>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        var error = new PreparedError(_owner.Api);
        try
        {
            uint status = _owner.Api.WindowBackgroundRgb(
                lease.Token, out float red, out float green, out float blue, out ulong errorToken);
            return status == UiStatus.Ok
                ? UiResult<RgbColor>.Success(new RgbColor(red, green, blue))
                : UiResult<RgbColor>.Failure(error.Capture(status, errorToken));
        }
        catch
        {
            return UiResult<RgbColor>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TrySetBackgroundRgb(RgbColor color)
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<bool>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        var error = new PreparedError(_owner.Api);
        try
        {
            uint status = _owner.Api.WindowSetBackgroundRgb(
                lease.Token, color.Red, color.Green, color.Blue, out ulong errorToken);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(error.Capture(status, errorToken));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TryResetBackgroundRgb()
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<bool>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        var error = new PreparedError(_owner.Api);
        try
        {
            uint status = _owner.Api.WindowResetBackgroundRgb(lease.Token, out ulong errorToken);
            return status == UiStatus.Ok
                ? UiResult<bool>.Success(true)
                : UiResult<bool>.Failure(error.Capture(status, errorToken));
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public RgbColor BackgroundRgb
    {
        get => TryGetBackgroundRgb().GetValueOrThrow();
        set => TrySetBackgroundRgb(value).GetValueOrThrow();
    }

    public void ResetBackgroundRgb() => TryResetBackgroundRgb().GetValueOrThrow();

    public UiResult<UiCloseDisposition> TryClose() => _owner.TryClose();

    public void Dispose() => _owner.Dispose();
}
