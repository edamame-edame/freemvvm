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
import time


ROOT = Path(__file__).resolve().parents[1]
VENV = ROOT / ".venv"
BIN = VENV / ("Scripts" if os.name == "nt" else "bin")
REPORT = ROOT / ".local/verification/bootstrap.json"


def msvc_environment(environment):
    """Import the compiler environment into child processes only."""
    if os.name != "nt":
        return environment
    if not (environment.get("INCLUDE") and environment.get("LIB")):
        installer = Path(os.environ.get("ProgramFiles(x86)", ""))
        vswhere = installer / "Microsoft Visual Studio/Installer/vswhere.exe"
        if not vswhere.is_file():
            raise RuntimeError("MSVC C++ Build Tools and vswhere are required")
        installation = subprocess.check_output(
            [
                str(vswhere),
                "-latest",
                "-products",
                "*",
                "-requires",
                "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                "-property",
                "installationPath",
            ],
            text=True,
            encoding="utf-8",
        ).strip()
        developer = Path(installation) / "Common7/Tools/VsDevCmd.bat"
        if not installation or not developer.is_file():
            raise RuntimeError(
                "A Visual Studio installation with x64 C++ tools is required"
            )
        # Initialize and read the vendor environment with a fixed command.
        # Capture it privately: inherited credentials must never be printed.
        command = (
            "call VsDevCmd.bat -no_logo -arch=x64 -host_arch=x64 >nul && set"
        )
        output = subprocess.check_output(
            ["cmd.exe", "/d", "/u", "/s", "/c", command],
            cwd=developer.parent,
            env=environment,
        ).decode("utf-16-le")
        for line in output.splitlines():
            key, separator, value = line.partition("=")
            if separator and key and not key.startswith("="):
                environment[key.upper()] = value
    environment["CC"] = "cl"
    environment["CXX"] = "cl"
    return environment


def capture(arguments, environment):
    arguments = resolve_executable(arguments, environment)
    return subprocess.check_output(
        arguments,
        cwd=ROOT,
        env=environment,
        text=True,
        encoding="utf-8",
        errors="replace",
        stderr=subprocess.STDOUT,
    ).strip()


def resolve_executable(arguments, environment):
    executable = shutil.which(str(arguments[0]), path=environment["PATH"])
    if not executable:
        raise RuntimeError(f"Executable not found: {arguments[0]}")
    return [executable, *arguments[1:]]


