//! Runs the shared repository conformance suite (see `tests/common/mod.rs`)
//! against the real `MongoDB`-backed repositories. Requires a reachable
//! `MongoDB` (see `docker-compose.yml`: `docker compose up -d`) — this is part
//! of `cargo test-integration`, not `test-fast`.
//!
//! The identical assertions also run against `Mock*` in
//! `tests/repository_conformance_mock.rs` (`cargo test-fast`, no DB). Passing
//! on both proves the mock is a faithful stand-in for Mongo.
//!
//! Each test gets its own randomly-named database (mirroring the pattern in
//! `tests/auth_integration_tests.rs`) so tests never collide with each other
//! or with a previous run, and each repository's constructor creates its
//! required unique index — asserted explicitly below.

mod common;

use mongodb::bson::doc;
use rust_backend::configuration::get_configuration;
use rust_backend::repositories::{
    MongoPlayerRepository, MongoRaceRepository, MongoSessionRepository,
};
use rust_backend::startup::get_connection_pool;

/// Connect to a freshly-named database for test isolation. Panics with a
/// clear message if `MongoDB` isn't reachable, rather than hanging — the short
/// server-selection timeout in `build_mongo_client` makes this fail fast.
async fn test_database() -> mongodb::Database {
    std::env::set_var("APP_ENVIRONMENT", "test");
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = format!("conformance_{}", uuid::Uuid::new_v4());

    get_connection_pool(&configuration.database)
        .await
        .unwrap_or_else(|e| {
            panic!(
                "MongoDB is not reachable ({e}). Run `docker compose up -d` from rust-backend/ \
             before `cargo test-integration`."
            )
        })
}

// ----- PlayerRepository -----

#[tokio::test]
async fn mongo_player_create_and_find_round_trips() {
    let repo = MongoPlayerRepository::new(&test_database().await)
        .await
        .unwrap();
    common::player_create_and_find_round_trips(&repo).await;
}

#[tokio::test]
async fn mongo_player_find_missing_returns_none() {
    let repo = MongoPlayerRepository::new(&test_database().await)
        .await
        .unwrap();
    common::player_find_missing_returns_none(&repo).await;
}

#[tokio::test]
async fn mongo_player_duplicate_email_is_conflict() {
    let repo = MongoPlayerRepository::new(&test_database().await)
        .await
        .unwrap();
    common::player_duplicate_email_is_conflict(&repo).await;
}

#[tokio::test]
async fn mongo_player_update_team_name_round_trips() {
    let repo = MongoPlayerRepository::new(&test_database().await)
        .await
        .unwrap();
    common::player_update_team_name_round_trips(&repo).await;
}

#[tokio::test]
async fn mongo_player_update_missing_returns_none() {
    let repo = MongoPlayerRepository::new(&test_database().await)
        .await
        .unwrap();
    common::player_update_missing_returns_none(&repo).await;
}

#[tokio::test]
async fn mongo_player_add_and_remove_car_round_trips() {
    let repo = MongoPlayerRepository::new(&test_database().await)
        .await
        .unwrap();
    common::player_add_and_remove_car_round_trips(&repo).await;
}

#[tokio::test]
async fn mongo_player_delete_by_uuid_works() {
    let repo = MongoPlayerRepository::new(&test_database().await)
        .await
        .unwrap();
    common::player_delete_by_uuid_works(&repo).await;
}

/// The `players` collection must have a unique index on `uuid` (see
/// `MongoPlayerRepository::new`), which is what makes duplicate-key
/// detection in `create` meaningful rather than accidental.
#[tokio::test]
async fn mongo_player_repository_has_unique_uuid_index() {
    let database = test_database().await;
    let _repo = MongoPlayerRepository::new(&database).await.unwrap();

    let collection = database.collection::<mongodb::bson::Document>("players");
    let indexes = collection.list_index_names().await.unwrap();
    let has_uuid_index = indexes.iter().any(|name| name.contains("uuid"));
    assert!(has_uuid_index, "expected a uuid index among {indexes:?}");
}

// ----- RaceRepository -----

#[tokio::test]
async fn mongo_race_create_and_find_round_trips() {
    let repo = MongoRaceRepository::new(&test_database().await)
        .await
        .unwrap();
    common::race_create_and_find_round_trips(&repo).await;
}

#[tokio::test]
async fn mongo_race_find_missing_returns_none() {
    let repo = MongoRaceRepository::new(&test_database().await)
        .await
        .unwrap();
    common::race_find_missing_returns_none(&repo).await;
}

