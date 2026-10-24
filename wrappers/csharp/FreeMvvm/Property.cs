using System;

namespace FreeMvvm;

public sealed class UiProperty<T> : IDisposable
{
    private readonly TokenOwner _owner;

    private UiProperty(TokenOwner owner) => _owner = owner;

    public static UiResult<UiProperty<T>> TryCreate(UiRuntime runtime, T initial)
    {
        if (runtime is null)
        {
            return UiResult<UiProperty<T>>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        if (typeof(T) == typeof(bool))
        {
            byte value = (bool)(object)initial! ? (byte)1 : (byte)0;
            return TryCreateCore(
                runtime.Owner,
                static (INativeApi api, ulong owner, byte initialValue, long _, out ulong property, out ulong error) =>
                    api.PropertyCreateBool(owner, initialValue, out property, out error),
                value,
                0);
        }

        if (typeof(T) == typeof(long))
        {
            long value = (long)(object)initial!;
            return TryCreateCore(
                runtime.Owner,
                static (INativeApi api, ulong owner, byte _, long initialValue, out ulong property, out ulong error) =>
                    api.PropertyCreateI64(owner, initialValue, out property, out error),
                0,
                value);
        }

        return UiResult<UiProperty<T>>.Failure(new UiErrorSnapshot(UiStatus.Unsupported));
    }

    public UiResult<T> TryGet()
    {
        UiResult<CallLease> acquired = _owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return UiResult<T>.Failure(acquired.Error!);
        }

        using CallLease lease = acquired.Value;
        try
        {
            if (typeof(T) == typeof(bool))
            {
                uint status = _owner.Api.PropertyGetBool(lease.Token, out byte value);
                if (status != UiStatus.Ok)
                {
                    return UiResult<T>.Failure(new UiErrorSnapshot(status));
                }

                if (value > 1)
                {
                    return UiResult<T>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
                }

                return UiResult<T>.Success((T)(object)(value == 1));
            }

            if (typeof(T) == typeof(long))
            {
                uint status = _owner.Api.PropertyGetI64(lease.Token, out long value);
                return status == UiStatus.Ok
                    ? UiResult<T>.Success((T)(object)value)
                    : UiResult<T>.Failure(new UiErrorSnapshot(status));
            }

            return UiResult<T>.Failure(new UiErrorSnapshot(UiStatus.Unsupported));
        }
        catch
        {
            return UiResult<T>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TrySet(T value)
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
            if (typeof(T) == typeof(bool))
            {
                byte nativeValue = (bool)(object)value! ? (byte)1 : (byte)0;
                status = _owner.Api.PropertySetBool(lease.Token, nativeValue, out errorToken);
            }
            else if (typeof(T) == typeof(long))
            {
                long nativeValue = (long)(object)value!;
                status = _owner.Api.PropertySetI64(lease.Token, nativeValue, out errorToken);
            }
            else
            {
                return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.Unsupported));
            }
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

    private static UiResult<UiProperty<T>> TryCreateCore(
        TokenOwner runtimeOwner,
        CreateNative createNative,
        byte initialBool,
        long initialI64)
    {
        var propertyOwner = new TokenOwner(runtimeOwner.Api);
        var error = new PreparedError(runtimeOwner.Api);
        UiResult<CallLease> acquired = runtimeOwner.AcquireLease();
        if (acquired.IsFailure)
        {
            propertyOwner.Dispose();
            return UiResult<UiProperty<T>>.Failure(acquired.Error!);
        }

        uint status;
        ulong property;
        ulong errorToken;
        using (CallLease lease = acquired.Value)
        {
            try
            {
                status = createNative(runtimeOwner.Api, lease.Token, initialBool, initialI64, out property, out errorToken);
            }
            catch
            {
                propertyOwner.Dispose();
                return UiResult<UiProperty<T>>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
            }
        }

        if (status != UiStatus.Ok)
        {
            if (property != 0)
            {
                propertyOwner.Adopt(property);
            }

            propertyOwner.Dispose();
            return UiResult<UiProperty<T>>.Failure(error.Capture(status, errorToken));
        }

        if (property == 0)
        {
            propertyOwner.Dispose();
            return UiResult<UiProperty<T>>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }

        propertyOwner.Adopt(property);
        return UiResult<UiProperty<T>>.Success(new UiProperty<T>(propertyOwner));
    }

    private delegate uint CreateNative(
        INativeApi api,
        ulong runtime,
        byte initialBool,
        long initialI64,
        out ulong property,
        out ulong error);
}
