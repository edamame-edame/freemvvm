"""Consumer roundtrip for UTF-8 Property values through the C ABI."""

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

    values = ("", "ASCII", "日本語", "絵文字🙂", "before\0after")
    with Runtime.create() as runtime:
        if not runtime.is_ui_thread():
            raise RuntimeError(
                "Runtime was not created on the current UI thread"
            )

        for value in values:
            with Property.create(runtime, value) as prop:
                if prop.get() != value:
                    raise AssertionError(
                        f"UTF-8 initial value did not roundtrip: {value!r}"
                    )
                updated = value + " 更新"
                prop.set(updated)
                if prop.get() != updated:
                    raise AssertionError(
                        f"UTF-8 update did not roundtrip: {updated!r}"
                    )

    print("Python UTF-8 Property C ABI consumer: OK")


if __name__ == "__main__":
    main()
