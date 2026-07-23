# FROZEN — read-only history (2026-07)

This directory belongs to the retired Kiro spec system. It was first superseded
by OpenSpec (see `docs/migration/kiro-to-openspec.md`), and the project has since
adopted **BMAD-METHOD v6** as its only workflow: planning artifacts, epics/stories,
and review/test artifacts live in `_bmad-output/` (see root `CLAUDE.md`).

- `specs/` — 12 legacy feature specs (requirements/design/tasks). Still useful as
  descriptions of behavior that shipped; never edit, never treat as current truth.
- `steering/` — legacy Kiro steering docs. Surviving rules (branch-per-feature,
  worktree discipline, docs layout) are folded into root `CLAUDE.md`.
- `settings/` — retired. The Kiro MCP config (`settings/mcp.json`) was removed
  because it contained a hardcoded credential; rotate any token that lived there.
