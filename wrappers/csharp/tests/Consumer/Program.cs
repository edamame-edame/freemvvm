using System;
using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Threading;
using FreeMvvm;

namespace FreeMvvm.Consumer;

internal static class Program
{
    private static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.WriteLine("Pass the absolute path to the host C ABI library.");
            return 1;
        }

        UiResult<UiRuntime> create = UiRuntime.TryCreate(new RuntimeOptions
        {
            NativeArtifactPath = args[0],
        });
        if (create.IsFailure)
        {
            Console.Error.WriteLine($"Runtime creation failed: {create.Error?.RawStatus}");
            return 2;
        }

        using UiRuntime runtime = create.Value;
        UiResult<bool> isUiThread = runtime.IsUiThread();
        if (isUiThread.IsFailure || !isUiThread.Value)
        {
            Console.Error.WriteLine("Runtime did not identify its creating thread as the UI thread.");
            return 3;
        }

        UiResult<UiProperty<bool>> boolCreate = UiProperty<bool>.TryCreate(runtime, true);
        if (boolCreate.IsFailure)
        {
            Console.Error.WriteLine($"BOOL property creation failed: {boolCreate.Error?.RawStatus}");
            return 4;
        }

        using UiProperty<bool> boolProperty = boolCreate.Value;
        if (!boolProperty.TryGet().Value || boolProperty.TrySet(false).IsFailure || boolProperty.TryGet().Value)
        {
            Console.Error.WriteLine("BOOL property did not round trip true/false values.");
            return 5;
        }

        UiResult<UiCloseDisposition> boolClose = boolProperty.TryClose();
        if (boolClose.IsFailure || boolClose.Value != UiCloseDisposition.Released)
        {
            Console.Error.WriteLine("BOOL property handle could not be closed.");
            return 6;
        }

        UiResult<UiProperty<long>> i64Create = UiProperty<long>.TryCreate(runtime, long.MinValue);
        if (i64Create.IsFailure)
        {
            Console.Error.WriteLine($"I64 property creation failed: {i64Create.Error?.RawStatus}");
            return 7;
        }

        using UiProperty<long> i64Property = i64Create.Value;
        if (i64Property.TryGet().Value != long.MinValue || i64Property.TrySet(long.MaxValue).IsFailure ||
            i64Property.TryGet().Value != long.MaxValue)
        {
            Console.Error.WriteLine("I64 property did not round trip its boundary values.");
            return 8;
        }

        UiResult<UiCloseDisposition> i64Close = i64Property.TryClose();
        if (i64Close.IsFailure || i64Close.Value != UiCloseDisposition.Released)
        {
            Console.Error.WriteLine("I64 property handle could not be closed.");
            return 9;
        }

        int notificationResult = RunNotificationConsumer(args[0]);
        if (notificationResult != 0)
        {
            return notificationResult;
        }

        int bindingResult = RunBindingConsumer(args[0]);
        if (bindingResult != 0)
        {
            return bindingResult;
        }

        long initialNaNBits = 0x7FF8_0000_0000_0042;
        long otherNaNBits = 0x7FF8_0000_0000_1234;
        long negativeZeroBits = long.MinValue;
        UiResult<UiProperty<double>> f64Create = UiProperty<double>.TryCreate(
            runtime,
            BitConverter.Int64BitsToDouble(0x3FF4_0000_0000_0000));
        if (f64Create.IsFailure)
        {
            Console.Error.WriteLine($"F64 property creation failed: {f64Create.Error?.RawStatus}");
            return 10;
        }

        using UiProperty<double> f64Property = f64Create.Value;
        if (BitConverter.DoubleToInt64Bits(f64Property.TryGet().Value) != 0x3FF4_0000_0000_0000 ||
            f64Property.TrySet(double.PositiveInfinity).IsFailure ||
            !double.IsPositiveInfinity(f64Property.TryGet().Value) ||
            f64Property.TrySet(double.NegativeInfinity).IsFailure ||
            !double.IsNegativeInfinity(f64Property.TryGet().Value))
        {
            Console.Error.WriteLine("F64 finite/infinity values did not round trip.");
            return 11;
        }

        if (f64Property.TrySet(BitConverter.Int64BitsToDouble(initialNaNBits)).IsFailure ||
            BitConverter.DoubleToInt64Bits(f64Property.TryGet().Value) != initialNaNBits ||
            f64Property.TrySet(BitConverter.Int64BitsToDouble(otherNaNBits)).IsFailure ||
            BitConverter.DoubleToInt64Bits(f64Property.TryGet().Value) != initialNaNBits)
        {
            Console.Error.WriteLine("F64 NaN payload or same-value storage was not preserved.");
            return 12;
        }

        if (f64Property.TrySet(BitConverter.Int64BitsToDouble(negativeZeroBits)).IsFailure ||
            BitConverter.DoubleToInt64Bits(f64Property.TryGet().Value) != negativeZeroBits ||
            f64Property.TrySet(0.0).IsFailure ||
            BitConverter.DoubleToInt64Bits(f64Property.TryGet().Value) != negativeZeroBits)
        {
            Console.Error.WriteLine("F64 signed-zero equivalence did not preserve the stored representation.");
            return 13;
        }

        UiResult<UiCloseDisposition> f64Close = f64Property.TryClose();
        if (f64Close.IsFailure || f64Close.Value != UiCloseDisposition.Released)
        {
            Console.Error.WriteLine("F64 property handle could not be closed.");
            return 14;
        }

        UiResult<UiProperty<string>> utf8Create = UiProperty<string>.TryCreate(runtime, "初期値");
        if (utf8Create.IsFailure)
        {
            Console.Error.WriteLine($"UTF-8 property creation failed: {utf8Create.Error?.RawStatus}");
            return 15;
        }

        using UiProperty<string> utf8Property = utf8Create.Value;
        string[] strings = [string.Empty, "ASCII", "こんにちは", "UI 🙂", "前\0後"];
        if (utf8Property.TryGet().Value != "初期値")
        {
            Console.Error.WriteLine("UTF-8 initial value did not round trip.");
            return 16;
        }

        foreach (string value in strings)
        {
            if (utf8Property.TrySet(value).IsFailure || utf8Property.TryGet().Value != value)
            {
                Console.Error.WriteLine("UTF-8 property did not preserve the exact string value.");
                return 17;
            }
        }

        UiResult<UiCloseDisposition> utf8Close = utf8Property.TryClose();
        if (utf8Close.IsFailure || utf8Close.Value != UiCloseDisposition.Released)
        {
            Console.Error.WriteLine("UTF-8 property handle could not be closed.");
            return 18;
        }

        const ulong decimalMaxCoefficient = (1UL << 58) - 1;
        (bool Negative, byte Scale, ulong Coefficient)[] decimalCases =
        [
            (false, 0, 0),
            (true, 31, 0),
            (false, 2, 100),
            (true, 0, decimalMaxCoefficient),
        ];
        foreach (var item in decimalCases)
        {
            DecimalValue expected = DecimalValue.TryFromParts(item.Negative, item.Scale, item.Coefficient).Value;
            UiResult<UiProperty<DecimalValue>> decimalCreate = UiProperty<DecimalValue>.TryCreate(runtime, expected);
            if (decimalCreate.IsFailure)
            {
                Console.Error.WriteLine($"DECIMAL property creation failed: {decimalCreate.Error?.RawStatus}");
                return 20;
            }

            using UiProperty<DecimalValue> decimalProperty = decimalCreate.Value;
            UiResult<DecimalValue> actual = decimalProperty.TryGet();
            if (actual.IsFailure || actual.Value.IsNegative != expected.IsNegative ||
                actual.Value.Scale != expected.Scale || actual.Value.Coefficient != expected.Coefficient)
            {
                Console.Error.WriteLine("DECIMAL create/get did not preserve canonical bits through the C ABI.");
                return 21;
            }

            DecimalValue changed = DecimalValue.TryFromParts(!item.Negative, item.Scale, item.Coefficient == 0 ? 1 : item.Coefficient - 1).Value;
            if (decimalProperty.TrySet(changed).IsFailure)
            {
                Console.Error.WriteLine("DECIMAL setter failed through the C ABI.");
                return 22;
            }

            actual = decimalProperty.TryGet();
            if (actual.IsFailure || actual.Value.IsNegative != changed.IsNegative ||
                actual.Value.Scale != changed.Scale || actual.Value.Coefficient != changed.Coefficient)
            {
                Console.Error.WriteLine("DECIMAL set/get did not preserve canonical bits through the C ABI.");
                return 23;
            }

            if (decimalProperty.TryClose().IsFailure)
            {
                Console.Error.WriteLine("DECIMAL property handle could not be closed.");
                return 24;
            }
        }

        UiResult<DecimalValue> fromDecimal = DecimalValue.TryFromDecimal(123.4500m);
        if (fromDecimal.IsFailure)
        {
            Console.Error.WriteLine("System.Decimal value should map exactly to DecimalValue.");
            return 25;
        }

        UiResult<decimal> toDecimal = fromDecimal.Value.TryToDecimal();
        if (toDecimal.IsFailure || toDecimal.Value != 123.4500m ||
            DecimalValue.TryFromParts(false, 31, 1).Value.TryToDecimal().IsSuccess)
        {
            Console.Error.WriteLine("System.Decimal conversions should preserve exact values and reject inexact scale.");
            return 26;
        }

        UiResult<ShutdownReceipt> request = runtime.RequestShutdown(ShutdownMode.Async);
        if (request.IsSuccess || request.Error?.RawStatus != 10)
        {
            Console.Error.WriteLine($"Expected UI_UNSUPPORTED (10), got success={request.IsSuccess}, status={request.Error?.RawStatus}.");
            return 27;
        }

        if (runtime.TryClose().IsFailure)
        {
            Console.Error.WriteLine("The Runtime handle could not be closed.");
            return 28;
        }

        Console.WriteLine("C# Runtime BOOL/I64/F64/UTF-8/DECIMAL Property and notification Subscription C ABI consumer: OK");
        return 0;
    }

    private static int RunNotificationConsumer(string nativeArtifactPath)
    {
        UiResult<UiRuntime> runtimeCreate = UiRuntime.TryCreate(new RuntimeOptions
        {
            NativeArtifactPath = nativeArtifactPath,
        });
        if (runtimeCreate.IsFailure)
        {
            Console.Error.WriteLine($"Notification Runtime creation failed: {runtimeCreate.Error?.RawStatus}");
            return 29;
        }

        using UiRuntime runtime = runtimeCreate.Value;
        if (!InstallTestBridge(runtime))
        {
            Console.Error.WriteLine("Could not register the C ABI test host bridge.");
            return 30;
        }

        UiResult<UiProperty<long>> propertyCreate = UiProperty<long>.TryCreate(runtime, 0);
        if (propertyCreate.IsFailure)
        {
            Console.Error.WriteLine($"Notification property creation failed: {propertyCreate.Error?.RawStatus}");
            return 31;
        }

        using UiProperty<long> property = propertyCreate.Value;
        int callbackCount = 0;
        long callbackValue = -1;
        UiSubscription subscription = property.TrySubscribe(() =>
        {
            callbackValue = property.TryGet().Value;
            callbackCount++;
        }).Value;
        UiResult<bool> set = UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        if (callbackCount != 0 || !RunDispatchTurn(runtime, () => set = property.TrySet(41)) ||
            set.IsFailure || callbackCount != 1 || callbackValue != 41)
        {
            Console.Error.WriteLine("C# notification callback did not observe the changed property value.");
            return 32;
        }

        UiResult<bool> canceledSet = UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        if (!subscription.TryCancel().Value || subscription.TryClose().IsFailure ||
            !RunDispatchTurn(runtime, () => canceledSet = property.TrySet(42)) ||
            canceledSet.IsFailure || callbackCount != 1)
        {
            Console.Error.WriteLine("C# subscription cancellation did not stop later notification callbacks.");
            return 33;
        }

        UiResult<UiProperty<long>> releasedPropertyCreate = UiProperty<long>.TryCreate(runtime, 0);
        if (releasedPropertyCreate.IsFailure)
        {
            Console.Error.WriteLine("Handle-release subscription property creation failed.");
            return 34;
        }

        using UiProperty<long> releasedProperty = releasedPropertyCreate.Value;
        int releasedHandleCallbackCount = 0;
        UiSubscription releasedSubscription = releasedProperty.TrySubscribe(() =>
        {
            _ = releasedProperty.TryGet().Value;
            releasedHandleCallbackCount++;
        }).Value;
        UiResult<bool> releasedSet = UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        if (releasedSubscription.TryClose().IsFailure ||
            !RunDispatchTurn(runtime, () => releasedSet = releasedProperty.TrySet(1)) ||
            releasedSet.IsFailure || releasedHandleCallbackCount != 1)
        {
            Console.Error.WriteLine("Releasing a subscription handle incorrectly canceled its active callback.");
            return 35;
        }

        if (runtime.Shutdown().IsFailure || runtime.TryClose().IsFailure)
        {
            Console.Error.WriteLine("Notification Runtime did not shut down and close cleanly.");
            return 36;
        }

        return 0;
    }

    private static int RunBindingConsumer(string nativeArtifactPath)
    {
        UiResult<UiRuntime> runtimeCreate = UiRuntime.TryCreate(new RuntimeOptions
        {
            NativeArtifactPath = nativeArtifactPath,
        });
        if (runtimeCreate.IsFailure)
        {
            Console.Error.WriteLine($"Binding Runtime creation failed: {runtimeCreate.Error?.RawStatus}");
            return 40;
        }

        using UiRuntime runtime = runtimeCreate.Value;
        if (!InstallTestBridge(runtime))
        {
            Console.Error.WriteLine("Could not register the Binding test host bridge.");
            return 41;
        }

        if (!VerifyBindingModes(runtime, false, true, true, false,
                static (owner, value) => UiProperty<bool>.TryCreate(owner, value),
                static property => property.TryGet(),
                static (property, value) => property.TrySet(value)))
        {
            return 42;
        }

        if (!VerifyBindingModes(runtime, 1L, 0L, 2L, 3L,
                static (owner, value) => UiProperty<long>.TryCreate(owner, value),
                static property => property.TryGet(),
                static (property, value) => property.TrySet(value)))
        {
            return 43;
        }

        if (!VerifyBindingModes(runtime, 1.5, 0.0, 2.5, 3.5,
                static (owner, value) => UiProperty<double>.TryCreate(owner, value),
                static property => property.TryGet(),
                static (property, value) => property.TrySet(value)))
        {
            return 44;
        }

        DecimalValue decimalOne = DecimalValue.TryFromParts(false, 0, 1).Value;
        DecimalValue decimalTwo = DecimalValue.TryFromParts(false, 0, 2).Value;
        DecimalValue decimalThree = DecimalValue.TryFromParts(false, 0, 3).Value;
        DecimalValue decimalFour = DecimalValue.TryFromParts(false, 0, 4).Value;
        if (!VerifyBindingModes(runtime, decimalOne, decimalTwo, decimalThree, decimalFour,
                static (owner, value) => UiProperty<DecimalValue>.TryCreate(owner, value),
                static property => property.TryGet(),
                static (property, value) => property.TrySet(value)))
        {
            return 45;
        }

        if (!VerifyBindingModes(runtime, "source", "target", "changed", "reverse",
                static (owner, value) => UiProperty<string>.TryCreate(owner, value),
                static property => property.TryGet(),
                static (property, value) => property.TrySet(value)))
        {
            return 46;
        }

        if (runtime.Shutdown().IsFailure || runtime.TryClose().IsFailure)
        {
            Console.Error.WriteLine("Binding Runtime did not shut down and close cleanly.");
            return 47;
        }

        Console.WriteLine("C# Property Binding modes and values across the host C ABI: OK");
        return 0;
    }

    private static bool VerifyBindingModes<T>(
        UiRuntime runtime,
        T sourceInitial,
        T targetInitial,
        T forwardValue,
        T reverseValue,
        Func<UiRuntime, T, UiResult<UiProperty<T>>> create,
        Func<UiProperty<T>, UiResult<T>> read,
        Func<UiProperty<T>, T, UiResult<bool>> write)
    {
        UiProperty<T> onceSource = create(runtime, sourceInitial).Value;
        UiProperty<T> onceTarget = create(runtime, targetInitial).Value;
        UiResult<UiBinding<T>?> once = onceSource.TryBindTo(onceTarget, UiBindingMode.OneTime);
        bool onceOk = once.IsSuccess && once.Value is null &&
            EqualityComparer<T>.Default.Equals(read(onceTarget).Value, sourceInitial);
        onceSource.Dispose();
        onceTarget.Dispose();
        if (!onceOk)
        {
            Console.Error.WriteLine($"{typeof(T).Name} OneTime Binding did not copy the source value.");
            return false;
        }

        UiProperty<T> oneWaySource = create(runtime, sourceInitial).Value;
        UiProperty<T> oneWayTarget = create(runtime, targetInitial).Value;
        UiBinding<T>? oneWay = oneWaySource.TryBindTo(oneWayTarget, UiBindingMode.OneWay).Value;
        UiResult<bool> oneWaySet = UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        bool oneWayOk = oneWay is not null && RunDispatchTurn(runtime, () => oneWaySet = write(oneWaySource, forwardValue)) &&
            oneWaySet.IsSuccess && EqualityComparer<T>.Default.Equals(read(oneWayTarget).Value, forwardValue);
        if (oneWayOk)
        {
            // Releasing the public handle must leave the Core-owned connection active.
            oneWayOk = oneWay!.TryClose().IsSuccess &&
                RunDispatchTurn(runtime, () => oneWaySet = write(oneWaySource, sourceInitial)) &&
                oneWaySet.IsSuccess && EqualityComparer<T>.Default.Equals(read(oneWayTarget).Value, sourceInitial);
        }

        oneWaySource.Dispose();
        oneWayTarget.Dispose();
        if (!oneWayOk)
        {
            Console.Error.WriteLine($"{typeof(T).Name} OneWay Binding propagation or handle-release lifetime failed.");
            return false;
        }

        UiProperty<T> twoWaySource = create(runtime, sourceInitial).Value;
        UiProperty<T> twoWayTarget = create(runtime, targetInitial).Value;
        using UiBinding<T> twoWay = twoWaySource.TryBindTo(twoWayTarget, UiBindingMode.TwoWay).Value!;
        UiResult<bool> twoWaySet = UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        bool twoWayOk = RunDispatchTurn(runtime, () => twoWaySet = write(twoWaySource, forwardValue)) &&
            twoWaySet.IsSuccess && EqualityComparer<T>.Default.Equals(read(twoWayTarget).Value, forwardValue) &&
            RunDispatchTurn(runtime, () => twoWaySet = write(twoWayTarget, reverseValue)) &&
            twoWaySet.IsSuccess && EqualityComparer<T>.Default.Equals(read(twoWaySource).Value, reverseValue) &&
            twoWay.TryCancel().IsSuccess &&
            RunDispatchTurn(runtime, () => twoWaySet = write(twoWaySource, sourceInitial)) &&
            twoWaySet.IsSuccess && EqualityComparer<T>.Default.Equals(read(twoWayTarget).Value, reverseValue) &&
            twoWay.TryClose().IsSuccess;
        twoWaySource.Dispose();
        twoWayTarget.Dispose();
        if (!twoWayOk)
        {
            Console.Error.WriteLine($"{typeof(T).Name} TwoWay propagation or cancellation failed.");
            return false;
        }

        return true;
    }

    private static bool InstallTestBridge(UiRuntime runtime)
    {
        GCHandle root = GCHandle.Alloc(new HostBridgeState());
        UiResult<CallLease> acquired = runtime.Owner.AcquireLease();
        if (acquired.IsFailure)
        {
            root.Free();
            return false;
        }

        using CallLease lease = acquired.Value;
        uint status = runtime.Owner.Api.RuntimeSetUiLoopBridge(
            lease.Token,
            TestBridgeEntryPoints.Wake,
            GCHandle.ToIntPtr(root),
            TestBridgeEntryPoints.Destroy,
            out _);
        if (status != UiStatus.Ok)
        {
            root.Free();
            return false;
        }

        return true;
    }

    private static bool RunDispatchTurn(UiRuntime runtime, Action action)
    {
        UiResult<CallLease> acquired = runtime.Owner.AcquireLease();
        if (acquired.IsFailure)
        {
            return false;
        }

        using CallLease lease = acquired.Value;
        action();
        return runtime.Owner.Api.RuntimeDispatchTurn(lease.Token) == UiStatus.Ok;
    }

    private sealed class HostBridgeState
    {
        internal int WakeCount;
    }

    private static unsafe class TestBridgeEntryPoints
    {
        internal static IntPtr Wake => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, uint, void>)&OnWake;

        internal static IntPtr Destroy => (IntPtr)(delegate* unmanaged[Cdecl]<IntPtr, void>)&OnDestroy;

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
        private static void OnWake(IntPtr userdata, uint priority)
        {
            try
            {
                if (GCHandle.FromIntPtr(userdata).Target is HostBridgeState state)
                {
                    Interlocked.Increment(ref state.WakeCount);
                }
            }
            catch
            {
            }
        }

        [UnmanagedCallersOnly(CallConvs = [typeof(CallConvCdecl)])]
        private static void OnDestroy(IntPtr userdata)
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
            }
        }
    }
}
