"""C++ wrapper command handlers."""

from pathlib import Path

from ..process import run


COMMANDS = (
    "cpp-runtime-test",
    "cpp-property-test",
    "cpp-property-utf8-test",
    "cpp-property-f64-test",
    "cpp-property-decimal-test",
    "cpp-property-subscription-test",
    "cpp-property-binding-test",
    "cpp-command-test",
    "cpp-controls-test",
    "cpp-input-test",
    "cpp-input-consumer-test",
    "cpp-package-test",
)
ROOT = Path(__file__).resolve().parents[3]

_TEST_TARGETS = {
    "cpp-property-test": (
        "freemvvm_cpp_property_tests_exe",
        r"^wrapper\.cpp\.property$",
        "cpp-property-test",
    ),
    "cpp-property-utf8-test": (
        "freemvvm_cpp_utf8_property_tests_exe",
        r"^wrapper\.cpp\.property\.utf8$",
        "cpp-property-utf8-test",
    ),
    "cpp-property-f64-test": (
        "freemvvm_cpp_f64_property_tests_exe",
        r"^wrapper\.cpp\.property\.f64$",
        "cpp-property-f64-test",
    ),
    "cpp-property-decimal-test": (
        "freemvvm_cpp_decimal64_property_tests_exe",
        r"^wrapper\.cpp\.property\.decimal$",
        "cpp-property-decimal-test",
    ),
    "cpp-property-subscription-test": (
        "freemvvm_cpp_property_subscription_tests_exe",
        r"^wrapper\.cpp\.property\.subscription$",
        "cpp-property-subscription-test",
    ),
    "cpp-property-binding-test": (
        "freemvvm_cpp_property_binding_tests_exe",
        r"^wrapper\.cpp\.property\.binding$",
        "cpp-property-binding-test",
    ),
    "cpp-command-test": (
        "freemvvm_cpp_command_tests_exe",
        r"^wrapper\.cpp\.command$",
        "cpp-command-test",
    ),
    "cpp-controls-test": (
        "freemvvm_cpp_ui_control_tests_exe",
        r"^wrapper\.cpp\.ui-controls$",
        "cpp-controls-test",
    ),
    "cpp-input-test": (
        "freemvvm_cpp_input_tests_exe",
        r"^wrapper\.cpp\.input$",
        "cpp-input-test",
    ),
}


def _run_configure(environment, results):
    run(["cmake", "--preset", "dev"], environment, results, "cmake-configure")


def _run_package_test(environment, results):
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


def handle(command, environment, results):
    """Run a C++ wrapper command if recognized."""
    if command not in COMMANDS:
        return False

    if command in {"cpp-package-test", "cpp-input-consumer-test"}:
        _run_package_test(environment, results)
        return True

    _run_configure(environment, results)
    if command == "cpp-runtime-test":
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
                r"^wrapper\.cpp\.runtime$",
                "--output-on-failure",
            ],
            environment,
            results,
            "cpp-runtime-test",
        )
        return True

    target, test_filter, gate = _TEST_TARGETS[command]
    run(
        ["cmake", "--build", "--preset", "dev", "--target", target],
        environment,
        results,
        f"{command}-build",
    )
    run(
        [
            "ctest",
            "--test-dir",
            "out/cmake/dev",
            "-R",
            test_filter,
            "--output-on-failure",
        ],
        environment,
        results,
        gate,
    )
    return True
