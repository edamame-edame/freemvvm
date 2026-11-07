"""Repository bootstrap and aggregate verification command handlers."""

import sys
from pathlib import Path

from ..process import run


COMMANDS = ("doctor", "check", "format", "minimum")
ROOT = Path(__file__).resolve().parents[3]


def _handle_check(environment, results):
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


def handle(command, environment, results):
    """Run a bootstrap command if recognized."""
    if command not in COMMANDS:
        return False
    if command == "check":
        _handle_check(environment, results)
    elif command == "format":
        run(["cargo", "fmt", "--all"], environment, results, "rust-format")
        run(
            ["ruff", "format", "tools", "wrappers/python/src"],
            environment,
            results,
            "python-format",
        )
    elif command == "minimum":
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
    return True
