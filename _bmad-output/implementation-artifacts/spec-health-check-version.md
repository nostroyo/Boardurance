---
title: 'Expose crate version on /health_check'
type: 'feature'
created: '2026-07-23'
status: 'done'
review_loop_iteration: 0
baseline_commit: 'e22fb3268dd9991ecaf850fbb7af2d58ca77aeaf'
context:
  - '{project-root}/_bmad-output/project-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `/health_check` returns `{status, message}` but no build identifier, so there is no cheap way to tell which build is running on test / preprod / prod (three Render environments deploying off `dev` and `main`).

**Approach:** Add a `version` field to `HealthResponse`, populated from `env!("CARGO_PKG_VERSION")` (compile-time constant, no runtime state), and regenerate the OpenAPI contract and the frontend types that are derived from it.

## Boundaries & Constraints

**Always:** Keep `status` as the first serialized field with value `"ok"` on the healthy path so the compact JSON body still contains the exact substring `"status":"ok"` (the Render deploy poll greps for it). Add `version` as an additive field. Keep the committed `docs/openapi.json` and `empty-project/src/types/api-generated.ts` in lockstep with the code (both have CI drift gates).

**Ask First:** Any change to the value or position of the existing `status` / `message` fields; introducing runtime/config plumbing instead of the compile-time constant.

**Never:** Do not change the HTTP status codes (200 on both healthy and degraded). Do not add a `service`/name field or any other field beyond `version` (out of scope). Do not touch auth, AppState, or the router state type.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Healthy | Mongo reachable | `200` + `{"status":"ok","message":"...","version":"<crate ver>"}` | N/A |
| Degraded | Mongo unreachable | `200` + `{"status":"degraded","message":"...","version":"<crate ver>"}` | N/A |
| Version wiring | any build | `version` equals `env!("CARGO_PKG_VERSION")`, never empty | compile-time constant, cannot be absent |

</frozen-after-approval>

## Code Map

- `rust-backend/src/routes/health_check.rs` -- `HealthResponse` struct (+ `ToSchema`) and the handler that builds both branches; add the `version` field + a no-Mongo unit test here.
- `rust-backend/src/startup.rs` -- utoipa `ApiDoc` registers the path + `HealthResponse` schema; snapshot test `committed_openapi_schema_is_up_to_date` asserts the committed contract matches live output (runs in `cargo test-fast`).
- `docs/openapi.json` -- committed OpenAPI contract; regenerated artifact (`HealthResponse` schema ~lines 3013-3027).
- `empty-project/src/types/api-generated.ts` -- frontend type generated from `docs/openapi.json`; regenerated artifact (`HealthResponse` ~lines 993-996).

## Tasks & Acceptance

**Execution:**
- [x] `rust-backend/src/routes/health_check.rs` -- add `pub version: String` to `HealthResponse` (declared AFTER `status` and `message`); set it to `env!("CARGO_PKG_VERSION").to_string()` in both the healthy and degraded branches -- exposes the build id while preserving field order.
- [x] `rust-backend/src/routes/health_check.rs` -- add a `#[cfg(test)]` unit test (no Mongo) asserting the constructed healthy response has `status == "ok"` and `version == env!("CARGO_PKG_VERSION")` and non-empty -- fast-lane guard on the wiring and the contract substring.
- [x] `docs/openapi.json` -- regenerate via `cargo run --bin dump_openapi > ../docs/openapi.json` (from `rust-backend/`) -- keep the committed contract in sync so the snapshot test and frontend gen pass.
- [x] `empty-project/src/types/api-generated.ts` -- regenerate via `npm run gen:api` (from `empty-project/`) -- keep the FE type in sync so `gen:api:check` passes.

**Acceptance Criteria:**
- Given a running server with Mongo reachable, when I GET `/health_check`, then the body contains `"status":"ok"` as a substring AND a `version` field equal to the crate version.
- Given the code change, when `cargo test-fast` runs, then both the new unit test and the OpenAPI snapshot test pass (committed `docs/openapi.json` is up to date).
- Given the regenerated contract, when frontend `gen:api:check` runs, then there is no diff in `api-generated.ts`.

## Verification

**Commands:**
- `.claude/scripts/be.ps1 fmt --check` -- expected: no formatting diff.
- `.claude/scripts/be.ps1 clippy --all-targets --all-features -- -D warnings -A clippy::too_many_lines -A clippy::cast_possible_truncation -A clippy::cast_precision_loss -A clippy::cast_sign_loss -A clippy::cast_possible_wrap -A clippy::match_wildcard_for_single_variants -A clippy::manual_let_else -A clippy::needless_pass_by_value -A clippy::needless_range_loop -A dead_code` -- expected: clean.
- `.claude/scripts/be.ps1 check --all-targets --all-features` -- expected: compiles.
- `.claude/scripts/be.ps1 test-fast` -- expected: green, including `committed_openapi_schema_is_up_to_date` and the new health_check unit test.
- `.claude/scripts/fe.ps1 npm run gen:api:check` -- expected: no diff in `api-generated.ts`.
- `.claude/scripts/fe.ps1 npx tsc --noEmit` -- expected: no type errors.
- `.claude/scripts/fe.ps1 npm run build` -- expected: build succeeds.

## Suggested Review Order

**The change (backend)**

- Entry point — the shared healthy/degraded constructors both the handler and the tests use (single source of truth for the body).
  [`health_check.rs:17`](../../rust-backend/src/routes/health_check.rs#L17)

- The `version` field on the response schema (serialized after `status`, so the deploy-poll substring survives).
  [`health_check.rs:7`](../../rust-backend/src/routes/health_check.rs#L7)

- Handler now returns the constructors instead of inline literals.
  [`health_check.rs:45`](../../rust-backend/src/routes/health_check.rs#L45)

**Contract artifacts (generated / regenerated)**

- OpenAPI schema gains a required `version` — regenerated, guarded by the snapshot test.
  [`openapi.json`](../../docs/openapi.json)

- Frontend type regenerated from the schema in lockstep.
  [`api-generated.ts`](../../empty-project/src/types/api-generated.ts)

**Tests & docs (supporting)**

- Fast-lane tests assert via the shared constructors: healthy body starts with `{"status":"ok"` + carries version; degraded body doesn't match the poll.
  [`health_check.rs:70`](../../rust-backend/src/routes/health_check.rs#L70)

- Test-router health double kept in parity with the new field.
  [`test_utils.rs:202`](../../rust-backend/src/test_utils.rs#L202)

- Hand-maintained route doc updated to show `version`.
  [`API_ROUTES.md:18`](../../docs/API_ROUTES.md#L18)
