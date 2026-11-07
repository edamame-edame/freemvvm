import json
from types import SimpleNamespace

from tools.dev_support.commands import platform


def test_platform_command_is_registered():
    assert platform.COMMANDS == ("linux-platform-docker",)


def test_linux_platform_docker_writes_provisional_success_report(
    monkeypatch, tmp_path
):
    monkeypatch.setattr(platform, "ROOT", tmp_path)
    outputs = iter(
        ["build complete\n", "sha256:test\n", "validation passed\n"]
    )
    commands = []

    def check_output(arguments, **kwargs):
        if arguments == ["git", "rev-parse", "HEAD"]:
            return "abc123\n"
        if arguments == ["git", "status", "--short"]:
            return "M pending.md\n"
        raise AssertionError(arguments)

    def run(arguments, **kwargs):
        commands.append(arguments)
        return SimpleNamespace(returncode=0, stdout=next(outputs))

    monkeypatch.setattr(platform.subprocess, "check_output", check_output)
    monkeypatch.setattr(platform.subprocess, "run", run)

    assert platform.run_linux_platform_docker() == 0

    report_path = tmp_path / ".local/verification/linux-platform-docker.json"
    report = json.loads(report_path.read_text(encoding="utf-8"))
    assert report["scope"] == "linux-platform-docker-provisional"
    assert report["status"] == "provisional_pass"
    assert report["qualification"] == (
        "provisional only; not hardware or release pass"
    )
    assert report["source_sha"] == "abc123"
    assert report["worktree_status"] == ["M pending.md"]
    assert report["image_id"] == "sha256:test"
    assert [entry["name"] for entry in report["commands"]] == [
        "linux-platform-docker-build",
        "linux-platform-docker-image-id",
        "linux-platform-docker-validation",
    ]
    assert report["commands"][-1]["argv"] == commands[-1]
    assert commands[-1][commands[-1].index("--mount") + 1] == (
        f"type=bind,source={tmp_path},target=/workspace"
    )


def test_linux_platform_docker_records_build_failure(monkeypatch, tmp_path):
    monkeypatch.setattr(platform, "ROOT", tmp_path)
    calls = []

    def check_output(arguments, **kwargs):
        return "abc123\n" if arguments[-1] == "HEAD" else ""

    def run(arguments, **kwargs):
        calls.append(arguments)
        return SimpleNamespace(returncode=1, stdout="docker unavailable")

    monkeypatch.setattr(platform.subprocess, "check_output", check_output)
    monkeypatch.setattr(platform.subprocess, "run", run)

    assert platform.run_linux_platform_docker() == 1

    report_path = tmp_path / ".local/verification/linux-platform-docker.json"
    report = json.loads(report_path.read_text(encoding="utf-8"))
    assert report["status"] == "blocked"
    assert report["error"] == (
        "linux-platform-docker-build failed with exit code 1"
    )
    assert report["commands"][0]["output_tail"] == "docker unavailable"
    assert len(calls) == 1
