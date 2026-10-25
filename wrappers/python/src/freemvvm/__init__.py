"""Safe Python facade for the FreeMvvm versioned C ABI."""

from __future__ import annotations

from contextlib import contextmanager
from dataclasses import dataclass
from enum import IntEnum
import queue
import threading
from typing import Iterator, final
import weakref

from . import _native


class MissingRuntimeError(ImportError):
    """The native FreeMvvm runtime could not be loaded."""


def _load_native_api() -> _native.NativeApi:
    """Private loader seam; consumers use Runtime.create()."""
    return _native.CtypesNativeApi.load()


@dataclass(frozen=True, slots=True)
class ErrorSnapshot:
    raw_status: int
    domain: int | None = None
    code: int | None = None
    message: str | None = None


class UiError(RuntimeError):
    """A native operation failure retaining its numeric ABI details."""

    def __init__(self, snapshot: ErrorSnapshot) -> None:
        self.snapshot = snapshot
        self.raw_status = snapshot.raw_status
        self.domain = snapshot.domain
        self.code = snapshot.code
        super().__init__(
            snapshot.message
            if snapshot.message is not None
            else f"FreeMvvm operation failed with status {snapshot.raw_status}"
        )


class ShutdownMode(IntEnum):
    ASYNC = _native.SHUTDOWN_ASYNC
    SYNC = _native.SHUTDOWN_SYNC


class ShutdownDisposition(IntEnum):
    COMPLETED = 0
    ACCEPTED_PENDING = 1


class ShutdownRequestState(IntEnum):
    PENDING = _native.SHUTDOWN_REQUEST_PENDING
    COMPLETED = _native.SHUTDOWN_REQUEST_COMPLETED
    HOST_UNAVAILABLE = _native.SHUTDOWN_REQUEST_HOST_UNAVAILABLE


@dataclass(frozen=True, slots=True)
class ShutdownSnapshot:
    raw_state: int
    reason: int

    @property
    def known_state(self) -> ShutdownRequestState | None:
        try:
            return ShutdownRequestState(self.raw_state)
        except ValueError:
            return None


class _OwnerCell:
    """Token owner with call leases and a preallocated finalizer ticket."""

    __slots__ = (
        "api",
        "token",
        "anchor",
        "condition",
        "active",
        "closing",
        "ticket",
    )

    def __init__(
        self,
        api: _native.NativeApi,
        token: int,
        anchor: _OwnerCell | None = None,
    ) -> None:
        self.api = api
        self.token = token
        self.anchor = anchor
        self.condition = threading.Condition(threading.RLock())
        self.active = 0
        self.closing = False
        self.ticket = _CleanupTicket(self)

    @contextmanager
    def lease(self) -> Iterator[int]:
        with self.condition:
            while self.closing:
                self.condition.wait()
            if self.token == 0:
                raise UiError(ErrorSnapshot(_native.CLOSED))
            self.active += 1
            token = self.token
        try:
            yield token
        finally:
            with self.condition:
                self.active -= 1
                if self.active == 0:
                    self.condition.notify_all()

    def close(self) -> None:
        with self.condition:
            while self.closing:
                self.condition.wait()
            if self.token == 0:
                return
            self.closing = True
            while self.active:
                self.condition.wait()
            token = self.token

        try:
            status = self.api.handle_release(token)
        finally:
            with self.condition:
                if "status" in locals() and status == _native.OK:
                    self._mark_released()
                self.closing = False
                self.condition.notify_all()
        if status != _native.OK:
            raise UiError(ErrorSnapshot(status))

    def _release_for_cleanup(self) -> bool:
        with self.condition:
            while self.closing:
                self.condition.wait()
            if self.token == 0:
                self._mark_released()
                return True
            self.closing = True
            while self.active:
                self.condition.wait()
            token = self.token

        try:
            status = self.api.handle_release(token)
        finally:
            with self.condition:
                if "status" in locals() and status == _native.OK:
                    self._mark_released()
                self.closing = False
                self.condition.notify_all()
        return status == _native.OK

    def _mark_released(self) -> None:
        self.token = 0
        self.anchor = None
        self.ticket.owner = None


class _CleanupTicket:
    __slots__ = ("owner", "api", "token", "anchor")

    def __init__(
        self,
        owner: _OwnerCell | None = None,
        api: _native.NativeApi | None = None,
        token: int = 0,
        anchor: _OwnerCell | None = None,
    ) -> None:
        self.owner = owner
        self.api = api
        self.token = token
        self.anchor = anchor

    def retry(self) -> bool:
        if self.owner is not None:
            return self.owner._release_for_cleanup()
        if self.token == 0 or self.api is None:
            return True
        if self.api.handle_release(self.token) == _native.OK:
            self.token = 0
            self.anchor = None
            return True
        return False


