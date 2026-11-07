"""C# wrapper command handlers."""

import os
from pathlib import Path
import sys

from ..process import run


COMMANDS = (
    "csharp-runtime-test",
    "csharp-runtime-consumer-test",
    "csharp-property-test",
    "csharp-property-consumer-test",
    "csharp-command-test",
    "csharp-command-consumer-test",
    "csharp-controls-test",
    "csharp-controls-consumer-test",
    "csharp-window-test",
    "csharp-window-consumer-test",
    "csharp-input-test",
    "csharp-input-consumer-test",
)
ROOT = Path(__file__).resolve().parents[3]
_RUNTIME_PROJECT = "wrappers/csharp/tests/RuntimeTests/RuntimeTests.csproj"
_CONSUMER_PROJECT = "wrappers/csharp/tests/Consumer/Consumer.csproj"

_UNIT_LABELS = {
    "csharp-runtime-test": "runtime",
    "csharp-property-test": "property",
    "csharp-command-test": "command",
    "csharp-controls-test": "controls",
    "csharp-input-test": "input",
    "csharp-window-test": "window",
}
_CONSUMER_LABELS = {
    "csharp-runtime-consumer-test": "runtime",
    "csharp-property-consumer-test": "property",
    "csharp-command-consumer-test": "command",
    "csharp-controls-consumer-test": "controls",
    "csharp-input-consumer-test": "input",
    "csharp-window-consumer-test": "window",
}


def _native_library_path():
    native_name = (
        "freemvvm_abi_1.dll"
        if os.name == "nt"
        else "libfreemvvm_abi_1.dylib"
        if sys.platform == "darwin"
        else "libfreemvvm_abi_1.so"
    )
    native_path = ROOT / "target/debug" / native_name
    if not native_path.is_file():
        raise RuntimeError(f"Host C ABI library was not built: {native_path}")
    return native_path


def _run_unit_test(command, environment, results):
    label = _UNIT_LABELS[command]
    run(
        ["dotnet", "restore", _RUNTIME_PROJECT, "--locked-mode"],
        environment,
        results,
        f"csharp-{label}-test-restore",
    )
    run(
        [
            "dotnet",
            "run",
            "--project",
            _RUNTIME_PROJECT,
            "--configuration",
            "Debug",
            "--no-restore",
        ],
        environment,
        results,
        f"csharp-{label}-unit-tests",
    )


def _run_consumer_test(command, environment, results):
    label = _CONSUMER_LABELS[command]
    native_build_gate = (
        "csharp-consumer-native-build"
        if label == "runtime"
        else f"csharp-{label}-consumer-native-build"
    )
    restore_gate = (
        "csharp-consumer-restore"
        if label == "runtime"
        else f"csharp-{label}-consumer-restore"
    )
    run(
        ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
        environment,
        results,
        native_build_gate,
    )
    run(
        ["dotnet", "restore", _CONSUMER_PROJECT, "--locked-mode"],
        environment,
        results,
        restore_gate,
    )
    native_path = _native_library_path()
    consumer_args = [
        "dotnet",
        "run",
        "--project",
        _CONSUMER_PROJECT,
        "--configuration",
        "Debug",
        "--no-restore",
        "--",
    ]
    if label == "input":
        consumer_args.append("--input-only")
    consumer_args.append(str(native_path.resolve()))
    consumer_gate = (
        "csharp-runtime-c-abi-consumer"
        if label == "runtime"
        else f"csharp-{label}-c-abi-consumer"
    )
    run(consumer_args, environment, results, consumer_gate)


def handle(command, environment, results):
    """Run a C# wrapper test or consumer command if recognized."""
    if command not in COMMANDS:
        return False
    if command in _UNIT_LABELS:
        _run_unit_test(command, environment, results)
    else:
        _run_consumer_test(command, environment, results)
    return True
