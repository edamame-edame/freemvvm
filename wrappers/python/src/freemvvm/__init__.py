"""Safe Python facade for the FreeMvvm versioned C ABI."""

from __future__ import annotations

from contextlib import contextmanager
from dataclasses import dataclass
from decimal import Decimal
from enum import IntEnum
import itertools
import queue
import sys
import threading
import traceback
from typing import Callable, Iterator, final
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


class BindingMode(IntEnum):
    ONE_TIME = 1
    ONE_WAY = 2
    TWO_WAY = 3


class BindingDirection(IntEnum):
    SOURCE_TO_TARGET = 1
    TARGET_TO_SOURCE = 2


class Orientation(IntEnum):
    VERTICAL = _native.ORIENTATION_VERTICAL
    HORIZONTAL = _native.ORIENTATION_HORIZONTAL


class Alignment(IntEnum):
    START = _native.ALIGN_START
    CENTER = _native.ALIGN_CENTER
    END = _native.ALIGN_END
    STRETCH = _native.ALIGN_STRETCH


@dataclass(frozen=True, slots=True)
class BindingErrorSnapshot:
    error: ErrorSnapshot
    rejected_property: Property
    direction: BindingDirection


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
            retired_count = _retired_roots.qsize()
            for _ in range(retired_count):
                try:
                    retired = _retired_roots.get_nowait()
                except queue.Empty:
                    break
                if hasattr(retired, "destroy"):
                    retired.destroy()
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


def _enqueue_cleanup_tickets(*tickets: _CleanupTicket) -> None:
    for ticket in tickets:
        _CleanupManager.enqueue(ticket)


_roots_lock = threading.Lock()
_roots: dict[int, object] = {}
_root_ids = itertools.count(1)
_retired_roots: queue.SimpleQueue[object] = queue.SimpleQueue()


def _root_add(root: object) -> int:
    with _roots_lock:
        key = next(_root_ids)
        _roots[key] = root
        return key


def _root_get(key: int) -> object | None:
    with _roots_lock:
        return _roots.get(key)


def _root_destroy(userdata: object) -> None:
    key = int(userdata or 0)
    with _roots_lock:
        root = _roots.pop(key, None)
    if root is not None and hasattr(root, "destroy"):
        _retired_roots.put(root)


_DESTROY_CALLBACK = _native.VoidCallback(_root_destroy)


def _report_callback_exception(error: BaseException) -> None:
    traceback.print_exception(
        type(error), error, error.__traceback__, file=sys.stderr
    )


class CommandState(IntEnum):
    IDLE = 0
    RUNNING = 1
    CANCELLING = 2
    SUCCEEDED = 3
    FAILED = 4
    CANCELLED = 5


class _StartRoot:
    __slots__ = ("start", "api", "runtime_cell", "runtime_owner", "trampoline")

    def __init__(
        self,
        start: object,
        api: _native.NativeApi,
        runtime_cell: _OwnerCell,
        runtime_owner: _OwnerCell,
    ) -> None:
        self.start = start
        self.api = api
        self.runtime_cell = runtime_cell
        self.runtime_owner = runtime_owner
        self.trampoline = _native.CommandStartCallback(self.invoke)

    def invoke(
        self,
        userdata: object,
        command_token: int,
        operation_id: int,
        cancel_token: int,
    ) -> int:
        try:
            command_status, owned_command = self.api.handle_retain(
                command_token
            )
            cancel_status, owned_cancel = self.api.handle_retain(cancel_token)
            with self.runtime_owner.lease() as runtime_token:
                runtime_status, owned_runtime = self.api.handle_retain(
                    runtime_token
                )
            if (
                command_status != _native.OK
                or cancel_status != _native.OK
                or runtime_status != _native.OK
            ):
                for status, token in (
                    (command_status, owned_command),
                    (cancel_status, owned_cancel),
                    (runtime_status, owned_runtime),
                ):
                    if token:
                        _CleanupManager.enqueue(
                            _CleanupTicket(
                                api=self.api,
                                token=token,
                                anchor=self.runtime_cell,
                            )
                        )
                return _native.INTERNAL_ERROR
            operation = CommandOperation._from_tokens(
                self.api,
                self.runtime_cell,
                owned_runtime,
                owned_command,
                owned_cancel,
                operation_id,
            )
            try:
                self.start(operation)  # type: ignore[operator]
            except BaseException as error:
                operation.close()
                _report_callback_exception(error)
                return _native.INTERNAL_ERROR
            return _native.OK
        except BaseException as error:
            _report_callback_exception(error)
            return _native.INTERNAL_ERROR

    def destroy(self) -> None:
        if not self.runtime_owner._release_for_cleanup():
            _retired_roots.put(self)


class _PostRoot:
    __slots__ = ("callback", "args", "operation", "terminal")

    def __init__(
        self,
        callback: object,
        args: tuple[object, ...],
        operation: CommandOperation | None = None,
        terminal: tuple[object, ...] | None = None,
    ) -> None:
        self.callback = callback
        self.args = args
        self.operation = operation
        self.terminal = terminal

    def invoke(self, _userdata: object) -> None:
        try:
            if self.terminal is not None and self.operation is not None:
                self.operation._invoke_terminal(self.terminal)
            else:
                self.callback(*self.args)  # type: ignore[operator]
        except BaseException as error:
            _report_callback_exception(error)

    def destroy(self) -> None:
        if self.operation is not None:
            self.operation._post_finished()


class _WindowCloseRoot:
    __slots__ = (
        "callback",
        "api",
        "runtime_cell",
        "window_cell",
        "trampoline",
    )

    def __init__(
        self,
        callback: object,
        api: _native.NativeApi,
        runtime_cell: _OwnerCell,
        window_cell: _OwnerCell,
    ) -> None:
        self.callback = callback
        self.api = api
        self.runtime_cell = runtime_cell
        self.window_cell = window_cell
        self.trampoline = _native.WindowCloseCallback(self.invoke)

    def invoke(self, _userdata: object, borrowed_token: int) -> None:
        request = CloseRequest(
            self.api, self.runtime_cell, int(borrowed_token)
        )
        try:
            self.callback(request)  # type: ignore[operator]
        except BaseException as error:
            _report_callback_exception(error)
        finally:
            request._expire()

    def destroy(self) -> None:
        self.callback = None
        self.trampoline = None
        self.window_cell = None
        self.runtime_cell = None


