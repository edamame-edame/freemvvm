import pytest

from tools.dev_support.commands import abi


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
