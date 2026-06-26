---
description: Record an Architecture Decision Record — capture a design decision, its context, alternatives, and accepted debt as a numbered, replayable artifact under docs/adr/.
argument-hint: "<short decision title>"
allowed-tools: Bash, PowerShell, Read, Grep, Glob, Write
---

You are recording an **Architecture Decision Record (ADR)** for Boardurance —
the talk's "trace": make the decision, the reasoning, and the debt it accepts
replayable later (and re-challengeable when the context changes). ADRs live in
`docs/adr/` as `NNNN-kebab-title.md`.

Decision: `$ARGUMENTS` (if empty, ask what decision to record).

## 1. Resolve the number
Glob `docs/adr/*.md`. The next number = highest existing `NNNN` + 1, zero-padded
to 4 digits (start at `0001`).

## 2. Gather content (infer from the code/diff/conversation; ask only for gaps)
- **Context** — the forces / problem that make this decision necessary.
- **Decision** — what we will do, in active voice ("We will …").
- **Alternatives considered** — each with a one-line why-not.
- **Consequences** — positive, negative, and **accepted debt / follow-ups**.
- **Status** — `Proposed` | `Accepted` (default `Accepted` if already decided).

## 3. Write `docs/adr/<NNNN>-<kebab-title>.md`

```
# <NNNN>. <Title>

- Status: <Accepted|Proposed>
- Date: <YYYY-MM-DD>
- Deciders: <who / "team">

## Context
<forces, constraints, the problem>

## Decision
We will <decision>.

## Alternatives considered
- <alternative> — <why not>

## Consequences
- (+) <benefit>
- (−) <cost>

## Follow-ups & accepted debt
- <deferred work we knowingly accept>
```
Link related ADRs with `[[NNNN]]`.

## 4. Report
Print the path + a one-line summary. **Do not implement the change** — an ADR
records a decision; the implementation is separate.
