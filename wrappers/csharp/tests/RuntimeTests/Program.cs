using System;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using FreeMvvm;

namespace FreeMvvm.RuntimeTests;

internal static class Program
{
    private static int Main()
    {
        Run(TestRuntimeAndShutdownReceiptPreserveAbiValues);
        Run(TestErrorSnapshotPreservesUnknownValues);
        Run(TestExplicitCloseFailureRetainsTokenForRetry);
        Run(TestCloseWaitsForCallLeaseAndRejectsNewCalls);
        Run(TestDisposeReleaseFailureMovesTokenToRetryTicket);
        Run(TestBoolAndI64PropertyRoundTripAndClose);
        Run(TestF64PropertyPreservesBitPatterns);
        Run(TestF64PropertyFailuresAndCloseRetry);
        Run(TestUtf8PropertyRoundTripAndStrictInput);
        Run(TestUtf8GetterRetriesAndRejectsInvalidBytes);
        Run(TestUtf8PropertyFailuresAndCloseRetry);
        Run(TestDecimalValuePartsAndExactConversions);
        Run(TestDecimalPropertyCanonicalBitsAndFailures);
        Run(TestUnsupportedPropertyTypeIsRejectedBeforeNativeCall);
        Run(TestPropertyFailuresPreserveStatusAndValue);
        Run(TestPropertyCreateAndGetterFailuresAndCloseRetry);
        Run(TestPropertyLeaseProtectsHandleDuringDispose);
        Run(TestPropertySubscriptionAndCallbackFailureHandling);
        Run(TestPropertyBindingModesErrorSnapshotAndSubscription);
        Run(TestCommandWorkerPostsTerminalResultsAndRetainsSnapshot);
        Run(TestCommandStartExceptionStaysInsideAbiBoundary);
        Run(TestCommandCreateFailureKeepsStartRootLocallyOwned);
        Run(TestUiControlsCreateTreePropertiesAndCommandAssociation);
        Run(TestWindowCloseRequestBorrowDeferResolveAndCallbackLifetime);
        return _failures == 0 ? 0 : 1;
    }

    private static int _failures;

    private static void Run(Action test)
    {
        try
        {
            test();
        }
        catch (Exception exception)
        {
            _failures++;
            Console.Error.WriteLine($"FAILED: {test.Method.Name}: {exception}");
        }
    }

    private static void TestRuntimeAndShutdownReceiptPreserveAbiValues()
    {
        var native = new FakeNativeApi
        {
            RuntimeHandle = 0xF000_0000_0000_0001,
            RequestHandle = 0xE000_0000_0000_0002,
            IsUiThread = 1,
            RequestStatus = UiStatus.Pending,
            RequestState = (uint)ShutdownRequestState.HostUnavailable,
            RequestReason = 0xF123_4567,
        };

        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        Check(runtime.IsUiThread().Value, "byte ui_bool should map to true");

        ShutdownReceipt receipt = runtime.RequestShutdown().Value;
        Check(native.LastRuntime == 0xF000_0000_0000_0001, "ui_handle must retain all 64 bits");
        Check(native.LastRequest == 0xE000_0000_0000_0002, "receipt token must retain all 64 bits");
        Check(receipt.Query().Value.RawState == (uint)ShutdownRequestState.HostUnavailable,
            "receipt query should preserve the raw state");
        Check(receipt.Query().Value.Reason == 0xF123_4567, "receipt query should preserve the raw reason");
        Check(receipt.TryClose().Value == UiCloseDisposition.Released, "receipt close should release the owned token");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime close should release the owned token");
        Check(native.ReleaseCount(0xE000_0000_0000_0002) == 1, "receipt should release exactly once");
        Check(native.ReleaseCount(0xF000_0000_0000_0001) == 1, "Runtime should release exactly once");
    }

    private static void TestErrorSnapshotPreservesUnknownValues()
    {
        var native = new FakeNativeApi
        {
            CreateStatus = 0xF123_4567,
            ErrorHandle = 0xD000_0000_0000_0003,
            ErrorStatusValue = 0xF123_4567,
            ErrorDomainValue = 0xABCD_0001,
            ErrorCodeValue = 0xFFFF_0002,
            ErrorMessageValue = [0xE3, 0x81, 0x82, 0x00, 0x74],
        };

        UiResult<UiRuntime> result = UiRuntime.TryCreate(native);
        Check(result.IsFailure, "unknown native status should fail Runtime creation");
        Check(result.Error?.RawStatus == 0xF123_4567, "unknown status must remain raw");
        Check(result.Error?.Domain == 0xABCD_0001 && result.Error.Code == 0xFFFF_0002,
            "unknown error domain and code must remain raw");
        byte[] message = result.Error!.MessageUtf8!;
        Check(message.AsSpan().SequenceEqual(native.ErrorMessageValue), "error message bytes must be copied exactly");
        message[0] = 0;
        Check(result.Error.MessageUtf8![0] == 0xE3, "error snapshot message must be immutable to callers");
        Check(native.ReleaseCount(0xD000_0000_0000_0003) == 1, "error handle should be released once");
    }

    private static void TestExplicitCloseFailureRetainsTokenForRetry()
    {
        var native = new FakeNativeApi { RuntimeHandle = 41 };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        native.SetReleaseFailures(41, 1);

        UiResult<UiCloseDisposition> first = runtime.TryClose();
        Check(first.IsFailure && first.Error?.RawStatus == UiStatus.InternalError,
            "explicit release failure should be returned");
        Check(runtime.IsUiThread().IsSuccess, "failed TryClose must retain an open token for retry");
        Check(runtime.TryClose().IsSuccess, "second close should retry and succeed");
        Check(native.ReleaseCount(41) == 2, "retry should issue one additional native release");
    }

    private static void TestCloseWaitsForCallLeaseAndRejectsNewCalls()
    {
        var native = new FakeNativeApi();
        var owner = new TokenOwner(native);
        owner.Adopt(73);
        CallLease lease = owner.AcquireLease().Value;

        UiResult<UiCloseDisposition> close = owner.TryClose();
        Check(close.IsSuccess && close.Value == UiCloseDisposition.Pending,
            "close with an active lease should report pending");
        Check(owner.AcquireLease().IsFailure, "closing owner must reject new calls");
        Check(native.ReleaseCount(73) == 0, "active call token must not be released");

        lease.Dispose();
        Check(SpinWait.SpinUntil(() => native.ReleaseCount(73) == 1, TimeSpan.FromSeconds(2)),
            "the final lease should release the token");
    }

    private static void TestDisposeReleaseFailureMovesTokenToRetryTicket()
    {
        var native = new FakeNativeApi();
        var owner = new TokenOwner(native);
        owner.Adopt(88);
        native.SetReleaseFailures(88, 1);

        owner.Dispose();
        Check(SpinWait.SpinUntil(() => native.ReleaseCount(88) >= 2, TimeSpan.FromSeconds(2)),
            "cleanup manager should retry a failed release ticket");
        Check(native.SuccessfulReleaseCount(88) == 1, "ticket should consume the token once");
    }

    private static void TestBoolAndI64PropertyRoundTripAndClose()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;

        UiProperty<bool> boolean = UiProperty<bool>.TryCreate(runtime, true).Value;
        Check(native.LastCreatedBool == 1, "true should map to ui_bool 1");
        Check(boolean.TryGet().Value, "BOOL initial value should be true");
        Check(boolean.TrySet(false).Value, "BOOL setter should succeed");
        Check(native.LastSetBool == 0 && !boolean.TryGet().Value, "false should map to ui_bool 0");
        Check(boolean.TryClose().Value == UiCloseDisposition.Released, "BOOL property should close");
        Check(boolean.TryGet().Error?.RawStatus == UiStatus.Closed, "closed BOOL property should reject calls");

