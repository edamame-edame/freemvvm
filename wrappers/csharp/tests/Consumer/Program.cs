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

        UiResult<ShutdownReceipt> request = runtime.RequestShutdown(ShutdownMode.Async);
        if (request.IsSuccess || request.Error?.RawStatus != 10)
        {
            Console.Error.WriteLine($"Expected UI_UNSUPPORTED (10), got success={request.IsSuccess}, status={request.Error?.RawStatus}.");
            return 10;
        }

        if (runtime.TryClose().IsFailure)
        {
            Console.Error.WriteLine("The Runtime handle could not be closed.");
            return 11;
        }

        Console.WriteLine("C# Runtime BOOL/I64 Property C ABI consumer: OK");
        return 0;
    }
}
