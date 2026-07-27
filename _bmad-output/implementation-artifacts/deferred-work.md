# Deferred Work

Findings surfaced during reviews that are real but out of the current story's scope.

- source_spec: `spec-health-check-version.md`
  summary: `/health_check` version can't distinguish two builds deployed at the same crate version — consider adding a git SHA / build timestamp.
  evidence: The spec's stated problem is telling which build runs on test/preprod/prod, but `env!("CARGO_PKG_VERSION")` only changes on a manual `Cargo.toml` bump, so successive `dev`→preprod and `main`→prod deploys without a version bump all report the same `"version"`. Raised by both review hunters (Blind Hunter #1). Touches the frozen intent (source was explicitly chosen as CARGO_PKG_VERSION), so it's a product decision for the human, not an auto-fix. A `git_sha` field (via a build script reading `GIT_SHA`/`vergen`) would make builds distinguishable.
