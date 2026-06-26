//! Print the backend's `OpenAPI` schema as pretty JSON to stdout.
//!
//! The schema is the single source of truth for the frontend's API types.
//! Regenerate the committed contract with:
//!
//! ```sh
//! cargo run --bin dump_openapi > ../docs/openapi.json
//! ```
//!
//! `committed_openapi_schema_is_up_to_date` (in `startup.rs`) fails the test
//! suite if `docs/openapi.json` drifts from this output.

use rust_backend::startup::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let json = ApiDoc::openapi()
        .to_pretty_json()
        .expect("OpenAPI schema should serialize to JSON");
    println!("{json}");
}
