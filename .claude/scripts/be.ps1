# be.ps1 — run a cargo command in rust-backend/ without the fragile
# "Set-Location …; $env:APP_ENVIRONMENT=…; cargo …" prefix. Sets the test
# environment automatically for test subcommands.
#
# Usage (from the repo root):
#   .claude/scripts/be.ps1 test-fast
#   .claude/scripts/be.ps1 check --all-targets --all-features
[CmdletBinding()]
param([Parameter(ValueFromRemainingArguments = $true)][string[]]$CargoArgs)

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location (Join-Path $repoRoot 'rust-backend')
if ($CargoArgs.Count -gt 0 -and $CargoArgs[0] -like 'test*') { $env:APP_ENVIRONMENT = 'test' }
& cargo @CargoArgs
# cargo's exit code is left in $LASTEXITCODE for the caller (no `exit`, which
# misbehaves when the call is piped).
