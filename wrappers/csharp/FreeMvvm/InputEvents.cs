using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

namespace FreeMvvm;

[Flags]
public enum InputEventMask : ulong
{
    KeyDown = 1UL << 0,
    KeyUp = 1UL << 1,
    TextCommit = 1UL << 2,
    PrimaryPointerDown = 1UL << 3,
    PrimaryPointerUp = 1UL << 4,
    All = KeyDown | KeyUp | TextCommit | PrimaryPointerDown | PrimaryPointerUp,
}

public enum InputEventKind : uint
{
    KeyDown = 1,
    KeyUp = 2,
    TextCommit = 3,
    PrimaryPointerDown = 4,
    PrimaryPointerUp = 5,
}

[Flags]
public enum InputModifiers : uint
{
    None = 0,
    Shift = 0x01,
    Control = 0x02,
    Alt = 0x04,
    Meta = 0x08,
}

public readonly record struct KeyIdentity(uint Value);

public readonly record struct PointDip(double X, double Y);

public sealed class InputEventView
{
    private static readonly UTF8Encoding StrictUtf8 = new(false, true);
    private readonly INativeApi _api;
    private readonly InputEventScope _scope;
    private readonly ulong _eventToken;

    internal InputEventView(INativeApi api, InputEventScope scope, ulong eventToken)
    {
        _api = api;
        _scope = scope;
        _eventToken = eventToken;
    }

    public InputEventKind Kind
    {
        get
        {
            EnsureActive();
            uint status = _api.InputEventKind(_eventToken, out uint kind);
            if (status != UiStatus.Ok || !Enum.IsDefined(typeof(InputEventKind), kind))
            {
                throw new InvalidOperationException("The input event is no longer available or has an unknown kind.");
            }

            return (InputEventKind)kind;
        }
    }

    public UiResult<KeyIdentity> TryGetKey()
    {
        if (!_scope.IsActive)
        {
            return UiResult<KeyIdentity>.Failure(new UiErrorSnapshot(UiStatus.InvalidHandle));
        }

        uint status = _api.InputEventKey(_eventToken, out uint identity, out _);
        return status == UiStatus.Ok
            ? UiResult<KeyIdentity>.Success(new KeyIdentity(identity))
            : UiResult<KeyIdentity>.Failure(new UiErrorSnapshot(status));
    }

    public UiResult<InputModifiers> TryGetModifiers()
    {
        if (!_scope.IsActive)
        {
            return UiResult<InputModifiers>.Failure(new UiErrorSnapshot(UiStatus.InvalidHandle));
        }

        uint status = _api.InputEventKey(_eventToken, out _, out uint modifiers);
        return status == UiStatus.Ok
            ? UiResult<InputModifiers>.Success((InputModifiers)modifiers)
            : UiResult<InputModifiers>.Failure(new UiErrorSnapshot(status));
    }

    public UiResult<string> TryGetText()
    {
        if (!_scope.IsActive)
        {
            return UiResult<string>.Failure(new UiErrorSnapshot(UiStatus.InvalidHandle));
        }

        try
        {
            uint status = _api.InputEventText(_eventToken, [], 0, out ulong length);
            if (status != UiStatus.BufferTooSmall && !(status == UiStatus.Ok && length == 0))
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(status));
            }

            if (length > int.MaxValue)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
            }

            byte[] buffer = new byte[(int)length];
            status = _api.InputEventText(_eventToken, buffer, length, out ulong actualLength);
            if (status != UiStatus.Ok || actualLength > length)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(status == UiStatus.Ok ? UiStatus.InternalError : status));
            }

            return UiResult<string>.Success(StrictUtf8.GetString(buffer, 0, (int)actualLength));
        }
        catch (Exception exception) when (exception is OutOfMemoryException or DecoderFallbackException)
        {
            return UiResult<string>.Failure(new UiErrorSnapshot(
                exception is OutOfMemoryException ? UiStatus.OutOfMemory : UiStatus.InvalidArgument));
        }
    }

    public UiResult<PointDip> TryGetPointerPosition()
    {
        if (!_scope.IsActive)
        {
            return UiResult<PointDip>.Failure(new UiErrorSnapshot(UiStatus.InvalidHandle));
        }

        uint status = _api.InputEventPointer(_eventToken, out double x, out double y, out _);
        return status == UiStatus.Ok
            ? UiResult<PointDip>.Success(new PointDip(x, y))
            : UiResult<PointDip>.Failure(new UiErrorSnapshot(status));
    }

    public UiResult<bool> TrySetHandled()
    {
        if (!_scope.IsActive)
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidHandle));
        }

        var preparedError = new PreparedError(_api);
        uint status;
        ulong error;
        try
        {
            status = _api.InputEventSetHandled(_eventToken, 1, out error);
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        return status == UiStatus.Ok
            ? UiResult<bool>.Success(true)
            : UiResult<bool>.Failure(preparedError.Capture(status, error));
    }

    private void EnsureActive()
    {
        if (!_scope.IsActive)
        {
            throw new InvalidOperationException("The input event view is valid only during its callback.");
        }
    }
}

internal sealed class InputEventScope
{
    private int _active = 1;
    internal bool IsActive => Volatile.Read(ref _active) == 1;
    internal void Invalidate() => Interlocked.Exchange(ref _active, 0);
}

internal sealed class InputCallbackRoot(INativeApi api, TokenOwner runtimeAnchor, CallbackHandlerState handlers, Action<InputEventView> callback)
{
    private int _destroyed;

    internal void Invoke(ulong borrowedEvent)
    {
        var scope = new InputEventScope();
        var view = new InputEventView(api, scope, borrowedEvent);
        try
        {
            callback(view);
        }
        catch (Exception exception)
        {
            handlers.Report(exception, UiCallbackFailureKind.InputEvent);
            _ = view.TrySetHandled();
        }
        finally
        {
            scope.Invalidate();
        }
    }

    internal void Destroy()
    {
        if (Interlocked.Exchange(ref _destroyed, 1) == 0)
        {
            runtimeAnchor.Dispose();
        }
    }
}

internal static unsafe class InputCallbackEntryPoints
{
    internal static IntPtr CallbackPointer =>
        (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, ulong, ulong, ulong, void>)&Invoke;

    internal static IntPtr DestroyPointer => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, void>)&Destroy;

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Invoke(IntPtr userdata, ulong currentWidget, ulong originalTarget, ulong borrowedEvent)
    {
        _ = currentWidget;
        _ = originalTarget;
        try
        {
            GCHandle root = GCHandle.FromIntPtr(userdata);
            if (root.Target is InputCallbackRoot callbackRoot)
            {
                callbackRoot.Invoke(borrowedEvent);
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
            if (root.Target is InputCallbackRoot callbackRoot)
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
