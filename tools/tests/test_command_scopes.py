from tools.dev_support.command_scopes import COMMAND_SCOPES, scope_for


def test_command_scope_registry_preserves_all_report_scopes():
    expected = {
        "abi-command-smoke": "abi-command-smoke",
        "rust-wrapper-runtime-test": "rust-wrapper-runtime-unit-test",
        "rust-wrapper-runtime-consumer-test": "rust-wrapper-runtime-consumer-test",  # noqa: E501
        "rust-wrapper-ui-controls-test": "rust-wrapper-ui-controls-test",
        "rust-wrapper-window-close-test": "rust-wrapper-window-close-test",
        "rust-wrapper-property-bool-i64-test": (
            "rust-wrapper-property-bool-i64-consumer-test"
        ),
        "rust-wrapper-property-utf8-test": (
            "rust-wrapper-property-utf8-consumer-test"
        ),
        "rust-wrapper-property-f64-test": (
            "rust-wrapper-property-f64-consumer-test"
        ),
        "rust-wrapper-property-decimal-test": (
            "rust-wrapper-property-decimal-consumer-test"
        ),
        "python-runtime-test": "python-runtime-unit-test",
        "python-runtime-consumer-test": "python-runtime-consumer-test",
        "python-property-bool-i64-test": "python-property-bool-i64-unit-test",
        "python-property-bool-i64-consumer-test": (
            "python-property-bool-i64-consumer-test"
        ),
        "python-property-utf8-test": "python-property-utf8-unit-test",
        "python-property-utf8-consumer-test": "python-property-utf8-consumer-test",  # noqa: E501
        "python-property-f64-test": "python-property-f64-unit-test",
        "python-property-f64-consumer-test": "python-property-f64-consumer-test",  # noqa: E501
        "python-property-decimal-test": "python-property-decimal-unit-test",
        "python-property-decimal-consumer-test": (
            "python-property-decimal-consumer-test"
        ),
        "python-binding-test": "python-binding-unit-test",
        "python-binding-consumer-test": "python-binding-consumer-test",
        "python-command-test": "python-command-unit-test",
        "python-command-consumer-test": "python-command-consumer-test",
        "python-controls-test": "python-controls-unit-test",
        "python-input-test": "python-input-event-wrapper-unit-test",
        "python-input-consumer-test": "python-input-event-consumer-test",
        "python-controls-consumer-test": "python-controls-consumer-test",
        "python-window-test": "python-window-unit-test",
        "python-window-consumer-test": "python-window-consumer-test",
        "python-window-peer-smoke-test": "python-window-peer-smoke-test",
        "csharp-runtime-test": "csharp-runtime-unit-test",
        "csharp-property-test": "csharp-property-unit-test",
        "csharp-command-test": "csharp-command-unit-test",
        "csharp-controls-test": "csharp-controls-unit-test",
        "csharp-input-test": "csharp-input-unit-test",
        "csharp-controls-consumer-test": "csharp-controls-consumer-test",
        "csharp-input-consumer-test": "csharp-input-consumer-test",
        "csharp-window-test": "csharp-window-unit-test",
        "csharp-window-consumer-test": "csharp-window-consumer-test",
        "csharp-command-consumer-test": "csharp-command-consumer-test",
        "csharp-runtime-consumer-test": "csharp-runtime-consumer-test",
        "csharp-property-consumer-test": "csharp-property-consumer-test",
        "cpp-package-test": "cpp-package-integration-test",
        "cpp-property-test": "cpp-property-unit-test",
        "cpp-property-utf8-test": "cpp-property-utf8-unit-test",
        "cpp-property-f64-test": "cpp-property-f64-unit-test",
        "cpp-property-decimal-test": "cpp-property-decimal-unit-test",
        "cpp-property-subscription-test": "cpp-property-subscription-unit-test",  # noqa: E501
        "cpp-property-binding-test": "cpp-property-binding-unit-test",
        "cpp-command-test": "cpp-command-unit-test",
        "cpp-runtime-test": "cpp-runtime-unit-test",
    }

    assert COMMAND_SCOPES == expected


def test_unmapped_commands_keep_the_development_bootstrap_scope():
    assert scope_for("doctor") == "development-bootstrap"
    assert scope_for("check") == "development-bootstrap"
    assert scope_for("format") == "development-bootstrap"
    assert scope_for("minimum") == "development-bootstrap"
    assert scope_for("linux-platform-docker") == "development-bootstrap"
    assert scope_for("unknown-command") == "development-bootstrap"
