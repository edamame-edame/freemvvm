"""Consumer roundtrip for Python DECIMAL Property values through the C ABI."""

from decimal import Decimal
import ctypes
import sys

import freemvvm
from freemvvm import Property, Runtime
from freemvvm import _native


def _assert_decimal_tuple(actual: Decimal, expected: Decimal) -> None:
    if actual.as_tuple() != expected.as_tuple():
        raise AssertionError(
            f"Decimal representation mismatch: {actual.as_tuple()} "
            f"!= {expected.as_tuple()}"
        )


def main() -> None:
    if len(sys.argv) > 1:
        library_path = sys.argv[1]
        freemvvm._load_native_api = lambda: _native.CtypesNativeApi(
            ctypes.CDLL(library_path)
        )

    roundtrips = (
        (Decimal("0"), Decimal("0")),
        (Decimal("-0.00"), Decimal("-0.00")),
        (Decimal("1.00"), Decimal("1.00")),
        (Decimal("1E-31"), Decimal("1E-31")),
        (Decimal("10E-32"), Decimal("1E-31")),
        (
            Decimal("288230376151711743.0"),
            Decimal("288230376151711743"),
        ),
        (Decimal("-288230376151711743"), Decimal("-288230376151711743")),
    )
    with Runtime.create() as runtime:
        if not runtime.is_ui_thread():
            raise RuntimeError(
                "Runtime was not created on the current UI thread"
            )

        for initial, expected in roundtrips:
            with Property.create(runtime, initial) as prop:
                _assert_decimal_tuple(prop.get(), expected)
                prop.set(expected)
                _assert_decimal_tuple(prop.get(), expected)

        with Property.create(runtime, Decimal("1.0")) as prop:
            prop.set(Decimal("1.00"))
            _assert_decimal_tuple(prop.get(), Decimal("1.0"))

        with Property.create(runtime, Decimal("-0.00")) as prop:
            prop.set(Decimal("0.0"))
            _assert_decimal_tuple(prop.get(), Decimal("-0.00"))

    print("Python DECIMAL Property C ABI consumer: OK")


if __name__ == "__main__":
    main()
