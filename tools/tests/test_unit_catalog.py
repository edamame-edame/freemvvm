import json

import pytest

from tools.dev_support.unit_catalog import CatalogError, load_catalog


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value), encoding="utf-8")
    return path


def test_loads_schema_two_catalog(tmp_path):
    manifest = write_json(
        tmp_path / "units.yaml",
        {
            "schema": 2,
            "units": [
                {
                    "id": "UNIT-CORE-001",
                    "depends_on": ["LEGACY-MISSING-UNIT"],
                    "cases": [],
                    "tests": [],
                }
            ],
        },
    )

    assert load_catalog(manifest)[0]["id"] == "UNIT-CORE-001"


def test_loads_schema_three_shards_in_stable_order(tmp_path):
    manifest = write_json(
        tmp_path / "units.yaml",
        {"schema": 3, "unit_shards": "units/*.yaml"},
    )
    write_json(
        tmp_path / "units" / "zeta.yaml",
        {"units": [{"unit_name": "UNIT-Z", "path": {}, "cases": []}]},
    )
    write_json(
        tmp_path / "units" / "alpha.yaml",
        {"units": [{"unit_name": "UNIT-A", "path": {}, "cases": []}]},
    )

    assert [unit["unit_name"] for unit in load_catalog(manifest)] == [
        "UNIT-A",
        "UNIT-Z",
    ]


@pytest.mark.parametrize("pattern", ["", "../outside/*.yaml", "/tmp/*.yaml"])
def test_rejects_unsafe_or_empty_shard_pattern(tmp_path, pattern):
    manifest = write_json(
        tmp_path / "units.yaml", {"schema": 3, "unit_shards": pattern}
    )

    with pytest.raises(CatalogError):
        load_catalog(manifest)


def test_rejects_duplicate_unit_names_across_shards(tmp_path):
    manifest = write_json(
        tmp_path / "units.yaml",
        {"schema": 3, "unit_shards": "units/*.yaml"},
    )
    record = {"unit_name": "UNIT-A", "path": {}, "cases": []}
    write_json(tmp_path / "units" / "one.yaml", {"units": [record]})
    write_json(tmp_path / "units" / "two.yaml", {"units": [record]})

    with pytest.raises(CatalogError, match="Duplicate unit name"):
        load_catalog(manifest)


def test_rejects_shard_pattern_without_matches(tmp_path):
    manifest = write_json(
        tmp_path / "units.yaml",
        {"schema": 3, "unit_shards": "units/*.yaml"},
    )

    with pytest.raises(CatalogError, match="matched no files"):
        load_catalog(manifest)


def test_rejects_missing_dependencies(tmp_path):
    manifest = write_json(
        tmp_path / "units.yaml",
        {"schema": 3, "unit_shards": "units/*.yaml"},
    )
    write_json(
        tmp_path / "units" / "one.yaml",
        {
            "units": [
                {
                    "unit_name": "UNIT-A",
                    "path": {},
                    "depends_on": ["UNIT-MISSING"],
                    "cases": [],
                }
            ]
        },
    )

    with pytest.raises(CatalogError, match="unresolved dependencies"):
        load_catalog(manifest)


def test_rejects_invalid_case_command(tmp_path):
    manifest = write_json(
        tmp_path / "units.yaml",
        {"schema": 3, "unit_shards": "units/*.yaml"},
    )
    write_json(
        tmp_path / "units" / "one.yaml",
        {
            "units": [
                {
                    "unit_name": "UNIT-A",
                    "path": {},
                    "cases": [
                        {
                            "case_name": "case-a",
                            "command": ["cargo", "test", 4],
                        }
                    ],
                }
            ]
        },
    )

    with pytest.raises(CatalogError, match="command"):
        load_catalog(manifest)