class CloseRequest:
    """A borrowed close request valid only during its callback invocation."""

    __slots__ = ("_api", "_runtime_cell", "_token", "_active", "_state")

    def __init__(
        self, api: _native.NativeApi, runtime_cell: _OwnerCell, token: int
    ) -> None:
        self._api = api
        self._runtime_cell = runtime_cell
        self._token = token
        self._active = True
        self._state = 0

    def _expire(self) -> None:
        self._active = False
        self._token = 0

    def _begin_decision(self) -> int:
        if not self._active or self._state != 0:
            raise UiError(ErrorSnapshot(_native.CLOSED))
        self._state = 1
        return self._token

    def _finish_decision(self, succeeded: bool) -> None:
        self._state = 2 if succeeded else 0

    def _resolve(self, decision: int) -> None:
        token = self._begin_decision()
        succeeded = False
        try:
            with self._runtime_cell.lease() as runtime_token:
                _check_ui_thread(self._api, runtime_token)
                status, error = self._api.window_close_request_resolve(
                    token, decision
                )
            _check_result(
                self._api,
                status,
                error,
                _CleanupTicket(api=self._api, anchor=self._runtime_cell),
            )
            succeeded = True
        finally:
            self._finish_decision(succeeded)

    def accept(self) -> None:
        self._resolve(_native.CLOSE_ACCEPT)

    def decline(self) -> None:
        self._resolve(_native.CLOSE_DECLINE)

    def defer(self) -> DeferredCloseRequest:
        token = self._begin_decision()
        succeeded = False
        try:
            with self._runtime_cell.lease() as runtime_token:
                _check_ui_thread(self._api, runtime_token)
                status, owned, error = self._api.window_close_request_defer(
                    token
                )
            if status != _native.OK or error or not owned:
                _dispose_unexpected(
                    self._api,
                    owned,
                    _CleanupTicket(api=self._api, anchor=self._runtime_cell),
                )
                _check_result(
                    self._api,
                    status,
                    error,
                    _CleanupTicket(api=self._api, anchor=self._runtime_cell),
                )
                raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
            result = DeferredCloseRequest._from_owned_token(
                self._api, owned, self._runtime_cell
            )
            succeeded = True
            self._expire()
            return result
        finally:
            self._finish_decision(succeeded)


class DeferredCloseRequest:
    """Own a deferred request; resolution and handle release are separate."""

    __slots__ = (
        "_cell",
        "_runtime_cell",
        "_finalizer",
        "_decision_lock",
        "_decision_state",
        "__weakref__",
    )

    def __init__(self) -> None:
        raise TypeError("Use CloseRequest.defer() to own a deferred request")

    @classmethod
    def _from_owned_token(
        cls, api: _native.NativeApi, token: int, runtime_cell: _OwnerCell
    ) -> DeferredCloseRequest:
        if token == 0:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        cell = _OwnerCell(api, token, runtime_cell)
        instance = object.__new__(cls)
        instance._cell = cell
        instance._runtime_cell = runtime_cell
        instance._finalizer = weakref.finalize(
            instance, _CleanupManager.enqueue, cell.ticket
        )
        instance._decision_lock = threading.Lock()
        instance._decision_state = 0
        return instance

    def _resolve(self, decision: int) -> None:
        with self._decision_lock:
            if self._decision_state != 0:
                raise UiError(ErrorSnapshot(_native.CLOSED))
            self._decision_state = 1
        succeeded = False
        try:
            _CleanupManager.retry_pending()
            api = self._cell.api
            with self._runtime_cell.lease() as runtime_token:
                _check_ui_thread(api, runtime_token)
                with self._cell.lease() as token:
                    status, error = api.window_close_request_resolve(
                        token, decision
                    )
            _check_result(
                api,
                status,
                error,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
            succeeded = True
        finally:
            with self._decision_lock:
                self._decision_state = 2 if succeeded else 0

    def accept(self) -> None:
        self._resolve(_native.CLOSE_ACCEPT)

    def decline(self) -> None:
        self._resolve(_native.CLOSE_DECLINE)

    def close(self) -> None:
        _CleanupManager.retry_pending()
        self._cell.close()
        if self._cell.token == 0:
            self._finalizer.detach()

    def __enter__(self) -> DeferredCloseRequest:
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
            try:
                self.close()
            except UiError as close_error:
                if hasattr(exception, "add_note"):
                    exception.add_note(
                        "FreeMvvm deferred request close also failed: "
                        f"{close_error}"
                    )
        return False


class WindowCloseSubscription:
    """Own the close-request Subscription; cancel and release are separate."""

    __slots__ = (
        "_cell",
        "_runtime_cell",
        "_window_cell",
        "_finalizer",
        "_cancel_lock",
        "_cancelled",
        "__weakref__",
    )

    def __init__(self) -> None:
        raise TypeError("Use Window.on_close_request() to register a callback")

    @classmethod
    def _from_owned_token(
        cls,
        api: _native.NativeApi,
        token: int,
        runtime_cell: _OwnerCell,
        window_cell: _OwnerCell,
    ) -> WindowCloseSubscription:
        if token == 0:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        cell = _OwnerCell(api, token, runtime_cell)
        instance = object.__new__(cls)
        instance._cell = cell
        instance._runtime_cell = runtime_cell
        instance._window_cell = window_cell
        instance._finalizer = weakref.finalize(
            instance, _CleanupManager.enqueue, cell.ticket
        )
        instance._cancel_lock = threading.Condition(threading.Lock())
        instance._cancelled = False
        return instance

    def cancel(self) -> None:
        with self._cancel_lock:
            while self._cancelled is None:
                self._cancel_lock.wait()
            if self._cancelled:
                return
            self._cancelled = None
        succeeded = False
        try:
            with self._runtime_cell.lease() as runtime_token:
                _check_ui_thread(self._cell.api, runtime_token)
                with self._cell.lease() as token:
                    status, error = self._cell.api.subscription_cancel(token)
            _check_result(
                self._cell.api,
                status,
                error,
                _CleanupTicket(api=self._cell.api, anchor=self._runtime_cell),
            )
            succeeded = True
        finally:
            with self._cancel_lock:
                self._cancelled = succeeded
                self._cancel_lock.notify_all()

    def close(self) -> None:
        _CleanupManager.retry_pending()
        self._cell.close()
        if self._cell.token == 0:
            self._finalizer.detach()

    def __enter__(self) -> WindowCloseSubscription:
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
            self.cancel()
            self.close()
        else:
            try:
                self.cancel()
                self.close()
            except UiError as close_error:
                if hasattr(exception, "add_note"):
                    exception.add_note(
                        "FreeMvvm close-request subscription cleanup also "
                        f"failed: {close_error}"
                    )
        return False


def _invoke_window_close_callback(userdata: object, request: int) -> None:
    root = _root_get(int(userdata or 0))
    if isinstance(root, _WindowCloseRoot):
        root.invoke(userdata, request)


_WINDOW_CLOSE_CALLBACK = _native.WindowCloseCallback(
    _invoke_window_close_callback
)


_POST_CALLBACK = _native.VoidCallback(
    lambda userdata: (
        root.invoke(userdata)
        if (root := _root_get(int(userdata or 0))) is not None
        else None
    )
)


def _post_root(
    api: _native.NativeApi, runtime_token: int, root: _PostRoot
) -> None:
    key = _root_add(root)
    status, error = api.runtime_post(
        runtime_token, _POST_CALLBACK, key, _DESTROY_CALLBACK
    )
    if status != _native.OK:
        _root_destroy(key)
        raise UiError(ErrorSnapshot(status))
    if error:
        raise UiError(
            _snapshot_error(
                api, _native.INTERNAL_ERROR, error, _CleanupTicket(api=api)
            )
        )


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


def _check_ui_thread(api: _native.NativeApi, runtime: int) -> None:
    status, value = api.runtime_is_ui_thread(runtime)
    if status != _native.OK:
        raise UiError(ErrorSnapshot(status))
    if value == 0:
        raise UiError(ErrorSnapshot(_native.WRONG_THREAD))
    if value != 1:
        raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))