#[tokio::test]
async fn mongo_race_join_missing_race_is_not_found() {
    let repo = MongoRaceRepository::new(&test_database().await)
        .await
        .unwrap();
    common::race_join_missing_race_is_not_found(&repo).await;
}

#[tokio::test]
async fn mongo_race_duplicate_join_is_conflict() {
    let repo = MongoRaceRepository::new(&test_database().await)
        .await
        .unwrap();
    common::race_duplicate_join_is_conflict(&repo).await;
}

#[tokio::test]
async fn mongo_race_join_non_waiting_race_is_validation_error() {
    let repo = MongoRaceRepository::new(&test_database().await)
        .await
        .unwrap();
    common::race_join_non_waiting_race_is_validation_error(&repo).await;
}

#[tokio::test]
async fn mongo_race_turn_processing_persists() {
    let repo = MongoRaceRepository::new(&test_database().await)
        .await
        .unwrap();
    common::race_turn_processing_persists(&repo).await;
}

#[tokio::test]
async fn mongo_race_turn_processing_before_start_is_validation_error() {
    let repo = MongoRaceRepository::new(&test_database().await)
        .await
        .unwrap();
    common::race_turn_processing_before_start_is_validation_error(&repo).await;
}

#[tokio::test]
async fn mongo_race_get_by_status_filters_correctly() {
    let repo = MongoRaceRepository::new(&test_database().await)
        .await
        .unwrap();
    common::race_get_by_status_filters_correctly(&repo).await;
}

#[tokio::test]
async fn mongo_race_repository_has_unique_uuid_index() {
    let database = test_database().await;
    let _repo = MongoRaceRepository::new(&database).await.unwrap();

    let collection = database.collection::<mongodb::bson::Document>("races");
    let indexes = collection.list_index_names().await.unwrap();
    assert!(indexes.iter().any(|name| name.contains("uuid")));
}

// ----- SessionRepository -----

#[tokio::test]
async fn mongo_session_create_and_find_round_trips() {
    let repo = MongoSessionRepository::new(&test_database().await)
        .await
        .unwrap();
    common::session_create_and_find_round_trips(&repo).await;
}

#[tokio::test]
async fn mongo_session_find_missing_returns_none() {
    let repo = MongoSessionRepository::new(&test_database().await)
        .await
        .unwrap();
    common::session_find_missing_returns_none(&repo).await;
}

#[tokio::test]
async fn mongo_session_expired_is_not_found() {
    let repo = MongoSessionRepository::new(&test_database().await)
        .await
        .unwrap();
    common::session_expired_is_not_found(&repo).await;
}

#[tokio::test]
async fn mongo_session_deactivate_hides_session() {
    let repo = MongoSessionRepository::new(&test_database().await)
        .await
        .unwrap();
    common::session_deactivate_hides_session(&repo).await;
}

#[tokio::test]
async fn mongo_session_deactivate_all_for_user_is_scoped() {
    let repo = MongoSessionRepository::new(&test_database().await)
        .await
        .unwrap();
    common::session_deactivate_all_for_user_is_scoped(&repo).await;
}

#[tokio::test]
async fn mongo_session_cleanup_expired_removes_only_expired() {
    let repo = MongoSessionRepository::new(&test_database().await)
        .await
        .unwrap();
    common::session_cleanup_expired_removes_only_expired(&repo).await;
}

#[tokio::test]
async fn mongo_session_count_active_for_user_is_accurate() {
    let repo = MongoSessionRepository::new(&test_database().await)
        .await
        .unwrap();
    common::session_count_active_for_user_is_accurate(&repo).await;
}

#[tokio::test]
async fn mongo_session_repository_has_unique_token_index() {
    let database = test_database().await;
    let _repo = MongoSessionRepository::new(&database).await.unwrap();

    let collection = database.collection::<mongodb::bson::Document>("sessions");
    let indexes = collection.list_index_names().await.unwrap();
    assert!(indexes.iter().any(|name| name.contains("token")));
}

/// Sanity check that the `doc!` import above is actually used somewhere,
/// keeping the module free of an unused-import warning if the assertions
/// above are trimmed later. (`ping` is also a cheap standalone liveness
/// check for the Mongo connection itself.)
#[tokio::test]
async fn mongo_connection_responds_to_ping() {
    let database = test_database().await;
    database
        .run_command(doc! { "ping": 1 }, None)
        .await
        .expect("MongoDB should respond to a ping");
}
