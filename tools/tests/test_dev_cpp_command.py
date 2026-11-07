import pytest

from tools.dev_support.commands import cpp


@pytest.mark.parametrize(
    ("command", "target", "test_filter", "gate"),
    [
        (
            "cpp-property-test",
            "freemvvm_cpp_property_tests_exe",
            r"^wrapper\.cpp\.property$",
            "cpp-property-test",
        ),
        (
            "cpp-property-utf8-test",
            "freemvvm_cpp_utf8_property_tests_exe",
            r"^wrapper\.cpp\.property\.utf8$",
            "cpp-property-utf8-test",
        ),
        (
            "cpp-property-f64-test",
            "freemvvm_cpp_f64_property_tests_exe",
            r"^wrapper\.cpp\.property\.f64$",
            "cpp-property-f64-test",
        ),
        (
            "cpp-property-decimal-test",
            "freemvvm_cpp_decimal64_property_tests_exe",
            r"^wrapper\.cpp\.property\.decimal$",
            "cpp-property-decimal-test",
        ),
        (
            "cpp-property-subscription-test",
            "freemvvm_cpp_property_subscription_tests_exe",
            r"^wrapper\.cpp\.property\.subscription$",
            "cpp-property-subscription-test",
        ),
        (
            "cpp-property-binding-test",
            "freemvvm_cpp_property_binding_tests_exe",
            r"^wrapper\.cpp\.property\.binding$",
            "cpp-property-binding-test",
        ),
        (
            "cpp-command-test",
            "freemvvm_cpp_command_tests_exe",
            r"^wrapper\.cpp\.command$",
            "cpp-command-test",
        ),
        (
            "cpp-controls-test",
            "freemvvm_cpp_ui_control_tests_exe",
            r"^wrapper\.cpp\.ui-controls$",
            "cpp-controls-test",
        ),
        (
            "cpp-input-test",
            "freemvvm_cpp_input_tests_exe",
            r"^wrapper\.cpp\.input$",
            "cpp-input-test",
        ),
    ],
)
def test_cpp_handler_preserves_configure_build_and_test(
    monkeypatch, command, target, test_filter, gate
):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, environment, results, name))

    monkeypatch.setattr(cpp, "run", record_run)
    environment = {"PATH": "test-path"}
    results = []

    assert cpp.handle(command, environment, results) is True

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
            f"{command}-build",
        ),
        (
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
        ),
    ]


def test_cpp_runtime_handler_preserves_no_tests_and_runtime_gates(monkeypatch):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, name))

    monkeypatch.setattr(cpp, "run", record_run)

    assert cpp.handle("cpp-runtime-test", {}, []) is True

    assert [name for _arguments, name in calls] == [
        "cmake-configure",
        "cpp-runtime-no-tests-configure",
        "cpp-runtime-no-tests-build",
        "cpp-runtime-test-build",
        "cpp-runtime-test",
    ]
    assert calls[-1][0] == [
        "ctest",
        "--test-dir",
        "out/cmake/dev",
        "-R",
        r"^wrapper\.cpp\.runtime$",
        "--output-on-failure",
    ]


@pytest.mark.parametrize(
    "command", ["cpp-package-test", "cpp-input-consumer-test"]
)
def test_cpp_package_commands_preserve_relocatable_consumer_gates(
    monkeypatch, command
):
    calls = []

    def record_run(arguments, environment, results, name):
        calls.append((arguments, name))

    monkeypatch.setattr(cpp, "run", record_run)

    assert cpp.handle(command, {}, []) is True

    assert [name for _arguments, name in calls] == [
        "package-test-clean",
        "cpp-package-configure",
        "cpp-package-build",
        "cpp-package-install",
        "cpp-package-relocate",
        "cpp-package-consumer-configure",
        "cpp-package-consumer-build",
        "cpp-package-consumer-test",
    ]
    assert (
        calls[5][0][-1]
        == f"-DCMAKE_PREFIX_PATH={cpp.ROOT / 'out/cmake/package-moved'}"
    )


def test_cpp_handler_leaves_unknown_commands_unhandled():
    assert cpp.handle("unknown-command", {}, []) is False
