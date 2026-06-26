---
description: Write a blameless postmortem for an incident or recurring failure (e.g. after the 3-attempt rule trips) — root cause, fix, detection gap, and follow-ups — under docs/postmortems/.
argument-hint: "<short incident title>"
allowed-tools: Bash, PowerShell, Read, Grep, Glob, Write
---

You are writing a **blameless postmortem** for Boardurance — the talk's "chaque
problème rencontré devient une opportunité d'améliorer la méthode". The goal is
not blame; it is to shrink time-to-detect / time-to-fix next time. Postmortems
live in `docs/postmortems/` as `YYYY-MM-DD-kebab-title.md`.

**Trigger this when:** a fix has failed ~3 times on the same problem (the
CLAUDE.md termination rule), after a prod incident, or when a defect escaped to a
late stage (PR review / prod).

Incident: `$ARGUMENTS` (if empty, ask).

## Gather (infer from the conversation/diff/logs; ask only for gaps)
- **Summary** — one line: what broke and the impact.
- **Timeline** — what happened and what was tried, in order.
- **Root cause** — the actual cause, not the symptom.
- **Resolution** — the fix that worked.
- **Detection gap** — *why it wasn't caught earlier.* The most valuable section.
- **Follow-up actions** — concrete and owned: tests, CLAUDE.md rules, hooks, or
  guards that would catch this class automatically next time.
- **Accepted debt** — what was knowingly deferred.
- **Method note** — if this is the **3rd** postmortem touching the same
  method/area, flag a meta-review of the method itself (the talk's méta-postmortem).

## Write `docs/postmortems/<YYYY-MM-DD>-<kebab-title>.md`

```
# Postmortem — <title>

- Date: <YYYY-MM-DD> | Severity: <low|med|high> | Status: <resolved|monitoring>

## Summary
## Timeline
## Root cause
## Resolution
## Detection gap
## Follow-up actions
- [ ] <action>
## Accepted debt
## Method note
```

## Report
Print the path + the single highest-value follow-up. **Propose** follow-ups;
don't apply them here (unless trivial) — they are separate, reviewable changes.
