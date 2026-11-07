"""Python wrapper command handlers."""

import os
from pathlib import Path
import sys

from ..process import run


COMMANDS = (
    "python-runtime-test",
    "python-runtime-consumer-test",
    "python-property-bool-i64-test",
    "python-property-bool-i64-consumer-test",
    "python-property-utf8-test",
    "python-property-utf8-consumer-test",
    "python-property-f64-test",
    "python-property-f64-consumer-test",
    "python-property-decimal-test",
    "python-property-decimal-consumer-test",
    "python-binding-test",
    "python-binding-consumer-test",
    "python-command-test",
    "python-command-consumer-test",
    "python-controls-test",
    "python-controls-consumer-test",
    "python-input-test",
    "python-input-consumer-test",
    "python-window-test",
    "python-window-consumer-test",
    "python-window-peer-smoke-test",
)
ROOT = Path(__file__).resolve().parents[3]
_SOURCE_PATH = ROOT / "wrappers/python/src"
_TESTS_PATH = ROOT / "wrappers/python/tests"
_EXAMPLES_PATH = ROOT / "wrappers/python/examples"

_UNIT_TESTS = {
    "python-runtime-test": (
        "test_runtime.py",
        "python-runtime-unit-tests",
        "source",
    ),
    "python-property-bool-i64-test": (
        "test_property_bool_i64.py",
        "python-property-bool-i64-unit-tests",
        "source",
    ),
    "python-property-utf8-test": (
        "test_property_utf8.py",
        "python-property-utf8-unit-tests",
        "source",
    ),
    "python-property-f64-test": (
        "test_property_f64.py",
        "python-property-f64-unit-tests",
        "source",
    ),
    "python-property-decimal-test": (
        "test_property_decimal.py",
        "python-property-decimal-unit-tests",
        "source",
    ),
    "python-binding-test": (
        "test_binding.py",
        "python-binding-unit-tests",
        "source",
    ),
    "python-command-test": (
        "test_command.py",
        "python-command-unit-tests",
        "source-tests",
    ),
    "python-controls-test": (
        "test_controls.py",
        "python-controls-unit-tests",
        "source-tests",
    ),
    "python-input-test": (
        "test_input_events.py",
        "python-input-test",
        "source-tests-examples",
    ),
    "python-input-consumer-test": (
        "test_input_consumer.py",
        "python-input-consumer-test",
        "source-tests-examples",
    ),
    "python-window-test": (
        "test_window.py",
        "python-window-unit-tests",
        "source-tests",
    ),
}
_CONSUMERS = {
    "python-runtime-consumer-test": (
        "runtime_consumer.py",
        "python-runtime-consumer-native-build",
        "python-runtime-c-abi-consumer",
    ),
    "python-property-bool-i64-consumer-test": (
        "property_bool_i64_consumer.py",
        "python-property-consumer-native-build",
        "python-property-bool-i64-c-abi-consumer",
    ),
    "python-property-utf8-consumer-test": (
        "property_utf8_consumer.py",
        "python-property-consumer-native-build",
        "python-property-utf8-c-abi-consumer",
    ),
    "python-property-f64-consumer-test": (
        "property_f64_consumer.py",
        "python-property-consumer-native-build",
        "python-property-f64-c-abi-consumer",
    ),
    "python-property-decimal-consumer-test": (
        "property_decimal_consumer.py",
        "python-property-consumer-native-build",
        "python-property-decimal-c-abi-consumer",
    ),
    "python-binding-consumer-test": (
        "binding_consumer.py",
        "python-binding-consumer-native-build",
        "python-binding-c-abi-consumer",
    ),
    "python-command-consumer-test": (
        "command_consumer.py",
        "python-command-consumer-native-build",
        "python-command-c-abi-consumer",
    ),
    "python-controls-consumer-test": (
        "controls_consumer.py",
        "python-controls-consumer-native-build",
        "python-controls-c-abi-consumer",
    ),
    "python-window-consumer-test": (
        "window_consumer.py",
        "python-window-consumer-native-build",
        "python-window-c-abi-consumer",
    ),
}


