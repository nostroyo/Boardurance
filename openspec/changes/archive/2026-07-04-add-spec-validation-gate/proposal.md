# Add spec validation gate

## Why

The OpenSpec migration (feat/openspec-migration) makes `openspec/specs/` the
current-truth source. Without a validation gate, spec edits can drift out of
the enforced format. CLAUDE.md's Definition of done now requires
`openspec validate --all --strict` whenever anything under `openspec/` changes
— this change records that gate in the `ci-cd` capability spec.

## What Changes

- `ci-cd`: one ADDED requirement — Spec validation gate.

Capabilities touched: `ci-cd`. No ADR needed (process documentation only, no
architectural tradeoff).

## Non-goals

- No CI workflow change (the gate is a local Definition-of-done step for now;
  wiring it into GitHub Actions would be a follow-up change).
