"""Write machine-readable development verification reports."""

import json
from pathlib import Path


def write_json_report(path, report):
    """Write an indented UTF-8 JSON report with a trailing newline."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
