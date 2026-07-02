# pre-push-verify.ps1 — full Definition-of-done verify loop, run at `git push`
# time so CI-only failures (clippy in particular) are caught before they ever
# reach GitHub Actions, not just fmt/check as the Stop hook does per-turn.
#
# Installed via `.git/hooks/pre-push` (see install-git-hooks.ps1) so it fires
# for `git push` from ANY worktree of this repo (worktrees share .git/hooks).
#
# Bypass with `git push --no-verify` when you genuinely need to (e.g. pushing
# a WIP branch nobody's building CI for yet).

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)   # .claude/scripts -> repo root
$failures = @()

function Invoke-Step([string]$label, [string]$dir, [string]$cmd) {
    Write-Output "  -> $label"
    Push-Location $dir
    try {
        $out = & cmd /c "$cmd 2>&1"
        $code = $LASTEXITCODE
    } finally { Pop-Location }
    if ($code -ne 0) {
        $tail = ($out | Select-Object -Last 60) -join "`n"
        $script:failures += "### $label  (exit $code)`n$tail"
    }
}

# Which areas changed relative to what's already on the remote? (Falls back to
# "check both" if there's no upstream yet, e.g. a brand-new branch's first push.)
# Building the ref into a plain variable first avoids PowerShell mangling the
# literal `@{u}` braces when passing it straight to a native command.
$branch = git -C $repo rev-parse --abbrev-ref HEAD
$upstreamRef = "$branch@{upstream}"
$paths = $null
try {
    $upstream = git -C $repo rev-parse --abbrev-ref --symbolic-full-name $upstreamRef 2>$null
    if ($LASTEXITCODE -eq 0 -and $upstream) {
        $paths = git -C $repo diff --name-only "$upstream..HEAD"
    }
} catch {
    $paths = $null   # no upstream (first push of a new branch) -> verify everything
}

if ($null -eq $paths) {
    $backend = $true; $frontend = $true
} else {
    $backend  = [bool]($paths | Where-Object { $_ -like 'rust-backend/*' })
    $frontend = [bool]($paths | Where-Object { $_ -like 'empty-project/*' })
}

if (-not $backend -and -not $frontend) {
    Write-Output "pre-push: no backend/frontend changes relative to upstream, skipping verify."
    exit 0
}

Write-Output "pre-push: running full verify loop for '$branch' (backend=$backend frontend=$frontend)..."

if ($backend) {
    $rb = Join-Path $repo 'rust-backend'
    Invoke-Step 'backend: cargo fmt --check' $rb 'cargo fmt --check'
    Invoke-Step 'backend: cargo clippy' $rb 'cargo clippy --all-targets --all-features -- -D warnings -A clippy::too_many_lines -A clippy::cast_possible_truncation -A clippy::cast_precision_loss -A clippy::cast_sign_loss -A clippy::cast_possible_wrap -A clippy::match_wildcard_for_single_variants -A clippy::manual_let_else -A clippy::needless_pass_by_value -A clippy::needless_range_loop -A dead_code'
    Invoke-Step 'backend: cargo check' $rb 'cargo check --all-targets --all-features'
    Invoke-Step 'backend: cargo test-fast' $rb 'cargo test-fast'
}

if ($frontend) {
    $fe = Join-Path $repo 'empty-project'
    Invoke-Step 'frontend: tsc --noEmit' $fe 'npx tsc --noEmit'
    Invoke-Step 'frontend: npm test' $fe 'npm run test -- --run'
}

if ($failures.Count -eq 0) {
    Write-Output "pre-push: all checks passed."
    exit 0
}

[Console]::Error.WriteLine(
    "pre-push: $($failures.Count) check(s) failed -- push aborted. Fix these, or " +
    "`git push --no-verify` if you really mean to push anyway:`n`n" +
    ($failures -join "`n`n"))
exit 1