def _dispose_unexpected(
    api: _native.NativeApi, token: int, release_ticket: _CleanupTicket
) -> None:
    if not token:
        return
    if release_ticket.owner is not None:
        release_ticket.anchor = release_ticket.owner.anchor
    release_ticket.owner = None
    release_ticket.api = api
    release_ticket.token = token
    if api.handle_release(token) == _native.OK:
        release_ticket.token = 0
    else:
        _CleanupManager.enqueue(release_ticket)


def _dispose_unexpected_binding(
    api: _native.NativeApi, token: int, release_ticket: _CleanupTicket
) -> None:
    if not token:
        return
    try:
        _, error = api.binding_cancel(token)
    except BaseException:
        _dispose_unexpected(api, token, release_ticket)
        raise
    if error:
        _dispose_unexpected(
            api,
            error,
            _CleanupTicket(api=api, anchor=release_ticket.anchor),
        )
    _dispose_unexpected(api, token, release_ticket)


_DECIMAL_MAX_COEFFICIENT = (1 << 58) - 1
_DECIMAL_MAX_COEFFICIENT_DIGITS = tuple(
    int(digit) for digit in str(_DECIMAL_MAX_COEFFICIENT)
)
_DECIMAL_MAX_SCALE = 31
_UINT64_MAX = (1 << 64) - 1


def _decimal_digits_exceed_max(
    digits: tuple[int, ...], start: int, end: int
) -> bool:
    length = end - start
    if length > len(_DECIMAL_MAX_COEFFICIENT_DIGITS):
        return True
    if length < len(_DECIMAL_MAX_COEFFICIENT_DIGITS):
        return False
    return digits[start:end] > _DECIMAL_MAX_COEFFICIENT_DIGITS


def _decimal_to_bits(value: Decimal) -> int:
    """Map a finite built-in Decimal to the exact DECIMAL ABI encoding."""
    if type(value) is not Decimal:
        raise TypeError("DECIMAL Property values must be built-in Decimal")

    sign, digits, exponent = value.as_tuple()
    if not isinstance(exponent, int):
        raise ValueError("DECIMAL Property values must be finite")

    start = 0
    while start < len(digits) and digits[start] == 0:
        start += 1
    if start == len(digits):
        scale = -exponent if exponent < 0 else 0
        if scale > _DECIMAL_MAX_SCALE:
            scale = 0
        return (sign << 63) | (scale << 58)

    if exponent >= 0:
        significant_length = len(digits) - start
        candidate_length = significant_length + exponent
        max_digits = len(_DECIMAL_MAX_COEFFICIENT_DIGITS)
        if candidate_length > max_digits:
            raise OverflowError("Decimal value exceeds DECIMAL Property range")
        if candidate_length == max_digits:
            padded_digits = digits[start:] + (0,) * exponent
            if padded_digits > _DECIMAL_MAX_COEFFICIENT_DIGITS:
                raise OverflowError(
                    "Decimal value exceeds DECIMAL Property range"
                )
        coefficient = int("".join(str(digit) for digit in digits[start:]))
        if exponent:
            coefficient *= 10**exponent
        scale = 0
    else:
        scale = -exponent
        end = len(digits)
        while (
            scale > 0
            and (
                scale > _DECIMAL_MAX_SCALE
                or _decimal_digits_exceed_max(digits, start, end)
            )
            and digits[end - 1] == 0
        ):
            end -= 1
            scale -= 1

        if scale > _DECIMAL_MAX_SCALE or _decimal_digits_exceed_max(
            digits, start, end
        ):
            raise OverflowError("Decimal value exceeds DECIMAL Property range")
        coefficient = int("".join(str(digit) for digit in digits[start:end]))

    return (sign << 63) | (scale << 58) | coefficient


def _decimal_from_bits(bits: int) -> Decimal:
    """Construct a Decimal tuple without context-sensitive arithmetic."""
    if type(bits) is not int or not 0 <= bits <= _UINT64_MAX:
        raise ValueError("DECIMAL ABI returned an invalid uint64 value")
    sign = (bits >> 63) & 1
    scale = (bits >> 58) & 0x1F
    coefficient = bits & _DECIMAL_MAX_COEFFICIENT
    digits = (
        tuple(int(digit) for digit in str(coefficient))
        if coefficient
        else (0,)
    )
    return Decimal((sign, digits, -scale))


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
            _check_ui_thread(self._cell.api, token)


