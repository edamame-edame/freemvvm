using System;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;

namespace FreeMvvm;

public sealed class UiSubscription : IDisposable
{
    private readonly TokenOwner _owner;

    internal UiSubscription(TokenOwner owner) => _owner = owner;

    public UiResult<bool> TryCancel()
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
        try
        {
            status = _owner.Api.SubscriptionCancel(lease.Token, out errorToken);
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        return status == UiStatus.Ok
            ? UiResult<bool>.Success(true)
            : UiResult<bool>.Failure(error.Capture(status, errorToken));
    }

    public UiResult<UiCloseDisposition> TryClose() => _owner.TryClose();

    public void Dispose() => _owner.Dispose();
}

internal static unsafe class PropertyCallbackEntryPoints
{
    internal static IntPtr CallbackPointer => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, void>)&Invoke;

    internal static IntPtr DestroyPointer => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, void>)&Destroy;

    [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
    private static void Invoke(IntPtr userdata)
    {
        try
        {
            GCHandle root = GCHandle.FromIntPtr(userdata);
            if (root.Target is PropertyCallbackRoot callbackRoot)
            {
                callbackRoot.Invoke();
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
                root.Free();
            }
        }
        catch
        {
            // Destroy must not throw through the C ABI.
        }
    }
}
