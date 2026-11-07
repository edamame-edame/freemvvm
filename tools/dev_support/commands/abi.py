"""C and C++ ABI smoke command handlers."""

from ..process import run


COMMAND_TARGETS = {
    "abi-smoke": ("freemvvm_abi_runtime_smoke", "abi-runtime-smoke"),
    "abi-command-smoke": (
        "freemvvm_abi_command_smoke",
        "abi-command-smoke",
    ),
    "abi-smoke-cpp": (
        "freemvvm_abi_cpp_runtime_smoke",
        "abi-cpp-runtime-smoke",
    ),
}
COMMANDS = tuple(COMMAND_TARGETS)


def handle(command, environment, results):
    """Run the selected ABI probe and report whether it was handled."""
    target = COMMAND_TARGETS.get(command)
    if target is None:
        return False

    target_name, gate_name = target
    run(["cmake", "--preset", "dev"], environment, results, "cmake-configure")
    run(
        ["cmake", "--build", "--preset", "dev", "--target", target_name],
        environment,
        results,
        gate_name,
    )
    return True
