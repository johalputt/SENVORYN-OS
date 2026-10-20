#!/usr/bin/env python3
"""Generate or verify the SHA-256 manifest for the Senvoryn contract pack.

The manifest covers the *contract artifacts* — the machine-readable contracts,
the reference model, the tests, the decision records, and the normative
specification documents. Living documents (README, ROADMAP, CONTRIBUTING, …)
are deliberately excluded: they change often and their integrity is not what
the manifest is protecting.

Usage:
    python scripts/generate-checksums.py --write   # regenerate SHA256SUMS.txt
    python scripts/generate-checksums.py --check   # verify (exit 1 on mismatch)

Exit codes:
    0  manifest matches / was written successfully
    1  mismatch, missing file, or unexpected extra file
    2  bad usage
"""
from __future__ import annotations

import argparse
import hashlib
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "SHA256SUMS.txt"

# Contract artifacts, in manifest order. Anything not listed here is out of
# scope for the checksum manifest.
INCLUDE_DIRS = (
    "schemas",
    "state-machines",
    "errors",
    "examples",
    "decisions",
    "reference",
    "tests",
    "evidence",
    "docs/specs",
    "docs/constitution",
)

# Deliberately NOT covered by the manifest:
#
#   requirements-dev.txt — this is a dependency manifest, not a contract
#       artifact. It changes whenever a dependency is updated (including
#       automatically, by Dependabot), so including it would make routine
#       dependency bumps look like tampering. Dependency integrity is handled
#       by the dependency-review CI job and the release SBOM / license
#       inventory instead.
#
#   README.md, ROADMAP.md, CONTRIBUTING.md, … — living documents that change
#       often and are protected by review rather than by a checksum.
#
# The manifest protects the artifacts whose *meaning* must not drift silently:
# the schemas, the state machines, the denial registry, the reference model,
# the tests, the decision records, the evidence, and the normative specs.
INCLUDE_FILES = ()

# Never hashed: build noise, caches, editor droppings.
EXCLUDE_DIR_NAMES = {"__pycache__", ".git", ".mypy_cache", ".pytest_cache", ".ruff_cache"}
EXCLUDE_SUFFIXES = {".pyc", ".pyo", ".swp", ".orig", ".rej"}


def _is_excluded(path: Path) -> bool:
    if path.suffix in EXCLUDE_SUFFIXES:
        return True
    return any(part in EXCLUDE_DIR_NAMES for part in path.parts)


def collect() -> list[Path]:
    """Return the contract artifacts, sorted for a stable manifest."""
    found: list[Path] = []

    for rel_dir in INCLUDE_DIRS:
        base = ROOT / rel_dir
        if not base.is_dir():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            if _is_excluded(path.relative_to(ROOT)):
                continue
            found.append(path)

    for rel_file in INCLUDE_FILES:
        path = ROOT / rel_file
        if path.is_file():
            found.append(path)

    return sorted(set(found), key=lambda p: p.relative_to(ROOT).as_posix())


def sha256(path: Path) -> str:
    """Hash a file.

    Text files are hashed over *canonical* content: CRLF and CR line endings
    are normalized to LF first. Without this, the manifest would depend on
    the checkout platform's line-ending settings, and a checksum computed on
    Windows would not match the same file checked out on Linux in CI.

    Binary files (anything containing a NUL byte — for example the .docx
    archives) are hashed as raw bytes.
    """
    raw = path.read_bytes()
    if b"\x00" not in raw:
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            text = None
        if text is not None:
            canonical = text.replace("\r\n", "\n").replace("\r", "\n").encode("utf-8")
            return hashlib.sha256(canonical).hexdigest()

    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build_manifest() -> str:
    lines = []
    for path in collect():
        rel = path.relative_to(ROOT).as_posix()
        lines.append(f"{sha256(path)}  {rel}")
    return "\n".join(lines) + "\n"


def parse_manifest(text: str) -> dict[str, str]:
    entries: dict[str, str] = {}
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        digest, _, name = line.partition("  ")
        if not digest or not name:
            continue
        entries[name.strip()] = digest.strip()
    return entries


def check() -> int:
    if not MANIFEST.exists():
        print(f"FAIL: {MANIFEST.name} not found. Run: python scripts/generate-checksums.py --write")
        return 1

    expected = parse_manifest(MANIFEST.read_text(encoding="utf-8"))
    actual = {p.relative_to(ROOT).as_posix(): sha256(p) for p in collect()}

    problems: list[str] = []

    for name in sorted(set(expected) - set(actual)):
        problems.append(f"MISSING   {name}  (listed in manifest but not on disk)")
    for name in sorted(set(actual) - set(expected)):
        problems.append(f"UNTRACKED {name}  (on disk but not listed in the manifest)")
    for name in sorted(set(expected) & set(actual)):
        if expected[name] != actual[name]:
            problems.append(f"CHANGED   {name}")

    if problems:
        print(f"FAIL: {len(problems)} checksum problem(s)")
        for problem in problems:
            print(f"  {problem}")
        print("\nIf the change was intentional, regenerate with:")
        print("  python scripts/generate-checksums.py --write")
        return 1

    print(f"OK: {len(actual)} contract artifact(s) match the manifest.")
    return 0


def write() -> int:
    content = build_manifest()
    MANIFEST.write_text(content, encoding="utf-8", newline="\n")
    count = len([line for line in content.splitlines() if line.strip()])
    print(f"Wrote {MANIFEST.name} with {count} artifact(s).")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--write", action="store_true", help="regenerate the manifest")
    group.add_argument("--check", action="store_true", help="verify the manifest")
    args = parser.parse_args(argv)

    if args.write:
        return write()
    return check()


if __name__ == "__main__":
    sys.exit(main())
