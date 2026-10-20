<#
.SYNOPSIS
    Senvoryn OS — local verification (Windows PowerShell).

.DESCRIPTION
    Runs the full local verification suite:
      1. SHA-256 manifest integrity for the contract artifacts
      2. JSON Schema Draft 2020-12 validation
      3. The executable contract test suite
      4. The Rust workspace test suite
      5. The internal link check

.EXAMPLE
    pwsh -File scripts/verify-pack.ps1
#>

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-Location -Path (Join-Path $PSScriptRoot '..')

$python = if ($env:PYTHON) { $env:PYTHON } else { 'python' }
if (-not (Get-Command $python -ErrorAction SilentlyContinue)) {
    Write-Error 'No Python interpreter found. Install Python 3.11+ and retry.'
    exit 1
}

function Write-Step {
    param([string]$Title)
    Write-Host ('-' * 60) -ForegroundColor DarkGray
    Write-Host ">> $Title" -ForegroundColor Cyan
    Write-Host ('-' * 60) -ForegroundColor DarkGray
}

Write-Step '1/5  Contract artifact integrity (SHA-256)'
& $python 'scripts/generate-checksums.py' --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Step '2/5  JSON Schema validation'
@'
import json, sys
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
    except Exception as exc:
        failed += 1
        print(f"FAIL  {path.name}: {exc}")
print(f"{len(schemas) - failed}/{len(schemas)} schema(s) valid (Draft 2020-12)")
sys.exit(1 if failed else 0)
'@ | & $python -
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Step '3/5  Contract test suite'
$env:PYTHONPATH = 'reference'
& $python -m unittest discover -s tests -v
$testExit = $LASTEXITCODE

Write-Step '4/5  Rust workspace test suite'
# The Rust workspace carries the canonical profile, the SCP oracle and the
# Change Capsule executor. A toolchain is not assumed: without one, the step
# is skipped rather than failed, and the skip is said out loud.
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    cargo test --workspace
    if ($LASTEXITCODE -ne 0 -and $testExit -eq 0) { $testExit = $LASTEXITCODE }
} else {
    Write-Host 'SKIP: cargo is not on PATH. The Rust workspace was not tested.'
}

Write-Step '5/5  Internal link check'
if ($testExit -eq 0) {
    & $python 'scripts/check-links.py'
    if ($LASTEXITCODE -ne 0) { $testExit = $LASTEXITCODE }
}

Write-Host ('-' * 60) -ForegroundColor DarkGray
if ($testExit -eq 0) {
    Write-Host 'Verification complete.' -ForegroundColor Green
    Write-Host ''
    Write-Host 'Reminder: these checks validate the CONTRACT PACK only.'
    Write-Host 'They say nothing about an operating system, because none exists yet.'
    Write-Host ''
    Write-Host 'The mutation gate is not part of this suite: it needs a Rust toolchain'
    Write-Host 'and several minutes. Run it separately with:'
    Write-Host '    python scripts/mutation-gate.py'
} else {
    Write-Host 'Verification FAILED.' -ForegroundColor Red
}
Write-Host ('-' * 60) -ForegroundColor DarkGray

exit $testExit