@final
class Property:
    """Owns a BOOL, signed-I64, F64, DECIMAL, or UTF-8 Property token."""

    __slots__ = (
        "_cell",
        "_runtime_cell",
        "_kind",
        "_finalizer",
        "__weakref__",
    )

    _BOOL = 1
    _I64 = 2
    _UTF8 = 3
    _F64 = 4
    _DECIMAL = 6
    _I64_MIN = -(1 << 63)
    _I64_MAX = (1 << 63) - 1

    def __init__(self) -> None:
        raise TypeError(
            "Use Property.create(runtime, initial) to create a Property"
        )

    @classmethod
    def create(
        cls, runtime: Runtime, initial: bool | int | float | Decimal | str
    ) -> Property:
        if not isinstance(runtime, Runtime):
            raise TypeError("runtime must be a freemvvm.Runtime")

        if type(initial) is bool:
            kind = cls._BOOL
            native_initial = int(initial)
        elif type(initial) is int:
            if not cls._I64_MIN <= initial <= cls._I64_MAX:
                raise OverflowError(
                    "Property int value must fit in signed I64"
                )
            kind = cls._I64
            native_initial = initial
        elif type(initial) is float:
            kind = cls._F64
            native_initial = initial
        elif type(initial) is Decimal:
            kind = cls._DECIMAL
            native_initial = _decimal_to_bits(initial)
        elif type(initial) is str:
            kind = cls._UTF8
            native_initial = initial.encode("utf-8", errors="strict")
        else:
            raise TypeError(
                "Property initial value must be built-in bool, int, float, "
                "Decimal, or str"
            )

        _CleanupManager.retry_pending()
        api = runtime._cell.api
        cell = _OwnerCell(api, 0, runtime._cell)
        error_ticket = _CleanupTicket(api=api, anchor=runtime._cell)
        with runtime._cell.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            if kind == cls._BOOL:
                status, token, error = api.property_create_bool(
                    runtime_token, native_initial
                )
            elif kind == cls._I64:
                status, token, error = api.property_create_i64(
                    runtime_token, native_initial
                )
            elif kind == cls._F64:
                status, token, error = api.property_create_f64(
                    runtime_token, native_initial
                )
            elif kind == cls._DECIMAL:
                status, token, error = api.property_create_decimal(
                    runtime_token, native_initial
                )
            else:
                status, token, error = api.property_create_utf8(
                    runtime_token, native_initial
                )

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
        instance._runtime_cell = runtime._cell
        instance._kind = kind
        instance._finalizer = weakref.finalize(
            instance, _CleanupManager.enqueue, cell.ticket
        )
        return instance

    @contextmanager
    def _ui_call(self) -> Iterator[tuple[_native.NativeApi, int]]:
        _CleanupManager.retry_pending()
        api = self._cell.api
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with self._cell.lease() as property_token:
                yield api, property_token

    @classmethod
    def _from_owned_token(
        cls,
        api: _native.NativeApi,
        token: int,
        runtime_cell: _OwnerCell,
        kind: int,
    ) -> Property:
        if token == 0:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        cell = _OwnerCell(api, token, runtime_cell)
        instance = object.__new__(cls)
        instance._cell = cell
        instance._runtime_cell = runtime_cell
        instance._kind = kind
        instance._finalizer = weakref.finalize(
            instance, _CleanupManager.enqueue, cell.ticket
        )
        return instance

    def get(self) -> bool | int | float | Decimal | str:
        with self._ui_call() as (api, token):
            if self._kind == self._BOOL:
                status, value = api.property_get_bool(token)
            elif self._kind == self._I64:
                status, value = api.property_get_i64(token)
            elif self._kind == self._F64:
                status, value = api.property_get_f64(token)
            elif self._kind == self._DECIMAL:
                status, value = api.property_get_decimal(token)
            else:
                status, length = api.property_get_utf8(token, None)
                if status not in (_native.OK, _native.BUFFER_TOO_SMALL):
                    raise UiError(ErrorSnapshot(status))
                if length > 2**31 - 1:
                    raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
                while True:
                    try:
                        buffer = bytearray(length)
                    except MemoryError:
                        raise
                    status, copied = api.property_get_utf8(token, buffer)
                    if status == _native.BUFFER_TOO_SMALL:
                        query_status, length = api.property_get_utf8(
                            token, None
                        )
                        if query_status not in (
                            _native.OK,
                            _native.BUFFER_TOO_SMALL,
                        ):
                            raise UiError(ErrorSnapshot(query_status))
                        if length > 2**31 - 1:
                            raise UiError(
                                ErrorSnapshot(_native.INTERNAL_ERROR)
                            )
                        continue
                    if status != _native.OK:
                        raise UiError(ErrorSnapshot(status))
                    if copied > len(buffer):
                        raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
                    try:
                        return buffer[:copied].decode("utf-8", errors="strict")
                    except UnicodeDecodeError as error:
                        raise UiError(
                            ErrorSnapshot(_native.INTERNAL_ERROR)
                        ) from error

        if status != _native.OK:
            raise UiError(ErrorSnapshot(status))
        if self._kind == self._BOOL:
            if value not in (0, 1):
                raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
            return bool(value)
        if self._kind == self._DECIMAL:
            try:
                return _decimal_from_bits(value)
            except ValueError as error:
                raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR)) from error
        return value

    def set(self, value: bool | int | float | Decimal | str) -> None:
        if self._kind == self._BOOL:
            if type(value) is not bool:
                raise TypeError("BOOL Property values must be built-in bool")
            native_value = int(value)
        elif self._kind == self._I64:
            if type(value) is not int:
                raise TypeError("I64 Property values must be built-in int")
            if not self._I64_MIN <= value <= self._I64_MAX:
                raise OverflowError(
                    "Property int value must fit in signed I64"
                )
            native_value = value
        elif self._kind == self._F64:
            if type(value) is not float:
                raise TypeError("F64 Property values must be built-in float")
            native_value = value
        elif self._kind == self._DECIMAL:
            if type(value) is not Decimal:
                raise TypeError(
                    "DECIMAL Property values must be built-in Decimal"
                )
            native_value = _decimal_to_bits(value)
        else:
            if type(value) is not str:
                raise TypeError("UTF-8 Property values must be built-in str")
            native_value = value.encode("utf-8", errors="strict")

        _CleanupManager.retry_pending()
        error_ticket = _CleanupTicket(
            api=self._cell.api, anchor=self._runtime_cell
        )
        with self._ui_call() as (api, token):
            if self._kind == self._BOOL:
                status, error = api.property_set_bool(token, native_value)
            elif self._kind == self._I64:
                status, error = api.property_set_i64(token, native_value)
            elif self._kind == self._F64:
                status, error = api.property_set_f64(token, native_value)
            elif self._kind == self._DECIMAL:
                status, error = api.property_set_decimal(token, native_value)
            else:
                status, error = api.property_set_utf8(token, native_value)
        _check_result(api, status, error, error_ticket)

    def close(self) -> None:
        _CleanupManager.retry_pending()
        self._cell.close()
        if self._cell.token == 0:
            self._finalizer.detach()

    def try_close(self) -> UiError | None:
        try:
            self.close()
        except UiError as error:
            return error
        return None

    def __enter__(self) -> Property:
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
                    f"FreeMvvm Property close also failed: {error}"
                )
        return False


@final
class Binding:
    """Own a Property Binding handle; close releases but does not cancel it."""

    __slots__ = (
        "_cell",
        "_runtime_cell",
        "_kind",
        "_finalizer",
        "__weakref__",
    )

    def __init__(self) -> None:
        raise TypeError("Use Binding.create() to create a Binding")

    @classmethod
    def create(
        cls, source: Property, target: Property, mode: BindingMode
    ) -> Binding | None:
        if not isinstance(source, Property) or not isinstance(
            target, Property
        ):
            raise TypeError(
                "source and target must be freemvvm.Property values"
            )
        if not isinstance(mode, BindingMode):
            raise TypeError("mode must be a freemvvm.BindingMode")
        if source._runtime_cell is not target._runtime_cell:
            raise UiError(ErrorSnapshot(_native.WRONG_RUNTIME))
        if source._kind != target._kind:
            raise UiError(ErrorSnapshot(_native.WRONG_TYPE))

        _CleanupManager.retry_pending()
        api = source._cell.api
        runtime_cell = source._runtime_cell
        binding_cell = _OwnerCell(api, 0, runtime_cell)
        error_ticket = _CleanupTicket(api=api, anchor=runtime_cell)
        with runtime_cell.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with source._cell.lease() as source_token:
                with target._cell.lease() as target_token:
                    status, binding_token, error_token = api.binding_create(
                        source_token, target_token, int(mode)
                    )

        if status == _native.OK and error_token == 0:
            if mode == BindingMode.ONE_TIME and binding_token == 0:
                return None
            if (
                mode in (BindingMode.ONE_WAY, BindingMode.TWO_WAY)
                and binding_token
            ):
                binding_cell.token = binding_token
                instance = object.__new__(cls)
                instance._cell = binding_cell
                instance._runtime_cell = runtime_cell
                instance._kind = source._kind
                instance._finalizer = weakref.finalize(
                    instance,
                    _CleanupManager.enqueue,
                    binding_cell.ticket,
                )
                return instance

        result_status = (
            status if status != _native.OK else _native.INTERNAL_ERROR
        )
        if binding_token:
            _dispose_unexpected_binding(
                api, binding_token, binding_cell.ticket
            )
        raise UiError(
            _snapshot_error(api, result_status, error_token, error_ticket)
        )

    def cancel(self) -> None:
        _CleanupManager.retry_pending()
        error_ticket = _CleanupTicket(
            api=self._cell.api, anchor=self._runtime_cell
        )
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(self._cell.api, runtime_token)
            with self._cell.lease() as binding_token:
                status, error = self._cell.api.binding_cancel(binding_token)
        _check_result(self._cell.api, status, error, error_ticket)

    def error_snapshot(self) -> BindingErrorSnapshot | None:
        _CleanupManager.retry_pending()
        api = self._cell.api
        error_ticket = _CleanupTicket(api=api, anchor=self._runtime_cell)
        property_ticket = _CleanupTicket(api=api, anchor=self._runtime_cell)
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with self._cell.lease() as binding_token:
                status, error_token, rejected_token, raw_direction = (
                    api.binding_error_snapshot(binding_token)
                )

        if status != _native.OK:
            _dispose_unexpected(api, error_token, error_ticket)
            _dispose_unexpected(api, rejected_token, property_ticket)
            raise UiError(ErrorSnapshot(status))
        if error_token == rejected_token == raw_direction == 0:
            return None

        try:
            direction = BindingDirection(raw_direction)
        except ValueError:
            direction = None
        if not error_token or not rejected_token or direction is None:
            _dispose_unexpected(api, error_token, error_ticket)
            _dispose_unexpected(api, rejected_token, property_ticket)
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))

        error = _snapshot_error(api, _native.OK, error_token, error_ticket)
        try:
            rejected_property = Property._from_owned_token(
                api, rejected_token, self._runtime_cell, self._kind
            )
        except BaseException:
            _dispose_unexpected(api, rejected_token, property_ticket)
            raise
        return BindingErrorSnapshot(error, rejected_property, direction)

    def close(self) -> None:
        _CleanupManager.retry_pending()
        self._cell.close()
        if self._cell.token == 0:
            self._finalizer.detach()

    def try_close(self) -> UiError | None:
        try:
            self.close()
        except UiError as error:
            return error
        return None


