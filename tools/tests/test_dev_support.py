import json
import os
import sys

from tools.dev_support.environment import bootstrap_environment
from tools.dev_support.process import resolve_executable, run
from tools.dev_support.reports import write_json_report


def test_bootstrap_environment_preserves_host_values_and_pins_tools():
    host = {"PATH": "host-path", "CUSTOM": "kept"}

    result = bootstrap_environment(host, "venv-bin", ";")

    assert result["PATH"] == "venv-bin;host-path"
    assert result["CUSTOM"] == "kept"
    assert result["PYTHONUTF8"] == "1"
    assert result["PYTHONIOENCODING"] == "utf-8"
    assert result["DOTNET_CLI_TELEMETRY_OPTOUT"] == "1"
    assert result["DOTNET_NOLOGO"] == "1"
    assert host == {"PATH": "host-path", "CUSTOM": "kept"}


def test_write_json_report_creates_parent_and_keeps_report_shape(tmp_path):
    report = {"status": "passed", "results": [{"exit_code": 0}]}
    destination = tmp_path / "verification" / "check.json"

    write_json_report(destination, report)

    assert json.loads(destination.read_text(encoding="utf-8")) == report
    assert destination.read_bytes().endswith(b"\n")


def test_resolve_executable_uses_child_path_and_keeps_arguments():
    resolved = resolve_executable([sys.executable, "-V"], {"PATH": os.environ["PATH"]})

    assert resolved[0].lower().endswith(("python.exe", "python"))
    assert resolved[1:] == ["-V"]


def test_run_records_gate_result_shape(capsys):
    results = []

    run([sys.executable, "-c", "pass"], {"PATH": os.environ["PATH"]}, results, "probe")

    assert results[0]["gate"] == "probe"
    assert results[0]["command"] == [sys.executable, "-c", "pass"]
    assert results[0]["exit_code"] == 0
    assert isinstance(results[0]["elapsed_seconds"], float)
    assert "[probe]" in capsys.readouterr().out
