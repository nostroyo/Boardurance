---
description: Autopilot factory — dequeue N tasks from .kiro/specs, run each through the chain (implement → verify) in its own git worktree, and leave a reviewable branch per task. Bounded; never pushes or merges.
argument-hint: "[N] [spec-name]  (N defaults to 2; optional spec to scope the queue)"
allowed-tools: Bash, PowerShell, Read, Grep, Glob, Edit, Write, Agent
---

You are the **autopilot** for Boardurance — the talk's factory: run several tasks
through the full chain in parallel, each isolated in its own git worktree, and
leave a reviewable branch per task. **You never push or merge** — the human
reviews (via `/review-gate`) and merges. "Le merge reste mon outil."

Args `$ARGUMENTS`: first token = N (max concurrent tasks, default **2**, hard cap
**2** on this 4-core machine); optional second token = a spec name to scope the queue.

## 1. Build the queue
Collect unchecked tasks (`- [ ]`) from `.kiro/specs/<spec>/tasks.md` (all specs if
none given). Take the first **N** (default 2, never more than 2). Derive a kebab
`task-id` per task (e.g. `ai-solo-mode-3`). Show the chosen tasks and **STOP for
confirmation** if N > 2, or if a task touches schema / migrations / auth.

## 2. Per task — isolate, implement, verify (run the ≤N concurrently)
Spawn one Agent per task (so they run in parallel). Each agent:
1. Creates its worktree: `.claude/scripts/new-worktree.ps1 <task-id>` → note the
   `WORKTREE` path and `BRANCH`.
2. Implements **only** that task, editing files **under the worktree path** (never
   the main checkout). Plan first, then implement.
3. Runs the verify loop in the worktree — backend `be.ps1 check` + `be.ps1 test-fast`,
   and/or frontend `fe.ps1 npx tsc --noEmit` (the wrappers resolve to the
   worktree's own `rust-backend`/`empty-project` via `$PSScriptRoot`).
4. On green: commit in the worktree
   (`git -C <worktree> add -A; git -C <worktree> commit -m "auto(<task-id>): <summary>"`),
   tick the task `- [ ]` → `- [x]` in that worktree's `tasks.md`, commit that too.
5. On the **3rd failed attempt** at the same error: STOP, leave the worktree for
   inspection, return the blocker (the termination rule).
   Return `{ task, branch, status: committed|blocked, summary, verify }`.

## 3. Report
Print a table: task → branch → committed/blocked → one-line summary. For each
committed branch, recommend `/review-gate` then a human merge. **Do not push or
merge.** List leftover worktrees and the cleanup command per task:
`.claude/scripts/rm-worktree.ps1 <task-id> -DeleteBranch` (run once a branch is
merged or abandoned).

## Guardrails
- **Never more than 2 concurrent agents** (4 cores — parallel cargo builds thrash beyond that).
- Each agent edits **only** inside its own worktree; no cross-task collisions.
- No push, no merge, no touching `main`. Branches are `auto/<task-id>`.
- Honour the repo's Always/Never rules and Definition of done inside each worktree.
