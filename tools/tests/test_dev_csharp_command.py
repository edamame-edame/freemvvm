import pytest

from tools.dev_support.commands import csharp


@pytest.mark.parametrize(
    ("command", "label"),
    [
        ("csharp-runtime-test", "runtime"),
        ("csharp-property-test", "property"),
        ("csharp-command-test", "command"),
        ("csharp-controls-test", "controls"),
        ("csharp-input-test", "input"),
        ("csharp-window-test", "window"),
    ],
)
def test_csharp_handler_preserves_runtime_unit_gates(
    monkeypatch, command, label
):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, environment, results, name))

    monkeypatch.setattr(csharp, "run", record_run)
    environment = {"PATH": "test-path"}
    results = []

    assert csharp.handle(command, environment, results) is True

    assert [call[3] for call in calls] == [
        f"csharp-{label}-test-restore",
        f"csharp-{label}-unit-tests",
    ]
    assert calls[0][0] == [
        "dotnet",
        "restore",
        "wrappers/csharp/tests/RuntimeTests/RuntimeTests.csproj",
        "--locked-mode",
    ]
    assert calls[1][0] == [
        "dotnet",
        "run",
        "--project",
        "wrappers/csharp/tests/RuntimeTests/RuntimeTests.csproj",
        "--configuration",
        "Debug",
        "--no-restore",
    ]


@pytest.mark.parametrize(
    ("command", "label", "native_gate", "restore_gate", "consumer_gate"),
    [
        (
            "csharp-runtime-consumer-test",
            "runtime",
            "csharp-consumer-native-build",
            "csharp-consumer-restore",
            "csharp-runtime-c-abi-consumer",
        ),
        (
            "csharp-property-consumer-test",
            "property",
            "csharp-property-consumer-native-build",
            "csharp-property-consumer-restore",
            "csharp-property-c-abi-consumer",
        ),
        (
            "csharp-command-consumer-test",
            "command",
            "csharp-command-consumer-native-build",
            "csharp-command-consumer-restore",
            "csharp-command-c-abi-consumer",
        ),
        (
            "csharp-controls-consumer-test",
            "controls",
            "csharp-controls-consumer-native-build",
            "csharp-controls-consumer-restore",
            "csharp-controls-c-abi-consumer",
        ),
        (
            "csharp-window-consumer-test",
            "window",
            "csharp-window-consumer-native-build",
            "csharp-window-consumer-restore",
            "csharp-window-c-abi-consumer",
        ),
        (
            "csharp-input-consumer-test",
            "input",
            "csharp-input-consumer-native-build",
            "csharp-input-consumer-restore",
            "csharp-input-c-abi-consumer",
        ),
    ],
)
def test_csharp_handler_preserves_native_consumer_gates(
    monkeypatch,
    tmp_path,
    command,
    label,
    native_gate,
    restore_gate,
    consumer_gate,
):
    calls = []
    native_dir = tmp_path / "target" / "debug"
    native_dir.mkdir(parents=True)
    native_name = (
        "freemvvm_abi_1.dll"
        if csharp.os.name == "nt"
        else "libfreemvvm_abi_1.dylib"
        if csharp.sys.platform == "darwin"
        else "libfreemvvm_abi_1.so"
    )
    native_path = native_dir / native_name
    native_path.touch()

    def record_run(arguments, environment, results, name):
        calls.append((arguments, name))

    monkeypatch.setattr(csharp, "ROOT", tmp_path)
    monkeypatch.setattr(csharp, "run", record_run)

    assert csharp.handle(command, {}, []) is True

    assert [name for _arguments, name in calls] == [
        native_gate,
        restore_gate,
        consumer_gate,
    ]
    consumer_arguments = calls[-1][0]
    expected_consumer_arguments = [
        "dotnet",
        "run",
        "--project",
        "wrappers/csharp/tests/Consumer/Consumer.csproj",
        "--configuration",
        "Debug",
        "--no-restore",
        "--",
    ]
    if label == "input":
        expected_consumer_arguments.append("--input-only")
    expected_consumer_arguments.append(str(native_path.resolve()))
    assert consumer_arguments == expected_consumer_arguments


def test_csharp_handler_leaves_unknown_commands_unhandled():
    assert csharp.handle("unknown-command", {}, []) is False
