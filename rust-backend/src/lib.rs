pub mod app_state;
pub mod configuration;
pub mod domain;
pub mod middleware;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod startup;
pub mod telemetry;

// Make test_utils available for integration tests
#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;
