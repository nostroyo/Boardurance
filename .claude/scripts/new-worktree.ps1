# new-worktree.ps1 — create an isolated git worktree + branch for one autopilot
# task, so parallel agents never touch the same files (the talk's per-agent
# isolation). The worktree lives OUTSIDE the repo (sibling dir) so it isn't
# tracked by the repo itself.
#
# Usage:  .claude/scripts/new-worktree.ps1 <task-id> [-BaseRef main]
# Creates  <parent>/Boardurance-worktrees/<task-id>  on branch  auto/<task-id>.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$TaskId,
    [string]$BaseRef = 'main'
)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)   # .claude/scripts -> repo root
$wtRoot = Join-Path (Split-Path -Parent $repoRoot) 'Boardurance-worktrees'
$path = Join-Path $wtRoot $TaskId
$branch = "auto/$TaskId"

if (Test-Path $path) { Write-Output "ERROR: worktree path already exists: $path"; exit 1 }
New-Item -ItemType Directory -Force -Path $wtRoot | Out-Null

git -C $repoRoot worktree add -b $branch $path $BaseRef
if ($LASTEXITCODE -ne 0) { Write-Output "ERROR: 'git worktree add' failed (branch '$branch' may already exist)"; exit 1 }

Write-Output "WORKTREE: $path"
Write-Output "BRANCH:   $branch"
