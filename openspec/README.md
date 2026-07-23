# FROZEN — read-only history (2026-07)

OpenSpec is retired as the project's spec workflow. The project now runs entirely
on **BMAD-METHOD v6**: planning artifacts, epics/stories, and review/test artifacts
live in `_bmad-output/` (see root `CLAUDE.md`, section "Method: BMAD").

What remains here stays valuable as a snapshot of shipped behavior:

- `specs/<capability>/spec.md` — 8 capability specs (SHALL + GIVEN/WHEN/THEN).
  Still accurate descriptions of existing behavior at freeze time; never edit,
  never treat as current truth. New behavior is specified in BMAD PRDs/stories.
- `changes/` — historical change proposals. `add-multiplayer-turn-sync` was
  in flight at freeze time; that feature continues under BMAD planning.
- `project.md`, `config.yaml` — kept for historical context only. The `/opsx:*`
  commands and `openspec-*` skills were removed; `openspec validate` is no
  longer part of the definition of done.
