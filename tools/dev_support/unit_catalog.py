"""Load and validate the implementation-unit catalog."""

import json
from pathlib import Path


class CatalogError(ValueError):
    """Raised when a unit catalog or one of its shards is invalid."""


def load_catalog(manifest_path):
    """Return the catalog records from a schema 2 or schema 3 manifest."""
    manifest_path = Path(manifest_path).resolve()
    manifest = _read_json(manifest_path)
    if not isinstance(manifest, dict):
        raise CatalogError(
            f"Catalog manifest must be an object: {manifest_path}"
        )

    schema = manifest.get("schema")
    if schema == 2:
        records = manifest.get("units")
        if not isinstance(records, list):
            raise CatalogError("Schema 2 manifest must contain a units array")
        return _validate_records(records, "id", validate_dependencies=False)
    if schema != 3:
        raise CatalogError(f"Unsupported unit catalog schema: {schema!r}")

    pattern = manifest.get("unit_shards")
    if not isinstance(pattern, str) or not pattern:
        raise CatalogError("Schema 3 manifest must define unit_shards")
    relative_pattern = Path(pattern)
    if (
        relative_pattern.is_absolute()
        or relative_pattern.anchor
        or ".." in relative_pattern.parts
    ):
        raise CatalogError(
            "unit_shards must stay within the manifest directory"
        )

    shard_paths = sorted(
        (
            path
            for path in manifest_path.parent.glob(pattern)
            if path.is_file()
        ),
        key=lambda path: path.as_posix(),
    )
    if not shard_paths:
        raise CatalogError(f"unit_shards matched no files: {pattern}")

    records = []
    for shard_path in shard_paths:
        shard = _read_json(shard_path)
        if not isinstance(shard, dict) or not isinstance(
            shard.get("units"), list
        ):
            raise CatalogError(
                f"Shard must contain a units array: {shard_path}"
            )
        records.extend(shard["units"])
    return _validate_records(records, "unit_name", validate_dependencies=True)


def _read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise CatalogError(
            f"Cannot read JSON-compatible YAML {path}: {error}"
        ) from error


def _validate_records(records, name_field, validate_dependencies):
    names = set()
    for index, record in enumerate(records):
        location = f"unit record {index}"
        if not isinstance(record, dict):
            raise CatalogError(f"{location} must be an object")
        name = record.get(name_field)
        if not isinstance(name, str) or not name:
            raise CatalogError(f"{location} must define {name_field}")
        if name in names:
            raise CatalogError(f"Duplicate unit name: {name}")
        names.add(name)
        _validate_record_shape(record, name, name_field == "unit_name")

    if validate_dependencies:
        for record in records:
            dependencies = record.get("depends_on", [])
            missing = sorted(set(dependencies) - names)
            if missing:
                raise CatalogError(
                    f"Unit {record[name_field]} has unresolved dependencies: "
                    + ", ".join(missing)
                )
    return records


def _validate_record_shape(record, name, sharded_schema):
    dependencies = record.get("depends_on", [])
    if not isinstance(dependencies, list) or not all(
        isinstance(item, str) and item for item in dependencies
    ):
        raise CatalogError(f"Unit {name} depends_on must be a list of names")

    paths = record.get("path") if sharded_schema else None
    if sharded_schema:
        if not isinstance(paths, dict):
            raise CatalogError(f"Unit {name} path must be an object")
        for key, value in paths.items():
            values = value if isinstance(value, list) else [value]
            if not all(isinstance(item, str) and item for item in values):
                raise CatalogError(
                    f"Unit {name} path.{key} must contain paths"
                )

    cases = record.get("cases", [])
    if not isinstance(cases, list):
        raise CatalogError(f"Unit {name} cases must be a list")
    if sharded_schema:
        case_names = set()
        for case in cases:
            if not isinstance(case, dict):
                raise CatalogError(f"Unit {name} cases must contain objects")
            case_name = case.get("case_name")
            if not isinstance(case_name, str) or not case_name:
                raise CatalogError(f"Unit {name} case must define case_name")
            if case_name in case_names:
                raise CatalogError(
                    f"Unit {name} has duplicate case_name: {case_name}"
                )
            case_names.add(case_name)
            command = case.get("command", [])
            if not isinstance(command, list) or not all(
                isinstance(item, str) and item for item in command
            ):
                raise CatalogError(
                    f"Unit {name} case {case_name} command must be a list "
                    "of arguments"
                )
    else:
        tests = record.get("tests", [])
        if not isinstance(tests, list):
            raise CatalogError(f"Unit {name} tests must be a list")
        for test in tests:
            if not isinstance(test, dict) or not isinstance(
                test.get("command"), list
            ):
                raise CatalogError(
                    f"Unit {name} tests must define command arrays"
                )
            if not all(
                isinstance(item, str) and item for item in test["command"]
            ):
                raise CatalogError(
                    f"Unit {name} test commands must be non-empty strings"
                )


def main():
    """Validate the repository unit catalog and print the unit count."""
    root = Path(__file__).resolve().parents[2]
    records = load_catalog(root / "tools/units.yaml")
    print(f"Unit catalog: {len(records)} units valid")


if __name__ == "__main__":
    main()
