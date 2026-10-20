#!/usr/bin/env python3
"""Lint every JSON document in the repository and validate the JSON Schemas.

Two checks:
  1. Every ``*.json`` file in the repository parses as JSON.
  2. Every ``schemas/*.schema.json`` is a valid JSON Schema Draft 2020-12.

Usage:
    python scripts/check-json.py

Exit codes:
    0  all documents parse and all schemas are valid
    1  at least one failure
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

SKIP_DIRS = {".git", "__pycache__", ".mypy_cache", ".pytest_cache", ".ruff_cache", "node_modules", ".venv", "venv"}


def iter_json_files() -> list[Path]:
    files: list[Path] = []
    for path in ROOT.rglob("*.json"):
        if any(part in SKIP_DIRS for part in path.relative_to(ROOT).parts):
            continue
        files.append(path)
    return sorted(files)


def main() -> int:
    failures = 0
    files = iter_json_files()

    for path in files:
        rel = path.relative_to(ROOT).as_posix()
        try:
            json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            failures += 1
            print(f"FAIL  {rel}: {exc}")

    print(f"{len(files) - failures}/{len(files)} JSON document(s) parse cleanly")

    try:
        import jsonschema
    except ImportError:
        print("SKIP: schema validation (jsonschema not installed)")
        return 1 if failures else 0

    schemas = sorted((ROOT / "schemas").glob("*.schema.json"))
    schema_failures = 0
    for path in schemas:
        rel = path.relative_to(ROOT).as_posix()
        try:
            jsonschema.Draft202012Validator.check_schema(json.loads(path.read_text(encoding="utf-8")))
        except Exception as exc:  # noqa: BLE001
            schema_failures += 1
            failures += 1
            print(f"FAIL  {rel}: {exc}")

    print(f"{len(schemas) - schema_failures}/{len(schemas)} schema(s) valid (JSON Schema Draft 2020-12)")

    if failures:
        print(f"\n{failures} failure(s).")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
