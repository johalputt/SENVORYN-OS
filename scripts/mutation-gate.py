#!/usr/bin/env python3
"""The mutation gate: prove that every control has a test that fails when it is broken.

A control that has never been seen to fail on the thing it forbids has proved
nothing. This script breaks each control in turn and requires a **specific named
test** to go red. A mutation that survives is a control that is not tested, and
the script fails.

Three ways this has produced a false green before, and each is designed out here:

1. **A test that calls the function under test to judge its output** compares the
   code with itself. The mutations here change files under ``src/`` and the
   tests that catch them live under ``tests/``, so no test judges its own
   mutation.
2. **A seed that trips several rules kills a mutation of none of them.** Each
   mutation names the case that must catch it, and the script checks that the
   failing output mentions *that case*. A mutation caught by a different case
   does not count.
3. **Two guards for one outcome are individually unkillable by an outcome-only
   assertion.** The vectors assert the specific reason code, and two of the
   mutations change nothing but the reason code, so a suite that merely checked
   "it failed" would not catch them.

The mutated files are restored byte for byte from memory in a ``finally``
block, so a failure, a crash or a Ctrl-C leaves the working tree exactly as it
was found.

Usage::

    python scripts/mutation-gate.py                    # run every mutation
    python scripts/mutation-gate.py M11-risk-downgrade # run one
    python scripts/mutation-gate.py --list
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "scripts" / "mutations.json"


def load_manifest():
    with open(MANIFEST, encoding="utf-8") as handle:
        return json.load(handle)


def apply_mutation(path: Path, find: str, replace: str) -> int:
    """Replace every occurrence of `find` and return how many were replaced.

    Returning the count is the point: a mutation that matched nothing would
    leave the code untouched, the test would stay green, and the script would
    report the control as untested for the wrong reason.
    """
    original = path.read_bytes()
    occurrences = original.count(find.encode("utf-8"))
    if occurrences == 0:
        raise SystemExit(
            f"the mutation did not apply: {path.relative_to(ROOT)} does not contain {find!r}"
        )
    path.write_bytes(original.replace(find.encode("utf-8"), replace.encode("utf-8")))
    return occurrences


def run_test(package: str, test_file: str, test_name: str):
    command = [
        "cargo",
        "test",
        "-p",
        package,
        "--test",
        test_file,
        test_name,
        "--",
        "--exact",
    ]
    environment = dict(os.environ)
    environment.setdefault("CARGO_TERM_COLOR", "never")
    completed = subprocess.run(
        command, cwd=ROOT, env=environment, capture_output=True, text=True
    )
    return completed.returncode, completed.stdout + completed.stderr


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("mutation", nargs="?", help="run only this mutation by id")
    parser.add_argument("--list", action="store_true", help="list the mutations and exit")
    arguments = parser.parse_args()

    manifest = load_manifest()
    mutations = manifest["mutations"]

    if arguments.list:
        for mutation in mutations:
            print(f"{mutation['id']:<36} gate {mutation['gate']:>2}  {mutation['control']}")
        print(f"\n{len(mutations)} mutation(s)")
        return 0

    if arguments.mutation:
        selected = [m for m in mutations if m["id"] == arguments.mutation]
        if not selected:
            print(f"no mutation with id {arguments.mutation!r}", file=sys.stderr)
            return 2
        mutations = selected

    if shutil.which("cargo") is None:
        print("cargo is not on PATH", file=sys.stderr)
        return 2

    survivors = []
    invalid = []
    for mutation in mutations:
        identifier = mutation["id"]
        path = ROOT / mutation["file"]
        original = path.read_bytes()
        print(f"[{identifier}] {mutation['control']}")
        try:
            occurrences = apply_mutation(path, mutation["find"], mutation["replace"])
            print(f"    mutated {mutation['file']} ({occurrences} occurrence(s))")
            code, output = run_test(
                mutation["package"], mutation["test_file"], mutation["test"]
            )
        finally:
            path.write_bytes(original)

        expected = mutation["expect_output_contains"]
        if code == 0:
            survivors.append(identifier)
            print(f"    SURVIVED — {mutation['test']} stayed green. The control is not tested.")
            continue
        if "error[" in output or "could not compile" in output:
            invalid.append(identifier)
            print("    INVALID — the mutation does not compile. It proves nothing.")
            print("    " + "\n    ".join(output.splitlines()[:12]))
            continue
        if mutation["test"] not in output:
            survivors.append(identifier)
            print(f"    SURVIVED — a different test went red, not {mutation['test']}.")
            print("    A mutation caught by the wrong test proves nothing.")
            continue
        if expected not in output:
            survivors.append(identifier)
            print(f"    SURVIVED — {mutation['test']} went red, but not about {expected!r}.")
            print("    A mutation caught by the wrong case proves nothing.")
            continue
        print(f"    killed by {mutation['test']} (mentions {expected!r})")

    print()
    if survivors or invalid:
        if survivors:
            print(f"FAIL: {len(survivors)} mutation(s) survived:")
            for identifier in survivors:
                print(f"  {identifier}")
            print("A surviving mutation is a control that no test enforces.")
        if invalid:
            print(f"FAIL: {len(invalid)} mutation(s) were invalid:")
            for identifier in invalid:
                print(f"  {identifier}")
            print("An invalid mutation is one that does not compile. It proves nothing.")
        return 1
    print(f"OK: all {len(mutations)} mutation(s) were killed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
