import pytest

from tools.dev_support.commands import bootstrap


_CHECK_GATES = (
    "document-links",
    "unit-catalog-validation",
    "unit-catalog-tests",
    "python-format",
    "python-lint",
    "rust-format",
    "rust-clippy",
    "rust-build",
    "rust-test-harness-build",
    "rust-wrapper-runtime-unit",
    "rust-wrapper-runtime-consumer",
    "rust-wrapper-property-bool-i64-consumer",
    "rust-wrapper-property-utf8-consumer",
    "rust-wrapper-property-f64-consumer",
    "rust-wrapper-property-decimal-consumer",
    "cmake-configure",
    "c-cpp-build",
    "c-cpp-probes",
    "cpp-package-integration",
    "dotnet-restore",
    "dotnet-build",
    "csharp-runtime-unit",
    "csharp-runtime-consumer",
    "csharp-property-unit",
    "csharp-property-consumer",
    "csharp-command-unit",
    "csharp-command-consumer",
    "csharp-controls-unit",
    "csharp-controls-consumer",
    "csharp-window-unit",
    "csharp-window-consumer",
    "python-dependencies",
    "python-runtime-unit",
    "python-property-bool-i64-unit",
    "python-property-utf8-unit",
    "python-property-f64-unit",
    "python-property-decimal-unit",
    "python-binding-unit",
    "python-command-unit",
    "python-controls-unit",
    "python-window-unit",
    "python-input-event-unit",
    "python-input-event-consumer",
    "python-runtime-consumer",
    "python-property-bool-i64-consumer",
    "python-property-utf8-consumer",
    "python-property-f64-consumer",
    "python-property-decimal-consumer",
    "python-binding-consumer",
    "python-command-consumer",
    "python-controls-consumer",
    "python-window-consumer",
    "python-wheel-build",
    "python-wheel-install",
    "python-import",
)


def test_bootstrap_check_preserves_all_aggregate_gates(monkeypatch):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, environment, results, name))

    monkeypatch.setattr(bootstrap, "run", record_run)
    environment = {"PATH": "test-path"}
    results = []

    assert bootstrap.handle("check", environment, results) is True

    assert tuple(call[3] for call in calls) == _CHECK_GATES
    assert all(call[1] is environment and call[2] is results for call in calls)
    assert calls[0][0] == [bootstrap.sys.executable, "tools/check_docs.py"]
    assert calls[1][0] == [
        bootstrap.sys.executable,
        "tools/dev_support/unit_catalog.py",
    ]
    assert calls[-3][0][-2] == str(bootstrap.ROOT / "out/python")
    assert calls[-2][0][3:6] == ["install", "--no-deps", "--force-reinstall"]
    assert calls[-1][0][-1] == "import freemvvm; print('Python facade: OK')"


@pytest.mark.parametrize(
    ("command", "expected_calls"),
    [
        (
            "format",
            [
                (["cargo", "fmt", "--all"], "rust-format"),
                (
                    ["ruff", "format", "tools", "wrappers/python/src"],
                    "python-format",
                ),
            ],
        ),
        (
            "minimum",
            [
                (
                    [
                        "cargo",
                        "+1.85.0",
                        "check",
                        "--workspace",
                        "--all-targets",
                        "--locked",
                    ],
                    "rust-minimum",
                )
            ],
        ),
        ("doctor", []),
    ],
)
def test_bootstrap_single_commands_keep_existing_steps(
    monkeypatch, command, expected_calls
):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, name))

    monkeypatch.setattr(bootstrap, "run", record_run)

    assert bootstrap.handle(command, {}, []) is True
    assert calls == expected_calls


def test_bootstrap_handler_leaves_unknown_commands_unhandled():
    assert bootstrap.handle("unknown-command", {}, []) is False
