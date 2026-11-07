"""Rust wrapper command handlers."""

import os
from pathlib import Path
import sys

from ..process import run


COMMANDS = (
    "rust-wrapper-runtime-test",
    "rust-wrapper-runtime-consumer-test",
    "rust-wrapper-ui-controls-test",
    "rust-wrapper-window-close-test",
    "rust-wrapper-property-bool-i64-test",
    "rust-wrapper-property-utf8-test",
    "rust-wrapper-property-f64-test",
    "rust-wrapper-property-decimal-test",
)
ROOT = Path(__file__).resolve().parents[3]


def _native_environment(environment):
    target_dir = Path(environment.get("CARGO_TARGET_DIR", ROOT / "target"))
    if not target_dir.is_absolute():
        target_dir = ROOT / target_dir
    native_dir = target_dir / "debug"
    child_environment = dict(environment)
    child_environment["FREEMVVM_NATIVE_LIB_DIR"] = str(native_dir)
    if os.name == "nt":
        child_environment["PATH"] = (
            str(native_dir) + os.pathsep + child_environment["PATH"]
        )
    elif sys.platform == "darwin":
        child_environment["DYLD_LIBRARY_PATH"] = (
            str(native_dir)
            + os.pathsep
            + child_environment.get("DYLD_LIBRARY_PATH", "")
        )
    else:
        child_environment["LD_LIBRARY_PATH"] = (
            str(native_dir)
            + os.pathsep
            + child_environment.get("LD_LIBRARY_PATH", "")
        )
    return child_environment


def handle(command, environment, results):
    """Run a Rust wrapper test or consumer command if recognized."""
    if command not in COMMANDS:
        return False

    run(
        ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
        environment,
        results,
        "rust-wrapper-native-build",
    )
    native_environment = _native_environment(environment)

    if command == "rust-wrapper-window-close-test":
        run(
            ["cargo", "test", "--locked", "-p", "freemvvm"],
            native_environment,
            results,
            "rust-wrapper-window-close-tests",
        )
        run(
            [
                "cargo",
                "run",
                "--locked",
                "-p",
                "freemvvm",
                "--example",
                "window_close_consumer",
            ],
            native_environment,
            results,
            "rust-wrapper-window-close-consumer",
        )
    elif command == "rust-wrapper-ui-controls-test":
        run(
            ["cargo", "test", "--locked", "-p", "freemvvm"],
            native_environment,
            results,
            "rust-wrapper-ui-controls-tests",
        )
        run(
            [
                "cargo",
                "run",
                "--locked",
                "-p",
                "freemvvm",
                "--example",
                "ui_controls_consumer",
            ],
            native_environment,
            results,
            "rust-wrapper-ui-controls-consumer",
        )
    elif command == "rust-wrapper-runtime-test":
        run(
            ["cargo", "test", "--locked", "-p", "freemvvm"],
            native_environment,
            results,
            "rust-wrapper-runtime-tests",
        )
    else:
        example, gate = {
            "rust-wrapper-runtime-consumer-test": (
                "runtime_consumer",
                "rust-wrapper-runtime-consumer",
            ),
            "rust-wrapper-property-bool-i64-test": (
                "property_bool_i64_consumer",
                "rust-wrapper-property-bool-i64-consumer",
            ),
            "rust-wrapper-property-utf8-test": (
                "property_utf8_consumer",
                "rust-wrapper-property-utf8-consumer",
            ),
            "rust-wrapper-property-f64-test": (
                "property_f64_consumer",
                "rust-wrapper-property-f64-consumer",
            ),
            "rust-wrapper-property-decimal-test": (
                "property_decimal_consumer",
                "rust-wrapper-property-decimal-consumer",
            ),
        }[command]
        run(
            [
                "cargo",
                "run",
                "--locked",
                "-p",
                "freemvvm",
                "--example",
                example,
            ],
            native_environment,
            results,
            gate,
        )
    return True


SCOPES = {
    "rust-wrapper-runtime-test": "rust-wrapper-runtime-unit-test",
    "rust-wrapper-runtime-consumer-test": "rust-wrapper-runtime-consumer-test",
    "rust-wrapper-ui-controls-test": "rust-wrapper-ui-controls-test",
    "rust-wrapper-window-close-test": "rust-wrapper-window-close-test",
    "rust-wrapper-property-bool-i64-test": (
        "rust-wrapper-property-bool-i64-consumer-test"
    ),
    "rust-wrapper-property-utf8-test": (
        "rust-wrapper-property-utf8-consumer-test"
    ),
    "rust-wrapper-property-f64-test": (
        "rust-wrapper-property-f64-consumer-test"
    ),
    "rust-wrapper-property-decimal-test": (
        "rust-wrapper-property-decimal-consumer-test"
    ),
}
