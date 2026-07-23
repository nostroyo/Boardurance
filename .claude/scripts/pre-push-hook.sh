#!/bin/sh
# Tracked source for .git/hooks/pre-push (git doesn't version .git/hooks/, so
# this copy lives here for review/history; install-git-hooks.ps1 copies it
# into place). Delegates to pre-push-verify.ps1 for the actual checks.
#
# Deliberately does NOT read stdin (git feeds ref-update lines there): in at
# least one harness/environment this hung indefinitely waiting on EOF that
# never arrived, blocking a real `git push` for 5+ minutes. The hook doesn't
# need that data, so just don't touch stdin at all -- redirect it from
# /dev/null instead of reading it, which can't block.
set -e
repo_root="$(git rev-parse --show-toplevel)"
verify_script="$repo_root/.claude/scripts/pre-push-verify.ps1"

# This hook is installed repo-wide (shared .git/hooks across worktrees), but
# a branch checked out from BEFORE this tooling was merged into dev won't
# have pre-push-verify.ps1 in its own working tree yet. Don't hard-block
# those pushes -- warn and let them through rather than punishing branches
# for predating the rollout.
if [ ! -f "$verify_script" ]; then
    echo "pre-push: $verify_script not found on this branch (predates the" >&2
    echo "pre-push hook tooling) -- skipping the full verify loop for this push." >&2
    exit 0
fi

# Hard ceiling so a future hang (a different one than the stdin issue above)
# fails the push loudly after 10 minutes instead of blocking it forever.
timeout 600 powershell.exe -NoProfile -ExecutionPolicy Bypass \
    -File "$verify_script" < /dev/null
