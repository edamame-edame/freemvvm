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

    private static void Check(bool condition, string message)
    {
        if (!condition)
        {
            throw new InvalidOperationException(message);
        }
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
        private ulong _nextProperty = 300;
        private ulong _nextSubscription = 900;

        internal ulong RuntimeHandle { get; init; } = 101;
        internal ulong RequestHandle { get; init; } = 202;
        internal uint CreateStatus { get; init; }
        internal ulong ErrorHandle { get; init; }
        internal uint ErrorStatusValue { get; set; } = UiStatus.InternalError;
        internal uint ErrorDomainValue { get; set; } = 1;
        internal uint ErrorCodeValue { get; set; } = 1;
        internal byte[] ErrorMessageValue { get; init; } = [];
        internal byte IsUiThread { get; init; } = 1;
        internal uint RequestStatus { get; init; } = UiStatus.Ok;
        internal uint RequestState { get; init; } = (uint)ShutdownRequestState.Completed;
        internal uint RequestReason { get; init; }
        internal uint PropertyCreateStatus { get; set; } = UiStatus.Ok;
        internal uint PropertyGetStatus { get; set; } = UiStatus.Ok;
        internal uint PropertySetStatus { get; set; } = UiStatus.Ok;
        internal uint PropertySubscribeStatus { get; set; } = UiStatus.Ok;
        internal uint SubscriptionCancelStatus { get; set; } = UiStatus.Ok;
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

        public uint RuntimeShutdown(ulong runtime, out ulong error)
        {
            LastRuntime = runtime;
            error = 0;
            return UiStatus.Ok;
        }

        public uint HandleRelease(ulong owned)
        {
            _releaseCounts.AddOrUpdate(owned, 1, static (_, count) => count + 1);
            while (true)
            {
                int failures = Read(_releaseFailures, owned);
                if (failures <= 0)
                {
                    _successfulReleases.AddOrUpdate(owned, 1, static (_, count) => count + 1);
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
    }
}
