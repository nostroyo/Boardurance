# pre-push-verify.ps1 — full Definition-of-done verify loop, run at `git push`
# time so CI-only failures (clippy in particular) are caught before they ever
# reach GitHub Actions, not just fmt/check as the Stop hook does per-turn.
#
# Installed via `.git/hooks/pre-push` (see install-git-hooks.ps1) so it fires
# for `git push` from ANY worktree of this repo (worktrees share .git/hooks).
#
# Bypass with `git push --no-verify` when you genuinely need to (e.g. pushing
# a WIP branch nobody's building CI for yet).

# NOT 'Stop': every Invoke-Step call below redirects a native command's
# stderr (2>&1) to capture failure output, but PowerShell wraps EACH stderr
# line from a native exe as a NativeCommandError -- with ErrorActionPreference
# 'Stop' that throws and aborts the whole script on cargo's ordinary build
# progress chatter (e.g. "Compiling proc-macro2"), long before the real
# exit code is even known. Control flow here is exit-code-driven throughout,
# not exception-driven, so 'Continue' is correct, not just tolerated.
$ErrorActionPreference = 'Continue'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)   # .claude/scripts -> repo root
$failures = @()

# Invoked via `powershell.exe -NoProfile` from the git hook shim (deliberately,
# for determinism/speed), which means PATH additions that only live in the
# user's $PROFILE — cargo/rustup and Node.js both turned out to be PATH-via-
# profile-only on the machine this was built on, not the persistent
# user/system PATH — are NOT present. Add the standard install locations
# explicitly rather than depending on any one profile's setup; harmless if
# already on PATH.
foreach ($p in @("$env:USERPROFILE\.cargo\bin", 'C:\Program Files\nodejs')) {
    if ((Test-Path $p) -and ($env:PATH -notlike "*$p*")) {
        $env:PATH = "$p;$env:PATH"
    }
}

# Invoke the executable directly (no `cmd /c "...string..."` layer) -- that
# extra hop proved unreliable in the exact process-spawn chain a git hook
# uses (raw `powershell.exe` subprocess, no console/profile), silently
# producing an empty exit code instead of actually running anything.
function Invoke-Step([string]$label, [string]$dir, [string]$exe, [string[]]$argList) {
    Write-Output "  -> $label"
    Push-Location $dir
    try {
        $out = & $exe @argList 2>&1
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
    Invoke-Step 'backend: cargo fmt --check' $rb 'cargo' @('fmt', '--check')
    Invoke-Step 'backend: cargo clippy' $rb 'cargo' @(
        'clippy', '--all-targets', '--all-features', '--',
        '-D', 'warnings',
        '-A', 'clippy::too_many_lines', '-A', 'clippy::cast_possible_truncation',
        '-A', 'clippy::cast_precision_loss', '-A', 'clippy::cast_sign_loss',
        '-A', 'clippy::cast_possible_wrap', '-A', 'clippy::match_wildcard_for_single_variants',
        '-A', 'clippy::manual_let_else', '-A', 'clippy::needless_pass_by_value',
        '-A', 'clippy::needless_range_loop', '-A', 'dead_code'
    )
    Invoke-Step 'backend: cargo check' $rb 'cargo' @('check', '--all-targets', '--all-features')
    Invoke-Step 'backend: cargo test-fast' $rb 'cargo' @('test-fast')
}

if ($frontend) {
    $fe = Join-Path $repo 'empty-project'
    Invoke-Step 'frontend: tsc --noEmit' $fe 'npx' @('tsc', '--noEmit')
    Invoke-Step 'frontend: npm test' $fe 'npm' @('run', 'test', '--', '--run')
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
