# Boardurance (Racing Tycoon)

A turn-based racing tycoon game. Two parts:

- **`rust-backend/`** — Rust + Axum API, MongoDB, JWT auth. See `rust-backend/CLAUDE.md`.
- **`empty-project/`** — the **frontend** (Vite + React 19 + TypeScript + Tailwind). Despite the name, this is the web client. See `empty-project/CLAUDE.md`.

## Environment

- OS: Windows 11. Shell is **PowerShell** — use `$env:VAR` (not `$VAR`), `$null` (not `/dev/null`), backtick for line continuation.
- Process: feature work runs on **BMAD-METHOD v6** — see "Method: BMAD" below. Planning artifacts, epics/stories, and review/test artifacts live in `_bmad-output/`; **that is the spec source of truth**. The pre-BMAD spec systems are **frozen, read-only history** (still accurate as descriptions of existing behavior, never edited, never treated as current truth): `openspec/` capability specs and legacy `.kiro/specs/` (mapping: `docs/migration/kiro-to-openspec.md`). Bugfix/feature writeups land in `docs/bugfixes/` and `docs/features/`.
- **Command wrappers** (set cwd + test env for you, instead of a fragile `Set-Location …; $env:…; cargo …` prefix): `.claude/scripts/be.ps1 <cargo args>` (e.g. `be.ps1 test-fast`, `be.ps1 check --all-targets --all-features`) and `.claude/scripts/fe.ps1 <npm/npx args>` (e.g. `fe.ps1 npx tsc --noEmit`). The PowerShell cwd persists between calls — cd into an area once rather than re-prefixing every command.
- **Concurrent sessions — never `git checkout` in the bare `Boardurance/` repo.** Multiple Claude Code sessions/agents can be active on this repo at once, and the bare checkout is a shared resource: one session switching its branch silently changes what every other session reads. This has caused real incidents (a session mid-review-gate read stale pre-migration code because another session checked out an unrelated branch underneath it; this very file has been edited out from under a concurrent session too). Keep the bare `Boardurance/` checkout pinned to `dev` and do **all** actual work — human-directed feature branches included, not just automated tasks — in an isolated `git worktree`:
  - Branch-per-feature convention (all work — interactive or agent-driven): `git worktree add ../Boardurance-worktrees/<slug> -b feat/<slug> dev` (or `fix/<slug>`) instead of `git checkout -b` in the bare repo; remove with `git worktree remove ../Boardurance-worktrees/<slug>` after the PR merges.
- **Pre-push hook enforces the full backend/frontend verify loop** (including `clippy`, which the Stop hook skips for speed — see below) before code leaves the machine, so CI-only failures get caught locally instead. Run `.claude/scripts/install-git-hooks.ps1` once per clone/machine (worktrees share `.git/hooks`, so once per main checkout covers all of them). Bypass a single push with `git push --no-verify` when you genuinely need to.

## Method: BMAD v6 (the workflow)

Installed: BMad Core + BMM v6.10.0 + Test Architect (TEA) v1.19.1 — skills in `.claude/skills/bmad-*`, config in `_bmad/` (durable overrides go in `_bmad/custom/config.toml`, never in the generated files), artifacts in `_bmad-output/`. Lost? **`bmad-help`** inspects the project and recommends the next step. **Step-by-step how-to (personas, per-phase commands, cheat sheet): [`docs/BMAD_WORKFLOW_GUIDE.md`](docs/BMAD_WORKFLOW_GUIDE.md).**

Feature work flows through the four BMAD phases:

1. **Analysis** (optional, for fuzzy ideas): `bmad-brainstorming`, `bmad-product-brief`, `bmad-domain-research` / `bmad-technical-research`.
2. **Planning**: `bmad-prd` → PRD in `_bmad-output/planning-artifacts/`; `bmad-ux` when UI is involved; `bmad-validate-prd` to check completeness.
3. **Solutioning**: `bmad-architecture` → architecture doc (decisions, trade-offs, accepted debt are recorded HERE); `bmad-create-epics-and-stories` → epics/stories; `bmad-check-implementation-readiness` before any code.
4. **Implementation**: `bmad-sprint-planning` to sequence stories, then per story: `bmad-create-story` → `bmad-dev-story` (implements; must end with the verify loop below green) → `bmad-code-review` → merge. `bmad-retrospective` closes the feature. `bmad-dev-auto` may run the story loop unattended — same rules apply.

