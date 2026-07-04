# Boardurance (Racing Tycoon)

A turn-based racing tycoon game. Two parts:

- **`rust-backend/`** — Rust + Axum API, MongoDB, JWT auth. See `rust-backend/CLAUDE.md`.
- **`empty-project/`** — the **frontend** (Vite + React 19 + TypeScript + Tailwind). Despite the name, this is the web client. See `empty-project/CLAUDE.md`.

## Environment

- OS: Windows 11. Shell is **PowerShell** — use `$env:VAR` (not `$VAR`), `$null` (not `/dev/null`), backtick for line continuation.
- Process: feature work is specced with **OpenSpec**. Current truth lives in `openspec/specs/<capability>/spec.md`; every change goes through `openspec/changes/<change>/` (proposal → approve → implement `tasks.md` → `openspec archive`, which merges the deltas into `specs/` and moves the change to `changes/archive/`). Start a change with `/opsx:propose`. Legacy `.kiro/specs/` is **deprecated, frozen, read-only history** — never edit it, never treat it as current truth (mapping: `docs/migration/kiro-to-openspec.md`). Bugfix/feature writeups land in `docs/bugfixes/` and `docs/features/`.
- **Command wrappers** (set cwd + test env for you, instead of a fragile `Set-Location …; $env:…; cargo …` prefix): `.claude/scripts/be.ps1 <cargo args>` (e.g. `be.ps1 test-fast`, `be.ps1 check --all-targets --all-features`) and `.claude/scripts/fe.ps1 <npm/npx args>` (e.g. `fe.ps1 npx tsc --noEmit`). The PowerShell cwd persists between calls — cd into an area once rather than re-prefixing every command.

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

**Specs** (whenever anything under `openspec/` changed):
```
openspec validate --all --strict
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

- **Plan first** for non-trivial work: design the approach before editing, then implement, then run the verify loop. Use `openspec/specs/` as the source of truth for current behavior and the active change's `openspec/changes/<change>/` (proposal + delta specs + tasks) for what "done" means; archive the change (`openspec archive`) once merged.
- **Termination rule:** after **3 failed attempts** at the same error with no new information, stop and summarize the blocker rather than retrying the same strategy.
- **Read the full error** (stack trace / clippy span / failing assertion) before revising — distinguish a recoverable error from a hard blocker.
- **Review gate before shipping:** before opening a PR or merging to `main`, run `/review-gate` (spec-conformance + correctness + security judges) and resolve any **BLOCK**. The verdict is recorded under `docs/reviews/`.
- **Record decisions (`/adr`):** for any non-trivial architecture/design tradeoff, write an Architecture Decision Record under `docs/adr/` — the decision, why, alternatives, and accepted debt.
- **Postmortem on repeated failure (`/postmortem`):** when the 3-attempt termination rule trips, after a prod incident, or when a defect escapes to PR/prod, write a blameless postmortem under `docs/postmortems/` (root cause, detection gap, follow-ups). Feed the follow-ups back as rules/hooks — every problem improves the method.