def _native_path(environment):
    target_dir = Path(environment.get("CARGO_TARGET_DIR", ROOT / "target"))
    if not target_dir.is_absolute():
        target_dir = ROOT / target_dir
    native_name = (
        "freemvvm_abi_1.dll"
        if os.name == "nt"
        else "libfreemvvm_abi_1.dylib"
        if sys.platform == "darwin"
        else "libfreemvvm_abi_1.so"
    )
    native_path = target_dir / "debug" / native_name
    if not native_path.is_file():
        raise RuntimeError(f"Host C ABI library was not built: {native_path}")
    return native_path


def _python_path(environment, parts):
    paths = {
        "source": _SOURCE_PATH,
        "tests": _TESTS_PATH,
        "examples": _EXAMPLES_PATH,
    }
    child_environment = dict(environment)
    child_environment["PYTHONPATH"] = os.pathsep.join(
        [
            *(str(paths[part]) for part in parts),
            environment.get("PYTHONPATH", ""),
        ]
    ).rstrip(os.pathsep)
    return child_environment


def _handle_unit_test(command, environment, results):
    test_file, gate, path_group = _UNIT_TESTS[command]
    path_parts = {
        "source": ("source",),
        "source-tests": ("source", "tests"),
        "source-tests-examples": ("source", "tests", "examples"),
    }[path_group]
    child_environment = _python_path(environment, path_parts)
    run(
        [
            sys.executable,
            "-m",
            "pytest",
            f"wrappers/python/tests/{test_file}",
        ],
        child_environment,
        results,
        gate,
    )


def _handle_consumer(command, environment, results):
    example, native_build_gate, consumer_gate = _CONSUMERS[command]
    run(
        ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
        environment,
        results,
        native_build_gate,
    )
    native_path = _native_path(environment)
    child_environment = _python_path(environment, ("source",))
    run(
        [
            sys.executable,
            f"wrappers/python/examples/{example}",
            str(native_path.resolve()),
        ],
        child_environment,
        results,
        consumer_gate,
    )


def _handle_window_peer_smoke(environment, results):
    if not sys.platform.startswith("linux"):
        raise RuntimeError("Window peer smoke requires a Linux Platform host")
    run(
        ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
        environment,
        results,
        "python-window-peer-smoke-native-build",
    )
    native_path = _native_path(environment)
    child_environment = _python_path(environment, ("source",))
    run(
        [
            sys.executable,
            "wrappers/python/examples/window_peer_smoke.py",
            str(native_path.resolve()),
        ],
        child_environment,
        results,
        "python-window-peer-platform-smoke",
    )


def handle(command, environment, results):
    """Run a Python wrapper test or consumer command if recognized."""
    if command not in COMMANDS:
        return False
    if command in _UNIT_TESTS:
        _handle_unit_test(command, environment, results)
    elif command in _CONSUMERS:
        _handle_consumer(command, environment, results)
    else:
        _handle_window_peer_smoke(environment, results)
    return True


SCOPES = {
    "python-runtime-test": "python-runtime-unit-test",
    "python-runtime-consumer-test": "python-runtime-consumer-test",
    "python-property-bool-i64-test": "python-property-bool-i64-unit-test",
    "python-property-bool-i64-consumer-test": (
        "python-property-bool-i64-consumer-test"
    ),
    "python-property-utf8-test": "python-property-utf8-unit-test",
    "python-property-utf8-consumer-test": "python-property-utf8-consumer-test",
    "python-property-f64-test": "python-property-f64-unit-test",
    "python-property-f64-consumer-test": "python-property-f64-consumer-test",
    "python-property-decimal-test": "python-property-decimal-unit-test",
    "python-property-decimal-consumer-test": (
        "python-property-decimal-consumer-test"
    ),
    "python-binding-test": "python-binding-unit-test",
    "python-binding-consumer-test": "python-binding-consumer-test",
    "python-command-test": "python-command-unit-test",
    "python-command-consumer-test": "python-command-consumer-test",
    "python-controls-test": "python-controls-unit-test",
    "python-input-test": "python-input-event-wrapper-unit-test",
    "python-input-consumer-test": "python-input-event-consumer-test",
    "python-controls-consumer-test": "python-controls-consumer-test",
    "python-window-test": "python-window-unit-test",
    "python-window-consumer-test": "python-window-consumer-test",
    "python-window-peer-smoke-test": "python-window-peer-smoke-test",
}
