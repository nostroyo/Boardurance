# Boardurance (Racing Tycoon)

A turn-based racing tycoon game. Two parts:

- **`rust-backend/`** — Rust + Axum API, MongoDB, JWT auth. See `rust-backend/CLAUDE.md`.
- **`empty-project/`** — the **frontend** (Vite + React 19 + TypeScript + Tailwind). Despite the name, this is the web client. See `empty-project/CLAUDE.md`.

## Environment

- OS: Windows 11. Shell is **PowerShell** — use `$env:VAR` (not `$VAR`), `$null` (not `/dev/null`), backtick for line continuation.
- Process: feature work is specced under `.kiro/specs/<feature>/` (requirements → design → tasks). Bugfix/feature writeups land in `docs/bugfixes/` and `docs/features/`.

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

## Always / Never (non-negotiable guardrails)

These are hard rules, born from real incidents. They override convenience.

- **Never** skip, disable (`.skip`/`.only`/`#[ignore]`), delete, or weaken a test — and **never** reduce coverage — to make a check pass. A green suite that doesn't exercise the behavior is worse than a red one. If a test is genuinely wrong, fix it and say why.
- **Never** write secrets, tokens, passwords, or personal data (emails, user identifiers) into logs, error messages, commit messages, or fixtures. Redact before logging.
- **Always** enforce tenant isolation: every query that reads or writes user/org-scoped data must filter by the authenticated tenant. A passing test on a single tenant does **not** prove isolation — add a cross-tenant negative test for any new data-access path.
- **Always** gate user-facing strings through i18n — no hardcoded display text. A literal in a component is a bug.
- **Never** touch production data or run migrations against prod. Local/dev is freestyle; anything beyond that waits for explicit human approval.

## Loop discipline

- **Plan first** for non-trivial work: design the approach before editing, then implement, then run the verify loop. Use existing `.kiro/specs/` as the source of truth for what "done" means per feature.
- **Termination rule:** after **3 failed attempts** at the same error with no new information, stop and summarize the blocker rather than retrying the same strategy.
- **Read the full error** (stack trace / clippy span / failing assertion) before revising — distinguish a recoverable error from a hard blocker.
- **Review gate before shipping:** before opening a PR or merging to `main`, run `/review-gate` (spec-conformance + correctness + security judges) and resolve any **BLOCK**. The verdict is recorded under `docs/reviews/`.
