import pytest

from tools.dev_support.commands import abi
from tools.dev_support.commands import rust


@pytest.mark.parametrize(
    ("command", "target", "gate"),
    [
        ("abi-smoke", "freemvvm_abi_runtime_smoke", "abi-runtime-smoke"),
        (
            "abi-command-smoke",
            "freemvvm_abi_command_smoke",
            "abi-command-smoke",
        ),
        (
            "abi-smoke-cpp",
            "freemvvm_abi_cpp_runtime_smoke",
            "abi-cpp-runtime-smoke",
        ),
    ],
)
def test_abi_handler_preserves_configure_and_build_commands(
    monkeypatch, command, target, gate
):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, environment, results, name))

    monkeypatch.setattr(abi, "run", record_run)
    environment = {"PATH": "test-path"}
    results = []

    assert abi.handle(command, environment, results) is True

    assert calls == [
        (
            ["cmake", "--preset", "dev"],
            environment,
            results,
            "cmake-configure",
        ),
        (
            ["cmake", "--build", "--preset", "dev", "--target", target],
            environment,
            results,
            gate,
        ),
    ]


def test_abi_handler_leaves_unknown_commands_unhandled():
    assert abi.handle("unknown-command", {}, []) is False


@pytest.mark.parametrize(
    ("command", "tail"),
    [
        (
            "rust-wrapper-runtime-test",
            (
                ["cargo", "test", "--locked", "-p", "freemvvm"],
                "rust-wrapper-runtime-tests",
            ),
        ),
        (
            "rust-wrapper-runtime-consumer-test",
            (
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "freemvvm",
                    "--example",
                    "runtime_consumer",
                ],
                "rust-wrapper-runtime-consumer",
            ),
        ),
        (
            "rust-wrapper-ui-controls-test",
            (
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "freemvvm",
                    "--example",
                    "ui_controls_consumer",
                ],
                "rust-wrapper-ui-controls-consumer",
            ),
        ),
        (
            "rust-wrapper-window-close-test",
            (
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "freemvvm",
                    "--example",
                    "window_close_consumer",
                ],
                "rust-wrapper-window-close-consumer",
            ),
        ),
        (
            "rust-wrapper-property-bool-i64-test",
            (
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "freemvvm",
                    "--example",
                    "property_bool_i64_consumer",
                ],
                "rust-wrapper-property-bool-i64-consumer",
            ),
        ),
        (
            "rust-wrapper-property-utf8-test",
            (
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "freemvvm",
                    "--example",
                    "property_utf8_consumer",
                ],
                "rust-wrapper-property-utf8-consumer",
            ),
        ),
        (
            "rust-wrapper-property-f64-test",
            (
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "freemvvm",
                    "--example",
                    "property_f64_consumer",
                ],
                "rust-wrapper-property-f64-consumer",
            ),
        ),
        (
            "rust-wrapper-property-decimal-test",
            (
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "freemvvm",
                    "--example",
                    "property_decimal_consumer",
                ],
                "rust-wrapper-property-decimal-consumer",
            ),
        ),
    ],
)
def test_rust_handler_preserves_native_build_and_command(
    monkeypatch, command, tail
):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, environment, results, name))

    monkeypatch.setattr(rust, "run", record_run)
    environment = {"PATH": "test-path"}
    results = []

    assert rust.handle(command, environment, results) is True

    assert calls[0] == (
        ["cargo", "build", "--locked", "-p", "freemvvm-c-abi"],
        environment,
        results,
        "rust-wrapper-native-build",
    )
    if command in {
        "rust-wrapper-ui-controls-test",
        "rust-wrapper-window-close-test",
    }:
        assert calls[1][0] == [
            "cargo",
            "test",
            "--locked",
            "-p",
            "freemvvm",
        ]
        assert calls[1][3] in {
            "rust-wrapper-ui-controls-tests",
            "rust-wrapper-window-close-tests",
        }
    assert calls[-1][0] == tail[0]
    assert calls[-1][3] == tail[1]
    native_environment = calls[-1][1]
    assert native_environment["FREEMVVM_NATIVE_LIB_DIR"] == str(
        rust.ROOT / "target" / "debug"
    )
    assert native_environment is not environment
    assert environment == {"PATH": "test-path"}


def test_rust_handler_leaves_unknown_commands_unhandled():
    assert rust.handle("unknown-command", {}, []) is False
