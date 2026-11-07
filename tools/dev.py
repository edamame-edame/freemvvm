"""Run repository-local bootstrap gates; these are not product unit tests."""

import argparse
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import sysconfig

from dev_support.environment import bootstrap_environment
from dev_support.commands.abi import COMMANDS as ABI_COMMANDS
from dev_support.commands.abi import handle as handle_abi_command
from dev_support.commands.cpp import COMMANDS as CPP_COMMANDS
from dev_support.commands.cpp import handle as handle_cpp_command
from dev_support.commands.csharp import COMMANDS as CSHARP_COMMANDS
from dev_support.commands.csharp import handle as handle_csharp_command
from dev_support.commands.platform import COMMANDS as PLATFORM_COMMANDS
from dev_support.commands.platform import (
    run_linux_platform_docker,
)
from dev_support.commands.rust import COMMANDS as RUST_COMMANDS
from dev_support.commands.rust import handle as handle_rust_command
from dev_support.process import capture, msvc_environment, run
from dev_support.reports import write_json_report


ROOT = Path(__file__).resolve().parents[1]
VENV = ROOT / ".venv"
BIN = VENV / ("Scripts" if os.name == "nt" else "bin")
REPORT = ROOT / ".local/verification/bootstrap.json"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command",
        choices=[
            "doctor",
            "check",
            *ABI_COMMANDS,
            *CPP_COMMANDS,
            *RUST_COMMANDS,
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
            *CSHARP_COMMANDS,
            "format",
            "minimum",
            *PLATFORM_COMMANDS,
        ],
    )
    args = parser.parse_args()
    if args.command in PLATFORM_COMMANDS:
        return run_linux_platform_docker()
    if Path(sys.prefix).resolve() != VENV.resolve():
        parser.error(
            "Run with the repository .venv Python; see DEVELOPMENT.md"
        )
    if sys.version_info[:2] != (3, 13) or sysconfig.get_config_var(
        "Py_GIL_DISABLED"
    ):
        parser.error("Bootstrap requires the standard CPython 3.13 build")
    environment = bootstrap_environment(os.environ, BIN, os.pathsep)
    results = []
    report = {
        "scope": (
            "abi-command-smoke"
            if args.command == "abi-command-smoke"
            else "rust-wrapper-runtime-unit-test"
            if args.command == "rust-wrapper-runtime-test"
            else "rust-wrapper-runtime-consumer-test"
            if args.command == "rust-wrapper-runtime-consumer-test"
            else "rust-wrapper-ui-controls-test"
            if args.command == "rust-wrapper-ui-controls-test"
            else "rust-wrapper-window-close-test"
            if args.command == "rust-wrapper-window-close-test"
            else "rust-wrapper-property-bool-i64-consumer-test"
            if args.command == "rust-wrapper-property-bool-i64-test"
            else "rust-wrapper-property-utf8-consumer-test"
            if args.command == "rust-wrapper-property-utf8-test"
            else "rust-wrapper-property-f64-consumer-test"
            if args.command == "rust-wrapper-property-f64-test"
            else "rust-wrapper-property-decimal-consumer-test"
            if args.command == "rust-wrapper-property-decimal-test"
            else "python-runtime-unit-test"
            if args.command == "python-runtime-test"
            else "python-runtime-consumer-test"
            if args.command == "python-runtime-consumer-test"
            else "python-property-bool-i64-unit-test"
            if args.command == "python-property-bool-i64-test"
            else "python-property-bool-i64-consumer-test"
            if args.command == "python-property-bool-i64-consumer-test"
            else "python-property-utf8-unit-test"
            if args.command == "python-property-utf8-test"
            else "python-property-utf8-consumer-test"
            if args.command == "python-property-utf8-consumer-test"
            else "python-property-f64-unit-test"
            if args.command == "python-property-f64-test"
            else "python-property-f64-consumer-test"
            if args.command == "python-property-f64-consumer-test"
            else "python-property-decimal-unit-test"
            if args.command == "python-property-decimal-test"
            else "python-property-decimal-consumer-test"
            if args.command == "python-property-decimal-consumer-test"
            else "python-binding-unit-test"
            if args.command == "python-binding-test"
            else "python-binding-consumer-test"
            if args.command == "python-binding-consumer-test"
            else "python-command-unit-test"
            if args.command == "python-command-test"
            else "python-command-consumer-test"
            if args.command == "python-command-consumer-test"
            else "python-controls-unit-test"
            if args.command == "python-controls-test"
            else "python-input-event-wrapper-unit-test"
            if args.command == "python-input-test"
            else "python-input-event-consumer-test"
            if args.command == "python-input-consumer-test"
            else "python-controls-consumer-test"
            if args.command == "python-controls-consumer-test"
            else "python-window-unit-test"
            if args.command == "python-window-test"
            else "python-window-consumer-test"
            if args.command == "python-window-consumer-test"
            else "python-window-peer-smoke-test"
            if args.command == "python-window-peer-smoke-test"
            else "csharp-runtime-unit-test"
            if args.command == "csharp-runtime-test"
            else "csharp-property-unit-test"
            if args.command == "csharp-property-test"
            else "csharp-command-unit-test"
            if args.command == "csharp-command-test"
            else "csharp-controls-unit-test"
            if args.command == "csharp-controls-test"
            else "csharp-input-unit-test"
            if args.command == "csharp-input-test"
            else "csharp-controls-consumer-test"
            if args.command == "csharp-controls-consumer-test"
            else "csharp-input-consumer-test"
            if args.command == "csharp-input-consumer-test"
            else "csharp-window-unit-test"
            if args.command == "csharp-window-test"
            else "csharp-window-consumer-test"
            if args.command == "csharp-window-consumer-test"
            else "csharp-command-consumer-test"
            if args.command == "csharp-command-consumer-test"
            else "csharp-runtime-consumer-test"
            if args.command == "csharp-runtime-consumer-test"
            else "csharp-property-consumer-test"
            if args.command == "csharp-property-consumer-test"
            else "cpp-package-integration-test"
            if args.command == "cpp-package-test"
            else "cpp-property-unit-test"
            if args.command == "cpp-property-test"
            else "cpp-property-utf8-unit-test"
            if args.command == "cpp-property-utf8-test"
            else "cpp-property-f64-unit-test"
            if args.command == "cpp-property-f64-test"
            else "cpp-property-decimal-unit-test"
            if args.command == "cpp-property-decimal-test"
            else "cpp-property-subscription-unit-test"
            if args.command == "cpp-property-subscription-test"
            else "cpp-property-binding-unit-test"
            if args.command == "cpp-property-binding-test"
            else "cpp-command-unit-test"
            if args.command == "cpp-command-test"
            else "cpp-runtime-unit-test"
            if args.command == "cpp-runtime-test"
            else "development-bootstrap"
        ),
        "product_tests": (
            "Rust Runtime safe wrapper unit tests executed"
            if args.command == "rust-wrapper-runtime-test"
            else "Rust Runtime consumer test executed against host C ABI"
            if args.command == "rust-wrapper-runtime-consumer-test"
            else "Rust UI Controls focused and C ABI consumer tests executed"
            if args.command == "rust-wrapper-ui-controls-test"
            else (
                "Rust Window close-request focused and C ABI consumer tests "
                "executed"
            )
            if args.command == "rust-wrapper-window-close-test"
            else "Rust BOOL/I64 Property consumer test executed"
            if args.command == "rust-wrapper-property-bool-i64-test"
            else "Rust UTF-8 Property consumer test executed"
            if args.command == "rust-wrapper-property-utf8-test"
            else "Rust F64 Property consumer test executed"
            if args.command == "rust-wrapper-property-f64-test"
            else "Rust DECIMAL Property consumer test executed"
            if args.command == "rust-wrapper-property-decimal-test"
            else "Python Runtime wrapper unit tests executed"
            if args.command == "python-runtime-test"
            else "Python Runtime consumer test executed against host C ABI"
            if args.command == "python-runtime-consumer-test"
            else "Python BOOL/I64 Property wrapper unit tests executed"
            if args.command == "python-property-bool-i64-test"
            else (
                "Python BOOL/I64 Property consumer test executed "
                "against host C ABI"
            )
            if args.command == "python-property-bool-i64-consumer-test"
            else "Python UTF-8 Property wrapper unit tests executed"
            if args.command == "python-property-utf8-test"
            else (
                "Python UTF-8 Property consumer test executed "
                "against host C ABI"
            )
            if args.command == "python-property-utf8-consumer-test"
            else "Python F64 Property wrapper unit tests executed"
            if args.command == "python-property-f64-test"
            else (
                "Python F64 Property consumer test executed against host C ABI"
            )
            if args.command == "python-property-f64-consumer-test"
            else "Python DECIMAL Property wrapper unit tests executed"
            if args.command == "python-property-decimal-test"
            else (
                "Python DECIMAL Property consumer test executed "
                "against host C ABI"
            )
            if args.command == "python-property-decimal-consumer-test"
            else "Python Binding wrapper unit tests executed"
            if args.command == "python-binding-test"
            else "Python Binding consumer test executed against host C ABI"
            if args.command == "python-binding-consumer-test"
            else "Python AsyncCommand wrapper unit tests executed"
            if args.command == "python-command-test"
            else (
                "Python AsyncCommand consumer test executed against host C ABI"
            )
            if args.command == "python-command-consumer-test"
            else "Python Controls wrapper unit tests executed"
            if args.command == "python-controls-test"
            else "Python Input event wrapper unit tests executed"
            if args.command == "python-input-test"
            else "Python Input event consumer test executed with a fake host"
            if args.command == "python-input-consumer-test"
            else "Python Controls consumer test executed against host C ABI"
            if args.command == "python-controls-consumer-test"
            else "Python Window peer/close-request wrapper unit tests executed"
            if args.command == "python-window-test"
            else (
                "Python Window peer/close-request consumer test executed "
                "against host C ABI"
            )
            if args.command == "python-window-consumer-test"
            else (
                "Python Window peer open/close smoke executed against "
                "the configured host Platform"
            )
            if args.command == "python-window-peer-smoke-test"
            else "C# Runtime wrapper unit tests executed"
            if args.command == "csharp-runtime-test"
            else "C# Property wrapper unit tests executed"
            if args.command == "csharp-property-test"
            else "C# asynchronous Command wrapper unit tests executed"
            if args.command == "csharp-command-test"
            else "C# UI Control wrapper unit tests executed"
            if args.command == "csharp-controls-test"
            else "C# Input event wrapper unit tests executed"
            if args.command == "csharp-input-test"
            else "C# UI Control consumer test executed against host C ABI"
            if args.command == "csharp-controls-consumer-test"
            else "C# Input event consumer test executed against host C ABI"
            if args.command == "csharp-input-consumer-test"
            else "C# Window wrapper unit tests executed"
            if args.command == "csharp-window-test"
            else "C# Window consumer test executed against host C ABI"
            if args.command == "csharp-window-consumer-test"
            else "C# Command consumer test executed against host C ABI"
            if args.command == "csharp-command-consumer-test"
            else "C# Runtime consumer test executed against the host C ABI"
            if args.command == "csharp-runtime-consumer-test"
            else "C# Property consumer test executed against host C ABI"
            if args.command == "csharp-property-consumer-test"
            else "cpp package consumer test executed"
            if args.command == "cpp-package-test"
            else "C++ Property BOOL/I64 wrapper unit test executed"
            if args.command == "cpp-property-test"
            else "C++ Property UTF-8 wrapper unit test executed"
            if args.command == "cpp-property-utf8-test"
            else "C++ Property F64 wrapper unit test executed"
            if args.command == "cpp-property-f64-test"
            else "C++ Property DECIMAL wrapper unit test executed"
            if args.command == "cpp-property-decimal-test"
            else "C++ Property subscription unit test executed"
            if args.command == "cpp-property-subscription-test"
            else "C++ Property Binding wrapper unit test executed"
            if args.command == "cpp-property-binding-test"
            else "C++ asynchronous Command wrapper unit tests executed"
            if args.command == "cpp-command-test"
            else "C ABI asynchronous Command consumer smoke executed"
            if args.command == "abi-command-smoke"
            else "C++/C# Runtime and package integration tests executed"
            if args.command == "check"
            else "cpp-runtime unit test executed"
            if args.command == "cpp-runtime-test"
            else "not implemented"
        ),
        "host": platform.platform(),
        "python": platform.python_version(),
        "results": results,
        "status": "failed",
    }
    try:
        environment = msvc_environment(environment)
        # VsDevCmd may prepend tool paths; keep the pinned CMake/Ninja first.
        environment["PATH"] = str(BIN) + os.pathsep + environment["PATH"]
        for tool in [
            "rustc",
            "cargo",
            "cmake",
            "ninja",
            "ctest",
            "dotnet",
            "git",
            "ruff",
        ]:
            if not shutil.which(tool, path=environment["PATH"]):
                raise RuntimeError(
                    f"Required executable missing from child PATH: {tool}"
                )
        versions = {
            "rust": capture(["rustc", "--version"], environment),
            "cargo": capture(["cargo", "--version"], environment),
            "cmake": capture(["cmake", "--version"], environment).splitlines()[
                0
            ],
            "ninja": capture(["ninja", "--version"], environment),
            "dotnet": capture(["dotnet", "--version"], environment),
            "pytest": importlib.metadata.version("pytest"),
            "ruff": importlib.metadata.version("ruff"),
        }
        report["tools"] = versions
        if os.name == "nt":
            report["compiler_environment"] = {
                "msvc_tools": environment.get("VCTOOLSVERSION"),
                "windows_sdk": environment.get("WINDOWSSDKVERSION"),
                "target_arch": environment.get("VSCMD_ARG_TGT_ARCH"),
            }
        else:
            report["compiler_environment"] = {
                "cxx": capture(["c++", "--version"], environment).splitlines()[
                    0
                ]
            }
        print(json.dumps(versions, indent=2), flush=True)
        if not versions["rust"].startswith("rustc 1.90.0 "):
            raise RuntimeError(
                "Use the rust-toolchain.toml pin; check RUSTUP_TOOLCHAIN"
            )
        for package in [
            "cmake",
            "ninja",
            "build",
            "pytest",
            "setuptools",
            "wheel",
            "ruff",
        ]:
            pins = (ROOT / "tools/requirements-dev.in").read_text(
                encoding="utf-8"
            )
            expected = next(
                line.split("==")[1]
                for line in pins.splitlines()
                if line.startswith(package + "==")
            )
            if importlib.metadata.version(package) != expected:
                raise RuntimeError(
                    f"Install the pinned development requirements: {package}"
                )
        expected_sdk = json.loads(
            (ROOT / "global.json").read_text(encoding="utf-8")
        )["sdk"]["version"]
        if versions["dotnet"] != expected_sdk:
            raise RuntimeError("The exact .NET SDK in global.json is required")
        if args.command == "format":
            run(["cargo", "fmt", "--all"], environment, results, "rust-format")
            run(
                ["ruff", "format", "tools", "wrappers/python/src"],
                environment,
                results,
                "python-format",
            )
        elif args.command in RUST_COMMANDS:
            handle_rust_command(args.command, environment, results)
        elif args.command in CSHARP_COMMANDS:
            handle_csharp_command(args.command, environment, results)
        elif args.command == "python-runtime-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, test_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_runtime.py",
                ],
                test_environment,
                results,
                "python-runtime-unit-tests",
            )
        elif args.command == "python-runtime-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-runtime-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
            if not target_dir.is_absolute():
                target_dir = ROOT / target_dir
            native_dir = target_dir / "debug"
            native_name = (
                "freemvvm_abi_1.dll"
                if os.name == "nt"
                else "libfreemvvm_abi_1.dylib"
                if sys.platform == "darwin"
                else "libfreemvvm_abi_1.so"
            )
            native_path = native_dir / native_name
            if not native_path.is_file():
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/runtime_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-runtime-c-abi-consumer",
            )
        elif args.command == "python-property-bool-i64-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, test_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_property_bool_i64.py",
                ],
                test_environment,
                results,
                "python-property-bool-i64-unit-tests",
            )
        elif args.command == "python-property-bool-i64-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-property-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
            if not target_dir.is_absolute():
                target_dir = ROOT / target_dir
            native_dir = target_dir / "debug"
            native_name = (
                "freemvvm_abi_1.dll"
                if os.name == "nt"
                else "libfreemvvm_abi_1.dylib"
                if sys.platform == "darwin"
                else "libfreemvvm_abi_1.so"
            )
            native_path = native_dir / native_name
            if not native_path.is_file():
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/property_bool_i64_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-property-bool-i64-c-abi-consumer",
            )
        elif args.command == "python-property-utf8-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, test_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_property_utf8.py",
                ],
                test_environment,
                results,
                "python-property-utf8-unit-tests",
            )
        elif args.command == "python-property-utf8-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-property-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
            if not target_dir.is_absolute():
                target_dir = ROOT / target_dir
            native_dir = target_dir / "debug"
            native_name = (
                "freemvvm_abi_1.dll"
                if os.name == "nt"
                else "libfreemvvm_abi_1.dylib"
                if sys.platform == "darwin"
                else "libfreemvvm_abi_1.so"
            )
            native_path = native_dir / native_name
            if not native_path.is_file():
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/property_utf8_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-property-utf8-c-abi-consumer",
            )
        elif args.command == "python-property-f64-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, test_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_property_f64.py",
                ],
                test_environment,
                results,
                "python-property-f64-unit-tests",
            )
        elif args.command == "python-property-f64-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-property-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
            if not target_dir.is_absolute():
                target_dir = ROOT / target_dir
            native_dir = target_dir / "debug"
            native_name = (
                "freemvvm_abi_1.dll"
                if os.name == "nt"
                else "libfreemvvm_abi_1.dylib"
                if sys.platform == "darwin"
                else "libfreemvvm_abi_1.so"
            )
            native_path = native_dir / native_name
            if not native_path.is_file():
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/property_f64_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-property-f64-c-abi-consumer",
            )
        elif args.command == "python-property-decimal-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, test_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_property_decimal.py",
                ],
                test_environment,
                results,
                "python-property-decimal-unit-tests",
            )
        elif args.command == "python-property-decimal-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-property-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
            if not target_dir.is_absolute():
                target_dir = ROOT / target_dir
            native_dir = target_dir / "debug"
            native_name = (
                "freemvvm_abi_1.dll"
                if os.name == "nt"
                else "libfreemvvm_abi_1.dylib"
                if sys.platform == "darwin"
                else "libfreemvvm_abi_1.so"
            )
            native_path = native_dir / native_name
            if not native_path.is_file():
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/property_decimal_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-property-decimal-c-abi-consumer",
            )
        elif args.command == "python-binding-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, test_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_binding.py",
                ],
                test_environment,
                results,
                "python-binding-unit-tests",
            )
        elif args.command == "python-binding-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-binding-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
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
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/binding_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-binding-c-abi-consumer",
            )
        elif args.command == "python-command-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            tests_path = str(ROOT / "wrappers/python/tests")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [
                    source_path,
                    tests_path,
                    test_environment.get("PYTHONPATH", ""),
                ]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_command.py",
                ],
                test_environment,
                results,
                "python-command-unit-tests",
            )
        elif args.command == "python-command-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-command-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
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
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/command_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-command-c-abi-consumer",
            )
        elif args.command == "python-controls-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            tests_path = str(ROOT / "wrappers/python/tests")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [
                    source_path,
                    tests_path,
                    test_environment.get("PYTHONPATH", ""),
                ]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_controls.py",
                ],
                test_environment,
                results,
                "python-controls-unit-tests",
            )
        elif args.command in (
            "python-input-test",
            "python-input-consumer-test",
        ):
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            tests_path = str(ROOT / "wrappers/python/tests")
            examples_path = str(ROOT / "wrappers/python/examples")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [
                    source_path,
                    tests_path,
                    examples_path,
                    test_environment.get("PYTHONPATH", ""),
                ]
            ).rstrip(os.pathsep)
            test_file = (
                "test_input_events.py"
                if args.command == "python-input-test"
                else "test_input_consumer.py"
            )
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    f"wrappers/python/tests/{test_file}",
                ],
                test_environment,
                results,
                args.command,
            )
        elif args.command == "python-controls-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-controls-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
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
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/controls_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-controls-c-abi-consumer",
            )
        elif args.command == "python-window-test":
            test_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            tests_path = str(ROOT / "wrappers/python/tests")
            test_environment["PYTHONPATH"] = os.pathsep.join(
                [
                    source_path,
                    tests_path,
                    test_environment.get("PYTHONPATH", ""),
                ]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "wrappers/python/tests/test_window.py",
                ],
                test_environment,
                results,
                "python-window-unit-tests",
            )
        elif args.command == "python-window-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-window-consumer-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
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
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/window_consumer.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-window-c-abi-consumer",
            )
        elif args.command == "python-window-peer-smoke-test":
            if not sys.platform.startswith("linux"):
                raise RuntimeError(
                    "Window peer smoke requires a Linux Platform host"
                )
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "python-window-peer-smoke-native-build",
            )
            target_dir = Path(
                environment.get("CARGO_TARGET_DIR", ROOT / "target")
            )
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
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            consumer_environment = dict(environment)
            source_path = str(ROOT / "wrappers/python/src")
            consumer_environment["PYTHONPATH"] = os.pathsep.join(
                [source_path, consumer_environment.get("PYTHONPATH", "")]
            ).rstrip(os.pathsep)
            run(
                [
                    sys.executable,
                    "wrappers/python/examples/window_peer_smoke.py",
                    str(native_path.resolve()),
                ],
                consumer_environment,
                results,
                "python-window-peer-platform-smoke",
            )
        elif args.command == "minimum":
            run(
                [
                    "cargo",
                    "+1.85.0",
                    "check",
                    "--workspace",
                    "--all-targets",
                    "--locked",
                ],
                environment,
                results,
                "rust-minimum",
            )
        elif args.command in ABI_COMMANDS:
            handle_abi_command(args.command, environment, results)
        elif args.command in CPP_COMMANDS:
            handle_cpp_command(args.command, environment, results)
        elif args.command == "check":
            run(
                [sys.executable, "tools/check_docs.py"],
                environment,
                results,
                "document-links",
            )
            run(
                [sys.executable, "tools/dev_support/unit_catalog.py"],
                environment,
                results,
                "unit-catalog-validation",
            )
            run(
                [
                    sys.executable,
                    "-m",
                    "pytest",
                    "tools/tests/test_unit_catalog.py",
                ],
                environment,
                results,
                "unit-catalog-tests",
            )
            run(
                [
                    "ruff",
                    "format",
                    "--check",
                    "tools",
                    "wrappers/python/src",
                    "wrappers/python/tests",
                    "wrappers/python/examples",
                ],
                environment,
                results,
                "python-format",
            )
            run(
                [
                    "ruff",
                    "check",
                    "tools",
                    "wrappers/python/src",
                    "wrappers/python/tests",
                    "wrappers/python/examples",
                ],
                environment,
                results,
                "python-lint",
            )
            run(
                ["cargo", "fmt", "--all", "--", "--check"],
                environment,
                results,
                "rust-format",
            )
            run(
                [
                    "cargo",
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--locked",
                    "--",
                    "-D",
                    "warnings",
                ],
                environment,
                results,
                "rust-clippy",
            )
            run(
                ["cargo", "build", "--workspace", "--locked"],
                environment,
                results,
                "rust-build",
            )
            run(
                [
                    "cargo",
                    "test",
                    "--workspace",
                    "--all-targets",
                    "--locked",
                    "--no-run",
                ],
                environment,
                results,
                "rust-test-harness-build",
            )
            run(
                [sys.executable, "tools/dev.py", "rust-wrapper-runtime-test"],
                environment,
                results,
                "rust-wrapper-runtime-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "rust-wrapper-runtime-consumer-test",
                ],
                environment,
                results,
                "rust-wrapper-runtime-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "rust-wrapper-property-bool-i64-test",
                ],
                environment,
                results,
                "rust-wrapper-property-bool-i64-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "rust-wrapper-property-utf8-test",
                ],
                environment,
                results,
                "rust-wrapper-property-utf8-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "rust-wrapper-property-f64-test",
                ],
                environment,
                results,
                "rust-wrapper-property-f64-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "rust-wrapper-property-decimal-test",
                ],
                environment,
                results,
                "rust-wrapper-property-decimal-consumer",
            )
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                ["cmake", "--build", "--preset", "dev"],
                environment,
                results,
                "c-cpp-build",
            )
            run(
                ["ctest", "--preset", "dev", "--no-tests=error"],
                environment,
                results,
                "c-cpp-probes",
            )
            run(
                [sys.executable, "tools/dev.py", "cpp-package-test"],
                environment,
                results,
                "cpp-package-integration",
            )
            project = "wrappers/csharp/FreeMvvm/FreeMvvm.csproj"
            run(
                ["dotnet", "restore", project, "--locked-mode"],
                environment,
                results,
                "dotnet-restore",
            )
            run(
                [
                    "dotnet",
                    "build",
                    project,
                    "--configuration",
                    "Debug",
                    "--no-restore",
                ],
                environment,
                results,
                "dotnet-build",
            )
            run(
                [sys.executable, "tools/dev.py", "csharp-runtime-test"],
                environment,
                results,
                "csharp-runtime-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "csharp-runtime-consumer-test",
                ],
                environment,
                results,
                "csharp-runtime-consumer",
            )
            run(
                [sys.executable, "tools/dev.py", "csharp-property-test"],
                environment,
                results,
                "csharp-property-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "csharp-property-consumer-test",
                ],
                environment,
                results,
                "csharp-property-consumer",
            )
            run(
                [sys.executable, "tools/dev.py", "csharp-command-test"],
                environment,
                results,
                "csharp-command-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "csharp-command-consumer-test",
                ],
                environment,
                results,
                "csharp-command-consumer",
            )
            run(
                [sys.executable, "tools/dev.py", "csharp-controls-test"],
                environment,
                results,
                "csharp-controls-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "csharp-controls-consumer-test",
                ],
                environment,
                results,
                "csharp-controls-consumer",
            )
            run(
                [sys.executable, "tools/dev.py", "csharp-window-test"],
                environment,
                results,
                "csharp-window-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "csharp-window-consumer-test",
                ],
                environment,
                results,
                "csharp-window-consumer",
            )
            run(
                [sys.executable, "-m", "pip", "check"],
                environment,
                results,
                "python-dependencies",
            )
            run(
                [sys.executable, "tools/dev.py", "python-runtime-test"],
                environment,
                results,
                "python-runtime-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-property-bool-i64-test",
                ],
                environment,
                results,
                "python-property-bool-i64-unit",
            )
            run(
                [sys.executable, "tools/dev.py", "python-property-utf8-test"],
                environment,
                results,
                "python-property-utf8-unit",
            )
            run(
                [sys.executable, "tools/dev.py", "python-property-f64-test"],
                environment,
                results,
                "python-property-f64-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-property-decimal-test",
                ],
                environment,
                results,
                "python-property-decimal-unit",
            )
            run(
                [sys.executable, "tools/dev.py", "python-binding-test"],
                environment,
                results,
                "python-binding-unit",
            )
            run(
                [sys.executable, "tools/dev.py", "python-command-test"],
                environment,
                results,
                "python-command-unit",
            )
            run(
                [sys.executable, "tools/dev.py", "python-controls-test"],
                environment,
                results,
                "python-controls-unit",
            )
            run(
                [sys.executable, "tools/dev.py", "python-window-test"],
                environment,
                results,
                "python-window-unit",
            )
            run(
                [sys.executable, "tools/dev.py", "python-input-test"],
                environment,
                results,
                "python-input-event-unit",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-input-consumer-test",
                ],
                environment,
                results,
                "python-input-event-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-runtime-consumer-test",
                ],
                environment,
                results,
                "python-runtime-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-property-bool-i64-consumer-test",
                ],
                environment,
                results,
                "python-property-bool-i64-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-property-utf8-consumer-test",
                ],
                environment,
                results,
                "python-property-utf8-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-property-f64-consumer-test",
                ],
                environment,
                results,
                "python-property-f64-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-property-decimal-consumer-test",
                ],
                environment,
                results,
                "python-property-decimal-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-binding-consumer-test",
                ],
                environment,
                results,
                "python-binding-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-command-consumer-test",
                ],
                environment,
                results,
                "python-command-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-controls-consumer-test",
                ],
                environment,
                results,
                "python-controls-consumer",
            )
            run(
                [
                    sys.executable,
                    "tools/dev.py",
                    "python-window-consumer-test",
                ],
                environment,
                results,
                "python-window-consumer",
            )
            run(
                [
                    sys.executable,
                    "-m",
                    "build",
                    "--wheel",
                    "--no-isolation",
                    "--outdir",
                    str(ROOT / "out/python"),
                    "wrappers/python",
                ],
                environment,
                results,
                "python-wheel-build",
            )
            wheel = ROOT / "out/python/freemvvm-0.0.0-py3-none-any.whl"
            run(
                [
                    sys.executable,
                    "-m",
                    "pip",
                    "install",
                    "--no-deps",
                    "--force-reinstall",
                    str(wheel),
                ],
                environment,
                results,
                "python-wheel-install",
            )
            run(
                [
                    sys.executable,
                    "-c",
                    "import freemvvm; print('Python facade: OK')",
                ],
                environment,
                results,
                "python-import",
            )
        report["git_head"] = capture(["git", "rev-parse", "HEAD"], environment)
        report["dirty_worktree"] = bool(
            capture(["git", "status", "--porcelain"], environment)
        )
        tracked_inputs = capture(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard"],
            environment,
        )
        report["input_hashes"] = {
            name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
            for name in tracked_inputs.splitlines()
            if (ROOT / name).is_file()
        }
        report["status"] = "passed"
    except (
        RuntimeError,
        subprocess.CalledProcessError,
        OSError,
        UnicodeError,
    ) as error:
        report["error"] = str(error)
        print(f"Bootstrap failed: {error}", file=sys.stderr)
    finally:
        destination = REPORT.with_name(f"{args.command}.json")
        write_json_report(destination, report)
        print(f"Report: {destination.relative_to(ROOT)}", flush=True)
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
