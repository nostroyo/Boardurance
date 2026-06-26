---
description: Pre-PR judge gate — runs spec-conformance, correctness, and security judges on the branch diff, then writes a PASS/BLOCK verdict artifact.
argument-hint: "[feature-spec-name]  (optional; e.g. auth-middleware — defaults to inferring from the branch)"
allowed-tools: Bash, Read, Grep, Glob, Edit, Write, Agent, Skill
---

You are running the **review gate** — "la machine relit la machine" — the pre-PR
review step for Boardurance. Goal: catch the *looks-correct-but-isn't* class of
bug (tenant-data leaks, weakened tests, spec gaps) that CI cannot, BEFORE a PR is
opened. The human still owns the merge; this gate produces an on-record verdict.

Feature spec argument (optional): `$ARGUMENTS`

Work from the repo root `Boardurance/` (the git repo). Follow these steps in order.

## 1. Resolve scope

- `BASE=$(git merge-base origin/main HEAD)` and inspect the diff with
  `git diff --stat $BASE...HEAD` and `git diff $BASE...HEAD`. Also include
  uncommitted working-tree changes (`git diff` and `git status --porcelain`) so
  in-progress work is reviewed too.
- Determine changed areas: `rust-backend/` (backend) and/or `empty-project/`
  (frontend).
- Resolve the relevant spec folder under `.kiro/specs/`:
  - if `$ARGUMENTS` is given, use `.kiro/specs/$ARGUMENTS/`;
  - else infer from the branch name (`git branch --show-current`);
  - else if you cannot confidently map it, ask the user which spec applies (or
    confirm "no spec — ad hoc change", in which case skip the conformance judge
    but say so explicitly in the artifact).

## 2. Run THREE independent judges

Run them so their verdicts don't contaminate each other. Capture each judge's
findings with severity (high / medium / low) and file:line references.

1. **Spec-conformance judge** — spawn a subagent (Agent tool, `Explore` or
   general-purpose) with: the diff, and the resolved
   `.kiro/specs/<feature>/requirements.md` + `tasks.md`. It must check, per EARS
   acceptance criterion (`WHEN … THE … SHALL …`):
   - is the criterion implemented in the diff?
   - is there a functional/BDD test that exercises it, and does it actually run
     (not skipped/ignored)?
   Output a per-criterion **PASS / FAIL / N-A** checklist plus any gaps.

2. **Correctness judge** — invoke the `/code-review` skill on the current diff
   (correctness bugs, reuse, simplification, efficiency). Collect its findings.

3. **Security judge** — invoke the `/security-review` skill, and on top of its
   output explicitly verify the `CLAUDE.md` **Always / Never** rules:
   - tenant isolation: every user/org-scoped query filters by the authenticated
     tenant; a cross-tenant negative test exists for any new data-access path;
   - no secrets / tokens / passwords / PII (emails, user ids) in logs, errors,
     commit messages, or fixtures;
   - test integrity: no test was skipped (`.skip`/`.only`/`#[ignore]`), deleted,
     or weakened, and coverage was not reduced to pass;
   - i18n: no hardcoded user-facing strings; no prod-data / migration access.

## 3. Aggregate the verdict

Verdict is **BLOCK** if ANY of:
- an acceptance criterion is FAIL,
- a test was skipped / removed / weakened or coverage reduced,
- any **high**-severity correctness or security finding.

Otherwise **PASS** (medium/low findings are recorded as non-blocking notes).

## 4. Emit the traceable artifact

Write `docs/reviews/<branch>-<seq>.md` where `<branch>` is the sanitized current
branch name and `<seq>` is the next integer not already used for that branch in
`docs/reviews/`. Use this structure:

```
# Review gate — <branch>

- Date: <today>
- Base SHA: <BASE>  | Head SHA: <HEAD>
- Spec: .kiro/specs/<feature>/  (or "none — ad hoc")
- Changed areas: backend / frontend
- Verdict: PASS | BLOCK

## Acceptance-criteria checklist
- [PASS|FAIL|N-A] <criterion> — <note / test path>

## Correctness (code-review)
- [severity] <file:line> — <finding>

## Security (security-review + Always/Never)
- [severity] <file:line> — <finding>

## Blocking items (must fix before PR)
- ...

## Non-blocking notes
- ...
```

## 5. Report

Print the verdict. On BLOCK, give a concrete, ordered fix list. On PASS, say it's
ready for PR and point to the artifact path. Do NOT open the PR or push — the
human owns the merge.
