"""Safe Python facade for the FreeMvvm versioned C ABI."""

from __future__ import annotations

from contextlib import contextmanager
from dataclasses import dataclass
from decimal import Decimal
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


class BindingMode(IntEnum):
    ONE_TIME = 1
    ONE_WAY = 2
    TWO_WAY = 3


class BindingDirection(IntEnum):
    SOURCE_TO_TARGET = 1
    TARGET_TO_SOURCE = 2


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
    "Binding",
    "BindingDirection",
    "BindingErrorSnapshot",
    "BindingMode",
    "ErrorSnapshot",
    "MissingRuntimeError",
    "Property",
    "Runtime",
    "ShutdownDisposition",
    "ShutdownMode",
    "ShutdownRequest",
    "ShutdownRequestState",
    "ShutdownSnapshot",
    "UiContext",
    "UiError",
]