class _CleanupManager:
    """Finalizers hand off preallocated tickets through a non-waiting queue."""

    _queue: queue.SimpleQueue[_CleanupTicket] = queue.SimpleQueue()
    _drain_lock = threading.Lock()

    @classmethod
    def enqueue(cls, ticket: _CleanupTicket) -> None:
        cls._queue.put(ticket)

    @classmethod
    def retry_pending(cls) -> None:
        if not cls._drain_lock.acquire(blocking=False):
            return
        try:
            pending_count = cls._queue.qsize()
            for _ in range(pending_count):
                try:
                    ticket = cls._queue.get_nowait()
                except queue.Empty:
                    return
                if not ticket.retry():
                    cls._queue.put(ticket)
        finally:
            cls._drain_lock.release()


def _snapshot_error(
    api: _native.NativeApi,
    operation_status: int,
    error: int,
    release_ticket: _CleanupTicket,
) -> ErrorSnapshot:
    if not error:
        return ErrorSnapshot(operation_status)

    status_query, error_status = api.error_status(error)
    raw_status = (
        error_status if status_query == _native.OK else operation_status
    )
    code_query, domain, code = api.error_code(error)
    if code_query == _native.OK:
        domain_value: int | None = domain
        code_value: int | None = code
    else:
        domain_value = code_value = None

    message: str | None = None
    query_status, length = api.error_message_utf8(error, None)
    if (
        query_status in (_native.OK, _native.BUFFER_TOO_SMALL)
        and length <= 2**31 - 1
    ):
        if length == 0:
            message = ""
        else:
            try:
                buffer = bytearray(length)
            except MemoryError:
                buffer = None
            if buffer is not None:
                copy_status, copied = api.error_message_utf8(error, buffer)
                if copy_status == _native.OK and copied == length:
                    try:
                        message = buffer.decode("utf-8", errors="strict")
                    except UnicodeDecodeError:
                        message = None

    release_ticket.api = api
    release_ticket.token = error
    if api.handle_release(error) == _native.OK:
        release_ticket.token = 0
    else:
        _CleanupManager.enqueue(release_ticket)
    return ErrorSnapshot(raw_status, domain_value, code_value, message)


def _check_result(
    api: _native.NativeApi,
    status: int,
    error: int,
    release_ticket: _CleanupTicket,
    accepted: tuple[int, ...] = (_native.OK,),
) -> None:
    if status in accepted:
        if error:
            raise UiError(
                _snapshot_error(
                    api, _native.INTERNAL_ERROR, error, release_ticket
                )
            )
        return
    raise UiError(_snapshot_error(api, status, error, release_ticket))


def _dispose_unexpected(
    api: _native.NativeApi, token: int, release_ticket: _CleanupTicket
) -> None:
    if not token:
        return
    release_ticket.owner = None
    release_ticket.api = api
    release_ticket.token = token
    if api.handle_release(token) == _native.OK:
        release_ticket.token = 0
    else:
        _CleanupManager.enqueue(release_ticket)


