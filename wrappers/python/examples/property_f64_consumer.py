"""Consumer roundtrip for F64 Property values through the C ABI."""

import ctypes
import math
import sys

import freemvvm
from freemvvm import Property, Runtime
from freemvvm import _native


def _assert_same_f64(actual: float, expected: float) -> None:
    if math.isnan(expected):
        if not math.isnan(actual):
            raise AssertionError("NaN value did not roundtrip")
    elif expected == 0.0:
        if actual != 0.0 or math.copysign(1.0, actual) != math.copysign(
            1.0, expected
        ):
            raise AssertionError(f"signed zero did not roundtrip: {actual!r}")
    elif actual != expected:
        raise AssertionError(
            f"F64 value did not roundtrip: {actual!r} != {expected!r}"
        )


def main() -> None:
    if len(sys.argv) > 1:
        library_path = sys.argv[1]
        freemvvm._load_native_api = lambda: _native.CtypesNativeApi(
            ctypes.CDLL(library_path)
        )

    values = (1.25, -3.5, float("nan"), float("inf"), float("-inf"), 0.0, -0.0)
    with Runtime.create() as runtime:
        if not runtime.is_ui_thread():
            raise RuntimeError(
                "Runtime was not created on the current UI thread"
            )

        for value in values:
            with Property.create(runtime, value) as prop:
                _assert_same_f64(prop.get(), value)
                prop.set(7.75)
                if prop.get() != 7.75:
                    raise AssertionError("F64 update did not roundtrip")

    print("Python F64 Property C ABI consumer: OK")


if __name__ == "__main__":
    main()
