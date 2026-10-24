using System;
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

        UiResult<ShutdownReceipt> request = runtime.RequestShutdown(ShutdownMode.Async);
        if (request.IsSuccess || request.Error?.RawStatus != 10)
        {
            Console.Error.WriteLine($"Expected UI_UNSUPPORTED (10), got success={request.IsSuccess}, status={request.Error?.RawStatus}.");
            return 19;
        }

        if (runtime.TryClose().IsFailure)
        {
            Console.Error.WriteLine("The Runtime handle could not be closed.");
            return 20;
        }

        Console.WriteLine("C# Runtime BOOL/I64/F64/UTF-8 Property C ABI consumer: OK");
        return 0;
    }
}