@final
class CommandOperation:
    """Thread-independent lease for one accepted Command operation."""

    __slots__ = (
        "operation_id",
        "_runtime",
        "_command",
        "_cancel",
        "_lock",
        "_terminal_pending",
        "_closed",
    )

    def __init__(self) -> None:
        raise TypeError("CommandOperation values are supplied to start()")

    @classmethod
    def _from_tokens(
        cls,
        api: _native.NativeApi,
        runtime_cell: _OwnerCell,
        runtime_token: int,
        command_token: int,
        cancel_token: int,
        operation_id: int,
    ) -> CommandOperation:
        instance = object.__new__(cls)
        instance.operation_id = operation_id
        instance._runtime = _OwnerCell(api, runtime_token, runtime_cell)
        instance._command = _OwnerCell(api, command_token, runtime_cell)
        instance._cancel = _OwnerCell(api, cancel_token, runtime_cell)
        instance._lock = threading.Lock()
        instance._terminal_pending = False
        instance._closed = False
        return instance

    def is_cancel_requested(self) -> bool:
        with self._lock:
            if self._closed:
                raise UiError(ErrorSnapshot(_native.CLOSED))
        with self._cancel.lease() as token:
            status, value = self._cancel.api.command_cancel_token_requested(
                token
            )
        if status != _native.OK or value not in (0, 1):
            raise UiError(
                ErrorSnapshot(
                    status if status != _native.OK else _native.INTERNAL_ERROR
                )
            )
        return bool(value)

    def post(self, callback: object, *args: object) -> None:
        if not callable(callback):
            raise TypeError("callback must be callable")
        with self._lock:
            if self._closed:
                raise UiError(ErrorSnapshot(_native.CLOSED))
        with self._runtime.lease() as runtime_token:
            _post_root(
                self._runtime.api, runtime_token, _PostRoot(callback, args)
            )

    def _report(self, kind: str, payload: tuple[object, ...]) -> None:
        with self._lock:
            if self._closed:
                raise UiError(ErrorSnapshot(_native.CLOSED))
            if self._terminal_pending:
                raise UiError(ErrorSnapshot(_native.INVALID_ARGUMENT))
            self._terminal_pending = True
        root = _PostRoot(None, (), self, (kind, *payload))
        try:
            with self._runtime.lease() as runtime_token:
                _post_root(self._runtime.api, runtime_token, root)
        except BaseException:
            with self._lock:
                self._terminal_pending = False
            raise

    def report_success(self) -> None:
        self._report("success", ())

    def report_cancelled(self) -> None:
        self._report("cancelled", ())

    def report_failure(self, domain: int, code: int, message: str) -> None:
        if type(domain) is not int or not 0 <= domain <= 0xFFFFFFFF:
            raise ValueError("domain must be a built-in uint32 int")
        if type(code) is not int or not 0 <= code <= 0xFFFFFFFF:
            raise ValueError("code must be a built-in uint32 int")
        if type(message) is not str:
            raise TypeError("message must be a built-in str")
        self._report(
            "failure", (domain, code, message.encode("utf-8", "strict"))
        )

    def _invoke_terminal(self, terminal: tuple[object, ...]) -> None:
        kind, *payload = terminal
        with self._command.lease() as command_token:
            if kind == "success":
                status = self._command.api.command_complete_success(
                    command_token, self.operation_id
                )
            elif kind == "cancelled":
                status = self._command.api.command_complete_cancelled(
                    command_token, self.operation_id
                )
            else:
                domain, code, message = payload
                status = self._command.api.command_complete_failure(
                    command_token, self.operation_id, domain, code, message
                )
        if status != _native.OK:
            _report_callback_exception(UiError(ErrorSnapshot(status)))

    def _post_finished(self) -> None:
        with self._lock:
            release = self._closed
        if release:
            self._release_cells()

    def _release_cells(self) -> None:
        for cell in (self._cancel, self._command, self._runtime):
            if not cell._release_for_cleanup():
                _CleanupManager.enqueue(cell.ticket)

    def _close_cells(self) -> None:
        errors: list[UiError] = []
        for cell in (self._cancel, self._command, self._runtime):
            try:
                cell.close()
            except UiError as error:
                errors.append(error)
        if errors:
            with self._lock:
                self._closed = False
            raise errors[0]

    def close(self) -> None:
        with self._lock:
            if self._closed:
                return
            self._closed = True
            pending = self._terminal_pending
        if pending:
            return
        self._close_cells()


