import pytest

from tools.dev_support.commands import python


@pytest.mark.parametrize(
    ("command", "test_file", "gate", "path_parts"),
    [
        (
            "python-runtime-test",
            "test_runtime.py",
            "python-runtime-unit-tests",
            ("source",),
        ),
        (
            "python-property-bool-i64-test",
            "test_property_bool_i64.py",
            "python-property-bool-i64-unit-tests",
            ("source",),
        ),
        (
            "python-property-utf8-test",
            "test_property_utf8.py",
            "python-property-utf8-unit-tests",
            ("source",),
        ),
        (
            "python-property-f64-test",
            "test_property_f64.py",
            "python-property-f64-unit-tests",
            ("source",),
        ),
        (
            "python-property-decimal-test",
            "test_property_decimal.py",
            "python-property-decimal-unit-tests",
            ("source",),
        ),
        (
            "python-binding-test",
            "test_binding.py",
            "python-binding-unit-tests",
            ("source",),
        ),
        (
            "python-command-test",
            "test_command.py",
            "python-command-unit-tests",
            ("source", "tests"),
        ),
        (
            "python-controls-test",
            "test_controls.py",
            "python-controls-unit-tests",
            ("source", "tests"),
        ),
        (
            "python-input-test",
            "test_input_events.py",
            "python-input-test",
            ("source", "tests", "examples"),
        ),
        (
            "python-input-consumer-test",
            "test_input_consumer.py",
            "python-input-consumer-test",
            ("source", "tests", "examples"),
        ),
        (
            "python-window-test",
            "test_window.py",
            "python-window-unit-tests",
            ("source", "tests"),
        ),
    ],
)
def test_python_handler_preserves_unit_test_command_and_environment(
    monkeypatch, command, test_file, gate, path_parts
):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, environment, results, name))

    monkeypatch.setattr(python, "run", record_run)
    environment = {"PYTHONPATH": "existing-path", "PATH": "test-path"}
    results = []

    assert python.handle(command, environment, results) is True

    paths = {
        "source": python.ROOT / "wrappers/python/src",
        "tests": python.ROOT / "wrappers/python/tests",
        "examples": python.ROOT / "wrappers/python/examples",
    }
    expected_pythonpath = [
        *(str(paths[part]) for part in path_parts),
        "existing-path",
    ]
    assert calls == [
        (
            [
                python.sys.executable,
                "-m",
                "pytest",
                f"wrappers/python/tests/{test_file}",
            ],
            {
                **environment,
                "PYTHONPATH": python.os.pathsep.join(expected_pythonpath),
            },
            results,
            gate,
        )
    ]
    assert environment["PYTHONPATH"] == "existing-path"


@pytest.mark.parametrize(
    ("command", "example", "build_gate", "consumer_gate"),
    [
        (
            "python-runtime-consumer-test",
            "runtime_consumer.py",
            "python-runtime-consumer-native-build",
            "python-runtime-c-abi-consumer",
        ),
        (
            "python-property-bool-i64-consumer-test",
            "property_bool_i64_consumer.py",
            "python-property-consumer-native-build",
            "python-property-bool-i64-c-abi-consumer",
        ),
        (
            "python-property-utf8-consumer-test",
            "property_utf8_consumer.py",
            "python-property-consumer-native-build",
            "python-property-utf8-c-abi-consumer",
        ),
        (
            "python-property-f64-consumer-test",
            "property_f64_consumer.py",
            "python-property-consumer-native-build",
            "python-property-f64-c-abi-consumer",
        ),
        (
            "python-property-decimal-consumer-test",
            "property_decimal_consumer.py",
            "python-property-consumer-native-build",
            "python-property-decimal-c-abi-consumer",
        ),
        (
            "python-binding-consumer-test",
            "binding_consumer.py",
            "python-binding-consumer-native-build",
            "python-binding-c-abi-consumer",
        ),
        (
            "python-command-consumer-test",
            "command_consumer.py",
            "python-command-consumer-native-build",
            "python-command-c-abi-consumer",
        ),
        (
            "python-controls-consumer-test",
            "controls_consumer.py",
            "python-controls-consumer-native-build",
            "python-controls-c-abi-consumer",
        ),
        (
            "python-window-consumer-test",
            "window_consumer.py",
            "python-window-consumer-native-build",
            "python-window-c-abi-consumer",
        ),
    ],
)
def test_python_handler_preserves_native_consumer_commands(
    monkeypatch, tmp_path, command, example, build_gate, consumer_gate
):
    calls = []
    target_dir = tmp_path / "custom-target"
    native_dir = target_dir / "debug"
    native_dir.mkdir(parents=True)
    native_name = (
        "freemvvm_abi_1.dll"
        if python.os.name == "nt"
        else "libfreemvvm_abi_1.dylib"
        if python.sys.platform == "darwin"
        else "libfreemvvm_abi_1.so"
    )
    native_path = native_dir / native_name
    native_path.touch()

    def record_run(arguments, environment, results, name):
        calls.append((arguments, environment, name))

    monkeypatch.setattr(python, "ROOT", tmp_path)
    monkeypatch.setattr(python, "run", record_run)
    environment = {
        "CARGO_TARGET_DIR": "custom-target",
        "PYTHONPATH": "existing",
    }

    assert python.handle(command, environment, []) is True

    assert [name for _args, _env, name in calls] == [build_gate, consumer_gate]
    assert calls[0][0] == [
        "cargo",
        "build",
        "--locked",
        "-p",
        "freemvvm-c-abi",
    ]
    assert calls[0][1] is environment
    assert calls[1][0] == [
        python.sys.executable,
        f"wrappers/python/examples/{example}",
        str(native_path.resolve()),
    ]
    assert (
        calls[1][1]["PYTHONPATH"]
        == str(python._SOURCE_PATH) + python.os.pathsep + "existing"
    )


def test_python_window_peer_smoke_requires_linux_and_uses_platform_gate(
    monkeypatch, tmp_path
):
    calls = []
    native_dir = tmp_path / "target" / "debug"
    native_dir.mkdir(parents=True)
    native_name = (
        "freemvvm_abi_1.dll"
        if python.os.name == "nt"
        else "libfreemvvm_abi_1.dylib"
        if python.sys.platform == "darwin"
        else "libfreemvvm_abi_1.so"
    )
    (native_dir / native_name).touch()

    def record_run(arguments, environment, results, name):
        calls.append((arguments, name))

    monkeypatch.setattr(python, "ROOT", tmp_path)
    monkeypatch.setattr(python, "run", record_run)
    monkeypatch.setattr(python.sys, "platform", "linux")

    assert python.handle("python-window-peer-smoke-test", {}, []) is True
    assert [name for _args, name in calls] == [
        "python-window-peer-smoke-native-build",
        "python-window-peer-platform-smoke",
    ]
    assert calls[-1][0][1] == "wrappers/python/examples/window_peer_smoke.py"


def test_python_window_peer_smoke_rejects_non_linux_before_build(monkeypatch):
    calls = []
    monkeypatch.setattr(
        python,
        "run",
        lambda *args: calls.append(args),
    )
    monkeypatch.setattr(python.sys, "platform", "win32")

    with pytest.raises(RuntimeError, match="Linux Platform host"):
        python.handle("python-window-peer-smoke-test", {}, [])

    assert calls == []


def test_python_handler_leaves_unknown_commands_unhandled():
    assert python.handle("unknown-command", {}, []) is False
