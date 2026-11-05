"""Minimal consumer using the public Python Runtime API and installed C ABI."""

import ctypes
import sys

import freemvvm
from freemvvm import Runtime, ShutdownMode, UiError
from freemvvm import _native


def main() -> None:
    if len(sys.argv) > 1:
        library_path = sys.argv[1]
        freemvvm._load_native_api = lambda: _native.CtypesNativeApi(
            ctypes.CDLL(library_path)
        )
    runtime = Runtime.create()
    if not runtime.is_ui_thread():
        raise RuntimeError("Runtime was not created on the current UI thread")
    runtime.ui_context().check()
    try:
        request = runtime.request_shutdown(ShutdownMode.ASYNC)
    except UiError as error:
        if (
            error.raw_status != _native.UNSUPPORTED
        ):  # This consumer has no host loop bridge.
            raise
        print("Host loop bridge unavailable; shutdown was not accepted")
    else:
        snapshot = request.query()
        print(f"shutdown request state: {snapshot.known_state!r}")
        request.close()
    runtime.close()
    print("Python Runtime C ABI consumer: OK")


if __name__ == "__main__":
    main()