@final
class AsyncCommand:
    """Own an asynchronous command registered with the native Runtime."""

    __slots__ = (
        "_cell",
        "_runtime_cell",
        "_runtime_owner",
        "_finalizer",
        "__weakref__",
    )

    def __init__(self) -> None:
        raise TypeError("Use AsyncCommand.create() to create a command")

    @classmethod
    def create(
        cls,
        runtime: Runtime,
        start: object,
        eligibility: Property | None = None,
    ) -> AsyncCommand:
        if not isinstance(runtime, Runtime):
            raise TypeError("runtime must be a freemvvm.Runtime")
        if not callable(start):
            raise TypeError("start must be callable")
        if eligibility is not None and not isinstance(eligibility, Property):
            raise TypeError("eligibility must be a BOOL Property or None")
        runtime_cell = runtime._cell
        if eligibility is not None and (
            eligibility._runtime_cell is not runtime_cell
            or eligibility._kind != Property._BOOL
        ):
            raise UiError(ErrorSnapshot(_native.WRONG_TYPE))
        _CleanupManager.retry_pending()
        api = runtime_cell.api
        cell = _OwnerCell(api, 0, runtime_cell)
        error_ticket = _CleanupTicket(api=api, anchor=runtime_cell)
        with runtime_cell.lease() as runtime_token:
            retain_status, retained_runtime = api.handle_retain(runtime_token)
            if retain_status != _native.OK or not retained_runtime:
                raise UiError(
                    ErrorSnapshot(
                        retain_status
                        if retain_status != _native.OK
                        else _native.INTERNAL_ERROR
                    )
                )
            runtime_owner = _OwnerCell(api, retained_runtime, runtime_cell)
            facade_status, facade_runtime = api.handle_retain(runtime_token)
            if facade_status != _native.OK or not facade_runtime:
                runtime_owner.close()
                raise UiError(
                    ErrorSnapshot(
                        facade_status
                        if facade_status != _native.OK
                        else _native.INTERNAL_ERROR
                    )
                )
            facade_runtime_owner = _OwnerCell(
                api, facade_runtime, runtime_cell
            )
            root = _StartRoot(start, api, runtime_cell, runtime_owner)
            key = _root_add(root)
            if eligibility is None:
                eligibility_token = 0
                status, token, error = api.command_create_async(
                    runtime_token,
                    eligibility_token,
                    root.trampoline,
                    key,
                    _DESTROY_CALLBACK,
                )
            else:
                with eligibility._cell.lease() as eligibility_token:
                    status, token, error = api.command_create_async(
                        runtime_token,
                        eligibility_token,
                        root.trampoline,
                        key,
                        _DESTROY_CALLBACK,
                    )
        if status != _native.OK or not token or error:
            _root_destroy(key)
            if status != _native.OK:
                _CleanupManager.retry_pending()
            facade_runtime_owner.close()
            if token:
                _dispose_unexpected(api, token, cell.ticket)
            result_status = (
                status if status != _native.OK else _native.INTERNAL_ERROR
            )
            raise UiError(
                _snapshot_error(api, result_status, error, error_ticket)
            )
        cell.token = token
        instance = object.__new__(cls)
        instance._cell = cell
        instance._runtime_cell = runtime_cell
        instance._runtime_owner = facade_runtime_owner
        instance._finalizer = weakref.finalize(
            instance,
            _enqueue_cleanup_tickets,
            cell.ticket,
            facade_runtime_owner.ticket,
        )
        return instance

    @classmethod
    def _from_owned_token(
        cls,
        api: _native.NativeApi,
        token: int,
        runtime_cell: _OwnerCell,
    ) -> AsyncCommand:
        """Adopt an owned Command token returned by a Control getter."""
        if token == 0:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        runtime_owner = _OwnerCell(api, 0, runtime_cell)
        retain_status = _native.INTERNAL_ERROR
        retained_runtime = 0
        with runtime_cell.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            retain_status, retained_runtime = api.handle_retain(runtime_token)
        if retain_status != _native.OK or not retained_runtime:
            ticket = _CleanupTicket(api=api, anchor=runtime_cell)
            _dispose_unexpected(api, token, ticket)
            _dispose_unexpected(api, retained_runtime, runtime_owner.ticket)
            raise UiError(
                ErrorSnapshot(
                    retain_status
                    if retain_status != _native.OK
                    else _native.INTERNAL_ERROR
                )
            )
        runtime_owner.token = retained_runtime
        cell = _OwnerCell(api, token, runtime_cell)
        instance = object.__new__(cls)
        instance._cell = cell
        instance._runtime_cell = runtime_cell
        instance._runtime_owner = runtime_owner
        instance._finalizer = weakref.finalize(
            instance,
            _enqueue_cleanup_tickets,
            cell.ticket,
            runtime_owner.ticket,
        )
        return instance

    def execute(self) -> int:
        api = self._cell.api
        error_ticket = _CleanupTicket(api=api, anchor=self._runtime_cell)
        with self._runtime_owner.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with self._cell.lease() as command_token:
                status, operation_id, error = api.command_execute(
                    command_token
                )
        _check_result(api, status, error, error_ticket)
        if operation_id == 0:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        return operation_id

    def cancel(self, operation_id: int) -> None:
        if (
            type(operation_id) is not int
            or not 0 < operation_id <= 0xFFFFFFFFFFFFFFFF
        ):
            raise ValueError("operation_id must be a nonzero uint64 int")
        api = self._cell.api
        error_ticket = _CleanupTicket(api=api, anchor=self._runtime_cell)
        with self._runtime_owner.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with self._cell.lease() as command_token:
                status, error = api.command_cancel(command_token, operation_id)
        _check_result(api, status, error, error_ticket)

    def _property(self, can_execute: bool) -> Property:
        api = self._cell.api
        with self._runtime_owner.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with self._cell.lease() as token:
                status, prop = (
                    api.command_can_execute_property(token)
                    if can_execute
                    else api.command_state_property(token)
                )
        if status != _native.OK or not prop:
            raise UiError(
                ErrorSnapshot(
                    status if status != _native.OK else _native.INTERNAL_ERROR
                )
            )
        return Property._from_owned_token(
            api,
            prop,
            self._runtime_cell,
            Property._BOOL if can_execute else Property._I64,
        )

    def state_property(self) -> Property:
        return self._property(False)

    def can_execute_property(self) -> Property:
        return self._property(True)

    def failure_snapshot(self) -> ErrorSnapshot | None:
        api = self._cell.api
        with self._runtime_owner.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with self._cell.lease() as token:
                status, error = api.command_failure_snapshot(token)
        if status != _native.OK:
            raise UiError(ErrorSnapshot(status))
        if not error:
            return None
        return _snapshot_error(
            api,
            _native.OK,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )

    def close(self) -> None:
        _CleanupManager.retry_pending()
        if self._cell.token == 0:
            self._runtime_owner.close()
            self._finalizer.detach()
            return
        api = self._cell.api
        error_ticket = _CleanupTicket(api=api, anchor=self._runtime_cell)
        with self._runtime_owner.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with self._cell.lease() as token:
                status, error = api.command_close(token)
        _check_result(api, status, error, error_ticket)
        self._cell.close()
        if self._cell.token == 0:
            self._runtime_owner.close()
            self._finalizer.detach()

    def try_close(self) -> UiError | None:
        try:
            self.close()
        except UiError as error:
            return error
        return None


