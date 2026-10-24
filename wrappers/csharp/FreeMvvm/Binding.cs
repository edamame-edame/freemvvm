using System;
using System.Runtime.InteropServices;
using System.Threading;

namespace FreeMvvm;

public enum UiBindingMode : uint
{
    OneTime = 1,
    OneWay = 2,
    TwoWay = 3,
}

public enum UiBindingDirection : uint
{
    SourceToTarget = 1,
    TargetToSource = 2,
}

public sealed class UiBinding<T> : IDisposable
{
    private readonly TokenOwner _owner;
    private readonly CallbackHandlerState _callbackHandlers;

    private UiBinding(TokenOwner owner, CallbackHandlerState callbackHandlers)
    {
        _owner = owner;
        _callbackHandlers = callbackHandlers;
    }

    internal static UiResult<UiBinding<T>?> TryCreate(
        UiProperty<T> source,
        UiProperty<T> target,
        UiBindingMode mode)
    {
        if (source is null || target is null)
        {
            return UiResult<UiBinding<T>?>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        UiResult<CallLease> sourceLeaseResult = source.Owner.AcquireLease();
        if (sourceLeaseResult.IsFailure)
        {
            return UiResult<UiBinding<T>?>.Failure(sourceLeaseResult.Error!);
        }

        UiResult<CallLease> targetLeaseResult = target.Owner.AcquireLease();
        if (targetLeaseResult.IsFailure)
        {
            sourceLeaseResult.Value.Dispose();
            return UiResult<UiBinding<T>?>.Failure(targetLeaseResult.Error!);
        }

        using CallLease sourceLease = sourceLeaseResult.Value;
        using CallLease targetLease = targetLeaseResult.Value;
        TokenOwner? bindingOwner = null;
        PreparedError? preparedError = null;
        try
        {
            bindingOwner = new TokenOwner(source.Owner.Api);
            preparedError = new PreparedError(source.Owner.Api);
        }
        catch (OutOfMemoryException)
        {
            bindingOwner?.Dispose();
            return UiResult<UiBinding<T>?>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        uint status;
        ulong bindingToken = 0;
        ulong errorToken = 0;
        try
        {
            status = source.Owner.Api.BindingCreate(
                sourceLease.Token,
                targetLease.Token,
                (uint)mode,
                out bindingToken,
                out errorToken);
        }
        catch
        {
            bindingOwner.Dispose();
            return UiResult<UiBinding<T>?>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status != UiStatus.Ok)
        {
            if (bindingToken != 0)
            {
                AdoptAndDispose(bindingOwner, bindingToken);
            }
            else
            {
                bindingOwner.Dispose();
            }

            return UiResult<UiBinding<T>?>.Failure(preparedError.Capture(status, errorToken));
        }

        if (mode == UiBindingMode.OneTime)
        {
            if (bindingToken != 0)
            {
                AdoptAndDispose(bindingOwner, bindingToken);
                return UiResult<UiBinding<T>?>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
            }

            bindingOwner.Dispose();
            return UiResult<UiBinding<T>?>.Success(null);
        }

        if (bindingToken == 0)
        {
            bindingOwner.Dispose();
            return UiResult<UiBinding<T>?>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        UiBinding<T> binding;
        try
        {
            binding = new UiBinding<T>(bindingOwner, source.CallbackHandlers);
        }
        catch (OutOfMemoryException)
        {
            AdoptAndDispose(bindingOwner, bindingToken);
            return UiResult<UiBinding<T>?>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        bindingOwner.Adopt(bindingToken);
        return UiResult<UiBinding<T>?>.Success(binding);
    }

    public UiResult<bool> TryCancel()
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<bool>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        var preparedError = new PreparedError(_owner.Api);
        uint status;
        ulong errorToken;
        try
        {
            status = _owner.Api.BindingCancel(lease.Token, out errorToken);
        }
        catch
        {
            return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        return status == UiStatus.Ok
            ? UiResult<bool>.Success(true)
            : UiResult<bool>.Failure(preparedError.Capture(status, errorToken));
    }

    public UiResult<UiCloseDisposition> TryClose() => _owner.TryClose();

    public UiResult<UiBindingError<T>?> TryGetErrorSnapshot()
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<UiBindingError<T>?>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        TokenOwner? rejectedPropertyOwner = null;
        PreparedError? preparedError = null;
        try
        {
            rejectedPropertyOwner = new TokenOwner(_owner.Api);
            preparedError = new PreparedError(_owner.Api);
        }
        catch (OutOfMemoryException)
        {
            rejectedPropertyOwner?.Dispose();
            return UiResult<UiBindingError<T>?>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        uint status;
        ulong errorToken = 0;
        ulong rejectedPropertyToken = 0;
        uint rawDirection = 0;
        try
        {
            status = _owner.Api.BindingErrorSnapshot(
                lease.Token,
                out errorToken,
                out rejectedPropertyToken,
                out rawDirection);
        }
        catch
        {
            rejectedPropertyOwner.Dispose();
            return UiResult<UiBindingError<T>?>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status != UiStatus.Ok)
        {
            if (rejectedPropertyToken != 0)
            {
                AdoptAndDispose(rejectedPropertyOwner, rejectedPropertyToken);
            }
            else
            {
                rejectedPropertyOwner.Dispose();
            }

            return UiResult<UiBindingError<T>?>.Failure(preparedError.Capture(status, errorToken));
        }

        if (errorToken == 0 && rejectedPropertyToken == 0 && rawDirection == 0)
        {
            rejectedPropertyOwner.Dispose();
            return UiResult<UiBindingError<T>?>.Success(null);
        }

        UiErrorSnapshot error = preparedError.Capture(UiStatus.InternalError, errorToken);
        if (errorToken == 0 || rejectedPropertyToken == 0 ||
            (rawDirection != (uint)UiBindingDirection.SourceToTarget &&
             rawDirection != (uint)UiBindingDirection.TargetToSource))
        {
            if (rejectedPropertyToken != 0)
            {
                AdoptAndDispose(rejectedPropertyOwner, rejectedPropertyToken);
            }
            else
            {
                rejectedPropertyOwner.Dispose();
            }

            return UiResult<UiBindingError<T>?>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        UiBindingError<T> snapshot;
        try
        {
            UiProperty<T> rejectedProperty = UiProperty<T>.FromOwned(rejectedPropertyOwner, _callbackHandlers);
            snapshot = new UiBindingError<T>(error, rejectedProperty, (UiBindingDirection)rawDirection);
        }
        catch (OutOfMemoryException)
        {
            AdoptAndDispose(rejectedPropertyOwner, rejectedPropertyToken);
            return UiResult<UiBindingError<T>?>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        rejectedPropertyOwner.Adopt(rejectedPropertyToken);
        return UiResult<UiBindingError<T>?>.Success(snapshot);
    }

    public UiResult<UiSubscription> TrySubscribeError(Action callback)
    {
        if (callback is null)
        {
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        UiResult<CallLease> bindingLeaseResult = _owner.AcquireLease();
        if (bindingLeaseResult.IsFailure)
        {
            return UiResult<UiSubscription>.Failure(bindingLeaseResult.Error!);
        }

        TokenOwner? subscriptionOwner = null;
        UiSubscription? subscription = null;
        GCHandle callbackRoot = default;
        PreparedError? preparedError = null;
        try
        {
            subscriptionOwner = new TokenOwner(_owner.Api);
            subscription = new UiSubscription(subscriptionOwner);
            callbackRoot = GCHandle.Alloc(new PropertyCallbackRoot(
                callback,
                _callbackHandlers,
                UiCallbackFailureKind.BindingErrorNotification));
            preparedError = new PreparedError(_owner.Api);
        }
        catch (OutOfMemoryException)
        {
            if (callbackRoot.IsAllocated)
            {
                callbackRoot.Free();
            }

            subscriptionOwner?.Dispose();
            bindingLeaseResult.Value.Dispose();
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }

        uint status;
        ulong subscriptionToken = 0;
        ulong errorToken = 0;
        try
        {
            using CallLease bindingLease = bindingLeaseResult.Value;
            status = _owner.Api.BindingSubscribeError(
                bindingLease.Token,
                PropertyCallbackEntryPoints.CallbackPointer,
                GCHandle.ToIntPtr(callbackRoot),
                PropertyCallbackEntryPoints.DestroyPointer,
                out subscriptionToken,
                out errorToken);
        }
        catch
        {
            callbackRoot.Free();
            subscriptionOwner.Dispose();
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        if (status != UiStatus.Ok)
        {
            callbackRoot.Free();
            if (subscriptionToken != 0)
            {
                AdoptAndDispose(subscriptionOwner, subscriptionToken);
            }
            else
            {
                subscriptionOwner.Dispose();
            }

            return UiResult<UiSubscription>.Failure(preparedError.Capture(status, errorToken));
        }

        if (subscriptionToken == 0)
        {
            callbackRoot.Free();
            subscriptionOwner.Dispose();
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        try
        {
            subscriptionOwner.Adopt(subscriptionToken);
            return UiResult<UiSubscription>.Success(subscription);
        }
        catch (OutOfMemoryException)
        {
            // The native closure owns callbackRoot after successful registration.
            AdoptAndDispose(subscriptionOwner, subscriptionToken);
            return UiResult<UiSubscription>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }
    }

    public void Dispose() => _owner.Dispose();

    private static void AdoptAndDispose(TokenOwner owner, ulong token)
    {
        try
        {
            owner.Adopt(token);
            owner.Dispose();
        }
        catch
        {
            // The owner has a preallocated cleanup ticket; never throw through the wrapper boundary.
        }
    }
}

public sealed class UiBindingError<T> : IDisposable
{
    private readonly UiProperty<T> _rejectedProperty;
    private int _disposed;

    internal UiBindingError(UiErrorSnapshot error, UiProperty<T> rejectedProperty, UiBindingDirection direction)
    {
        Error = error;
        _rejectedProperty = rejectedProperty;
        Direction = direction;
    }

    public UiErrorSnapshot Error { get; }

    public UiProperty<T> RejectedProperty => _rejectedProperty;

    public UiBindingDirection Direction { get; }

    public UiResult<UiCloseDisposition> TryClose()
    {
        UiResult<UiCloseDisposition> result = _rejectedProperty.TryClose();
        if (result.IsSuccess && result.Value == UiCloseDisposition.Released)
        {
            Interlocked.Exchange(ref _disposed, 1);
        }

        return result;
    }

    public void Dispose()
    {
        if (Interlocked.Exchange(ref _disposed, 1) == 0)
        {
            _rejectedProperty.Dispose();
        }
    }
}
