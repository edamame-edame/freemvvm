using System;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Threading;
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
