# fe.ps1 — run an npm/npx command in empty-project/ without the "Set-Location …"
# prefix.
#
# Usage (from the repo root):
#   .claude/scripts/fe.ps1 npx tsc --noEmit
#   .claude/scripts/fe.ps1 npm run test -- --run
#   .claude/scripts/fe.ps1 npm run gen:api:check
[CmdletBinding()]
param([Parameter(Mandatory, ValueFromRemainingArguments = $true)][string[]]$CmdArgs)

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location (Join-Path $repoRoot 'empty-project')
$exe = $CmdArgs[0]
$rest = if ($CmdArgs.Count -gt 1) { $CmdArgs[1..($CmdArgs.Count - 1)] } else { @() }
& $exe @rest
# the command's exit code is left in $LASTEXITCODE for the caller.
