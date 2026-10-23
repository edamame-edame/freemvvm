using System;
using System.Threading;

namespace FreeMvvm;

public enum UiCloseDisposition
{
    Released,
    Pending,
}

internal sealed class TokenOwner
{
    private readonly object _sync = new();
    private readonly INativeApi _api;
    private readonly CleanupTicket _ticket;
    private ulong _token;
    private int _activeCalls;
    private OwnerState _state;
    private bool _disposeRequested;

    internal TokenOwner(INativeApi api)
    {
        _api = api;
        CleanupManager.EnsureStarted();
        _ticket = new CleanupTicket(api, this);
        _state = OwnerState.Open;
    }

    internal INativeApi Api => _api;

    ~TokenOwner() => Dispose();

    internal void Adopt(ulong token)
    {
        if (token == 0)
        {
            throw new ArgumentOutOfRangeException(nameof(token));
        }

        lock (_sync)
        {
            if (_token != 0 || _state != OwnerState.Open)
            {
                throw new InvalidOperationException("This owner already has a token.");
            }

            _token = token;
        }
    }

    internal UiResult<CallLease> AcquireLease()
    {
        lock (_sync)
        {
            if (_state != OwnerState.Open || _token == 0)
            {
                return UiResult<CallLease>.Failure(new UiErrorSnapshot(UiStatus.Closed));
            }

            var lease = new CallLease(this, _token);
            checked
            {
                _activeCalls++;
            }

            return UiResult<CallLease>.Success(lease);
        }
    }

    internal UiResult<UiCloseDisposition> TryClose()
    {
        ulong token;
        lock (_sync)
        {
            switch (_state)
            {
                case OwnerState.Closed:
                    return UiResult<UiCloseDisposition>.Success(UiCloseDisposition.Released);
                case OwnerState.PendingCleanup:
                    return UiResult<UiCloseDisposition>.Success(UiCloseDisposition.Pending);
                case OwnerState.Closing:
                case OwnerState.Releasing:
                    return UiResult<UiCloseDisposition>.Success(UiCloseDisposition.Pending);
                case OwnerState.Open:
                    if (_activeCalls != 0)
                    {
                        _state = OwnerState.Closing;
                        return UiResult<UiCloseDisposition>.Success(UiCloseDisposition.Pending);
                    }

                    _state = OwnerState.Releasing;
                    token = _token;
                    break;
                default:
                    return UiResult<UiCloseDisposition>.Failure(new UiErrorSnapshot(UiStatus.InternalError));
            }
        }

        uint status = ReleaseNative(token);
        lock (_sync)
        {
            if (status == UiStatus.Ok)
            {
                _token = 0;
                _state = OwnerState.Closed;
                return UiResult<UiCloseDisposition>.Success(UiCloseDisposition.Released);
            }

            if (_disposeRequested)
            {
                TransferToCleanup(token);
                return UiResult<UiCloseDisposition>.Failure(new UiErrorSnapshot(status));
            }

            _state = OwnerState.Open;
            return UiResult<UiCloseDisposition>.Failure(new UiErrorSnapshot(status));
        }
    }

    internal void Dispose()
    {
        ulong token = 0;
        lock (_sync)
        {
            if (_state is OwnerState.Closed or OwnerState.PendingCleanup)
            {
                return;
            }

            _disposeRequested = true;
            if (_state == OwnerState.Releasing)
            {
                return;
            }

            if (_activeCalls != 0)
            {
                _state = OwnerState.Closing;
                return;
            }

            _state = OwnerState.Releasing;
            token = _token;
        }

        uint status = ReleaseNative(token);
        lock (_sync)
        {
            if (status == UiStatus.Ok)
            {
                _token = 0;
                _state = OwnerState.Closed;
            }
            else
            {
                TransferToCleanup(token);
            }
        }
    }

    internal ulong TokenForLease => _token;

