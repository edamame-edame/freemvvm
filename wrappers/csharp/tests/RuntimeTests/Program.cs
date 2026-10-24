using System;
using System.Collections.Concurrent;
using System.Diagnostics;
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
        Run(TestUnsupportedPropertyTypeIsRejectedBeforeNativeCall);
        Run(TestPropertyFailuresPreserveStatusAndValue);
        Run(TestPropertyCreateAndGetterFailuresAndCloseRetry);
        Run(TestPropertyLeaseProtectsHandleDuringDispose);
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

        UiResult<UiProperty<string>> unsupported = UiProperty<string>.TryCreate(runtime, "not supported");
        Check(unsupported.IsFailure && unsupported.Error?.RawStatus == UiStatus.Unsupported,
            "unsupported managed property types should return UI_UNSUPPORTED");
        Check(native.PropertyCreateCount == 0, "unsupported type must be rejected before entering the ABI");
        Check(runtime.TryClose().Value == UiCloseDisposition.Released, "Runtime should remain usable");
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
        private ulong _nextProperty = 300;

        internal ulong RuntimeHandle { get; init; } = 101;
        internal ulong RequestHandle { get; init; } = 202;
        internal uint CreateStatus { get; init; }
        internal ulong ErrorHandle { get; init; }
        internal uint ErrorStatusValue { get; init; } = UiStatus.InternalError;
        internal uint ErrorDomainValue { get; init; } = 1;
        internal uint ErrorCodeValue { get; init; } = 1;
        internal byte[] ErrorMessageValue { get; init; } = [];
        internal byte IsUiThread { get; init; } = 1;
        internal uint RequestStatus { get; init; } = UiStatus.Ok;
        internal uint RequestState { get; init; } = (uint)ShutdownRequestState.Completed;
        internal uint RequestReason { get; init; }
        internal uint PropertyCreateStatus { get; set; } = UiStatus.Ok;
        internal uint PropertyGetStatus { get; set; } = UiStatus.Ok;
        internal uint PropertySetStatus { get; set; } = UiStatus.Ok;
        internal ulong PropertyErrorHandle { get; set; }
        internal byte? MalformedBoolValue { get; set; }
        internal bool BlockI64Get { get; init; }
        internal ManualResetEventSlim I64GetEntered { get; } = new(false);
        internal ManualResetEventSlim AllowI64Get { get; } = new(false);
        internal byte LastCreatedBool { get; private set; }
        internal byte LastSetBool { get; private set; }
        internal long LastCreatedI64 { get; private set; }
        internal long LastSetI64 { get; private set; }
        internal ulong BoolPropertyHandle { get; private set; }
        internal ulong I64PropertyHandle { get; private set; }
        internal int PropertyCreateCount { get; private set; }
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
    }
}
