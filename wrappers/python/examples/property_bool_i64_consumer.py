"""Consumer roundtrip for BOOL and signed-I64 Property values."""

import ctypes
import sys

import freemvvm
from freemvvm import Property, Runtime
from freemvvm import _native


def main() -> None:
    if len(sys.argv) > 1:
        library_path = sys.argv[1]
        freemvvm._load_native_api = lambda: _native.CtypesNativeApi(
            ctypes.CDLL(library_path)
        )

    with Runtime.create() as runtime:
        if not runtime.is_ui_thread():
            raise RuntimeError(
                "Runtime was not created on the current UI thread"
            )

        with Property.create(runtime, False) as bool_property:
            if bool_property.get() is not False:
                raise AssertionError("BOOL initial value did not roundtrip")
            bool_property.set(True)
            if bool_property.get() is not True:
                raise AssertionError("BOOL update did not roundtrip")
            bool_property.set(False)
            if bool_property.get() is not False:
                raise AssertionError("BOOL false update did not roundtrip")

        for value in (0, -(1 << 63), (1 << 63) - 1):
            with Property.create(runtime, value) as i64_property:
                if i64_property.get() != value:
                    raise AssertionError(
                        f"I64 initial value did not roundtrip: {value}"
                    )
                updated = (
                    (1 << 63) - 1 if value != (1 << 63) - 1 else -(1 << 63)
                )
                i64_property.set(updated)
                if i64_property.get() != updated:
                    raise AssertionError(
                        f"I64 update did not roundtrip: {updated}"
                    )

    print("Python BOOL/I64 Property C ABI consumer: OK")


if __name__ == "__main__":
    main()