**Scale to the work.** Small, well-understood change → **Quick Flow**: `bmad-quick-dev` (tech-spec instead of PRD, straight to implementation). Substantial feature → full method. No PRD for a two-line fix.

**Testing** is owned by TEA: `bmad-testarch-*` skills (test design, automation, CI, NFR, traceability) and `bmad-qa-generate-e2e-tests`; test artifacts land in `_bmad-output/test-artifacts/`.

**Tracking**: every epic/story that enters implementation gets a YouTrack **RACE** issue (To do → In Progress → In Review → Done). A story is only done with green CI on its PR.

**Project context**: `_bmad-output/project-context.md` carries the critical implementation rules BMAD agents load — update it when invariants change.

## Definition of "done" (local CI parity)

A change is **not done** until the relevant verify loop passes. Run the gate that matches what you touched; run both if you touched both. These mirror `.github/workflows/`.

**Backend** (run from `rust-backend/`):
```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings -A clippy::too_many_lines -A clippy::cast_possible_truncation -A clippy::cast_precision_loss -A clippy::cast_sign_loss -A clippy::cast_possible_wrap -A clippy::match_wildcard_for_single_variants -A clippy::manual_let_else -A clippy::needless_pass_by_value -A clippy::needless_range_loop -A dead_code
cargo check --all-targets --all-features
cargo test-fast
```

**Frontend** (run from `empty-project/`):
```
npx tsc --noEmit
npm run test -- --run
```

## Working efficiently (token & friction)

Derived from self-analysis of past sessions (`docs/self-improvement/`):

- Backend source files are large (`races.rs` ~4k lines, `race.rs` ~1.7k). **Locate with Grep/Glob, then Read specific ranges** (`offset`/`limit`) — don't Read whole large files, and don't re-Read a file you just edited (Edit/Write already confirm success). `Read` was the largest token sink *and* the top error source.
- Default `Grep` to `files_with_matches` + `head_limit`; switch to `content` mode only when you actually need the matching lines.

## Always / Never (non-negotiable guardrails)

These are hard rules, born from real incidents. They override convenience.

- **Never** skip, disable (`.skip`/`.only`/`#[ignore]`), delete, or weaken a test — and **never** reduce coverage — to make a check pass. A green suite that doesn't exercise the behavior is worse than a red one. If a test is genuinely wrong, fix it and say why.
- **Never** write secrets, tokens, passwords, or personal data (emails, user identifiers) into logs, error messages, commit messages, or fixtures. Redact before logging.
- **Always** enforce tenant isolation: every query that reads or writes user/org-scoped data must filter by the authenticated tenant. A passing test on a single tenant does **not** prove isolation — add a cross-tenant negative test for any new data-access path.
- **Always** gate user-facing strings through i18n — no hardcoded display text. A literal in a component is a bug.
- **Never** touch production data or run migrations against prod. Local/dev is freestyle; anything beyond that waits for explicit human approval.
- **Always** funnel variants of one concept through a single shared path — one turn-resolution helper, one player store. Re-implementing per call-site is how the solo turn paths and the player stores drifted (see `docs/reviews/`).

## Loop discipline

- **Plan first** for non-trivial work: run the BMAD planning phases (PRD → architecture → stories) before touching code; `bmad-quick-dev` is the sanctioned shortcut for small changes. The active story's acceptance criteria in `_bmad-output/` define what "done" means.
- **Termination rule:** after **3 failed attempts** at the same error with no new information, stop and summarize the blocker rather than retrying the same strategy.
- **Read the full error** (stack trace / clippy span / failing assertion) before revising — distinguish a recoverable error from a hard blocker.
- **Review gate before shipping:** before opening a PR into `dev` or `main`, run `bmad-code-review` on the change and resolve every blocking finding. Review artifacts live under `_bmad-output/implementation-artifacts/`.
- **Record decisions:** non-trivial architecture/design tradeoffs are captured in the feature's BMAD architecture document (decision, why, alternatives, accepted debt). Pre-BMAD ADRs remain in `docs/adr/` as history.
- **Retrospective on repeated failure:** when the 3-attempt rule trips, after a prod incident, or when a defect escapes to PR/prod, run `bmad-retrospective` (root cause, detection gap, follow-ups). Feed follow-ups back into `_bmad-output/project-context.md` and `_bmad/custom/` rules — every problem improves the method. Pre-BMAD postmortems remain in `docs/postmortems/`.
