# Self-improvement report — 2026-06-26

- Window: 30 days | Sessions: 5 | Tool calls: 1412
- Source: `.claude/scripts/analyze-activity.ps1` (mines `~/.claude/projects/*RAcingtycoon*/*.jsonl`)

## Toil → script (ranked)
- [359x] `Edit` / [335x] `Read` / [137x] `Grep` — core editing loop; high by nature, but see token/friction lenses below for narrowing.
- [~75x combined] `cd …\rust-backend ; $env:APP_ENVIRONMENT=… ; cargo …` (signatures `sh: Set-Location cargo` ×33, `sh: cd cargo` ×15, `sh: Set-Location =` ×27) → **proposed:** a backend/frontend wrapper script (e.g. `.claude/scripts/be.ps1`, `fe.ps1`) that cd's into the right dir, sets `APP_ENVIRONMENT`, and runs the command. Call `be test-fast` instead of the long fragile prefix.
- [91x] `preview_eval` (MCP) → expected for the in-browser UI work; no action.

## Token hogs → trim
- [~283,000 tok] `Read` (1.13M chars) — **by far the #1 cost.** **proposed:** locate with Glob/Grep first and read *ranges* (`offset`/`limit`), not whole files; never re-Read a file just edited (the harness confirms Edit/Write success). A ~30% cut ≈ **~85k tokens** reclaimed.
- [~60,000 tok] `Grep` (241K chars) → **proposed:** default to `files_with_matches` + `head_limit`; use `content` mode only when the lines are actually needed.
- [~23,000 tok] `Agent` (subagent reports) → expected cost of delegation; keep.

## Friction & incidents → rule
- [106 errs] `Read` → reading missing/wrong paths or re-reading. **proposed CLAUDE.md rule:** "Locate files with Glob/Grep before Read; do not Read a file you just edited." Directly removes the top friction source *and* a chunk of the Read token waste.
- [~25 errs] `cd`/`Set-Location` into `rust-backend` → the multi-statement shell prefix is fragile. **proposed:** the `be`/`fe` wrapper above removes the prefix → fewer cd failures.
- [4 errs] `cargo test-fast` → `APP_ENVIRONMENT` not set / DB. **proposed:** bake `APP_ENVIRONMENT=test` into the wrapper (or `.cargo/config.toml [env]`).
- Recurring review-gate theme (from `docs/reviews/`): "same concept implemented twice then drifted" (two turn paths, two player stores). **proposed CLAUDE.md rule:** "one shared path per concept — funnel variants through a single function/store."

## Top 3 recommended actions (highest leverage first)
1. **Cut `Read` cost + the 106 Read errors** — add a CLAUDE.md rule to locate-before-read, read ranges, and never re-Read edited files. Biggest single lever (~85k tokens + top friction).
2. **Add `be`/`fe` wrapper scripts** — eliminate the ~75 `cd …; $env…; cargo/npm …` repetitions and ~25 cd errors; bake in `APP_ENVIRONMENT`.
3. **Trim `Grep` output** — default to `files_with_matches` + `head_limit` (~reclaims a chunk of the ~60k Grep tokens).

## Notes
- 5 sessions is a small window; signal sharpens as more sessions accrue. Re-run `/self-improve` periodically (or schedule it) to track whether applied changes move the numbers.
