# rm-worktree.ps1 — remove an autopilot worktree (and optionally its branch).
# Run after the task's branch has been reviewed/merged (or abandoned).
#
# Usage:  .claude/scripts/rm-worktree.ps1 <task-id> [-DeleteBranch]
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$TaskId,
    [switch]$DeleteBranch
)
$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$path = Join-Path (Join-Path (Split-Path -Parent $repoRoot) 'Boardurance-worktrees') $TaskId
$branch = "auto/$TaskId"

git -C $repoRoot worktree remove $path --force 2>&1 | Out-Null
git -C $repoRoot worktree prune
if ($DeleteBranch) { git -C $repoRoot branch -D $branch 2>&1 | Out-Null }
Write-Output ("Removed worktree {0}{1}" -f $path, $(if ($DeleteBranch) { " and branch $branch" } else { "" }))
