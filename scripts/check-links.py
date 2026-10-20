#!/usr/bin/env python3
"""Check that every relative Markdown link in the repository resolves.

A broken link in a specification is a specification that cannot be followed,
and this repository makes a habit of citing its own artefacts. The check is a
tool rather than an inline block in the workflow so that it can be run
locally, in the same way as the other checks in `scripts/`.

Usage::

    python scripts/check-links.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
SKIP = ("http://", "https://", "mailto:", "#", "<")
SKIP_DIRS = {".git", "node_modules", "target", "__pycache__"}


def main() -> int:
    failures = 0
    checked = 0
    for path in sorted(ROOT.rglob("*.md")):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        text = path.read_text(encoding="utf-8")
        for target in LINK.findall(text):
            target = target.strip()
            if target.startswith(SKIP):
                continue
            target = target.split("#")[0].strip()
            if not target:
                continue
            checked += 1
            # Root-relative links (/FOO.md) resolve from the repo root;
            # everything else resolves relative to the containing file.
            resolved = (
                (ROOT / target.lstrip("/")) if target.startswith("/") else (path.parent / target)
            )
            if not resolved.exists():
                failures += 1
                print(f"FAIL  {path.relative_to(ROOT)}: broken link -> {target}")

    print(f"{checked - failures}/{checked} link(s) resolve")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