class Widget:
    """Own one UI Widget reference; concrete views are created by factories."""

    __slots__ = ("_cell", "_runtime_cell", "_finalizer", "__weakref__")

    def __init__(self) -> None:
        raise TypeError("Use a concrete Widget create() factory")

    @classmethod
    def _create(
        cls,
        runtime: Runtime,
        create: Callable[[_native.NativeApi, int], tuple[int, int, int]],
    ) -> Widget:
        if not isinstance(runtime, Runtime):
            raise TypeError("runtime must be a freemvvm.Runtime")
        _CleanupManager.retry_pending()
        api = runtime._cell.api
        runtime_cell = runtime._cell
        error_ticket = _CleanupTicket(api=api, anchor=runtime_cell)
        with runtime_cell.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            status, token, error = create(api, runtime_token)
        if status != _native.OK or not token or error:
            _dispose_unexpected(
                api, token, _CleanupTicket(api=api, anchor=runtime_cell)
            )
            _check_result(api, status, error, error_ticket)
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        return cls._from_owned_token(api, token, runtime_cell)

    @classmethod
    def _from_owned_token(
        cls,
        api: _native.NativeApi,
        token: int,
        runtime_cell: _OwnerCell,
    ) -> Widget:
        if token == 0:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        cell = _OwnerCell(api, token, runtime_cell)
        instance = object.__new__(cls)
        instance._cell = cell
        instance._runtime_cell = runtime_cell
        instance._finalizer = weakref.finalize(
            instance, _CleanupManager.enqueue, cell.ticket
        )
        return instance

    @contextmanager
    def _ui_call(self) -> Iterator[tuple[_native.NativeApi, int]]:
        _CleanupManager.retry_pending()
        api = self._cell.api
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(api, runtime_token)
            with self._cell.lease() as widget_token:
                yield api, widget_token

    def _property(
        self, getter: Callable[[int], tuple[int, int]], kind: int
    ) -> Property:
        with self._ui_call() as (api, widget_token):
            status, token = getter(widget_token)
        if status != _native.OK or not token:
            _dispose_unexpected(
                api,
                token,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
            raise UiError(
                ErrorSnapshot(
                    status if status != _native.OK else _native.INTERNAL_ERROR
                )
            )
        return Property._from_owned_token(api, token, self._runtime_cell, kind)

    @property
    def is_enabled(self) -> Property:
        return self._property(
            self._cell.api.widget_is_enabled_property, Property._BOOL
        )

    @property
    def accessible_name(self) -> Property:
        return self._property(
            self._cell.api.widget_accessible_name_property, Property._UTF8
        )

    def request_focus(self) -> None:
        with self._ui_call() as (api, token):
            status, error = api.widget_request_focus(token)
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )

    def close(self) -> None:
        _CleanupManager.retry_pending()
        self._cell.close()
        if self._cell.token == 0:
            self._finalizer.detach()


class Container(Widget):
    """Abstract facade for Controls with indexed multiple children."""

    __slots__ = ()

    def __init__(self) -> None:
        raise TypeError("Container is abstract; create a concrete container")

    @property
    def child_count(self) -> int:
        with self._ui_call() as (api, token):
            status, count = api.container_child_count(token)
        if status != _native.OK:
            raise UiError(ErrorSnapshot(status))
        return count

    def child_at(self, index: int) -> Widget:
        _validate_index(index, "index")
        with self._ui_call() as (api, token):
            status, child, error = api.container_child_at(token, index)
        if status != _native.OK or error:
            _dispose_unexpected(
                api,
                child,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
            _check_result(
                api,
                status,
                error,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
        if not child:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))
        return Widget._from_owned_token(api, child, self._runtime_cell)

    def insert(self, child: Widget, index: int) -> None:
        _validate_index(index, "index")
        if not isinstance(child, Widget):
            raise TypeError("child must be a freemvvm.Widget")
        if child._runtime_cell is not self._runtime_cell:
            raise UiError(ErrorSnapshot(_native.WRONG_RUNTIME))
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(self._cell.api, runtime_token)
            with self._cell.lease() as container_token:
                with child._cell.lease() as child_token:
                    status, error = self._cell.api.container_insert(
                        container_token, child_token, index
                    )
        _check_result(
            self._cell.api,
            status,
            error,
            _CleanupTicket(api=self._cell.api, anchor=self._runtime_cell),
        )

    def remove(self, child: Widget) -> None:
        if not isinstance(child, Widget):
            raise TypeError("child must be a freemvvm.Widget")
        if child._runtime_cell is not self._runtime_cell:
            raise UiError(ErrorSnapshot(_native.WRONG_RUNTIME))
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(self._cell.api, runtime_token)
            with self._cell.lease() as container_token:
                with child._cell.lease() as child_token:
                    status, error = self._cell.api.container_remove(
                        container_token, child_token
                    )
        _check_result(
            self._cell.api,
            status,
            error,
            _CleanupTicket(api=self._cell.api, anchor=self._runtime_cell),
        )

    def move(self, from_index: int, to_index: int) -> None:
        _validate_index(from_index, "from_index")
        _validate_index(to_index, "to_index")
        with self._ui_call() as (api, token):
            status, error = api.container_move(token, from_index, to_index)
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )


def _validate_index(value: int, name: str) -> None:
    if type(value) is not int or not 0 <= value <= 0xFFFFFFFFFFFFFFFF:
        raise ValueError(f"{name} must be a nonnegative uint64 int")


def _encode_control_text(value: str, name: str) -> bytes:
    if type(value) is not str:
        raise TypeError(f"{name} must be a built-in str")
    return value.encode("utf-8", errors="strict")


@final
class Window(Widget):
    """Own a Window Control and its independent native peer operations."""

    __slots__ = ()

    @classmethod
    def create(cls, runtime: Runtime) -> Window:
        return cls._create(
            runtime, lambda api, token: api.window_create(token)
        )

    @property
    def background_rgb(self) -> tuple[float, float, float]:
        with self._ui_call() as (api, token):
            status, red, green, blue, error = api.window_background_rgb(token)
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )
        return red, green, blue

    @background_rgb.setter
    def background_rgb(self, value: tuple[float, float, float]) -> None:
        if type(value) is not tuple or len(value) != 3:
            raise TypeError("background_rgb must be a 3-item tuple")
        red, green, blue = value
        with self._ui_call() as (api, token):
            status, error = api.window_set_background_rgb(
                token, red, green, blue
            )
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )

    def reset_background_rgb(self) -> None:
        with self._ui_call() as (api, token):
            status, error = api.window_reset_background_rgb(token)
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )

    @property
    def content(self) -> Widget | None:
        with self._ui_call() as (api, token):
            status, child, error = api.window_content(token)
        if status != _native.OK or error:
            _dispose_unexpected(
                api,
                child,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
            _check_result(
                api,
                status,
                error,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
        if not child:
            return None
        return Widget._from_owned_token(api, child, self._runtime_cell)

    def set_content(self, child: Widget) -> None:
        if not isinstance(child, Widget):
            raise TypeError("child must be a freemvvm.Widget")
        if child._runtime_cell is not self._runtime_cell:
            raise UiError(ErrorSnapshot(_native.WRONG_RUNTIME))
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(self._cell.api, runtime_token)
            with self._cell.lease() as window_token:
                with child._cell.lease() as child_token:
                    status, error = self._cell.api.window_set_content(
                        window_token, child_token
                    )
        _check_result(
            self._cell.api,
            status,
            error,
            _CleanupTicket(api=self._cell.api, anchor=self._runtime_cell),
        )

    def clear_content(self) -> None:
        with self._ui_call() as (api, token):
            status, error = api.window_clear_content(token)
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )

    def open(self) -> None:
        with self._ui_call() as (api, token):
            status, error = api.window_open(token)
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )

    def close_peer(self) -> None:
        with self._ui_call() as (api, token):
            status, error = api.window_close(token)
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )

    def on_close_request(
        self, callback: Callable[[CloseRequest], object]
    ) -> WindowCloseSubscription:
        if not callable(callback):
            raise TypeError("callback must be callable")
        api = self._cell.api
        root = _WindowCloseRoot(callback, api, self._runtime_cell, self._cell)
        root_key = _root_add(root)
        try:
            with self._ui_call() as (call_api, token):
                status, subscription, error = (
                    call_api.window_close_request_subscribe(
                        token,
                        root.trampoline,
                        root_key,
                        _DESTROY_CALLBACK,
                    )
                )
        except BaseException:
            _root_destroy(root_key)
            raise

        if status != _native.OK:
            _root_destroy(root_key)
            _dispose_unexpected(
                api,
                subscription,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
            _check_result(
                api,
                status,
                error,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )

        if error or not subscription:
            if subscription:
                cancel_status, _cancel_error = api.subscription_cancel(
                    subscription
                )
                if cancel_status == _native.OK:
                    _root_destroy(root_key)
                _dispose_unexpected(
                    api,
                    subscription,
                    _CleanupTicket(api=api, anchor=self._runtime_cell),
                )
            else:
                _root_destroy(root_key)
            _check_result(
                api,
                _native.INTERNAL_ERROR,
                error,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR))

        return WindowCloseSubscription._from_owned_token(
            api, subscription, self._runtime_cell, self._cell
        )

    @property
    def title(self) -> Property:
        return self._property(
            self._cell.api.window_title_property, Property._UTF8
        )


