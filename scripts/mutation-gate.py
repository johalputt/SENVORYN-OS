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
was found. A run that is killed outright cannot restore anything, so the
original is also written to a backup file beside the source, and the next run
refuses to start if the tree is not in the state the manifest expects.

Usage::

    python scripts/mutation-gate.py                    # run every mutation
    python scripts/mutation-gate.py M11-risk-downgrade # run one
    python scripts/mutation-gate.py --list

Do not pipe this script's output through a truncating filter. A pipeline that
closes early kills the process mid-mutation, and the tree is left broken.
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


def run_test(mutation):
    """Runs the test that must catch this mutation.

    The language follows the mutated file: a Rust file is checked by a Rust
    test, a Python file by a Python one. The manifest does not have to say
    which, and cannot say it wrongly.
    """
    if language_of(mutation) == "python":
        # A cached bytecode file can shadow the mutated source. CPython
        # invalidates a cache by the source's size and its modification time
        # in whole seconds, so a mutation that leaves the size unchanged
        # within the same second is read from the cache and never runs — the
        # gate would report a control as untested when the control is fine,
        # or credit a kill to a mutation that did nothing. The caches are
        # removed and no new ones are written.
        for directory in ("reference", "tests", "scripts"):
            shutil.rmtree(ROOT / directory / "__pycache__", ignore_errors=True)
        command = [sys.executable, "-B", "-m", "unittest", mutation["test"]]
        environment = dict(os.environ)
        environment["PYTHONPATH"] = str(ROOT / "reference")
        environment["PYTHONDONTWRITEBYTECODE"] = "1"
    else:
        command = [
            "cargo",
            "test",
            "-p",
            mutation["package"],
            "--test",
            mutation["test_file"],
            mutation["test"],
            "--",
            "--exact",
        ]
        environment = dict(os.environ)
        environment.setdefault("CARGO_TERM_COLOR", "never")
    completed = subprocess.run(
        command, cwd=ROOT, env=environment, capture_output=True, text=True
    )
    return completed.returncode, completed.stdout + completed.stderr


def language_of(mutation) -> str:
    """The language a mutation is written in, which is the language of the
    file it breaks."""
    return "python" if mutation["file"].endswith(".py") else "rust"


def is_invalid(mutation, output: str) -> bool:
    """Whether the mutation failed to build rather than failed a test.

    A mutation that does not compile proves nothing: the test never ran, so
    nothing was shown about the control it was supposed to break.
    """
    if language_of(mutation) == "python":
        markers = ("SyntaxError", "IndentationError", "ModuleNotFoundError", "ImportError")
    else:
        markers = ("error[", "could not compile")
    return any(marker in output for marker in markers)


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

    if "rust" in {language_of(mutation) for mutation in mutations} and shutil.which("cargo") is None:
        print("cargo is not on PATH", file=sys.stderr)
        return 2

    # A run that was killed part way through leaves the tree mutated, and the
    # next run would then mutate a file that is already broken — which proves
    # nothing and hides the damage. The tree is checked before anything is
    # touched, so a dirty tree is reported rather than compounded.
    dirty = []
    for mutation in mutations:
        path = ROOT / mutation["file"]
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        if mutation["find"] not in text:
            dirty.append((mutation["id"], mutation["file"]))
    if dirty:
        print("FAIL: the working tree is not in the state the manifest expects.", file=sys.stderr)
        for identifier, file in dirty:
            print(f"  {identifier}: the text to mutate is not in {file}", file=sys.stderr)
        print(
            "A previous run was probably interrupted. Restore the files and try again.",
            file=sys.stderr,
        )
        return 1

    survivors = []
    invalid = []
    for mutation in mutations:
        identifier = mutation["id"]
        path = ROOT / mutation["file"]
        original = path.read_bytes()
        # The original is also written beside the file. If this process is
        # killed outright, the `finally` block never runs, and the file on disk
        # is the only way back.
        with open(path, "rb") as handle:
            original = handle.read()
        backup = path.with_suffix(path.suffix + ".mutation-backup")
        backup.write_bytes(original)
        print(f"[{identifier}] {mutation['control']}")
        try:
            occurrences = apply_mutation(path, mutation["find"], mutation["replace"])
            print(f"    mutated {mutation['file']} ({occurrences} occurrence(s))")
            code, output = run_test(mutation)
        finally:
            path.write_bytes(original)
            backup.unlink(missing_ok=True)

        expected = mutation["expect_output_contains"]
        if code == 0:
            survivors.append(identifier)
            print(f"    SURVIVED — {mutation['test']} stayed green. The control is not tested.")
            continue
        if is_invalid(mutation, output):
            invalid.append(identifier)
            print("    INVALID — the mutation does not build. It proves nothing.")
            print("    " + "\n    ".join(output.splitlines()[:12]))
            continue
        if mutation["test"] not in output:
            survivors.append(identifier)
            print(f"    SURVIVED — a different test went red, not {mutation['test']}.")
            print("    A mutation caught by the wrong test proves nothing.")
            print("    " + "\n    ".join(output.splitlines()[:24]))
            continue
        if expected not in output:
            survivors.append(identifier)
            print(f"    SURVIVED — {mutation['test']} went red, but not about {expected!r}.")
            print("    A mutation caught by the wrong case proves nothing.")
            print("    " + "\n    ".join(output.splitlines()[:24]))
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
