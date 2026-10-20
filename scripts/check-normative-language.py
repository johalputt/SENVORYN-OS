#!/usr/bin/env python3
"""Guard the project's claim discipline.

Two checks:
  1. Every normative document declares a status.
  2. No document makes an unmarked absolute security claim.

This exists because the project's central promise is that claims are backed by
evidence. A CI check is the cheapest way to keep a marketing sentence from
quietly becoming stronger than the artifact behind it.

Usage:
    python scripts/check-normative-language.py

Exit codes:
    0  all documents pass
    1  at least one failure
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Documents that carry normative weight and therefore must declare a status.
NORMATIVE_DOCS = (
    "docs/specs/SENVORYN_Core_Contract_Spec_v0.1.md",
    "docs/constitution/SENVORYN_OS_Constitution_v0.6.md",
    "ARCHITECTURE.md",
    "GOVERNANCE.md",
    "SECURITY.md",
    "CONTRIBUTING.md",
    "RELEASE_PROCESS.md",
    "ROADMAP.md",
)

# A document must contain at least one of these markers to be considered
# status-declaring.
STATUS_MARKERS = (
    re.compile(r"^\*\*Status:\*\*", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^Status:", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\*\*Conformance", re.MULTILINE | re.IGNORECASE),
    re.compile(r"^\| Status \|", re.MULTILINE | re.IGNORECASE),
)

# Absolute claims that must never appear without qualification. These are
# deliberately narrow: they catch the phrases that are simply false, without
# flagging legitimate discussion of security properties.
BANNED_ABSOLUTE_CLAIMS = (
    "unhackable",
    "unbreakable",
    "100% secure",
    "completely secure",
    "impossible to compromise",
    "impossible to exploit",
    "cannot be exploited",
    "guaranteed secure",
    "guarantees security",
    "guaranteed to be secure",
    "zero risk",
    "zero vulnerabilities",
    "no possible attack",
    "perfectly secure",
)

# A banned phrase inside a sentence carrying one of these markers is a
# discussion or a prohibition, not a claim. The constitution itself says
# '"100% secure" ... is never a valid metric' — that must not fail this check.
NEGATION_MARKERS = (
    "never",
    "not ",
    "n't",
    "cannot",
    "no one",
    "none",
    "without",
    "invalid",
    "forbidden",
    "prohibited",
    "refuse",
    "reject",
    "disallow",
)

# Split a line into rough sentences so a negation can be scoped to the
# sentence that actually contains the phrase.
SENTENCE_SPLIT = re.compile(r"(?<=[.!?])\s+")

# The README must keep a plain-language disclaimer that this is not a
# working operating system.
README_DISCLAIMER_PATTERNS = (
    re.compile(r"does not contain a working operating system", re.IGNORECASE),
    re.compile(r"not a production operating system", re.IGNORECASE),
    re.compile(r"is not a production operating system", re.IGNORECASE),
)

# The README must keep the "read this first" status section.
README_STATUS_MARKERS = (
    re.compile(r"read this first", re.IGNORECASE),
    re.compile(r"what this repository (is|is not)", re.IGNORECASE),
    re.compile(r"is not a production", re.IGNORECASE),
)


def check_status_declaration() -> list[str]:
    problems: list[str] = []
    for rel in NORMATIVE_DOCS:
        path = ROOT / rel
        if not path.exists():
            problems.append(f"{rel}: file not found")
            continue
        text = path.read_text(encoding="utf-8")
        if not any(marker.search(text) for marker in STATUS_MARKERS):
            problems.append(f"{rel}: no status declaration found (expected a '**Status:**' marker)")
    return problems


def check_absolute_claims() -> list[str]:
    problems: list[str] = []
    targets = [ROOT / rel for rel in NORMATIVE_DOCS]
    targets.append(ROOT / "README.md")
    for path in targets:
        if not path.exists():
            continue
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            for sentence in SENTENCE_SPLIT.split(line):
                lowered = sentence.lower()
                for phrase in BANNED_ABSOLUTE_CLAIMS:
                    if phrase not in lowered:
                        continue
                    # A sentence that negates or prohibits the phrase is a
                    # discussion of the phrase, not a claim made with it.
                    if any(marker in lowered for marker in NEGATION_MARKERS):
                        continue
                    problems.append(
                        f"{path.relative_to(ROOT).as_posix()}:{number}: absolute claim '{phrase}'"
                    )
    return problems


def check_readme_disclaimer() -> list[str]:
    problems: list[str] = []
    path = ROOT / "README.md"
    if not path.exists():
        return ["README.md: file not found"]
    text = path.read_text(encoding="utf-8")
    if not any(p.search(text) for p in README_DISCLAIMER_PATTERNS):
        problems.append("README.md: missing the plain-language 'not a working operating system' disclaimer")
    if not any(p.search(text) for p in README_STATUS_MARKERS):
        problems.append("README.md: missing the 'read this first' status section")
    return problems


def check_conformance_label() -> list[str]:
    """The README must not advertise a conformance label it has not earned."""
    problems: list[str] = []
    path = ROOT / "README.md"
    if not path.exists():
        return problems
    text = path.read_text(encoding="utf-8")
    for label in ("Release-assured", "Profile-conformant"):
        if re.search(rf"\b{label}\b", text) and not re.search(rf"{label}.*(Not reached|❌|not reached)", text):
            problems.append(f"README.md: mentions '{label}' without marking it as not reached")
    return problems


def main() -> int:
    all_problems: list[str] = []
    all_problems += check_status_declaration()
    all_problems += check_absolute_claims()
    all_problems += check_readme_disclaimer()
    all_problems += check_conformance_label()

    if all_problems:
        print(f"FAIL: {len(all_problems)} claim-discipline problem(s)")
        for problem in all_problems:
            print(f"  {problem}")
        return 1

    print(f"OK: {len(NORMATIVE_DOCS)} normative document(s) declare status; no unmarked absolute claims.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
