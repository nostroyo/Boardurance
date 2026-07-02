#!/bin/sh
# Tracked source for .git/hooks/pre-push (git doesn't version .git/hooks/, so
# this copy lives here for review/history; install-git-hooks.ps1 copies it
# into place). Delegates to pre-push-verify.ps1 for the actual checks.
set -e
repo_root="$(git rev-parse --show-toplevel)"
# Drain stdin (git pre-push feeds ref update lines on stdin); unused here.
cat > /dev/null
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$repo_root/.claude/scripts/pre-push-verify.ps1"
