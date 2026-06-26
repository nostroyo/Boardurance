---
description: Self-analysis flywheel — mine Claude Code transcripts for toil, token waste, and friction, then propose scripts / rules / hooks to improve the workflow.
argument-hint: "[days]  (optional lookback window, default 30)"
allowed-tools: Bash, PowerShell, Read, Grep, Glob, Write
---

You are running the **self-improvement** pass for Boardurance — the talk's
"l'usine entretient l'usine": turn the agent's own activity into a prioritized
list of concrete improvements. Each repeated action becomes a candidate script,
each token-heavy command a trim, each recurring incident a rule. **Propose only —
do NOT apply changes.** The human reviews and decides.

Lookback window (days): `$ARGUMENTS` (default 30).

## 1. Gather signals (no external tools — mine the transcripts directly)

Run the analysis script, which reads `~/.claude/projects/*RAcingtycoon*/*.jsonl`
and aggregates the agent's tool activity:

```
pwsh -File .claude/scripts/analyze-activity.ps1 -Days <window> -TopN 15
```

(Use `powershell -File` if `pwsh` is unavailable.) It prints three sections:
**Toil** (most-repeated actions), **Token hogs** (actions by total result size),
and **Friction** (actions whose results contained errors).

Then add repo-side signals:
- `docs/reviews/` — recent review-gate verdicts: what BLOCKED, what recurs?
- `docs/bugfixes/` + any `postmortem` docs — repeated root causes.
- `Grep` for clustered `#[allow(...)]` / `eslint-disable` / `TODO|FIXME` (places
  the codebase fights its own lints — candidate rules).

## 2. Analyze — three lenses (the talk's framing)

1. **Toil → script.** High-frequency actions. For shell signatures repeated many
   times (e.g. a `cd … ; cargo …` pattern), propose a deterministic script or
   convention. For high-frequency `Read`/`Grep`, propose narrowing (locate with
   Glob/Grep before Read; read ranges, not whole files; don't re-Read edited files).
2. **Token hogs → trim.** Largest total-result actions. Propose narrowing flags,
   `Select-Object`/range reads, or a wrapper/hook that returns only what's needed.
   Quantify the reclaimable tokens (chars/4).
3. **Friction & incidents → rule.** Actions with many errors, repeated review-gate
   BLOCKs, recurring bug root causes. Propose a `CLAUDE.md` always/never rule, a
   hook, or an LSP/AST guard so the same mistake is caught automatically.

## 3. Emit the report

Write `docs/self-improvement/<YYYY-MM-DD>-self-improve.md` (today's date):

```
# Self-improvement report — <date>

- Window: <N> days | Sessions: <n> | Tool calls: <n>

## Toil → script (ranked)
- [Nx] <signature> → proposed: <script / convention>

## Token hogs → trim
- [~T tok] <signature> → proposed: <narrowing / filter>

## Friction & incidents → rule
- [N errs] <signature / pattern> → proposed: <CLAUDE.md rule | hook | guard>

## Top 3 recommended actions (highest leverage first)
1. ...
2. ...
3. ...
```

## 4. Report

Print the **Top 3 recommended actions** and the report path. Apply nothing.