    internal void ReleaseLease()
    {
        ulong token = 0;
        bool shouldRelease = false;
        lock (_sync)
        {
            if (_activeCalls <= 0)
            {
                return;
            }

            _activeCalls--;
            if (_activeCalls == 0 && _state == OwnerState.Closing)
            {
                _state = OwnerState.Releasing;
                token = _token;
                shouldRelease = true;
            }
        }

        if (!shouldRelease)
        {
            return;
        }

        uint status = ReleaseNative(token);
        lock (_sync)
        {
            if (status == UiStatus.Ok)
            {
                _token = 0;
                _state = OwnerState.Closed;
            }
            else
            {
                TransferToCleanup(token);
            }
        }
    }

    private uint ReleaseNative(ulong token)
    {
        try
        {
            return token == 0 ? UiStatus.Ok : _api.HandleRelease(token);
        }
        catch
        {
            return UiStatus.InternalError;
        }
    }

    private void TransferToCleanup(ulong token)
    {
        _token = 0;
        _state = OwnerState.PendingCleanup;
        _ticket.Adopt(token);
        CleanupManager.Enqueue(_ticket);
    }

    internal void CleanupTicketReleased()
    {
        lock (_sync)
        {
            if (_state == OwnerState.PendingCleanup)
            {
                _state = OwnerState.Closed;
            }
        }
    }

    private enum OwnerState
    {
        Open,
        Closing,
        Releasing,
        PendingCleanup,
        Closed,
    }
}

internal sealed class CallLease : IDisposable
{
    private TokenOwner? _owner;

    internal CallLease(TokenOwner owner, ulong token)
    {
        _owner = owner;
        Token = token;
    }

    internal ulong Token { get; }

    public void Dispose()
    {
        Interlocked.Exchange(ref _owner, null)?.ReleaseLease();
    }
}

internal sealed class CleanupTicket
{
    private readonly INativeApi _api;
    private readonly TokenOwner _owner;
    private ulong _token;

    internal CleanupTicket(INativeApi api, TokenOwner owner)
    {
        _api = api;
        _owner = owner;
    }

    internal CleanupTicket? Next { get; set; }

    internal void Adopt(ulong token) => _token = token;

    internal bool Retry()
    {
        uint status;
        try
        {
            status = _api.HandleRelease(_token);
        }
        catch
        {
            status = UiStatus.InternalError;
        }

        if (status != UiStatus.Ok)
        {
            return false;
        }

        _token = 0;
        _owner.CleanupTicketReleased();
        return true;
    }
}

internal static class CleanupManager
{
    private static readonly object Sync = new();
    private static readonly AutoResetEvent Signal = new(false);
    private static CleanupTicket? _head;
    private static CleanupTicket? _tail;

    static CleanupManager()
    {
        var worker = new Thread(Run) { IsBackground = true, Name = "FreeMvvm C# cleanup" };
        worker.Start();
    }

    internal static void EnsureStarted()
    {
        _ = Signal;
    }

    internal static void Enqueue(CleanupTicket ticket)
    {
        lock (Sync)
        {
            ticket.Next = null;
            if (_tail is null)
            {
                _head = ticket;
            }
            else
            {
                _tail.Next = ticket;
            }

            _tail = ticket;
        }

        Signal.Set();
    }

    private static void Run()
    {
        while (true)
        {
            Signal.WaitOne();
            CleanupTicket? ticket;
            while ((ticket = Dequeue()) is not null)
            {
                if (!ticket.Retry())
                {
                    Thread.Sleep(25);
                    Enqueue(ticket);
                }
            }
        }
    }

    private static CleanupTicket? Dequeue()
    {
        lock (Sync)
        {
            CleanupTicket? ticket = _head;
            if (ticket is null)
            {
                return null;
            }

            _head = ticket.Next;
            if (_head is null)
            {
                _tail = null;
            }

            ticket.Next = null;
            return ticket;
        }
    }
}
