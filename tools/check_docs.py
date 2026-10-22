"""Check links, anchors, tables, and fences in active documents."""

import re
from pathlib import Path
from urllib.parse import unquote


ROOT = Path(__file__).resolve().parents[1]


def anchors(path):
    text = path.read_text(encoding="utf-8-sig")
    result = set(re.findall(r'<a\s+id="([^"]+)"', text))
    for heading in re.findall(r"^#{1,6}\s+(.+)", text, re.M):
        heading = re.sub(r"<[^>]+>", "", heading).lower()
        result.add(re.sub(r"[^\w\- ]", "", heading).replace(" ", "-"))
    return result


def main():
    paths = [ROOT / "README.md", ROOT / "DEVELOPMENT.md"]
    requirements = ROOT / ".docs/11_requirements"
    paths += [requirements / "reqirements-specification.md"]
    paths += sorted(requirements.glob("RQ-*.md"))
    for directory in [
        "12_design",
        "13_detail_design",
        "14_secure_design",
        "30_release_test",
        "31_version_test",
        "32_unit_test",
    ]:
        paths += [
            path
            for path in (ROOT / ".docs" / directory).rglob("*.md")
            if "archive" not in path.parts
        ]
    errors = []
    links = 0
    for path in paths:
        if not path.is_file():
            errors.append(f"Missing document: {path.relative_to(ROOT)}")
            continue
        text = path.read_text(encoding="utf-8-sig")
        explicit = re.findall(r'<a\s+id="([^"]+)"', text)
        if len(set(explicit)) != len(explicit):
            errors.append(f"Duplicate anchor: {path.relative_to(ROOT)}")
        fence = False
        width = None
        for number, line in enumerate(text.splitlines(), 1):
            label = f"{path.relative_to(ROOT)}:{number}"
            if re.match(r"\s*(```|~~~)", line):
                fence = not fence
                width = None
                continue
            if fence:
                continue
            if line.startswith("|"):
                columns = len(re.split(r"(?<!\\)\|", line)) - 2
                if width is not None and width != columns:
                    errors.append(f"Table width: {label}")
                width = columns
            else:
                width = None
            for link in re.findall(r"\]\(([^)]+)\)", line):
                if re.match(r"[A-Za-z]+:", link):
                    continue
                filename, _, anchor = unquote(link.strip("<>")).partition("#")
                target = (
                    (path.parent / filename).resolve() if filename else path
                )
                links += 1
                if not target.exists():
                    errors.append(f"Missing link: {label}: {link}")
                elif anchor and anchor not in anchors(target):
                    errors.append(f"Missing anchor: {label}: {link}")
        if fence:
            errors.append(f"Unclosed fence: {path.relative_to(ROOT)}")
    print(f"Active documents: {len(paths)}, local links: {links}")
    for error in errors:
        print(error)
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(main())