def run(arguments, environment, results, name):
    print(f"\n[{name}] {' '.join(map(str, arguments))}", flush=True)
    started = time.monotonic()
    result = subprocess.run(
        resolve_executable(arguments, environment),
        cwd=ROOT,
        env=environment,
        check=False,
    )
    results.append(
        {
            "gate": name,
            "command": list(map(str, arguments)),
            "exit_code": result.returncode,
            "elapsed_seconds": round(time.monotonic() - started, 3),
        }
    )
    if result.returncode:
        raise RuntimeError(f"{name} failed with exit code {result.returncode}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command",
        choices=[
            "doctor",
            "check",
            "abi-smoke",
            "abi-smoke-cpp",
            "cpp-runtime-test",
            "cpp-property-test",
            "cpp-property-utf8-test",
            "cpp-property-f64-test",
            "cpp-property-decimal-test",
            "cpp-property-subscription-test",
            "cpp-property-binding-test",
            "cpp-package-test",
            "csharp-runtime-test",
            "csharp-runtime-consumer-test",
            "format",
            "minimum",
        ],
    )
    args = parser.parse_args()
    if Path(sys.prefix).resolve() != VENV.resolve():
        parser.error(
            "Run with the repository .venv Python; see DEVELOPMENT.md"
        )
    if sys.version_info[:2] != (3, 13) or sysconfig.get_config_var(
        "Py_GIL_DISABLED"
    ):
        parser.error("Bootstrap requires the standard CPython 3.13 build")
    environment = dict(os.environ)
    environment["PYTHONUTF8"] = "1"
    environment["PYTHONIOENCODING"] = "utf-8"
    environment["DOTNET_CLI_TELEMETRY_OPTOUT"] = "1"
    environment["DOTNET_NOLOGO"] = "1"
    environment["PATH"] = str(BIN) + os.pathsep + environment.get("PATH", "")
    results = []
    report = {
        "scope": (
            "csharp-runtime-unit-test"
            if args.command == "csharp-runtime-test"
            else "csharp-runtime-consumer-test"
            if args.command == "csharp-runtime-consumer-test"
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
            else "cpp-runtime-unit-test"
            if args.command == "cpp-runtime-test"
            else "development-bootstrap"
        ),
        "product_tests": (
            "C# Runtime wrapper unit tests executed"
            if args.command == "csharp-runtime-test"
            else "C# Runtime consumer test executed against the host C ABI"
            if args.command == "csharp-runtime-consumer-test"
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
        elif args.command == "csharp-runtime-test":
            project = "wrappers/csharp/tests/RuntimeTests/RuntimeTests.csproj"
            run(
                ["dotnet", "restore", project, "--locked-mode"],
                environment,
                results,
                "csharp-runtime-test-restore",
            )
            run(
                [
                    "dotnet",
                    "run",
                    "--project",
                    project,
                    "--configuration",
                    "Debug",
                    "--no-restore",
                ],
                environment,
                results,
                "csharp-runtime-unit-tests",
            )
        elif args.command == "csharp-runtime-consumer-test":
            run(
                ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
                environment,
                results,
                "csharp-consumer-native-build",
            )
            project = "wrappers/csharp/tests/Consumer/Consumer.csproj"
            run(
                ["dotnet", "restore", project, "--locked-mode"],
                environment,
                results,
                "csharp-consumer-restore",
            )
            native_name = (
                "freemvvm_abi_1.dll"
                if os.name == "nt"
                else "libfreemvvm_abi_1.dylib"
                if sys.platform == "darwin"
                else "libfreemvvm_abi_1.so"
            )
            native_path = ROOT / "target/debug" / native_name
            if not native_path.is_file():
                raise RuntimeError(
                    f"Host C ABI library was not built: {native_path}"
                )
            run(
                [
                    "dotnet",
                    "run",
                    "--project",
                    project,
                    "--configuration",
                    "Debug",
                    "--no-restore",
                    "--",
                    str(native_path.resolve()),
                ],
                environment,
                results,
                "csharp-runtime-c-abi-consumer",
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
        elif args.command == "abi-smoke":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_abi_runtime_smoke",
                ],
                environment,
                results,
                "abi-runtime-smoke",
            )
        elif args.command == "abi-smoke-cpp":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_abi_cpp_runtime_smoke",
                ],
                environment,
                results,
                "abi-cpp-runtime-smoke",
            )
        elif args.command == "cpp-runtime-test":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "-S",
                    ".",
                    "-B",
                    "out/cmake/cpp-no-tests",
                    "-G",
                    "Ninja",
                    "-DBUILD_TESTING=OFF",
                    "-DCMAKE_BUILD_TYPE=Debug",
                ],
                environment,
                results,
                "cpp-runtime-no-tests-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "out/cmake/cpp-no-tests",
                    "--target",
                    "freemvvm_cpp",
                ],
                environment,
                results,
                "cpp-runtime-no-tests-build",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_cpp_runtime_tests_exe",
                ],
                environment,
                results,
                "cpp-runtime-test-build",
            )
            run(
                [
                    "ctest",
                    "--test-dir",
                    "out/cmake/dev",
                    "-R",
                    "^wrapper\\.cpp\\.runtime$",
                    "--output-on-failure",
                ],
                environment,
                results,
                "cpp-runtime-test",
            )
        elif args.command == "cpp-property-test":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_cpp_property_tests_exe",
                ],
                environment,
                results,
                "cpp-property-test-build",
            )
            run(
                [
                    "ctest",
                    "--test-dir",
                    "out/cmake/dev",
                    "-R",
                    "^wrapper\\.cpp\\.property$",
                    "--output-on-failure",
                ],
                environment,
                results,
                "cpp-property-test",
            )
        elif args.command == "cpp-property-utf8-test":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_cpp_utf8_property_tests_exe",
                ],
                environment,
                results,
                "cpp-property-utf8-test-build",
            )
            run(
                [
                    "ctest",
                    "--test-dir",
                    "out/cmake/dev",
                    "-R",
                    "^wrapper\\.cpp\\.property\\.utf8$",
                    "--output-on-failure",
                ],
                environment,
                results,
                "cpp-property-utf8-test",
            )
        elif args.command == "cpp-property-f64-test":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_cpp_f64_property_tests_exe",
                ],
                environment,
                results,
                "cpp-property-f64-test-build",
            )
            run(
                [
                    "ctest",
                    "--test-dir",
                    "out/cmake/dev",
                    "-R",
                    "^wrapper\\.cpp\\.property\\.f64$",
                    "--output-on-failure",
                ],
                environment,
                results,
                "cpp-property-f64-test",
            )
        elif args.command == "cpp-property-decimal-test":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_cpp_decimal64_property_tests_exe",
                ],
                environment,
                results,
                "cpp-property-decimal-test-build",
            )
            run(
                [
                    "ctest",
                    "--test-dir",
                    "out/cmake/dev",
                    "-R",
                    "^wrapper\\.cpp\\.property\\.decimal$",
                    "--output-on-failure",
                ],
                environment,
                results,
                "cpp-property-decimal-test",
            )
        elif args.command == "cpp-property-subscription-test":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_cpp_property_subscription_tests_exe",
                ],
                environment,
                results,
                "cpp-property-subscription-test-build",
            )
            run(
                [
                    "ctest",
                    "--test-dir",
                    "out/cmake/dev",
                    "-R",
                    "^wrapper\\.cpp\\.property\\.subscription$",
                    "--output-on-failure",
                ],
                environment,
                results,
                "cpp-property-subscription-test",
            )
        elif args.command == "cpp-property-binding-test":
            run(
                ["cmake", "--preset", "dev"],
                environment,
                results,
                "cmake-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "--preset",
                    "dev",
                    "--target",
                    "freemvvm_cpp_property_binding_tests_exe",
                ],
                environment,
                results,
                "cpp-property-binding-test-build",
            )
            run(
                [
                    "ctest",
                    "--test-dir",
                    "out/cmake/dev",
                    "-R",
                    "^wrapper\\.cpp\\.property\\.binding$",
                    "--output-on-failure",
                ],
                environment,
                results,
                "cpp-property-binding-test",
            )
        elif args.command == "cpp-package-test":
            run(
                [
                    "cmake",
                    "-E",
                    "rm",
                    "-rf",
                    "out/cmake/package-stage",
                    "out/cmake/package-moved",
                    "out/cmake/package-consumer",
                    "out/cmake/cpp-package-build",
                ],
                environment,
                results,
                "package-test-clean",
            )
            run(
                [
                    "cmake",
                    "-S",
                    ".",
                    "-B",
                    "out/cmake/cpp-package-build",
                    "-G",
                    "Ninja",
                    "-DBUILD_TESTING=OFF",
                    "-DCMAKE_BUILD_TYPE=Release",
                ],
                environment,
                results,
                "cpp-package-configure",
            )
            run(
                [
                    "cmake",
                    "--build",
                    "out/cmake/cpp-package-build",
                    "--target",
                    "freemvvm_cpp",
                ],
                environment,
                results,
                "cpp-package-build",
            )
            run(
                [
                    "cmake",
                    "--install",
                    "out/cmake/cpp-package-build",
                    "--prefix",
                    "out/cmake/package-stage",
                ],
                environment,
                results,
                "cpp-package-install",
            )
            run(
                [
                    "cmake",
                    "-E",
                    "rename",
                    "out/cmake/package-stage",
                    "out/cmake/package-moved",
                ],
                environment,
                results,
                "cpp-package-relocate",
            )
            run(
                [
                    "cmake",
                    "-S",
                    "wrappers/cpp/tests/package_consumer",
                    "-B",
                    "out/cmake/package-consumer",
                    "-G",
                    "Ninja",
                    "-DCMAKE_BUILD_TYPE=Release",
                    f"-DCMAKE_PREFIX_PATH={ROOT / 'out/cmake/package-moved'}",
                ],
                environment,
                results,
                "cpp-package-consumer-configure",
            )
            run(
                ["cmake", "--build", "out/cmake/package-consumer"],
                environment,
                results,
                "cpp-package-consumer-build",
            )
            run(
                [
                    "ctest",
                    "--test-dir",
                    "out/cmake/package-consumer",
                    "--output-on-failure",
                ],
                environment,
                results,
                "cpp-package-consumer-test",
            )
        elif args.command == "check":
            run(
                [sys.executable, "tools/check_docs.py"],
                environment,
                results,
                "document-links",
            )
            run(
                ["ruff", "format", "--check", "tools", "wrappers/python/src"],
                environment,
                results,
                "python-format",
            )
            run(
                ["ruff", "check", "tools", "wrappers/python/src"],
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
                [sys.executable, "-m", "pip", "check"],
                environment,
                results,
                "python-dependencies",
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
                    "import ctypes, freemvvm; print('skeleton import: OK')",
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
        REPORT.parent.mkdir(parents=True, exist_ok=True)
        destination = REPORT.with_name(f"{args.command}.json")
        destination.write_text(
            json.dumps(report, indent=2) + "\n", encoding="utf-8"
        )
        print(f"Report: {destination.relative_to(ROOT)}", flush=True)
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
