using System;
using System.Text;

namespace FreeMvvm;

public sealed class UiProperty<T> : IDisposable
{
    private static readonly UTF8Encoding StrictUtf8 = new(false, true);
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
                static (INativeApi api, ulong owner, byte initialValue, long _, double _, ulong _, byte[]? _, out ulong property, out ulong error) =>
                    api.PropertyCreateBool(owner, initialValue, out property, out error),
                value,
                0,
                0,
                0,
                null);
        }

        if (typeof(T) == typeof(long))
        {
            long value = (long)(object)initial!;
            return TryCreateCore(
                runtime.Owner,
                static (INativeApi api, ulong owner, byte _, long initialValue, double _, ulong _, byte[]? _, out ulong property, out ulong error) =>
                    api.PropertyCreateI64(owner, initialValue, out property, out error),
                0,
                value,
                0,
                0,
                null);
        }

        if (typeof(T) == typeof(double))
        {
            double value = (double)(object)initial!;
            return TryCreateCore(
                runtime.Owner,
                static (INativeApi api, ulong owner, byte _, long _, double initialValue, ulong _, byte[]? _, out ulong property, out ulong error) =>
                    api.PropertyCreateF64(owner, initialValue, out property, out error),
                0,
                0,
                value,
                0,
                null);
        }

        if (typeof(T) == typeof(DecimalValue))
        {
            DecimalValue value = (DecimalValue)(object)initial!;
            return TryCreateCore(
                runtime.Owner,
                static (INativeApi api, ulong owner, byte _, long _, double _, ulong initialBits, byte[]? _, out ulong property, out ulong error) =>
                    api.PropertyCreateDecimal(owner, initialBits, out property, out error),
                0,
                0,
                0,
                value.Bits,
                null);
        }

        if (typeof(T) == typeof(string))
        {
            if (initial is not string text)
            {
                return UiResult<UiProperty<T>>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
            }

            try
            {
                byte[] bytes = StrictUtf8.GetBytes(text);
                return TryCreateCore(
                    runtime.Owner,
                    static (INativeApi api, ulong owner, byte _, long _, double _, ulong _, byte[]? initialUtf8, out ulong property, out ulong error) =>
                        api.PropertyCreateUtf8(owner, initialUtf8!, out property, out error),
                    0,
                    0,
                    0,
                    0,
                    bytes);
            }
            catch (EncoderFallbackException)
            {
                return UiResult<UiProperty<T>>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
            }
            catch (OutOfMemoryException)
            {
                return UiResult<UiProperty<T>>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
            }
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

            if (typeof(T) == typeof(double))
            {
                uint status = _owner.Api.PropertyGetF64(lease.Token, out double value);
                return status == UiStatus.Ok
                    ? UiResult<T>.Success((T)(object)value)
                    : UiResult<T>.Failure(new UiErrorSnapshot(status));
            }

            if (typeof(T) == typeof(DecimalValue))
            {
                uint status = _owner.Api.PropertyGetDecimal(lease.Token, out ulong bits);
                if (status != UiStatus.Ok)
                {
                    return UiResult<T>.Failure(new UiErrorSnapshot(status));
                }

                UiResult<DecimalValue> value = DecimalValue.TryFromParts(
                    (bits >> 63) != 0,
                    (byte)((bits >> 58) & 0x1F),
                    bits & ((1UL << 58) - 1));
                return value.IsSuccess
                    ? UiResult<T>.Success((T)(object)value.Value)
                    : UiResult<T>.Failure(value.Error!);
            }

            if (typeof(T) == typeof(string))
            {
                UiResult<string> result = TryGetUtf8(lease.Token);
                return result.IsSuccess
                    ? UiResult<T>.Success((T)(object)result.Value)
                    : UiResult<T>.Failure(result.Error!);
            }

            return UiResult<T>.Failure(new UiErrorSnapshot(UiStatus.Unsupported));
        }
        catch (OutOfMemoryException)
        {
            return UiResult<T>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }
        catch
        {
            return UiResult<T>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        }
    }

    public UiResult<bool> TrySet(T value)
    {
        byte[]? utf8Value = null;
        if (typeof(T) == typeof(string))
        {
            if (value is not string text)
            {
                return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
            }

            try
            {
                utf8Value = StrictUtf8.GetBytes(text);
            }
            catch (EncoderFallbackException)
            {
                return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
            }
            catch (OutOfMemoryException)
            {
                return UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
            }
        }

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
            else if (typeof(T) == typeof(double))
            {
                double nativeValue = (double)(object)value!;
                status = _owner.Api.PropertySetF64(lease.Token, nativeValue, out errorToken);
            }
            else if (typeof(T) == typeof(DecimalValue))
            {
                ulong nativeValue = ((DecimalValue)(object)value!).Bits;
                status = _owner.Api.PropertySetDecimal(lease.Token, nativeValue, out errorToken);
            }
            else if (typeof(T) == typeof(string))
            {
                status = _owner.Api.PropertySetUtf8(lease.Token, utf8Value!, out errorToken);
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
        long initialI64,
        double initialF64,
        ulong initialDecimal,
        byte[]? initialUtf8)
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
                status = createNative(runtimeOwner.Api, lease.Token, initialBool, initialI64, initialF64, initialDecimal, initialUtf8, out property, out errorToken);
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
        double initialF64,
        ulong initialDecimal,
        byte[]? initialUtf8,
        out ulong property,
        out ulong error);

    private UiResult<string> TryGetUtf8(ulong token)
    {
        while (true)
        {
            uint queryStatus = _owner.Api.PropertyGetUtf8(token, Array.Empty<byte>(), 0, out ulong requiredLength);
            if (queryStatus != UiStatus.Ok && queryStatus != UiStatus.BufferTooSmall)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(queryStatus));
            }

            if (queryStatus == UiStatus.Ok && requiredLength == 0)
            {
                return UiResult<string>.Success(string.Empty);
            }

            if (requiredLength == 0 || requiredLength > int.MaxValue)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(
                    requiredLength == 0 ? UiStatus.InternalError : UiStatus.OutOfMemory));
            }

            byte[] buffer;
            try
            {
                buffer = new byte[(int)requiredLength];
            }
            catch (OutOfMemoryException)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
            }

            uint readStatus = _owner.Api.PropertyGetUtf8(token, buffer, requiredLength, out ulong actualLength);
            if (readStatus == UiStatus.BufferTooSmall)
            {
                if (actualLength <= requiredLength)
                {
                    return UiResult<string>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
                }

                continue;
            }

            if (readStatus != UiStatus.Ok)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(readStatus));
            }

            if (actualLength > requiredLength)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
            }

            try
            {
                return UiResult<string>.Success(StrictUtf8.GetString(buffer, 0, (int)actualLength));
            }
            catch (DecoderFallbackException)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
            }
            catch (OutOfMemoryException)
            {
                return UiResult<string>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
            }
        }
    }
}