@final
class Runtime:
    """Own a Runtime token; close does not shut down the Runtime."""

    __slots__ = ("_cell", "_finalizer", "__weakref__")

    def __init__(self) -> None:
        raise TypeError("Use Runtime.create() to create a Runtime")

    @classmethod
    def create(cls) -> Runtime:
        _CleanupManager.retry_pending()
        try:
            api = _load_native_api()
        except (OSError, AttributeError) as error:
            raise MissingRuntimeError(str(error)) from error
        return cls._create_with_api(api)

    @classmethod
    def _create_with_api(cls, api: _native.NativeApi) -> Runtime:
        version_status, major, _, _ = api.get_version()
        if version_status != _native.OK:
            raise UiError(ErrorSnapshot(version_status))
        if major != 1:
            raise UiError(ErrorSnapshot(_native.UNSUPPORTED))

        error_ticket = _CleanupTicket(api=api)
        cell = _OwnerCell(api, 0)
        status, token, error = api.runtime_create()
        if status != _native.OK:
            if token:
                _dispose_unexpected(api, token, cell.ticket)
            raise UiError(_snapshot_error(api, status, error, error_ticket))
        if not token:
            if error:
                raise UiError(
                    _snapshot_error(
                        api, _native.INTERNAL_ERROR, error, error_ticket
                    )
                )
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        if error:
            _dispose_unexpected(api, token, cell.ticket)
            raise UiError(
                _snapshot_error(
                    api, _native.INTERNAL_ERROR, error, error_ticket
                )
            )
        cell.token = token
        instance = object.__new__(cls)
        instance._cell = cell
        instance._finalizer = weakref.finalize(
            instance, _CleanupManager.enqueue, cell.ticket
        )
        return instance

    @final
    def is_ui_thread(self) -> bool:
        _CleanupManager.retry_pending()
        with self._cell.lease() as token:
            status, value = self._cell.api.runtime_is_ui_thread(token)
        if status != _native.OK:
            raise UiError(ErrorSnapshot(status))
        if value not in (0, 1):
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        return bool(value)

    @final
    def ui_context(self) -> UiContext:
        context = UiContext(self._cell)
        context.check()
        return context

    @final
    def shutdown(self) -> ShutdownDisposition:
        _CleanupManager.retry_pending()
        error_ticket = _CleanupTicket(api=self._cell.api)
        with self._cell.lease() as token:
            status, error = self._cell.api.runtime_shutdown(token)
        if status == _native.OK:
            _check_result(self._cell.api, status, error, error_ticket)
            return ShutdownDisposition.COMPLETED
        if status == _native.PENDING:
            _check_result(
                self._cell.api,
                status,
                error,
                error_ticket,
                accepted=(_native.PENDING,),
            )
            return ShutdownDisposition.ACCEPTED_PENDING
        raise UiError(
            _snapshot_error(self._cell.api, status, error, error_ticket)
        )

    @final
    def request_shutdown(self, mode: ShutdownMode) -> ShutdownRequest:
        _CleanupManager.retry_pending()
        if isinstance(mode, bool):
            raise ValueError(f"unsupported shutdown mode: {mode!r}")
        try:
            selected_mode = ShutdownMode(mode)
        except (TypeError, ValueError) as error:
            raise ValueError(f"unsupported shutdown mode: {mode!r}") from error

        error_ticket = _CleanupTicket(api=self._cell.api, anchor=self._cell)
        request_cell = _OwnerCell(self._cell.api, 0, self._cell)
        with self._cell.lease() as token:
            status, request_token, error_token = (
                self._cell.api.runtime_request_shutdown(
                    token, int(selected_mode)
                )
            )

        if status not in (_native.OK, _native.PENDING):
            if request_token:
                _dispose_unexpected(
                    self._cell.api, request_token, request_cell.ticket
                )
            raise UiError(
                _snapshot_error(
                    self._cell.api, status, error_token, error_ticket
                )
            )
        if not request_token:
            if error_token:
                raise UiError(
                    _snapshot_error(
                        self._cell.api,
                        _native.INTERNAL_ERROR,
                        error_token,
                        error_ticket,
                    )
                )
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        if error_token:
            _dispose_unexpected(
                self._cell.api, request_token, request_cell.ticket
            )
            raise UiError(
                _snapshot_error(
                    self._cell.api,
                    _native.INTERNAL_ERROR,
                    error_token,
                    error_ticket,
                )
            )

        request_cell.token = request_token
        request = object.__new__(ShutdownRequest)
        request._cell = request_cell
        request._finalizer = weakref.finalize(
            request, _CleanupManager.enqueue, request_cell.ticket
        )
        return request

    @final
    def close(self) -> None:
        _CleanupManager.retry_pending()
        self._cell.close()
        if self._cell.token == 0:
            self._finalizer.detach()

    @final
    def try_close(self) -> UiError | None:
        try:
            self.close()
        except UiError as error:
            return error
        return None

    def __enter__(self) -> Runtime:
        with self._cell.lease():
            pass
        return self

    def __exit__(
        self,
        _exception_type: object,
        exception: BaseException | None,
        _tb: object,
    ) -> bool:
        if exception is None:
            self.close()
        else:
            error = self.try_close()
            if error is not None and hasattr(exception, "add_note"):
                exception.add_note(
                    f"FreeMvvm Runtime close also failed: {error}"
                )
        return False


@final
class UiContext:
    """Runtime-anchored UI-thread capability checked at each use."""

    __slots__ = ("_cell",)

    def __init__(self, cell: _OwnerCell) -> None:
        self._cell = cell

    def check(self) -> None:
        _CleanupManager.retry_pending()
        with self._cell.lease() as token:
            status, value = self._cell.api.runtime_is_ui_thread(token)
        if status != _native.OK:
            raise UiError(ErrorSnapshot(status))
        if value == 0:
            raise UiError(ErrorSnapshot(_native.WRONG_THREAD))
        if value != 1:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))


@final
class ShutdownRequest:
    """Owns a thread-independent shutdown completion receipt."""

    __slots__ = ("_cell", "_finalizer", "__weakref__")

    def __init__(self) -> None:
        raise TypeError(
            "ShutdownRequest values are returned by Runtime.request_shutdown()"
        )

    def query(self) -> ShutdownSnapshot:
        _CleanupManager.retry_pending()
        error_ticket = _CleanupTicket(
            api=self._cell.api, anchor=self._cell.anchor
        )
        with self._cell.lease() as token:
            status, state, reason, error = (
                self._cell.api.shutdown_request_query(token)
            )
        _check_result(self._cell.api, status, error, error_ticket)
        return ShutdownSnapshot(state, reason)

    def close(self) -> None:
        _CleanupManager.retry_pending()
        self._cell.close()
        if self._cell.token == 0:
            self._finalizer.detach()

    def __enter__(self) -> ShutdownRequest:
        with self._cell.lease():
            pass
        return self

    def __exit__(
        self, _exception_type: object, _exception: object, _tb: object
    ) -> bool:
        self.close()
        return False


__all__ = [
    "ErrorSnapshot",
    "MissingRuntimeError",
    "Runtime",
    "ShutdownDisposition",
    "ShutdownMode",
    "ShutdownRequest",
    "ShutdownRequestState",
    "ShutdownSnapshot",
    "UiContext",
    "UiError",
]