        UiProperty<long> integer = UiProperty<long>.TryCreate(runtime, long.MinValue).Value;
        Check(native.LastCreatedI64 == long.MinValue, "I64 minimum should pass without conversion");
        Check(integer.TryGet().Value == long.MinValue, "I64 minimum should round trip");
        Check(integer.TrySet(long.MaxValue).Value, "I64 maximum setter should succeed");
        Check(native.LastSetI64 == long.MaxValue && integer.TryGet().Value == long.MaxValue,
            "I64 maximum should round trip");
        Check(integer.TryClose().Value == UiCloseDisposition.Released, "I64 property should close");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after properties");
        Check(native.ReleaseCount(native.BoolPropertyHandle) == 1, "BOOL token should release once");
        Check(native.ReleaseCount(native.I64PropertyHandle) == 1, "I64 token should release once");
    }

    private static void TestUnsupportedPropertyTypeIsRejectedBeforeNativeCall()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;

        UiResult<UiProperty<DateTime>> unsupported = UiProperty<DateTime>.TryCreate(runtime, DateTime.UnixEpoch);
        Check(unsupported.IsFailure && unsupported.Error?.RawStatus == UiStatus.Unsupported,
            "unsupported managed property types should return UI_UNSUPPORTED");
        Check(native.PropertyCreateCount == 0, "unsupported type must be rejected before entering the ABI");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should remain usable");
    }

    private static void TestUtf8PropertyRoundTripAndStrictInput()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiProperty<string> property = UiProperty<string>.TryCreate(runtime, "初期値").Value;
        string[] values = [string.Empty, "ASCII", "こんにちは", "UI 🙂", "前\0後"];
        var strictUtf8 = new UTF8Encoding(false, true);

        foreach (string value in values)
        {
            Check(property.TrySet(value).IsSuccess, "valid UTF-16 string should set successfully");
            byte[] expectedBytes = strictUtf8.GetBytes(value);
            Check(native.LastSetUtf8.SequenceEqual(expectedBytes),
                "setter should pass the exact UTF-8 bytes and byte length");
            Check(property.TryGet().Value == value, "UTF-8 property should round trip the exact string");
        }

        Check(native.LastCreatedUtf8.SequenceEqual(strictUtf8.GetBytes("初期値")),
            "create should pass the initial UTF-8 bytes and byte length");
        int createCount = native.PropertyCreateCount;
        UiResult<UiProperty<string>> invalidCreate = UiProperty<string>.TryCreate(runtime, "bad\uD800");
        Check(invalidCreate.IsFailure && invalidCreate.Error?.RawStatus == UiStatus.InvalidArgument,
            "unpaired surrogate should be rejected as invalid input");
        Check(native.PropertyCreateCount == createCount, "invalid UTF-16 must not enter the native ABI");

        int setCount = native.PropertySetCount;
        UiResult<bool> invalidSet = property.TrySet("bad\uDC00");
        Check(invalidSet.IsFailure && invalidSet.Error?.RawStatus == UiStatus.InvalidArgument,
            "unpaired low surrogate should be rejected as invalid input");
        Check(native.PropertySetCount == setCount, "invalid UTF-16 setter must not enter the native ABI");
        Check(property.TryClose().Value == UiCloseDisposition.Released, "UTF-8 property should close");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after UTF-8 property");
    }

    private static void TestUtf8GetterRetriesAndRejectsInvalidBytes()
    {
        var native = new FakeNativeApi { GrowUtf8OnCopy = true, Utf8GrowthSuffix = Encoding.UTF8.GetBytes("追加") };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiProperty<string> property = UiProperty<string>.TryCreate(runtime, "buffer").Value;

        UiResult<string> grown = property.TryGet();
        Check(grown.IsSuccess && grown.Value == "buffer追加",
            "getter should re-query and retry when the value grows after size query");
        Check(native.ShortUtf8BufferWasUntouched,
            "buffer-too-small response should not expose a partially written buffer");

        native.Utf8GetterOverride = [0xC3, 0x28];
        UiResult<string> invalidBytes = property.TryGet();
        Check(invalidBytes.IsFailure && invalidBytes.Error?.RawStatus == UiStatus.InternalError,
            "invalid ABI UTF-8 should fail without replacement decoding");

        native.Utf8GetterOverride = null;
        native.PropertyGetStatus = 0xF123_4567;
        UiResult<string> failedGet = property.TryGet();
        Check(failedGet.IsFailure && failedGet.Error?.RawStatus == 0xF123_4567,
            "UTF-8 getter should preserve an unknown raw status");
        Check(property.TryClose().IsSuccess, "UTF-8 property should close after getter failures");
        Check(runtime.TryClose().IsSuccess, "Runtime should close after UTF-8 property");
    }

    private static void TestUtf8PropertyFailuresAndCloseRetry()
    {
        var native = new FakeNativeApi
        {
            PropertyCreateStatus = 0xF765_4321,
            PropertyErrorHandle = 0xC000_0000_0000_0008,
            ErrorStatusValue = 0xF765_4321,
            ErrorDomainValue = 0xABCD_0006,
            ErrorCodeValue = 0xFFFF_0007,
        };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;

        UiResult<UiProperty<string>> create = UiProperty<string>.TryCreate(runtime, "失敗");
        Check(create.IsFailure && create.Error?.RawStatus == 0xF765_4321,
            "unknown UTF-8 create status should remain raw");
        Check(create.Error?.Domain == 0xABCD_0006 && create.Error.Code == 0xFFFF_0007,
            "UTF-8 create error snapshot should preserve domain and code");

        native.PropertyCreateStatus = UiStatus.Ok;
        native.PropertyErrorHandle = 0;
        UiProperty<string> property = UiProperty<string>.TryCreate(runtime, "before").Value;
        native.PropertySetStatus = 0xF123_4567;
        native.PropertyErrorHandle = 0xC000_0000_0000_0009;
        native.ErrorStatusValue = 0xF123_4567;
        native.ErrorDomainValue = 0xABCD_0007;
        native.ErrorCodeValue = 0xFFFF_0008;
        UiResult<bool> set = property.TrySet("after");
        Check(set.IsFailure && set.Error?.RawStatus == 0xF123_4567,
            "unknown UTF-8 set status should remain raw");
        Check(set.Error?.Domain == 0xABCD_0007 && set.Error.Code == 0xFFFF_0008,
            "UTF-8 set error snapshot should preserve domain and code");
        Check(property.TryGet().Value == "before", "failed UTF-8 set should retain the prior value");

        native.PropertySetStatus = UiStatus.Ok;
        native.PropertyErrorHandle = 0;
        native.SetReleaseFailures(native.Utf8PropertyHandle, 1);
        UiResult<UiCloseDisposition> close = property.TryClose();
        Check(close.IsFailure && close.Error?.RawStatus == UiStatus.InternalError,
            "failed UTF-8 release should be reported");
        Check(property.TryGet().Value == "before", "failed close should retain a usable token for retry");
        Check(property.TryClose().Value == UiCloseDisposition.Released, "UTF-8 release should be retryable");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after UTF-8 property");
    }

    private static void TestDecimalValuePartsAndExactConversions()
    {
        const ulong maxCoefficient = (1UL << 58) - 1;
        UiResult<DecimalValue> maximum = DecimalValue.TryFromParts(true, 31, maxCoefficient);
        Check(maximum.IsSuccess && maximum.Value.IsNegative && maximum.Value.Scale == 31 &&
            maximum.Value.Coefficient == maxCoefficient,
            "DecimalValue parts should preserve sign, maximum scale, and maximum coefficient");
        Check(DecimalValue.TryFromParts(false, 32, 0).Error?.RawStatus == UiStatus.InvalidArgument,
            "scale above 31 should fail locally");
        Check(DecimalValue.TryFromParts(false, 0, maxCoefficient + 1).Error?.RawStatus == UiStatus.InvalidArgument,
            "coefficient above 58 bits should fail locally");

        decimal negativeZero = new(0, 0, 0, true, 5);
        DecimalValue negativeZeroValue = DecimalValue.TryFromDecimal(negativeZero).Value;
        Check(negativeZeroValue.IsNegative && negativeZeroValue.Scale == 5 && negativeZeroValue.Coefficient == 0,
            "System.Decimal negative zero sign and scale should be preserved when representable");

        ulong shrinkableCoefficient = maxCoefficient * 10;
        decimal shrinkable = new(
            unchecked((int)(uint)shrinkableCoefficient),
            unchecked((int)(uint)(shrinkableCoefficient >> 32)),
            0,
            false,
            1);
        DecimalValue shrunk = DecimalValue.TryFromDecimal(shrinkable).Value;
        Check(shrunk.Scale == 0 && shrunk.Coefficient == maxCoefficient,
            "an oversized coefficient should shrink only by exact trailing-zero removal");

        decimal nonShrinkable = new(0, 1 << 26, 0, false, 1);
        Check(DecimalValue.TryFromDecimal(nonShrinkable).Error?.RawStatus == UiStatus.InvalidArgument,
            "an oversized coefficient without a removable decimal zero should fail without rounding");

        DecimalValue exactAtSystemScale = DecimalValue.TryFromParts(false, 31, 1000).Value;
        UiResult<decimal> converted = exactAtSystemScale.TryToDecimal();
        Check(converted.IsSuccess && converted.Value == 0.0000000000000000000000000001m,
            "scale above 28 should convert only when exact trailing-zero reduction is possible");
        DecimalValue tooFine = DecimalValue.TryFromParts(false, 31, 1).Value;
        Check(tooFine.TryToDecimal().Error?.RawStatus == UiStatus.InvalidArgument,
            "a value below System.Decimal precision should fail instead of rounding");
    }

    private static void TestDecimalPropertyCanonicalBitsAndFailures()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        const ulong maxCoefficient = (1UL << 58) - 1;
        DecimalValue initial = DecimalValue.TryFromParts(false, 2, 100).Value;
        UiProperty<DecimalValue> property = UiProperty<DecimalValue>.TryCreate(runtime, initial).Value;
        Check(native.LastCreatedDecimalBits == ((ulong)2 << 58 | 100),
            "Decimal Property create should pass canonical bits directly");

        (bool Negative, byte Scale, ulong Coefficient)[] cases =
        [
            (false, 0, 0),
            (true, 31, 0),
            (false, 2, 100),
            (true, 0, maxCoefficient),
        ];
        foreach (var item in cases)
        {
            DecimalValue candidate = DecimalValue.TryFromParts(item.Negative, item.Scale, item.Coefficient).Value;
            Check(property.TrySet(candidate).IsSuccess, "Decimal Property set should accept valid canonical bits");
            Check(native.LastSetDecimalBits == candidate.Bits, "Decimal Property setter should not transform canonical bits");
            DecimalValue actual = property.TryGet().Value;
            Check(actual.Bits == candidate.Bits, "Decimal Property getter should preserve every canonical bit");
        }

        native.PropertyCreateStatus = 0xF765_4321;
        native.PropertyErrorHandle = 0xC000_0000_0000_0008;
        native.ErrorStatusValue = 0xF765_4321;
        native.ErrorDomainValue = 0xABCD_0006;
        native.ErrorCodeValue = 0xFFFF_0007;
        UiResult<UiProperty<DecimalValue>> failedCreate = UiProperty<DecimalValue>.TryCreate(runtime, initial);
        Check(failedCreate.IsFailure && failedCreate.Error?.RawStatus == 0xF765_4321 &&
            failedCreate.Error.Domain == 0xABCD_0006 && failedCreate.Error.Code == 0xFFFF_0007,
            "Decimal Property create should preserve native error status and snapshot");

        native.PropertyCreateStatus = UiStatus.Ok;
        native.PropertyErrorHandle = 0;
        native.PropertySetStatus = 0xF123_4567;
        native.PropertyErrorHandle = 0xC000_0000_0000_0009;
        native.ErrorStatusValue = 0xF123_4567;
        native.ErrorDomainValue = 0xABCD_0007;
        native.ErrorCodeValue = 0xFFFF_0008;
        DecimalValue before = property.TryGet().Value;
        UiResult<bool> failedSet = property.TrySet(DecimalValue.TryFromParts(true, 7, 42).Value);
        Check(failedSet.IsFailure && failedSet.Error?.RawStatus == 0xF123_4567 &&
            failedSet.Error.Domain == 0xABCD_0007 && failedSet.Error.Code == 0xFFFF_0008,
            "Decimal Property set should preserve native error status and snapshot");
        Check(property.TryGet().Value.Bits == before.Bits, "failed Decimal set should preserve the current bits");

        native.PropertySetStatus = UiStatus.Ok;
        native.PropertyErrorHandle = 0;
        native.PropertyGetStatus = 0xE123_4567;
        Check(property.TryGet().Error?.RawStatus == 0xE123_4567,
            "Decimal getter should preserve unknown raw status");
        native.PropertyGetStatus = UiStatus.Ok;

        native.SetReleaseFailures(native.DecimalPropertyHandle, 1);
        Check(property.TryClose().Error?.RawStatus == UiStatus.InternalError,
            "failed Decimal Property release should be reported");
        Check(property.TryGet().IsSuccess, "failed Decimal release should retain an open retryable token");
        Check(property.TryClose().Value == UiCloseDisposition.Released, "Decimal Property release should retry");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after Decimal Property");
    }

    private static void TestF64PropertyPreservesBitPatterns()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        long[] bitPatterns =
        [
            0x3FF4_0000_0000_0000,
            0x7FF8_0000_0000_0042,
            0x7FF8_0000_0000_1234,
            0x7FF0_0000_0000_0000,
            unchecked((long)0xFFF0_0000_0000_0000),
            0,
            long.MinValue,
        ];

        double initial = BitConverter.Int64BitsToDouble(bitPatterns[0]);
        UiProperty<double> property = UiProperty<double>.TryCreate(runtime, initial).Value;
        Check(native.LastCreatedF64Bits == bitPatterns[0], "finite F64 create should preserve the raw bits");
        Check(BitConverter.DoubleToInt64Bits(property.TryGet().Value) == bitPatterns[0],
            "finite F64 getter should preserve the raw bits");

        foreach (long bits in bitPatterns.Skip(1))
        {
            double value = BitConverter.Int64BitsToDouble(bits);
            Check(property.TrySet(value).IsSuccess, "F64 setter should accept the value");
            Check(native.LastSetF64Bits == bits, "F64 setter should forward the raw bits");
            Check(BitConverter.DoubleToInt64Bits(property.TryGet().Value) == bits,
                "F64 getter should return the raw bits");
        }

        Check(property.TryClose().Value == UiCloseDisposition.Released, "F64 property should close");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after F64 property");
        Check(native.ReleaseCount(native.F64PropertyHandle) == 1, "F64 token should release once");
    }

    private static void TestF64PropertyFailuresAndCloseRetry()
    {
        var native = new FakeNativeApi
        {
            PropertyCreateStatus = 0xF765_4321,
            PropertyErrorHandle = 0xC000_0000_0000_0006,
            ErrorStatusValue = 0xF765_4321,
            ErrorDomainValue = 0xABCD_0004,
            ErrorCodeValue = 0xFFFF_0005,
        };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;

        UiResult<UiProperty<double>> create = UiProperty<double>.TryCreate(runtime, 1.25);
        Check(create.IsFailure && create.Error?.RawStatus == 0xF765_4321,
            "unknown F64 create status should remain raw");
        Check(create.Error?.Domain == 0xABCD_0004 && create.Error.Code == 0xFFFF_0005,
            "F64 create error snapshot should preserve domain and code");

        native.PropertyCreateStatus = UiStatus.Ok;
        native.PropertyErrorHandle = 0;
        UiProperty<double> property = UiProperty<double>.TryCreate(runtime, 1.25).Value;
        native.PropertySetStatus = 0xF123_4567;
        native.PropertyErrorHandle = 0xC000_0000_0000_0007;
        native.ErrorStatusValue = 0xF123_4567;
        native.ErrorDomainValue = 0xABCD_0005;
        native.ErrorCodeValue = 0xFFFF_0006;
        UiResult<bool> set = property.TrySet(2.5);
        Check(set.IsFailure && set.Error?.RawStatus == 0xF123_4567,
            "unknown F64 set status should remain raw");
        Check(set.Error?.Domain == 0xABCD_0005 && set.Error.Code == 0xFFFF_0006,
            "F64 set error snapshot should preserve domain and code");
        Check(BitConverter.DoubleToInt64Bits(property.TryGet().Value) == BitConverter.DoubleToInt64Bits(1.25),
            "failed F64 setter should leave the stored value unchanged");
        Check(native.ReleaseCount(native.PropertyErrorHandle) == 1, "F64 set error token should release once");

        native.PropertySetStatus = UiStatus.Ok;
        native.PropertyErrorHandle = 0;
        native.PropertyGetStatus = UiStatus.WrongThread;
        UiResult<double> wrongThread = property.TryGet();
        Check(wrongThread.IsFailure && wrongThread.Error?.RawStatus == UiStatus.WrongThread,
            "F64 getter should preserve raw wrong-thread status");

        native.PropertyGetStatus = UiStatus.Ok;
        native.SetReleaseFailures(native.F64PropertyHandle, 1);
        UiResult<UiCloseDisposition> firstClose = property.TryClose();
        Check(firstClose.IsFailure && firstClose.Error?.RawStatus == UiStatus.InternalError,
            "failed F64 release should be reported");
        Check(property.TryGet().IsSuccess, "failed explicit F64 close should retain the token");
        Check(property.TryClose().Value == UiCloseDisposition.Released, "F64 release should be retryable");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after F64 property");
    }

    private static void TestPropertyFailuresPreserveStatusAndValue()
    {
        var native = new FakeNativeApi
        {
            PropertySetStatus = 0xF123_4567,
            PropertyErrorHandle = 0xC000_0000_0000_0004,
            ErrorStatusValue = 0xF123_4567,
            ErrorDomainValue = 0xABCD_0002,
            ErrorCodeValue = 0xFFFF_0003,
        };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiProperty<long> property = UiProperty<long>.TryCreate(runtime, 25).Value;

        UiResult<bool> set = property.TrySet(99);
        Check(set.IsFailure && set.Error?.RawStatus == 0xF123_4567, "unknown set status must remain raw");
        Check(set.Error?.Domain == 0xABCD_0002 && set.Error.Code == 0xFFFF_0003,
            "set error snapshot should preserve domain and code");
        Check(property.TryGet().Value == 25, "failed setter must not be reported as a committed value");
        Check(native.ReleaseCount(native.PropertyErrorHandle) == 1, "set error token should release once");

        native.PropertySetStatus = UiStatus.Ok;
        native.MalformedBoolValue = 2;
        UiProperty<bool> boolean = UiProperty<bool>.TryCreate(runtime, true).Value;
        UiResult<bool> invalidBool = boolean.TryGet();
        Check(invalidBool.IsFailure && invalidBool.Error?.RawStatus == UiStatus.InternalError,
            "a noncanonical ui_bool from the ABI should be rejected");
        boolean.Dispose();
        property.Dispose();
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after failed operations");
    }

    private static void TestPropertyLeaseProtectsHandleDuringDispose()
    {
        var native = new FakeNativeApi { BlockI64Get = true };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiProperty<long> property = UiProperty<long>.TryCreate(runtime, 37).Value;

        Task<UiResult<long>> getter = Task.Run(property.TryGet);
        Check(native.I64GetEntered.Wait(TimeSpan.FromSeconds(2)), "fake getter should enter native call");
        UiResult<UiCloseDisposition> close = property.TryClose();
        Check(close.IsSuccess && close.Value == UiCloseDisposition.Pending,
            "close during an active property call should report pending");
        Check(native.ReleaseCount(native.I64PropertyHandle) == 0, "active call must keep its property token alive");

        native.AllowI64Get.Set();
        Check(getter.Wait(TimeSpan.FromSeconds(2)) && getter.Result.Value == 37, "active getter should complete");
        Check(SpinWait.SpinUntil(() => native.ReleaseCount(native.I64PropertyHandle) == 1, TimeSpan.FromSeconds(2)),
            "final lease should release the property exactly once");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after property release");
    }

    private static void TestPropertySubscriptionAndCallbackFailureHandling()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiProperty<long> property = UiProperty<long>.TryCreate(runtime, 12).Value;
        int observedValue = 0;
        UiSubscription subscription = property.TrySubscribe(() => observedValue = (int)property.TryGet().Value).Value;
        Check(native.SubscriptionCount == 1, "subscribe should call the native ABI once");
        Check(observedValue == 0, "subscribe should not issue an initial notification");

        property.TrySet(42);
        native.InvokeSubscription(native.LastSubscription);
        Check(observedValue == 42, "notification callback should read the latest property value");

        ulong firstSubscriptionToken = native.LastSubscription;
        native.SetReleaseFailures(firstSubscriptionToken, 1);
        Check(subscription.TryClose().IsFailure,
            "failed subscription handle release should remain retryable");
        property.TrySet(43);
        native.InvokeSubscription(firstSubscriptionToken);
        Check(observedValue == 43, "releasing the subscription handle must not cancel notification delivery");
        Check(subscription.TryClose().Value == UiCloseDisposition.Released,
            "subscription handle release should succeed on retry");
        property.TrySet(44);
        native.InvokeSubscription(firstSubscriptionToken);
        Check(observedValue == 44, "successful handle release must leave the active registration intact");

        (WeakReference rootedCapture, UiSubscription rootedSubscription, ulong rootedToken) =
            SubscribeWithWeakCapture(property, native);
        CollectGarbage();
        Check(rootedCapture.IsAlive, "Core-owned callback root should keep its managed capture alive");
        native.InvokeSubscription(rootedToken);
        native.DestroySubscription(rootedToken);
        native.DestroySubscription(rootedToken);
        CollectGarbage();
        Check(!rootedCapture.IsAlive, "Core destroy should release the managed callback root exactly once");
        rootedSubscription.TryClose();

        native.PropertySubscribeStatus = UiStatus.InternalError;
        WeakReference failedCapture = SubscribeWithFailedWeakCapture(property);
        native.PropertySubscribeStatus = UiStatus.Ok;
        CollectGarbage();
        Check(!failedCapture.IsAlive, "failed registration should release its managed callback root");

        (WeakReference inFlightCapture, UiSubscription inFlightSubscription, ulong inFlightToken,
            ManualResetEventSlim callbackEntered, ManualResetEventSlim allowCallbackReturn) =
            SubscribeWithBlockingCapture(property, native);
        Task invocation = Task.Run(() => native.InvokeSubscription(inFlightToken));
        Check(callbackEntered.Wait(TimeSpan.FromSeconds(2)), "fake callback should enter before the lifetime check");
        CollectGarbage();
        Check(inFlightCapture.IsAlive, "in-flight invocation should keep its callback capture alive");
        allowCallbackReturn.Set();
        Check(invocation.Wait(TimeSpan.FromSeconds(2)), "in-flight callback should return");
        native.DestroySubscription(inFlightToken);
        inFlightSubscription.TryClose();
        callbackEntered.Dispose();
        allowCallbackReturn.Dispose();
        CollectGarbage();
        Check(!inFlightCapture.IsAlive, "destroy after callback completion should release its capture");

        int failuresReported = 0;
        UiCallbackFailure? reported = null;
        UiCallbackHandlerRegistration firstHandler = runtime.TrySetCallbackFailureHandler(failure =>
        {
            reported = failure;
            failuresReported++;
        }).Value;
        UiCallbackHandlerRegistration replacement = runtime.TrySetCallbackFailureHandler(_ => failuresReported += 10).Value;
        firstHandler.Dispose();

        UiSubscription throwing = property.TrySubscribe(static () => throw new InvalidOperationException("callback")).Value;
        ulong throwingToken = native.LastSubscription;
        native.InvokeSubscription(throwingToken);
        Check(failuresReported == 10, "disposing an old registration must retain its replacement");
        Check(reported is null, "a replaced handler must not receive later failures");

        int latestHandlerCalls = 0;
        replacement.Dispose();
        UiCallbackHandlerRegistration latestHandler = runtime.TrySetCallbackFailureHandler(failure =>
        {
            Check(failure.Kind == UiCallbackFailureKind.PropertyNotification,
                "callback failure should identify the notification kind");
            Check(failure.Exception is InvalidOperationException,
                "callback failure should preserve the caught managed exception");
            latestHandlerCalls++;
            throw new InvalidOperationException("handler");
        }).Value;

        var trace = new RecordingTraceListener();
        Trace.Listeners.Add(trace);
        try
        {
            native.InvokeSubscription(throwingToken);
            Check(latestHandlerCalls == 1, "the current handler should receive the callback failure");
            Check(trace.LineCount == 1, "a throwing handler should be reported to Trace exactly once");
        }
        finally
        {
            Trace.Listeners.Remove(trace);
        }

        latestHandler.Dispose();
        trace = new RecordingTraceListener();
        Trace.Listeners.Add(trace);
        try
        {
            native.InvokeSubscription(throwingToken);
            Check(trace.LineCount == 1, "an unhandled callback exception should be traced exactly once");
        }
        finally
        {
            Trace.Listeners.Remove(trace);
        }

        var throwingTrace = new ThrowingTraceListener();
        Trace.Listeners.Add(throwingTrace);
        try
        {
            native.InvokeSubscription(throwingToken);
        }
        finally
        {
            Trace.Listeners.Remove(throwingTrace);
        }

        throwing.TryCancel();
        Check(throwing.TryCancel().Value, "repeated cancellation of an inactive subscription should succeed");
        int beforeCanceledInvoke = latestHandlerCalls;
        native.InvokeSubscription(throwingToken);
        Check(latestHandlerCalls == beforeCanceledInvoke, "cancel should prevent new callback starts");

        native.SubscriptionCancelStatus = UiStatus.InternalError;
        UiSubscription retryable = property.TrySubscribe(static () => { }).Value;
        ulong retryableToken = native.LastSubscription;
        UiResult<bool> cancelFailure = retryable.TryCancel();
        Check(cancelFailure.IsFailure, "native cancel failure should be observable");
        native.SubscriptionCancelStatus = UiStatus.Ok;
        Check(retryable.TryCancel().IsSuccess, "failed cancellation should be retryable");
        retryable.TryClose();

        native.DestroySubscription(firstSubscriptionToken);
        native.DestroySubscription(throwingToken);
        native.DestroySubscription(retryableToken);

        runtime.TryClose();
        property.TryClose();
    }

    private sealed class RecordingTraceListener : TraceListener
    {
        internal int LineCount { get; private set; }

        public override void Write(string? message) { }

        public override void WriteLine(string? message) => LineCount++;
    }

    private sealed class ThrowingTraceListener : TraceListener
    {
        public override void Write(string? message) => throw new InvalidOperationException("trace");

        public override void WriteLine(string? message) => throw new InvalidOperationException("trace");
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static (WeakReference Capture, UiSubscription Subscription, ulong Token) SubscribeWithWeakCapture(
        UiProperty<long> property, FakeNativeApi native)
    {
        var capture = new object();
        var weakCapture = new WeakReference(capture);
        Action callback = () => GC.KeepAlive(capture);
        UiSubscription subscription = property.TrySubscribe(callback).Value;
        return (weakCapture, subscription, native.LastSubscription);
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static WeakReference SubscribeWithFailedWeakCapture(UiProperty<long> property)
    {
        var capture = new object();
        var weakCapture = new WeakReference(capture);
        Action callback = () => GC.KeepAlive(capture);
        Check(property.TrySubscribe(callback).IsFailure, "configured native registration failure should be reported");
        return weakCapture;
    }

    [MethodImpl(MethodImplOptions.NoInlining)]
    private static (WeakReference Capture, UiSubscription Subscription, ulong Token,
        ManualResetEventSlim Entered, ManualResetEventSlim AllowReturn) SubscribeWithBlockingCapture(
        UiProperty<long> property, FakeNativeApi native)
    {
        var capture = new object();
        var weakCapture = new WeakReference(capture);
        var entered = new ManualResetEventSlim();
        var allowReturn = new ManualResetEventSlim();
        Action callback = () =>
        {
            entered.Set();
            allowReturn.Wait(TimeSpan.FromSeconds(5));
            GC.KeepAlive(capture);
        };
        UiSubscription subscription = property.TrySubscribe(callback).Value;
        return (weakCapture, subscription, native.LastSubscription, entered, allowReturn);
    }

    private static void CollectGarbage()
    {
        GC.Collect();
        GC.WaitForPendingFinalizers();
        GC.Collect();
    }

    private static void TestPropertyCreateAndGetterFailuresAndCloseRetry()
    {
        var native = new FakeNativeApi
        {
            PropertyCreateStatus = 0xF765_4321,
            PropertyErrorHandle = 0xC000_0000_0000_0005,
            ErrorStatusValue = 0xF765_4321,
            ErrorDomainValue = 0xABCD_0003,
            ErrorCodeValue = 0xFFFF_0004,
        };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;

        UiResult<UiProperty<long>> create = UiProperty<long>.TryCreate(runtime, 1);
        Check(create.IsFailure && create.Error?.RawStatus == 0xF765_4321,
            "unknown create status should remain raw");
        Check(create.Error?.Domain == 0xABCD_0003 && create.Error.Code == 0xFFFF_0004,
            "create error snapshot should preserve domain and code");
        Check(native.ReleaseCount(native.PropertyErrorHandle) == 1, "create error token should release once");

        native.PropertyCreateStatus = UiStatus.Ok;
        UiProperty<long> property = UiProperty<long>.TryCreate(runtime, 11).Value;
        native.PropertyGetStatus = UiStatus.WrongThread;
        UiResult<long> wrongThread = property.TryGet();
        Check(wrongThread.IsFailure && wrongThread.Error?.RawStatus == UiStatus.WrongThread,
            "getter should preserve raw wrong-thread status");

        native.PropertyGetStatus = UiStatus.Ok;
        native.SetReleaseFailures(native.I64PropertyHandle, 1);
        UiResult<UiCloseDisposition> firstClose = property.TryClose();
        Check(firstClose.IsFailure && firstClose.Error?.RawStatus == UiStatus.InternalError,
            "failed property release should be reported");
        Check(property.TryGet().Value == 11, "failed explicit close should retain the property token");
        Check(property.TryClose().Value == UiCloseDisposition.Released, "property release should be retryable");
        Check(native.SuccessfulReleaseCount(native.I64PropertyHandle) == 1,
            "retried property release should consume its token once");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should close after property retry");
    }

    private static void TestPropertyBindingModesErrorSnapshotAndSubscription()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiProperty<bool> boolSource = UiProperty<bool>.TryCreate(runtime, true).Value;
        UiProperty<bool> boolTarget = UiProperty<bool>.TryCreate(runtime, false).Value;
        UiProperty<long> longSource = UiProperty<long>.TryCreate(runtime, 1).Value;
        UiProperty<long> longTarget = UiProperty<long>.TryCreate(runtime, 0).Value;
        UiProperty<double> doubleSource = UiProperty<double>.TryCreate(runtime, 1.5).Value;
        UiProperty<double> doubleTarget = UiProperty<double>.TryCreate(runtime, 0).Value;
        UiProperty<DecimalValue> decimalSource = UiProperty<DecimalValue>.TryCreate(
            runtime, DecimalValue.TryFromParts(false, 0, 15).Value).Value;
        UiProperty<DecimalValue> decimalTarget = UiProperty<DecimalValue>.TryCreate(
            runtime, DecimalValue.TryFromParts(false, 0, 0).Value).Value;
        UiProperty<string> stringSource = UiProperty<string>.TryCreate(runtime, "source").Value;
        UiProperty<string> stringTarget = UiProperty<string>.TryCreate(runtime, "target").Value;

        VerifyBindingModes(boolSource, boolTarget, native);
        VerifyBindingModes(longSource, longTarget, native);
        VerifyBindingModes(doubleSource, doubleTarget, native);
        VerifyBindingModes(decimalSource, decimalTarget, native);
        VerifyBindingModes(stringSource, stringTarget, native);

        native.BindingCreateStatus = 0xF123_4567;
        native.BindingOperationErrorToken = 0xD000_0000_0000_0101;
        native.ErrorStatusValue = 0xF123_4567;
        native.ErrorDomainValue = 0xABCD_0102;
        native.ErrorCodeValue = 0xFFFF_0103;
        native.ErrorMessageValue = [0x62, 0x69, 0x6E, 0x64];
        UiResult<UiBinding<long>?> failedCreate = longSource.TryBindTo(longTarget, UiBindingMode.OneWay);
        Check(failedCreate.IsFailure && failedCreate.Error?.RawStatus == 0xF123_4567,
            "binding create failure should preserve its raw status");
        Check(failedCreate.Error?.Domain == 0xABCD_0102 && failedCreate.Error.Code == 0xFFFF_0103,
            "binding create failure should preserve error detail");
        Check(native.ReleaseCount(native.BindingOperationErrorToken) == 1,
            "binding create error token should be released after managed capture");
        native.BindingCreateStatus = UiStatus.Ok;
        native.BindingOperationErrorToken = 0;

        UiBinding<long> binding = longSource.TryBindTo(longTarget, UiBindingMode.OneWay).Value!;
        UiResult<UiBindingError<long>?> noError = binding.TryGetErrorSnapshot();
        Check(noError.IsSuccess && noError.Value is null,
            "zero error snapshot outputs should produce a successful null value");

        native.BindingErrorToken = 0xD000_0000_0000_0104;
        native.BindingRejectedPropertyToken = 0xD000_0000_0000_0105;
        native.BindingDirectionValue = (uint)UiBindingDirection.TargetToSource;
        native.ErrorStatusValue = 0xF654_3210;
        native.ErrorDomainValue = 0xABCD_0106;
        native.ErrorCodeValue = 0xFFFF_0107;
        native.ErrorMessageValue = [0x6F, 0x6C, 0x64];
        UiBindingError<long> snapshot = binding.TryGetErrorSnapshot().Value!;
        Check(snapshot.Direction == UiBindingDirection.TargetToSource,
            "error snapshot should preserve propagation direction");
        Check(snapshot.Error.RawStatus == 0xF654_3210 && snapshot.Error.Domain == 0xABCD_0106 &&
            snapshot.Error.Code == 0xFFFF_0107 && snapshot.Error.Message == "old",
            "error snapshot should copy all native error details");
        native.ErrorMessageValue = [0x6E, 0x65, 0x77];
        Check(snapshot.Error.Message == "old", "captured error detail should remain immutable");
        Check(native.ReleaseCount(0xD000_0000_0000_0104) == 1,
            "native error snapshot token should be released after copying");
        Check(snapshot.TryClose().Value == UiCloseDisposition.Released,
            "closing an error snapshot should release its owned rejected Property token");
        Check(native.ReleaseCount(0xD000_0000_0000_0105) == 1,
            "rejected Property token should release exactly once");
        native.BindingErrorToken = 0;
        native.BindingRejectedPropertyToken = 0;

        UiCallbackFailure? callbackFailure = null;
        using UiCallbackHandlerRegistration handler = runtime.TrySetCallbackFailureHandler(
            failure => callbackFailure = failure).Value;
        int callbackCount = 0;
        UiSubscription errorSubscription = binding.TrySubscribeError(() =>
        {
            callbackCount++;
            throw new InvalidOperationException("binding callback");
        }).Value;
        Check(callbackCount == 0, "binding error registration should not send an initial callback");
        ulong subscriptionToken = native.LastSubscription;
        native.InvokeSubscription(subscriptionToken);
        Check(callbackCount == 1 && callbackFailure?.Kind == UiCallbackFailureKind.BindingErrorNotification,
            "binding error callback failures should identify their callback kind");
        Check(errorSubscription.TryClose().Value == UiCloseDisposition.Released,
            "closing the subscription handle should succeed");
        native.InvokeSubscription(subscriptionToken);
        Check(callbackCount == 2, "releasing the error subscription handle should not cancel the registration");
        native.DestroySubscription(subscriptionToken);

        native.BindingCancelStatus = UiStatus.WrongThread;
        Check(binding.TryCancel().IsFailure, "failed Binding cancel should be reported");
        native.BindingCancelStatus = UiStatus.Ok;
        Check(binding.TryCancel().Value, "Binding cancel should succeed on retry");
        ulong bindingToken = native.LastBinding;
        native.SetReleaseFailures(bindingToken, 1);
        UiResult<UiCloseDisposition> failedBindingClose = binding.TryClose();
        Check(failedBindingClose.IsFailure && failedBindingClose.Error?.RawStatus == UiStatus.InternalError,
            "failed Binding handle release should be reported and retained for retry");
        Check(binding.TryClose().Value == UiCloseDisposition.Released,
            "Binding handle release should be retryable and separate from cancellation");
        Check(native.ReleaseCount(bindingToken) == 2 && native.SuccessfulReleaseCount(bindingToken) == 1,
            "Binding handle should be consumed once after a failed release retry");

        boolSource.Dispose();
        boolTarget.Dispose();
        longSource.Dispose();
        longTarget.Dispose();
        doubleSource.Dispose();
        doubleTarget.Dispose();
        decimalSource.Dispose();
        decimalTarget.Dispose();
        stringSource.Dispose();
        stringTarget.Dispose();
        Check(runtime.TryClose().IsSuccess, "Runtime should close after Binding tests");
    }

    private static void VerifyBindingModes<T>(UiProperty<T> source, UiProperty<T> target, FakeNativeApi native)
    {
        foreach (UiBindingMode mode in Enum.GetValues<UiBindingMode>())
        {
            UiResult<UiBinding<T>?> result = source.TryBindTo(target, mode);
            Check(result.IsSuccess, $"{typeof(T).Name} {mode} Binding should succeed");
            Check(native.LastBindingSource != 0 && native.LastBindingTarget != 0 &&
                native.LastBindingMode == (uint)mode,
                "Binding should pass the correct source, target, and mode tokens");
            if (mode == UiBindingMode.OneTime)
            {
                Check(result.Value is null, "OneTime success should have no Binding wrapper");
                continue;
            }

            UiBinding<T> binding = result.Value!;
            ulong token = native.LastBinding;
            Check(binding.TryCancel().IsSuccess, "Binding cancel should succeed");
            Check(binding.TryClose().Value == UiCloseDisposition.Released,
                "Binding handle release should be independent from cancel");
            Check(native.ReleaseCount(token) == 1, "Binding wrapper should own one native Binding token");
        }
    }

    private static void Check(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
    }

    private static void TestCommandWorkerPostsTerminalResultsAndRetainsSnapshot()
    {
        var native = new FakeNativeApi
        {
            CommandFailureToken = 1601,
            ErrorStatusValue = 12,
            ErrorDomainValue = 7,
            ErrorCodeValue = 19,
            ErrorMessageValue = Encoding.UTF8.GetBytes("command failed"),
        };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiProperty<bool> eligibility = UiProperty<bool>.TryCreate(runtime, true).Value;
        UiCommandWorker? worker = null;
        UiCommand command = UiCommand.TryCreateAsync(runtime, eligibility, current => worker = current).Value;
        Check(native.LastCommandEligibility == native.BoolPropertyHandle, "Command must pass the eligibility handle to native");

        ulong operationId = command.TryExecute().Value;
        Check(worker is not null && worker.OperationId == operationId, "Start callback must receive the issued operation ID");
        Check(command.TryCancel(operationId).Value, "Command cancellation must be accepted");
        Check(worker!.IsCancellationRequested().Value == false, "The fake cancellation token should initially be clear");
        native.RuntimePostStatus = UiStatus.OutOfMemory;
        ulong rejectedPostCommandToken = native.LastRetainedToken + 1;
        Check(worker.TryReportSuccess().IsFailure, "A rejected Runtime post must remain an observable failure");
        Check(native.SuccessfulReleaseCount(rejectedPostCommandToken) == 1,
            "A rejected Runtime post must release its locally retained Command token");
        native.RuntimePostStatus = UiStatus.Ok;
        Check(worker.TryReportSuccess().Value, "Terminal success must enqueue a UI post");
        native.InvokeLastPost();
        Check(native.CommandCompletionCount == 1 && native.LastCompletionKind == 1, "Success must complete on the UI dispatch callback");
        worker.Dispose();

        ulong failedOperation = command.TryExecute().Value;
        Check(worker!.OperationId == failedOperation, "Each accepted execution must get its own worker");
        Check(worker.TryReportFailure(7, 19, "\uD800").Error?.RawStatus == UiStatus.InvalidArgument,
            "Invalid UTF-16 failure text must be rejected before allocating a post root");
        Check(worker.TryReportFailure(7, 19, "command failed").Value, "Failure must enqueue a UI post");
        native.InvokeLastPost();
        Check(native.LastCompletionKind == 2 && native.LastFailureDomain == 7 && native.LastFailureCode == 19, "Failure metadata must cross the post intact");
        Check(Encoding.UTF8.GetString(native.LastFailureMessage) == "command failed", "Failure text must be UTF-8 encoded without replacement");

        UiErrorSnapshot? snapshot = command.TryGetFailureSnapshot().Value;
        Check(snapshot is not null && snapshot.RawStatus == 12 && snapshot.Domain == 7 && snapshot.Code == 19, "Failure snapshot details must be copied from the owned ABI token");
        Check(snapshot!.Message == "command failed", "Failure snapshot must preserve its immutable message");
        snapshot.Dispose();
        Check(native.SuccessfulReleaseCount(1601) == 1, "Disposing the failure snapshot must release its owned native token");
        worker.Dispose();

        ulong cancelledOperation = command.TryExecute().Value;
        Check(worker!.OperationId == cancelledOperation, "Cancellation must be scoped to the current worker operation");
        Check(worker.TryReportCancelled().Value, "Cancellation terminal result must enqueue a UI post");
        native.InvokeLastPost();
        Check(native.LastCompletionKind == 3, "Cancelled completion must remain distinct from cancellation request");
        worker.Dispose();

        UiProperty<long> state = command.TryGetStateProperty().Value;
        UiProperty<bool> canExecute = command.TryGetCanExecuteProperty().Value;
        native.CommandCloseStatus = UiStatus.InternalError;
        Check(command.TryClose().IsFailure && native.CommandDestroyCount == 0,
            "A failed Command close must leave the callback root available for retry");
        native.CommandCloseStatus = UiStatus.Ok;
        Check(command.TryClose().Value, "Command close must be explicit and independent of handle release");
        Check(native.CommandDestroyCount == 1, "Command close must destroy the start callback root once");
        state.Dispose();
        canExecute.Dispose();
        command.Dispose();
        eligibility.Dispose();
        runtime.Dispose();
    }

    private static void TestCommandStartExceptionStaysInsideAbiBoundary()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiCallbackFailureKind? observedKind = null;
        using UiCallbackHandlerRegistration registration = runtime.TrySetCallbackFailureHandler(
            failure => observedKind = failure.Kind).Value;
        UiCommand command = UiCommand.TryCreateAsync(
            runtime,
            null,
            _ => throw new InvalidOperationException("start failed")).Value;

        Check(command.TryExecute().IsSuccess, "Accepted execution must remain accepted when its start callback fails");
        Check(observedKind == UiCallbackFailureKind.CommandStart, "Start exceptions must use the CommandStart handler kind");
        Check(native.LastStartStatus == UiStatus.InternalError, "Managed exceptions must be translated before crossing the C ABI");
        Check(native.CommandDestroyCount == 0, "The start root remains owned until explicit close");
        Check(command.TryClose().IsSuccess && native.CommandDestroyCount == 1, "Closing the command must destroy its callback root once");
        command.Dispose();
        runtime.Dispose();
    }

    private static void TestCommandCreateFailureKeepsStartRootLocallyOwned()
    {
        var native = new FakeNativeApi { CommandCreateStatus = UiStatus.OutOfMemory };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        int startCount = 0;
        UiResult<UiCommand> created = UiCommand.TryCreateAsync(runtime, null, _ => startCount++);

        Check(created.IsFailure && created.Error?.RawStatus == UiStatus.OutOfMemory,
            "Command creation failure must preserve its native status");
        Check(startCount == 0 && native.CommandStartCount == 0,
            "A rejected create must not invoke or transfer the start callback");
        Check(native.SuccessfulReleaseCount(1501) == 1 && native.SuccessfulReleaseCount(1502) == 1,
            "A rejected create must release wrapper and callback Runtime anchors locally");
        runtime.Dispose();
    }

    private static void TestUiControlsCreateTreePropertiesAndCommandAssociation()
    {
        var native = new FakeNativeApi();
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiWindow window = UiWindow.TryCreate(runtime).Value;
        UiStackPanel panel = UiStackPanel.TryCreate(runtime).Value;
        UiTextBox editor = UiTextBox.TryCreate(runtime, "initial").Value;
        UiButton button = UiButton.TryCreate(runtime, "Apply").Value;

        Check(window.TrySetContent(panel).IsSuccess, "Window should accept same-runtime content");
        Check(panel.TryInsert(editor, 0).IsSuccess, "Container should insert a child");
        Check(native.LastControlParent != 0 && native.LastControlChild != 0 && native.LastControlIndex == 0,
            "Container insert should forward parent, child, and index handles");
        var foreignNative = new FakeNativeApi();
        UiRuntime foreignRuntime = UiRuntime.TryCreate(foreignNative).Value;
        UiText foreignChild = UiText.TryCreate(foreignRuntime, "foreign").Value;
        int insertCalls = native.ContainerInsertCount;
        Check(panel.TryInsert(foreignChild, 1).Error?.RawStatus == UiStatus.WrongRuntime &&
            native.ContainerInsertCount == insertCalls,
            "A different API owner's child must be rejected before the ABI call");
        foreignChild.Dispose();
        foreignRuntime.Dispose();
        Check(panel.TrySetOrientation(UiControlOrientation.Horizontal).IsSuccess &&
            native.LastOrientation == (uint)UiControlOrientation.Horizontal,
            "Stack orientation should round trip through the ABI facade");
        int orientationCalls = native.StackPanelSetOrientationCount;
        Check(panel.TrySetOrientation((UiControlOrientation)99).IsFailure &&
            native.StackPanelSetOrientationCount == orientationCalls,
            "Unknown orientation values should be rejected before native code");
        Check(panel.TrySetChildAlignment(editor, UiChildAlignment.Stretch).IsSuccess &&
            native.LastAlignment == (uint)UiChildAlignment.Stretch,
            "Stack child alignment should be forwarded");
        Check(button.TryRequestFocus().IsSuccess && native.FocusRequestCount == 1,
            "Widget focus request should reach the native facade");

        UiProperty<string> value = editor.TryGetValueProperty().Value;
        Check(value.TrySet("edited").IsSuccess, "TextBox property should remain a normal bindable Property");
        Check(Encoding.UTF8.GetString(native.LastSetUtf8) == "edited", "TextBox Property should preserve UTF-8 input");

        UiCommand command = UiCommand.TryCreateAsync(runtime, null, _ => { }).Value;
        Check(button.TrySetCommand(command).IsSuccess && native.LastButtonCommand == native.CommandHandle,
            "Button should associate a same-runtime Command handle");
        Check(button.TrySetCommand(null).IsSuccess && native.LastButtonCommand == 0,
            "Button command association should be clearable");

        command.Dispose();
        value.Dispose();
        button.Dispose();
        editor.Dispose();
        panel.Dispose();
        window.Dispose();
        runtime.Dispose();
    }

    private static void TestWindowCloseRequestBorrowDeferResolveAndCallbackLifetime()
    {
        var native = new FakeNativeApi
        {
            WindowOpenStatus = UiStatus.Unsupported,
            WindowCloseStatus = UiStatus.Unsupported,
        };
        UiRuntime runtime = UiRuntime.TryCreate(native).Value;
        UiWindow window = UiWindow.TryCreate(runtime).Value;
        var failures = new System.Collections.Generic.List<UiCallbackFailure>();
        UiCallbackHandlerRegistration handler = runtime.TrySetCallbackFailureHandler(failures.Add).Value;
        UiWindowCloseRequest? borrowed = null;
        UiDeferredWindowCloseRequest? deferred = null;
        bool throwFromCallback = false;
        UiResult<bool> declined = UiResult<bool>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
        UiSubscription subscription = window.TrySubscribeCloseRequest(request =>
        {
            borrowed = request;
            if (throwFromCallback) throw new InvalidOperationException("close handler failed");
            declined = request.TryResolve(UiWindowCloseDecision.Decline);
            deferred = request.TryDefer().Value;
        }).Value;

        Check(window.TryOpen().Error?.RawStatus == UiStatus.Unsupported &&
            window.TryCloseWindow().Error?.RawStatus == UiStatus.Unsupported,
            "Window peer unsupported status should be preserved by both operations");
        native.InvokeWindowCloseRequest(0xCAFE);
        Check(declined.IsSuccess && native.LastCloseDecision == (uint)UiWindowCloseDecision.Decline && deferred is not null,
            "Callback should resolve borrowed request and create an owned deferred request");
        Check(borrowed!.TryResolve(UiWindowCloseDecision.Accept).Error?.RawStatus == UiStatus.Closed,
            "Borrowed close request should expire when its callback returns");
        Check(deferred!.TryResolve(UiWindowCloseDecision.Accept).IsSuccess &&
            native.LastCloseDecision == (uint)UiWindowCloseDecision.Accept,
            "Deferred request should remain resolvable after callback return");

        throwFromCallback = true;
        native.InvokeWindowCloseRequest(0xCAFF);
        Check(failures.Count == 1 && failures[0].Kind == UiCallbackFailureKind.WindowCloseRequest,
            "Managed close callback exceptions should be routed to the typed callback handler");
        Check(subscription.TryCancel().IsSuccess && subscription.TryClose().IsSuccess &&
            native.WindowCloseCallbackDestroyCount == 1,
            "Cancel and handle release should preserve one callback-root destroy");

        deferred.Dispose();
        handler.Dispose();
        window.Dispose();
        runtime.Dispose();
    }

    private sealed class FakeNativeApi : INativeApi
    {
        private readonly ConcurrentDictionary<ulong, int> _releaseFailures = new();
        private readonly ConcurrentDictionary<ulong, int> _releaseCounts = new();
        private readonly ConcurrentDictionary<ulong, int> _successfulReleases = new();
        private readonly ConcurrentDictionary<ulong, byte> _boolProperties = new();
        private readonly ConcurrentDictionary<ulong, long> _i64Properties = new();
        private readonly ConcurrentDictionary<ulong, long> _f64Properties = new();
        private readonly ConcurrentDictionary<ulong, ulong> _decimalProperties = new();
        private readonly ConcurrentDictionary<ulong, byte[]> _utf8Properties = new();
        private readonly ConcurrentDictionary<ulong, FakeSubscription> _subscriptions = new();
        private readonly ConcurrentDictionary<ulong, byte> _cancelledSubscriptions = new();
        private readonly ConcurrentDictionary<long, FakePost> _posts = new();
        private ulong _nextProperty = 300;
        private ulong _nextSubscription = 900;
        private ulong _nextBinding = 1200;
        private ulong _nextRetained = 1500;
        private long _nextPost;
        private IntPtr _commandStart;
        private IntPtr _commandStartUserdata;
        private IntPtr _commandStartDestroy;
        private bool _commandRootDestroyed;

        internal ulong RuntimeHandle { get; init; } = 101;
        internal ulong RequestHandle { get; init; } = 202;
        internal uint CreateStatus { get; init; }
        internal ulong ErrorHandle { get; init; }
        internal uint ErrorStatusValue { get; set; } = UiStatus.InternalError;
        internal uint ErrorDomainValue { get; set; } = 1;
        internal uint ErrorCodeValue { get; set; } = 1;
        internal byte[] ErrorMessageValue { get; set; } = [];
        internal byte IsUiThread { get; init; } = 1;
        internal uint RequestStatus { get; init; } = UiStatus.Ok;
        internal uint RequestState { get; init; } = (uint)ShutdownRequestState.Completed;
        internal uint RequestReason { get; init; }
        internal uint PropertyCreateStatus { get; set; } = UiStatus.Ok;
        internal uint PropertyGetStatus { get; set; } = UiStatus.Ok;
        internal uint PropertySetStatus { get; set; } = UiStatus.Ok;
        internal uint PropertySubscribeStatus { get; set; } = UiStatus.Ok;
        internal uint SubscriptionCancelStatus { get; set; } = UiStatus.Ok;
        internal uint BindingCreateStatus { get; set; } = UiStatus.Ok;
        internal uint BindingCancelStatus { get; set; } = UiStatus.Ok;
        internal uint BindingErrorSnapshotStatus { get; set; } = UiStatus.Ok;
        internal uint BindingSubscribeErrorStatus { get; set; } = UiStatus.Ok;
        internal uint CommandCreateStatus { get; set; } = UiStatus.Ok;
        internal uint CommandExecuteStatus { get; set; } = UiStatus.Ok;
        internal uint CommandCloseStatus { get; set; } = UiStatus.Ok;
        internal uint WindowOpenStatus { get; set; } = UiStatus.Ok;
        internal uint WindowCloseStatus { get; set; } = UiStatus.Ok;
        internal ulong LastControlParent { get; private set; }
        internal ulong LastControlChild { get; private set; }
        internal ulong LastControlIndex { get; private set; }
        internal uint LastOrientation { get; private set; }
        internal uint LastAlignment { get; private set; }
        internal ulong LastButtonCommand { get; private set; }
        internal int FocusRequestCount { get; private set; }
        internal int StackPanelSetOrientationCount { get; private set; }
        internal int ContainerInsertCount { get; private set; }
        internal uint LastCloseDecision { get; private set; }
        internal int WindowCloseCallbackDestroyCount { get; private set; }
        internal ulong LastControlHandle { get; private set; }
        private ulong _nextControl = 2000;
        private const ulong CloseSubscriptionToken = 2200;
        private IntPtr _windowCloseCallback;
        private IntPtr _windowCloseUserdata;
        private IntPtr _windowCloseDestroy;
        internal uint RuntimePostStatus { get; set; } = UiStatus.Ok;
        internal ulong CommandHandle { get; set; } = 1301;
        internal ulong CommandCancelToken { get; set; } = 1302;
        internal ulong CommandOperationErrorToken { get; set; }
        internal ulong CommandFailureToken { get; set; }
        internal ulong RuntimePostErrorToken { get; set; }
        internal ulong BindingErrorToken { get; set; }
        internal ulong BindingRejectedPropertyToken { get; set; }
        internal uint BindingDirectionValue { get; set; } = (uint)UiBindingDirection.SourceToTarget;
        internal ulong BindingOperationErrorToken { get; set; }
        internal ulong PropertyErrorHandle { get; set; }
        internal bool GrowUtf8OnCopy { get; set; }
        internal byte[] Utf8GrowthSuffix { get; set; } = [];
        internal byte[]? Utf8GetterOverride { get; set; }
        internal bool ShortUtf8BufferWasUntouched { get; private set; }
        internal byte? MalformedBoolValue { get; set; }
        internal bool BlockI64Get { get; init; }
        internal ManualResetEventSlim I64GetEntered { get; } = new(false);
        internal ManualResetEventSlim AllowI64Get { get; } = new(false);
        internal byte LastCreatedBool { get; private set; }
        internal byte LastSetBool { get; private set; }
        internal long LastCreatedI64 { get; private set; }
        internal long LastSetI64 { get; private set; }
        internal long LastCreatedF64Bits { get; private set; }
        internal long LastSetF64Bits { get; private set; }
        internal ulong LastCreatedDecimalBits { get; private set; }
        internal ulong LastSetDecimalBits { get; private set; }
        internal byte[] LastCreatedUtf8 { get; private set; } = [];
        internal byte[] LastSetUtf8 { get; private set; } = [];
        internal ulong BoolPropertyHandle { get; private set; }
        internal ulong I64PropertyHandle { get; private set; }
        internal ulong F64PropertyHandle { get; private set; }
        internal ulong DecimalPropertyHandle { get; private set; }
        internal ulong Utf8PropertyHandle { get; private set; }
        internal int PropertyCreateCount { get; private set; }
        internal int PropertySetCount { get; private set; }
        internal int SubscriptionCount { get; private set; }
        internal ulong LastSubscription { get; private set; }
        internal ulong LastRuntime { get; private set; }
        internal ulong LastRequest { get; private set; }
        internal ulong LastBindingSource { get; private set; }
        internal ulong LastBindingTarget { get; private set; }
        internal uint LastBindingMode { get; private set; }
        internal ulong LastBinding { get; private set; }
        internal int BindingCreateCount { get; private set; }
        internal int CommandCreateCount { get; private set; }
        internal int CommandStartCount { get; private set; }
        internal int CommandDestroyCount { get; private set; }
        internal int CommandCompletionCount { get; private set; }
        internal ulong LastCommand { get; private set; }
        internal ulong LastCommandEligibility { get; private set; }
        internal ulong LastOperationId { get; private set; }
        internal uint LastCompletionKind { get; private set; }
        internal uint LastStartStatus { get; private set; }
        internal ulong LastRetainedToken { get; private set; }
        internal uint LastFailureDomain { get; private set; }
        internal uint LastFailureCode { get; private set; }
        internal byte[] LastFailureMessage { get; private set; } = [];
        internal ulong LastPostId => unchecked((ulong)Volatile.Read(ref _nextPost));

        public uint WindowCreate(ulong runtime, out ulong window, out ulong error) => CreateControl(runtime, out window, out error);
        public uint StackPanelCreate(ulong runtime, out ulong panel, out ulong error) => CreateControl(runtime, out panel, out error);

        public uint TextCreate(ulong runtime, byte[] initial, out ulong text, out ulong error)
        {
            LastCreatedUtf8 = initial.ToArray();
            return CreateControl(runtime, out text, out error);
        }

        public uint TextBoxCreate(ulong runtime, byte[] initial, out ulong textBox, out ulong error)
        {
            LastCreatedUtf8 = initial.ToArray();
            return CreateControl(runtime, out textBox, out error);
        }

        public uint ButtonCreate(ulong runtime, byte[] initial, out ulong button, out ulong error)
        {
            LastCreatedUtf8 = initial.ToArray();
            return CreateControl(runtime, out button, out error);
        }

        private uint CreateControl(ulong runtime, out ulong control, out ulong error)
        {
            LastRuntime = runtime;
            control = ++_nextControl;
            LastControlHandle = control;
            error = 0;
            return UiStatus.Ok;
        }

        public uint WidgetIsEnabledProperty(ulong widget, out ulong property) => CreateControlProperty(out property);
        public uint WidgetAccessibleNameProperty(ulong widget, out ulong property) => CreateControlProperty(out property);
        public uint WindowTitleProperty(ulong window, out ulong property) => CreateControlProperty(out property, utf8: true);
        public uint WindowOpen(ulong window, out ulong error) { error = 0; return WindowOpenStatus; }
        public uint WindowClose(ulong window, out ulong error) { error = 0; return WindowCloseStatus; }
        public uint WindowCloseRequestSubscribe(ulong window, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong subscription, out ulong error)
        {
            _windowCloseCallback = callback;
            _windowCloseUserdata = userdata;
            _windowCloseDestroy = destroy;
            subscription = CloseSubscriptionToken;
            error = 0;
            return UiStatus.Ok;
        }
        public uint WindowCloseRequestDefer(ulong borrowedRequest, out ulong ownedRequest, out ulong error)
        {
            ownedRequest = 2201;
            error = 0;
            return UiStatus.Ok;
        }
        public uint WindowCloseRequestResolve(ulong request, uint decision, out ulong error)
        {
            LastCloseDecision = decision;
            error = 0;
            return UiStatus.Ok;
        }

        internal unsafe void InvokeWindowCloseRequest(ulong borrowedRequest)
        {
            ((delegate* unmanaged[Cdecl]<IntPtr, ulong, void>)_windowCloseCallback)(
                _windowCloseUserdata,
                borrowedRequest);
        }
        public uint TextContentProperty(ulong text, out ulong property) => CreateControlProperty(out property, utf8: true);
        public uint TextBoxValueProperty(ulong textBox, out ulong property) => CreateControlProperty(out property, utf8: true);
        public uint ButtonLabelProperty(ulong button, out ulong property) => CreateControlProperty(out property, utf8: true);

        private uint CreateControlProperty(out ulong property, bool utf8 = false)
        {
            property = ++_nextProperty;
            if (utf8) _utf8Properties[property] = [];
            else _boolProperties[property] = 1;
            return UiStatus.Ok;
        }

        public uint WidgetRequestFocus(ulong widget, out ulong error) { FocusRequestCount++; error = 0; return UiStatus.Ok; }
        public uint WindowContent(ulong window, out ulong child, out ulong error) { child = 0; error = 0; return UiStatus.Ok; }
        public uint WindowSetContent(ulong window, ulong child, out ulong error) { LastControlParent = window; LastControlChild = child; error = 0; return UiStatus.Ok; }
        public uint WindowClearContent(ulong window, out ulong error) { LastControlParent = window; error = 0; return UiStatus.Ok; }
        public uint ContainerInsert(ulong container, ulong child, ulong index, out ulong error) { ContainerInsertCount++; LastControlParent = container; LastControlChild = child; LastControlIndex = index; error = 0; return UiStatus.Ok; }
        public uint ContainerRemove(ulong container, ulong child, out ulong error) { LastControlParent = container; LastControlChild = child; error = 0; return UiStatus.Ok; }
        public uint ContainerMove(ulong container, ulong fromIndex, ulong toIndex, out ulong error) { LastControlParent = container; LastControlIndex = toIndex; error = 0; return UiStatus.Ok; }
        public uint ContainerChildCount(ulong container, out ulong count) { count = 0; return UiStatus.Ok; }
        public uint ContainerChildAt(ulong container, ulong index, out ulong child, out ulong error) { child = 0; error = 0; return UiStatus.InvalidArgument; }
        public uint StackPanelOrientation(ulong panel, out uint orientation) { orientation = LastOrientation == 0 ? 1u : LastOrientation; return UiStatus.Ok; }
        public uint StackPanelSetOrientation(ulong panel, uint orientation, out ulong error) { StackPanelSetOrientationCount++; LastOrientation = orientation; error = 0; return UiStatus.Ok; }
        public uint StackPanelSetChildAlign(ulong panel, ulong child, uint alignment, out ulong error) { LastControlParent = panel; LastControlChild = child; LastAlignment = alignment; error = 0; return UiStatus.Ok; }
        public uint ButtonSetCommand(ulong button, ulong command, out ulong error) { LastControlParent = button; LastButtonCommand = command; error = 0; return UiStatus.Ok; }
        public uint ButtonCommand(ulong button, out ulong command, out ulong error) { command = LastButtonCommand == 0 ? 0 : CommandHandle; error = 0; return UiStatus.Ok; }

        public uint RuntimeCreate(out ulong runtime, out ulong error)
        {
            runtime = CreateStatus == UiStatus.Ok ? RuntimeHandle : 0;
            error = ErrorHandle;
            return CreateStatus;
        }

        public uint RuntimeIsUiThread(ulong runtime, out byte isUiThread)
        {
            LastRuntime = runtime;
            isUiThread = IsUiThread;
            return UiStatus.Ok;
        }

        public uint RuntimeSetUiLoopBridge(ulong runtime, IntPtr wake, IntPtr userdata, IntPtr destroy, out ulong error)
        {
            error = 0;
            return UiStatus.Ok;
        }

        public uint RuntimeDispatchTurn(ulong runtime) => UiStatus.Ok;

        public uint RuntimePost(ulong runtime, IntPtr callback, IntPtr userdata, IntPtr destroy, out ulong error)
        {
            error = RuntimePostErrorToken;
            if (RuntimePostStatus == UiStatus.Ok)
            {
                long id = Interlocked.Increment(ref _nextPost);
                _posts[id] = new FakePost(callback, userdata, destroy);
            }

            return RuntimePostStatus;
        }

        public uint CommandCreateAsync(
            ulong runtime, ulong eligibility, IntPtr start, IntPtr userdata, IntPtr destroy,
            out ulong command, out ulong error)
        {
            CommandCreateCount++;
            LastRuntime = runtime;
            LastCommandEligibility = eligibility;
            error = CommandOperationErrorToken;
            command = CommandCreateStatus == UiStatus.Ok ? CommandHandle : 0;
            if (command != 0)
            {
                _commandStart = start;
                _commandStartUserdata = userdata;
                _commandStartDestroy = destroy;
                _commandRootDestroyed = false;
            }

            return CommandCreateStatus;
        }

        public unsafe uint CommandExecute(ulong command, out ulong operationId, out ulong error)
        {
            LastCommand = command;
            operationId = CommandExecuteStatus == UiStatus.Ok ? LastOperationId + 1 : 0;
            error = CommandOperationErrorToken;
            if (operationId == 0)
            {
                return CommandExecuteStatus == UiStatus.Ok ? UiStatus.InternalError : CommandExecuteStatus;
            }

            LastOperationId = operationId;
            CommandStartCount++;
            LastStartStatus = ((delegate* unmanaged[Cdecl]<IntPtr, ulong, ulong, ulong, uint>)_commandStart)(
                _commandStartUserdata,
                command,
                operationId,
                CommandCancelToken);
            return UiStatus.Ok;
        }

        public uint CommandCancel(ulong command, ulong operationId, out ulong error)
        {
            LastCommand = command;
            LastOperationId = operationId;
            error = CommandOperationErrorToken;
            return UiStatus.Ok;
        }

        public uint CommandClose(ulong command, out ulong error)
        {
            LastCommand = command;
            error = CommandOperationErrorToken;
            if (CommandCloseStatus == UiStatus.Ok)
            {
                DestroyCommandRoot();
            }

            return CommandCloseStatus;
        }

        public uint CommandStateProperty(ulong command, out ulong property)
        {
            LastCommand = command;
            property = ++_nextProperty;
            _i64Properties[property] = (long)UiCommandState.Idle;
            return UiStatus.Ok;
        }

        public uint CommandCanExecuteProperty(ulong command, out ulong property)
        {
            LastCommand = command;
            property = ++_nextProperty;
            _boolProperties[property] = 1;
            return UiStatus.Ok;
        }

        public uint CommandFailureSnapshot(ulong command, out ulong errorSnapshot)
        {
            LastCommand = command;
            errorSnapshot = CommandFailureToken;
            return UiStatus.Ok;
        }

        public uint CommandCancelTokenRequested(ulong token, out byte requested)
        {
            requested = 0;
            return token == 0 ? UiStatus.InvalidHandle : UiStatus.Ok;
        }

        public uint CommandCompleteSuccess(ulong command, ulong operationId)
        {
            RecordCompletion(command, operationId, 1);
            return UiStatus.Ok;
        }

        public unsafe uint CommandCompleteFailureUtf8(
            ulong command, ulong operationId, uint domain, uint code, byte* message, ulong length)
        {
            LastFailureDomain = domain;
            LastFailureCode = code;
            LastFailureMessage = length == 0 ? [] : new ReadOnlySpan<byte>(message, checked((int)length)).ToArray();
            RecordCompletion(command, operationId, 2);
            return UiStatus.Ok;
        }

        public uint CommandCompleteCancelled(ulong command, ulong operationId)
        {
            RecordCompletion(command, operationId, 3);
            return UiStatus.Ok;
        }

        public uint HandleRetain(ulong borrowed, out ulong owned)
        {
            owned = borrowed == 0 ? 0 : ++_nextRetained;
            LastRetainedToken = owned;
            return borrowed == 0 ? UiStatus.InvalidHandle : UiStatus.Ok;
        }

        public uint PropertyCreateBool(ulong runtime, byte initial, out ulong property, out ulong error)
        {
            PropertyCreateCount++;
            LastCreatedBool = initial;
            property = PropertyCreateStatus == UiStatus.Ok ? ++_nextProperty : 0;
            error = PropertyErrorHandle;
            if (property != 0)
            {
                BoolPropertyHandle = property;
                _boolProperties[property] = initial;
            }

            return PropertyCreateStatus;
        }

        public uint PropertyCreateI64(ulong runtime, long initial, out ulong property, out ulong error)
        {
            PropertyCreateCount++;
            LastCreatedI64 = initial;
            property = PropertyCreateStatus == UiStatus.Ok ? ++_nextProperty : 0;
            error = PropertyErrorHandle;
            if (property != 0)
            {
                I64PropertyHandle = property;
                _i64Properties[property] = initial;
            }

            return PropertyCreateStatus;
        }

        public uint PropertyCreateF64(ulong runtime, double initial, out ulong property, out ulong error)
        {
            PropertyCreateCount++;
            LastCreatedF64Bits = BitConverter.DoubleToInt64Bits(initial);
            property = PropertyCreateStatus == UiStatus.Ok ? ++_nextProperty : 0;
            error = PropertyErrorHandle;
            if (property != 0)
            {
                F64PropertyHandle = property;
                _f64Properties[property] = LastCreatedF64Bits;
            }

            return PropertyCreateStatus;
        }

        public uint PropertyCreateDecimal(ulong runtime, ulong initial, out ulong property, out ulong error)
        {
            PropertyCreateCount++;
            LastCreatedDecimalBits = initial;
            property = PropertyCreateStatus == UiStatus.Ok ? ++_nextProperty : 0;
            error = PropertyErrorHandle;
            if (property != 0)
            {
                DecimalPropertyHandle = property;
                _decimalProperties[property] = initial;
            }

            return PropertyCreateStatus;
        }

        public uint PropertyCreateUtf8(ulong runtime, byte[] initial, out ulong property, out ulong error)
        {
            PropertyCreateCount++;
            LastCreatedUtf8 = initial.ToArray();
            property = PropertyCreateStatus == UiStatus.Ok ? ++_nextProperty : 0;
            error = PropertyErrorHandle;
            if (property != 0)
            {
                Utf8PropertyHandle = property;
                _utf8Properties[property] = LastCreatedUtf8;
            }

            return PropertyCreateStatus;
        }

        public uint PropertyGetBool(ulong property, out byte value)
        {
            value = MalformedBoolValue ?? (_boolProperties.TryGetValue(property, out byte current) ? current : (byte)0);
            return PropertyGetStatus;
        }

        public uint PropertyGetI64(ulong property, out long value)
        {
            I64GetEntered.Set();
            if (BlockI64Get)
            {
                AllowI64Get.Wait(TimeSpan.FromSeconds(5));
            }

            _i64Properties.TryGetValue(property, out value);
            return PropertyGetStatus;
        }

        public uint PropertyGetF64(ulong property, out double value)
        {
            _f64Properties.TryGetValue(property, out long bits);
            value = BitConverter.Int64BitsToDouble(bits);
            return PropertyGetStatus;
        }

        public uint PropertyGetDecimal(ulong property, out ulong value)
        {
            _decimalProperties.TryGetValue(property, out value);
            return PropertyGetStatus;
        }

        public uint PropertyGetUtf8(ulong property, byte[] buffer, ulong capacity, out ulong length)
        {
            length = 0;
            if (PropertyGetStatus != UiStatus.Ok)
            {
                return PropertyGetStatus;
            }

            if (!_utf8Properties.TryGetValue(property, out byte[]? stored))
            {
                return UiStatus.InvalidHandle;
            }

            byte[] value = Utf8GetterOverride ?? stored;
            if (capacity == 0)
            {
                length = (ulong)value.Length;
                return value.Length == 0 ? UiStatus.Ok : UiStatus.BufferTooSmall;
            }

            if (GrowUtf8OnCopy)
            {
                GrowUtf8OnCopy = false;
                value = value.Concat(Utf8GrowthSuffix).ToArray();
                _utf8Properties[property] = value;
                if (buffer.Length > 0 && buffer.All(static item => item == 0))
                {
                    ShortUtf8BufferWasUntouched = true;
                }

                length = (ulong)value.Length;
                return UiStatus.BufferTooSmall;
            }

            if ((ulong)value.Length > capacity)
            {
                length = (ulong)value.Length;
                return UiStatus.BufferTooSmall;
            }

            value.CopyTo(buffer, 0);
            length = (ulong)value.Length;
            return UiStatus.Ok;
        }

        public uint PropertySetBool(ulong property, byte value, out ulong error)
        {
            LastSetBool = value;
            error = PropertyErrorHandle;
            if (PropertySetStatus == UiStatus.Ok)
            {
                _boolProperties[property] = value;
            }

            return PropertySetStatus;
        }

        public uint PropertySetI64(ulong property, long value, out ulong error)
        {
            LastSetI64 = value;
            error = PropertyErrorHandle;
            if (PropertySetStatus == UiStatus.Ok)
            {
                _i64Properties[property] = value;
            }

            return PropertySetStatus;
        }

        public uint PropertySetF64(ulong property, double value, out ulong error)
        {
            LastSetF64Bits = BitConverter.DoubleToInt64Bits(value);
            error = PropertyErrorHandle;
            if (PropertySetStatus == UiStatus.Ok)
            {
                _f64Properties[property] = LastSetF64Bits;
            }

            return PropertySetStatus;
        }

        public uint PropertySetUtf8(ulong property, byte[] value, out ulong error)
        {
            PropertySetCount++;
            LastSetUtf8 = value.ToArray();
            error = PropertyErrorHandle;
            if (PropertySetStatus == UiStatus.Ok)
            {
                _utf8Properties[property] = LastSetUtf8;
            }

            return PropertySetStatus;
        }

        public uint RuntimeRequestShutdown(ulong runtime, uint mode, out ulong request, out ulong error)
        {
            LastRuntime = runtime;
            LastRequest = RequestHandle;
            request = RequestHandle;
            error = 0;
            return RequestStatus;
        }

        public uint ShutdownRequestQuery(ulong request, out uint state, out uint reason, out ulong error)
        {
            LastRequest = request;
            state = RequestState;
            reason = RequestReason;
            error = 0;
            return UiStatus.Ok;
        }

        public uint PropertySetDecimal(ulong property, ulong value, out ulong error)
        {
            LastSetDecimalBits = value;
            error = PropertyErrorHandle;
            if (PropertySetStatus == UiStatus.Ok)
            {
                _decimalProperties[property] = value;
            }

            return PropertySetStatus;
        }

        public uint PropertySubscribe(
            ulong property, IntPtr callback, IntPtr userdata, IntPtr destroy,
            out ulong subscription, out ulong error)
        {
            SubscriptionCount++;
            subscription = PropertySubscribeStatus == UiStatus.Ok ? ++_nextSubscription : 0;
            LastSubscription = subscription;
            error = PropertyErrorHandle;
            if (subscription != 0)
            {
                _subscriptions[subscription] = new FakeSubscription(callback, userdata, destroy);
            }

            return PropertySubscribeStatus;
        }

        public uint SubscriptionCancel(ulong subscription, out ulong error)
        {
            error = PropertyErrorHandle;
            if (SubscriptionCancelStatus == UiStatus.Ok)
            {
                _cancelledSubscriptions[subscription] = 1;
            }

            return SubscriptionCancelStatus;
        }

        public uint BindingCreate(ulong source, ulong target, uint mode, out ulong binding, out ulong error)
        {
            BindingCreateCount++;
            LastBindingSource = source;
            LastBindingTarget = target;
            LastBindingMode = mode;
            binding = BindingCreateStatus == UiStatus.Ok && mode != (uint)UiBindingMode.OneTime
                ? ++_nextBinding
                : 0;
            LastBinding = binding;
            error = BindingOperationErrorToken;
            return BindingCreateStatus;
        }

        public uint BindingCancel(ulong binding, out ulong error)
        {
            LastBinding = binding;
            error = BindingOperationErrorToken;
            return BindingCancelStatus;
        }

        public uint BindingErrorSnapshot(ulong binding, out ulong errorSnapshot, out ulong rejectedProperty, out uint direction)
        {
            LastBinding = binding;
            errorSnapshot = BindingErrorToken;
            rejectedProperty = BindingRejectedPropertyToken;
            direction = BindingErrorToken == 0 && BindingRejectedPropertyToken == 0 ? 0 : BindingDirectionValue;
            return BindingErrorSnapshotStatus;
        }

        public uint BindingSubscribeError(
            ulong binding, IntPtr callback, IntPtr userdata, IntPtr destroy,
            out ulong subscription, out ulong error)
        {
            LastBinding = binding;
            SubscriptionCount++;
            subscription = BindingSubscribeErrorStatus == UiStatus.Ok ? ++_nextSubscription : 0;
            LastSubscription = subscription;
            error = BindingOperationErrorToken;
            if (subscription != 0)
            {
                _subscriptions[subscription] = new FakeSubscription(callback, userdata, destroy);
            }

            return BindingSubscribeErrorStatus;
        }

        internal unsafe void InvokeSubscription(ulong subscription)
        {
            if (_subscriptions.TryGetValue(subscription, out FakeSubscription? state) &&
                !_cancelledSubscriptions.ContainsKey(subscription))
            {
                ((delegate* unmanaged[Cdecl]<IntPtr, void>)state.Callback)(state.Userdata);
            }
        }

        internal unsafe void DestroySubscription(ulong subscription)
        {
            if (_subscriptions.TryRemove(subscription, out FakeSubscription? state))
            {
                ((delegate* unmanaged[Cdecl]<IntPtr, void>)state.Destroy)(state.Userdata);
            }
        }

        internal unsafe void InvokeLastPost()
        {
            long id = Interlocked.Read(ref _nextPost);
            if (_posts.TryRemove(id, out FakePost post))
            {
                ((delegate* unmanaged[Cdecl]<IntPtr, void>)post.Callback)(post.Userdata);
                ((delegate* unmanaged[Cdecl]<IntPtr, void>)post.Destroy)(post.Userdata);
            }
        }

        private unsafe void DestroyCommandRoot()
        {
            if (_commandRootDestroyed || _commandStartDestroy == IntPtr.Zero)
            {
                return;
            }

            _commandRootDestroyed = true;
            CommandDestroyCount++;
            ((delegate* unmanaged[Cdecl]<IntPtr, void>)_commandStartDestroy)(_commandStartUserdata);
        }

        private void RecordCompletion(ulong command, ulong operationId, uint kind)
        {
            LastCommand = command;
            LastOperationId = operationId;
            LastCompletionKind = kind;
            CommandCompletionCount++;
        }

        public uint RuntimeShutdown(ulong runtime, out ulong error)
        {
            LastRuntime = runtime;
            error = 0;
            return UiStatus.Ok;
        }

        public unsafe uint HandleRelease(ulong owned)
        {
            _releaseCounts.AddOrUpdate(owned, 1, static (_, count) => count + 1);
            while (true)
            {
                int failures = Read(_releaseFailures, owned);
                if (failures <= 0)
                {
                    _successfulReleases.AddOrUpdate(owned, 1, static (_, count) => count + 1);
                    if (owned == CloseSubscriptionToken && _windowCloseDestroy != IntPtr.Zero)
                    {
                        IntPtr destroy = _windowCloseDestroy;
                        _windowCloseDestroy = IntPtr.Zero;
                        WindowCloseCallbackDestroyCount++;
                        ((delegate* unmanaged[Cdecl]<IntPtr, void>)destroy)(_windowCloseUserdata);
                    }

                    return UiStatus.Ok;
                }

                if (_releaseFailures.TryUpdate(owned, failures - 1, failures))
                {
                    return UiStatus.InternalError;
                }
            }
        }

        public uint ErrorStatus(ulong error, out uint status)
        {
            status = ErrorStatusValue;
            return UiStatus.Ok;
        }

        public uint ErrorCode(ulong error, out uint domain, out uint code)
        {
            domain = ErrorDomainValue;
            code = ErrorCodeValue;
            return UiStatus.Ok;
        }

        public unsafe uint ErrorMessageUtf8(ulong error, byte* buffer, ulong capacity, out ulong length)
        {
            length = (ulong)ErrorMessageValue.Length;
            if (capacity == 0)
            {
                return UiStatus.Ok;
            }

            if (capacity < length)
            {
                return UiStatus.BufferTooSmall;
            }

            for (int index = 0; index < ErrorMessageValue.Length; index++)
            {
                buffer[index] = ErrorMessageValue[index];
            }

            return UiStatus.Ok;
        }

        public uint GetVersion(out uint major, out uint minor, out uint patch)
        {
            major = 1;
            minor = 0;
            patch = 0;
            return UiStatus.Ok;
        }

        internal void SetReleaseFailures(ulong token, int count) => _releaseFailures[token] = count;

        internal int ReleaseCount(ulong token) => Read(_releaseCounts, token);

        internal int SuccessfulReleaseCount(ulong token) => Read(_successfulReleases, token);

        private static int Read(ConcurrentDictionary<ulong, int> values, ulong token) =>
            values.TryGetValue(token, out int value) ? value : 0;

        private sealed record FakeSubscription(IntPtr Callback, IntPtr Userdata, IntPtr Destroy);

        private readonly record struct FakePost(IntPtr Callback, IntPtr Userdata, IntPtr Destroy);
    }
}
