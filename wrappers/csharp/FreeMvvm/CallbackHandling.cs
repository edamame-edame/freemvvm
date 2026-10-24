using System;
using System.Diagnostics;
using System.Threading;

namespace FreeMvvm;

public enum UiCallbackFailureKind
{
    PropertyNotification,
}

public sealed class UiCallbackFailure
{
    internal UiCallbackFailure(UiCallbackFailureKind kind, Exception exception)
    {
        Kind = kind;
        Exception = exception;
    }

    public UiCallbackFailureKind Kind { get; }

    public Exception Exception { get; }
}

public sealed class UiCallbackHandlerRegistration : IDisposable
{
    private CallbackHandlerState? _state;
    private CallbackHandlerRecord? _record;

    internal UiCallbackHandlerRegistration(CallbackHandlerState state, CallbackHandlerRecord record)
    {
        _state = state;
        _record = record;
    }

    public void Dispose()
    {
        CallbackHandlerState? state = Interlocked.Exchange(ref _state, null);
        CallbackHandlerRecord? record = Interlocked.Exchange(ref _record, null);
        if (state is not null && record is not null)
        {
            state.Remove(record);
        }
    }
}

internal sealed class CallbackHandlerRecord(Action<UiCallbackFailure> handler)
{
    internal Action<UiCallbackFailure> Handler { get; } = handler;
}

internal sealed class CallbackHandlerState
{
    private CallbackHandlerRecord? _current;

    internal static void ReportUnexpected(Exception? exception) => TraceFailure(exception);

    internal UiResult<UiCallbackHandlerRegistration> TrySet(Action<UiCallbackFailure> handler)
    {
        if (handler is null)
        {
            return UiResult<UiCallbackHandlerRegistration>.Failure(new UiErrorSnapshot(UiStatus.InvalidArgument));
        }

        try
        {
            var record = new CallbackHandlerRecord(handler);
            var registration = new UiCallbackHandlerRegistration(this, record);
            Interlocked.Exchange(ref _current, record);
            return UiResult<UiCallbackHandlerRegistration>.Success(registration);
        }
        catch (OutOfMemoryException)
        {
            return UiResult<UiCallbackHandlerRegistration>.Failure(new UiErrorSnapshot(UiStatus.OutOfMemory));
        }
    }

    internal void Remove(CallbackHandlerRecord record)
    {
        Interlocked.CompareExchange(ref _current, null, record);
    }

    internal void Report(Exception exception)
    {
        CallbackHandlerRecord? record = Volatile.Read(ref _current);
        if (record is null)
        {
            TraceFailure(exception);
            return;
        }

        try
        {
            record.Handler(new UiCallbackFailure(UiCallbackFailureKind.PropertyNotification, exception));
        }
        catch (Exception handlerException)
        {
            TraceFailure(handlerException);
        }
    }

    private static void TraceFailure(Exception? exception)
    {
        try
        {
            Trace.TraceError(
                exception is null
                    ? "A FreeMvvm callback or callback failure handler failed."
                    : $"A FreeMvvm callback or callback failure handler failed: {exception}");
        }
        catch
        {
            // Diagnostics must never escape an unmanaged callback boundary.
        }
    }
}

internal sealed class PropertyCallbackRoot(Action callback, CallbackHandlerState handlers)
{
    internal void Invoke()
    {
        try
        {
            callback();
        }
        catch (Exception exception)
        {
            handlers.Report(exception);
        }
    }
}
