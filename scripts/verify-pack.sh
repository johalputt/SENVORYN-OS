#!/usr/bin/env bash
#
# Senvoryn OS — local verification (POSIX shells)
#
# Runs the full local verification suite:
#   1. SHA-256 manifest integrity for the contract artifacts
#   2. JSON Schema Draft 2020-12 validation
#   3. The executable contract test suite
#
# Usage:
#   bash scripts/verify-pack.sh
#
# Override the interpreter with PYTHON=/path/to/python if needed.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# ---------------------------------------------------------------------------
# Locate a working Python interpreter.
#
# `command -v python3` is not enough: on some systems it resolves to a stub
# (for example the Microsoft Store app-execution alias on Windows) that exists
# but cannot run. Probe each candidate for real.
# ---------------------------------------------------------------------------
if [ -z "${PYTHON:-}" ]; then
  for candidate in python3 python; do
    if command -v "$candidate" >/dev/null 2>&1 && "$candidate" --version >/dev/null 2>&1; then
      PYTHON="$candidate"
      break
    fi
  done
fi

if [ -z "${PYTHON:-}" ]; then
  echo "ERROR: no working Python interpreter found." >&2
  echo "       Install Python 3.11+ and retry, or set PYTHON=/path/to/python." >&2
  exit 1
fi

echo "Using interpreter: $PYTHON ($("$PYTHON" --version 2>&1))"

hr()  { printf '%s\n' "------------------------------------------------------------"; }
step() { hr; printf '>> %s\n' "$1"; hr; }

# ---------------------------------------------------------------------------
step "1/4  Contract artifact integrity (SHA-256)"
"$PYTHON" scripts/generate-checksums.py --check

# ---------------------------------------------------------------------------
step "2/4  JSON Schema validation"
"$PYTHON" - <<'PY'
import json
import sys
from pathlib import Path

try:
    import jsonschema
except ImportError:
    print("SKIP: jsonschema not installed (pip install -r requirements-dev.txt)")
    sys.exit(0)

root = Path.cwd()
schemas = sorted((root / "schemas").glob("*.schema.json"))
failed = 0
for path in schemas:
    try:
        jsonschema.Draft202012Validator.check_schema(json.loads(path.read_text(encoding="utf-8")))
    except Exception as exc:  # noqa: BLE001
        failed += 1
        print(f"FAIL  {path.name}: {exc}")

print(f"{len(schemas) - failed}/{len(schemas)} schema(s) valid (Draft 2020-12)")
sys.exit(1 if failed else 0)
PY

# ---------------------------------------------------------------------------
step "3/4  Contract test suite"
PYTHONPATH=reference "$PYTHON" -m unittest discover -s tests -v

# ---------------------------------------------------------------------------
step "4/4  Internal link check"
"$PYTHON" scripts/check-links.py

# ---------------------------------------------------------------------------
hr
echo "Verification complete."
echo
echo "Reminder: these checks validate the CONTRACT PACK only."
echo "They say nothing about an operating system, because none exists yet."
echo
echo "The mutation gate is not part of this suite: it needs a Rust toolchain"
echo "and several minutes. Run it separately with:"
echo "    python scripts/mutation-gate.py"
hr
