# install-git-hooks.ps1 — copy tracked hook sources into .git/hooks/, where
# git actually looks for them (that directory is never tracked by git itself).
# Run once per machine/clone. Worktrees share the main repo's .git/hooks/, so
# running this once from the main checkout covers every worktree too.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$gitCommonDir = (git -C $repoRoot rev-parse --git-common-dir).Trim()
if (-not [System.IO.Path]::IsPathRooted($gitCommonDir)) {
    $gitCommonDir = Join-Path $repoRoot $gitCommonDir
}
$hooksDir = Join-Path $gitCommonDir 'hooks'
New-Item -ItemType Directory -Force -Path $hooksDir | Out-Null

$src = Join-Path $PSScriptRoot 'pre-push-hook.sh'
$dst = Join-Path $hooksDir 'pre-push'
Copy-Item -Path $src -Destination $dst -Force
# Git for Windows' bash honors the executable bit via its own fstab mapping;
# harmless no-op on filesystems that don't track it.
& git -C $repoRoot update-index --chmod=+x -- .claude/scripts/pre-push-hook.sh 2>$null

Write-Output "Installed pre-push hook -> $dst"
Write-Output "(shared by every worktree of this repo; bypass a single push with 'git push --no-verify')"
