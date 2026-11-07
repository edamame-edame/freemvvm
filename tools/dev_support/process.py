"""Process helpers shared by repository bootstrap commands."""

import os
from pathlib import Path
import shutil
import subprocess
import time


ROOT = Path(__file__).resolve().parents[2]


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
