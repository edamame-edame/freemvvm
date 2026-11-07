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
from dev_support.command_scopes import scope_for
from dev_support.commands.bootstrap import COMMANDS as BOOTSTRAP_COMMANDS
from dev_support.commands.bootstrap import handle as handle_bootstrap_command
from dev_support.commands.abi import COMMANDS as ABI_COMMANDS
from dev_support.commands.abi import handle as handle_abi_command
from dev_support.commands.cpp import COMMANDS as CPP_COMMANDS
from dev_support.commands.cpp import handle as handle_cpp_command
from dev_support.commands.csharp import COMMANDS as CSHARP_COMMANDS
from dev_support.commands.csharp import handle as handle_csharp_command
from dev_support.commands.python import COMMANDS as PYTHON_COMMANDS
from dev_support.commands.python import handle as handle_python_command
from dev_support.commands.platform import COMMANDS as PLATFORM_COMMANDS
from dev_support.commands.platform import (
    run_linux_platform_docker,
)
from dev_support.commands.rust import COMMANDS as RUST_COMMANDS
from dev_support.commands.rust import handle as handle_rust_command
from dev_support.process import capture, msvc_environment
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
            *BOOTSTRAP_COMMANDS[:2],
            *ABI_COMMANDS,
            *CPP_COMMANDS,
            *RUST_COMMANDS,
            *PYTHON_COMMANDS,
            *CSHARP_COMMANDS,
            *BOOTSTRAP_COMMANDS[2:],
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
        "scope": scope_for(args.command),
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
        if args.command in BOOTSTRAP_COMMANDS:
            handle_bootstrap_command(args.command, environment, results)
        elif args.command in RUST_COMMANDS:
            handle_rust_command(args.command, environment, results)
        elif args.command in CSHARP_COMMANDS:
            handle_csharp_command(args.command, environment, results)
        elif args.command in PYTHON_COMMANDS:
            handle_python_command(args.command, environment, results)
        elif args.command in ABI_COMMANDS:
            handle_abi_command(args.command, environment, results)
        elif args.command in CPP_COMMANDS:
            handle_cpp_command(args.command, environment, results)
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
