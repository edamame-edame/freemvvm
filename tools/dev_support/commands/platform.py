"""Platform command handlers."""

import subprocess
import sys
from pathlib import Path

from ..reports import write_json_report


COMMANDS = ("linux-platform-docker",)
ROOT = Path(__file__).resolve().parents[3]


def run_linux_platform_docker():
    """Build and run the provisional Linux Platform Docker validation."""
    report_path = ROOT / ".local/verification/linux-platform-docker.json"
    image = "freemvvm-linux-platform:debian12"
    report = {
        "scope": "linux-platform-docker-provisional",
        "status": "blocked",
        "qualification": "provisional only; not hardware or release pass",
        "source_sha": None,
        "image_id": None,
        "commands": [],
    }

    def execute(name, arguments):
        print(f"\n[{name}] {' '.join(map(str, arguments))}", flush=True)
        result = subprocess.run(
            arguments,
            cwd=ROOT,
            check=False,
            text=True,
            encoding="utf-8",
            errors="replace",
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
        )
        print(result.stdout, end="", flush=True)
        report["commands"].append(
            {
                "name": name,
                "argv": list(map(str, arguments)),
                "exit_code": result.returncode,
                "output_tail": result.stdout[-3000:],
            }
        )
        if result.returncode:
            raise RuntimeError(
                f"{name} failed with exit code {result.returncode}"
            )
        return result.stdout.strip()

    try:
        report["source_sha"] = subprocess.check_output(
            ["git", "rev-parse", "HEAD"],
            cwd=ROOT,
            text=True,
            encoding="utf-8",
        ).strip()
        report["worktree_status"] = subprocess.check_output(
            ["git", "status", "--short"],
            cwd=ROOT,
            text=True,
            encoding="utf-8",
        ).splitlines()
        execute(
            "linux-platform-docker-build",
            [
                "docker",
                "build",
                "--pull",
                "-f",
                "docker/linux-platform/Dockerfile",
                "-t",
                image,
                ".",
            ],
        )
        report["image_id"] = execute(
            "linux-platform-docker-image-id",
            ["docker", "image", "inspect", "--format={{.Id}}", image],
        )
        execute(
            "linux-platform-docker-validation",
            [
                "docker",
                "run",
                "--rm",
                "--mount",
                f"type=bind,source={ROOT},target=/workspace",
                "--workdir",
                "/workspace",
                image,
                "bash",
                "tools/linux_platform_docker.sh",
            ],
        )
        report["status"] = "provisional_pass"
    except (RuntimeError, subprocess.CalledProcessError, OSError) as error:
        report["error"] = str(error)
        print(
            f"Linux platform Docker validation unavailable/failed: {error}",
            file=sys.stderr,
        )
    finally:
        write_json_report(report_path, report)
        print(f"Report: {report_path.relative_to(ROOT)}", flush=True)
    return 0 if report["status"] == "provisional_pass" else 1