@final
class StackPanel(Container):
    """Own an indexed multi-child StackPanel Control."""

    __slots__ = ()

    @classmethod
    def create(cls, runtime: Runtime) -> StackPanel:
        return cls._create(
            runtime, lambda api, token: api.stack_panel_create(token)
        )

    @property
    def orientation(self) -> Orientation:
        with self._ui_call() as (api, token):
            status, orientation = api.stack_panel_orientation(token)
        if status != _native.OK:
            raise UiError(ErrorSnapshot(status))
        try:
            return Orientation(orientation)
        except ValueError as error:
            raise UiError(ErrorSnapshot(_native.INTERNAL_ERROR)) from error

    @orientation.setter
    def orientation(self, value: Orientation) -> None:
        if type(value) is not Orientation:
            raise TypeError("orientation must be an Orientation")
        with self._ui_call() as (api, token):
            status, error = api.stack_panel_set_orientation(token, int(value))
        _check_result(
            api,
            status,
            error,
            _CleanupTicket(api=api, anchor=self._runtime_cell),
        )

    def set_child_align(self, child: Widget, alignment: Alignment) -> None:
        if not isinstance(child, Widget):
            raise TypeError("child must be a freemvvm.Widget")
        if type(alignment) is not Alignment:
            raise TypeError("alignment must be an Alignment")
        if child._runtime_cell is not self._runtime_cell:
            raise UiError(ErrorSnapshot(_native.WRONG_RUNTIME))
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(self._cell.api, runtime_token)
            with self._cell.lease() as panel_token:
                with child._cell.lease() as child_token:
                    status, error = self._cell.api.stack_panel_set_child_align(
                        panel_token, child_token, int(alignment)
                    )
        _check_result(
            self._cell.api,
            status,
            error,
            _CleanupTicket(api=self._cell.api, anchor=self._runtime_cell),
        )


@final
class Text(Widget):
    """Read-only UTF-8 text Control facade."""

    __slots__ = ()

    @classmethod
    def create(cls, runtime: Runtime, content: str = "") -> Text:
        initial = _encode_control_text(content, "content")
        return cls._create(
            runtime, lambda api, token: api.text_create(token, initial)
        )

    @property
    def content(self) -> Property:
        return self._property(
            self._cell.api.text_content_property, Property._UTF8
        )


@final
class TextBox(Widget):
    """Text input Control facade for its committed UTF-8 value."""

    __slots__ = ()

    @classmethod
    def create(cls, runtime: Runtime, value: str = "") -> TextBox:
        initial = _encode_control_text(value, "value")
        return cls._create(
            runtime, lambda api, token: api.text_box_create(token, initial)
        )

    @property
    def value(self) -> Property:
        return self._property(
            self._cell.api.text_box_value_property, Property._UTF8
        )


@final
class Button(Widget):
    """Button facade for its label and optional shared AsyncCommand."""

    __slots__ = ()

    @classmethod
    def create(cls, runtime: Runtime, label: str = "") -> Button:
        initial = _encode_control_text(label, "label")
        return cls._create(
            runtime, lambda api, token: api.button_create(token, initial)
        )

    @property
    def label(self) -> Property:
        return self._property(
            self._cell.api.button_label_property, Property._UTF8
        )

    def set_command(self, command: AsyncCommand | None) -> None:
        if command is not None and not isinstance(command, AsyncCommand):
            raise TypeError("command must be an AsyncCommand or None")
        if (
            command is not None
            and command._runtime_cell is not self._runtime_cell
        ):
            raise UiError(ErrorSnapshot(_native.WRONG_RUNTIME))
        with self._runtime_cell.lease() as runtime_token:
            _check_ui_thread(self._cell.api, runtime_token)
            with self._cell.lease() as button_token:
                if command is None:
                    status, error = self._cell.api.button_set_command(
                        button_token, 0
                    )
                else:
                    with command._cell.lease() as command_token:
                        status, error = self._cell.api.button_set_command(
                            button_token, command_token
                        )
        _check_result(
            self._cell.api,
            status,
            error,
            _CleanupTicket(api=self._cell.api, anchor=self._runtime_cell),
        )

    @property
    def command(self) -> AsyncCommand | None:
        with self._ui_call() as (api, token):
            status, command, error = api.button_command(token)
        if status != _native.OK or error:
            _dispose_unexpected(
                api,
                command,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
            _check_result(
                api,
                status,
                error,
                _CleanupTicket(api=api, anchor=self._runtime_cell),
            )
        if not command:
            return None
        return AsyncCommand._from_owned_token(api, command, self._runtime_cell)


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
    "Alignment",
    "AsyncCommand",
    "Binding",
    "BindingDirection",
    "BindingErrorSnapshot",
    "BindingMode",
    "Button",
    "CommandOperation",
    "CommandState",
    "Container",
    "CloseRequest",
    "DeferredCloseRequest",
    "ErrorSnapshot",
    "MissingRuntimeError",
    "Orientation",
    "Property",
    "Runtime",
    "StackPanel",
    "ShutdownDisposition",
    "ShutdownMode",
    "ShutdownRequest",
    "ShutdownRequestState",
    "ShutdownSnapshot",
    "Text",
    "TextBox",
    "UiContext",
    "UiError",
    "Widget",
    "Window",
    "WindowCloseSubscription",
]
