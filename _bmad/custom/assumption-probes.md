# Assumption Probes — verify before you build

Team protocol, loaded as a persistent fact by `bmad-create-story`, `bmad-dev-story`,
`bmad-quick-dev`, and `bmad-create-epics-and-stories` (via `_bmad/custom/<skill>.toml`).
Inspired by spike/probe discipline: **an assumption the plan depends on is verified
empirically BEFORE code is built on top of it** — a probe failing after 10 minutes is
cheap; the same discovery after 2 days of implementation is not.

## Definitions

- **Assumption** — a statement the plan depends on that nobody has verified against
  reality. Typical classes: library/framework behavior ("axum extractor X yields Y"),
  environment capability ("works without Mongo"), external contract (API shape, wire
  format), performance ("poll every 2s is fine"), data shape ("openapi-typescript emits
  a union for this enum"), tooling ("this codegen preserves field order").
- **Load-bearing** — if the assumption is false, the story's approach (not just a
  detail) is wrong: tasks would be re-planned, an AC becomes unreachable, or a contract
  breaks.
- **Probe** — the smallest *executable* check that settles the assumption: a one-liner,
  a scratch `#[test]`, a `curl`, a tiny script run against the real dependency.
  Throwaway by default (scratchpad, not committed) — only its **result** is recorded.
  A probe that deserves to live on becomes a regression test, explicitly.
- **Spike story** — a probe too big for a story preamble (new tech, external system,
  perf target). A dedicated leading story whose deliverable is a **decision note**, not
  production code.

## Status taxonomy

| Status | Meaning | Evidence required |
|---|---|---|
| `VERIFIED-BY-READING` | Settled by reading real code/docs | `file:line` or doc link, one-line quote |
| `VERIFIED-BY-PROBE` | Settled empirically | probe command + observed output (verbatim, trimmed) |
| `UNVERIFIED` | Not yet settled | — blocks `ready-for-dev` if load-bearing |
| `ACCEPTED-RISK` | Human explicitly accepts building without verifying | who accepted + fallback plan if false |

## The rules

1. **Every story file carries an `## Assumption Probes` table** (columns: Assumption /
   Load-bearing? / Status / Evidence). "No load-bearing assumptions" is a legitimate
   entry — state it explicitly rather than omitting the section.
2. **A story with a load-bearing `UNVERIFIED` assumption is not `ready-for-dev`.**
   Probe it during story creation, or mark `ACCEPTED-RISK` (human call only).
3. **dev-story gate**: before starting Task 1, execute any probe still `UNVERIFIED`,
   record command + output in the table, and update the status. On probe **FAIL**:
   HALT — do not implement on a falsified assumption. Small miss → amend the story
   tasks (human approves); approach-level miss → `bmad-correct-course`.
4. **Timebox**: a probe is ≤ 15 minutes; if it needs more, it's a spike story —
   escalate to the human (choices: spike story / accepted-risk / redesign).
5. **Epic level**: when an epic rests on a major unverified assumption, create a
   leading **spike story** (e.g. story N.0) with the probe question as its AC and a
   decision note in `_bmad-output/planning-artifacts/` as its deliverable.
6. **Honesty rule**: a probe must be able to fail. A "probe" that asserts what the test
   itself constructs proves nothing (same trap as a tautological unit test).

## Worked example (this repo)

> Story: "expose crate version on /health_check".
> Assumption: *the Render deploy poll greps the compact JSON for `"status":"ok"`, and
> serde keeps `status` first* — load-bearing (deploy breaks if false).
> Probe: `serde_json::to_string(&HealthResponse::healthy())` in a scratch test →
> observed `{"status":"ok",...` → `VERIFIED-BY-PROBE`.
> Counter-example: the deploy workflow's grep pattern itself was `VERIFIED-BY-READING`
> at `.github/workflows/deploy.yml:57`.
