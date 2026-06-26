# Postmortem — Invalid OpenAPI schema (17 dangling $refs) shipped undetected

- Date: 2026-06-26 | Severity: med | Status: resolved

## Summary
The backend's generated OpenAPI document referenced 17 component schemas it never
registered (dangling `$ref`s). Swagger UI rendered it leniently, so it went
unnoticed — but the document was invalid for any strict consumer and blocked
frontend type generation (`openapi-typescript`) the moment we tried it.

## Timeline
- While building the front↔back contract gate, ran `openapi-typescript` against
  the dumped schema → failed with "Can't resolve $ref" ×17.
- Refs pointed at unregistered types: `UserRole`, `LapCharacteristic`,
  `MovementProbability`, `PerformanceCalculation`, `PitStopRequest`, plus `Uuid`
  and chrono `DateTime`, plus two fully-qualified-path refs in `PlayerSpecificData`.

## Root cause
`ApiDoc`'s `components(schemas(...))` list was incomplete: several `ToSchema`
types were referenced by fields but never registered; foreign types (`Uuid`,
`DateTime`) had no utoipa integration enabled; and two fields used a
fully-qualified path (`crate::domain::BoostUsageRecord`), which utoipa named
`crate.domain.BoostUsageRecord` — not matching the registered `BoostUsageRecord`.

## Resolution
Registered the missing schemas, enabled utoipa `uuid` + `chrono` features, and
switched the two fields to the plain type name. The schema now validates and
codegen succeeds.

## Detection gap
Nothing validated the OpenAPI document. Swagger UI tolerates dangling refs, so
the invalid contract was invisible until a strict consumer (codegen) hit it.

## Follow-up actions
- [x] `dump_openapi` bin + `committed_openapi_schema_is_up_to_date` test so the
      schema is regenerated and checked in `cargo test`.
- [x] Frontend `gen:api:check` CI step that fails on type drift.
- [ ] Consider a stricter OpenAPI lint (e.g. Spectral) for semantic issues beyond
      dangling refs.

## Accepted debt
- The hand-written `race-api.ts` consumers haven't migrated to the generated
  types yet; they coexist. Tracked separately.

## Method note
First postmortem in this area — no meta-review needed yet. General lesson:
"renders fine in the UI" is not "valid" — gate a machine-readable contract with a
machine consumer, not a human viewer.
