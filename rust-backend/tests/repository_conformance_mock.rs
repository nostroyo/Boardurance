//! Runs the shared repository conformance suite (see `tests/common/mod.rs`)
//! against the in-memory `Mock*` repositories. No database needed — this is
//! part of `cargo test-fast`.
//!
//! The identical assertions also run against `Mongo*` in
//! `tests/repository_conformance_mongo.rs` (`cargo test-integration`). Passing
//! on both proves the mock is a faithful stand-in for Mongo.

mod common;

use rust_backend::repositories::{MockPlayerRepository, MockRaceRepository, MockSessionRepository};

// ----- PlayerRepository -----

#[tokio::test]
async fn mock_player_create_and_find_round_trips() {
    common::player_create_and_find_round_trips(&MockPlayerRepository::new()).await;
}

#[tokio::test]
async fn mock_player_find_missing_returns_none() {
    common::player_find_missing_returns_none(&MockPlayerRepository::new()).await;
}

#[tokio::test]
async fn mock_player_duplicate_email_is_conflict() {
    common::player_duplicate_email_is_conflict(&MockPlayerRepository::new()).await;
}

#[tokio::test]
async fn mock_player_update_team_name_round_trips() {
    common::player_update_team_name_round_trips(&MockPlayerRepository::new()).await;
}

#[tokio::test]
async fn mock_player_update_missing_returns_none() {
    common::player_update_missing_returns_none(&MockPlayerRepository::new()).await;
}

#[tokio::test]
async fn mock_player_add_and_remove_car_round_trips() {
    common::player_add_and_remove_car_round_trips(&MockPlayerRepository::new()).await;
}

#[tokio::test]
async fn mock_player_delete_by_uuid_works() {
    common::player_delete_by_uuid_works(&MockPlayerRepository::new()).await;
}

// ----- RaceRepository -----

#[tokio::test]
async fn mock_race_create_and_find_round_trips() {
    common::race_create_and_find_round_trips(&MockRaceRepository::new()).await;
}

#[tokio::test]
async fn mock_race_find_missing_returns_none() {
    common::race_find_missing_returns_none(&MockRaceRepository::new()).await;
}

#[tokio::test]
async fn mock_race_join_missing_race_is_not_found() {
    common::race_join_missing_race_is_not_found(&MockRaceRepository::new()).await;
}

#[tokio::test]
async fn mock_race_duplicate_join_is_conflict() {
    common::race_duplicate_join_is_conflict(&MockRaceRepository::new()).await;
}

#[tokio::test]
async fn mock_race_join_non_waiting_race_is_validation_error() {
    common::race_join_non_waiting_race_is_validation_error(&MockRaceRepository::new()).await;
}

#[tokio::test]
async fn mock_race_turn_processing_persists() {
    common::race_turn_processing_persists(&MockRaceRepository::new()).await;
}

#[tokio::test]
async fn mock_race_turn_processing_before_start_is_validation_error() {
    common::race_turn_processing_before_start_is_validation_error(&MockRaceRepository::new()).await;
}

#[tokio::test]
async fn mock_race_get_by_status_filters_correctly() {
    common::race_get_by_status_filters_correctly(&MockRaceRepository::new()).await;
}

// ----- SessionRepository -----

#[tokio::test]
async fn mock_session_create_and_find_round_trips() {
    common::session_create_and_find_round_trips(&MockSessionRepository::new()).await;
}

#[tokio::test]
async fn mock_session_find_missing_returns_none() {
    common::session_find_missing_returns_none(&MockSessionRepository::new()).await;
}

#[tokio::test]
async fn mock_session_expired_is_not_found() {
    common::session_expired_is_not_found(&MockSessionRepository::new()).await;
}

#[tokio::test]
async fn mock_session_deactivate_hides_session() {
    common::session_deactivate_hides_session(&MockSessionRepository::new()).await;
}

#[tokio::test]
async fn mock_session_deactivate_all_for_user_is_scoped() {
    common::session_deactivate_all_for_user_is_scoped(&MockSessionRepository::new()).await;
}

#[tokio::test]
async fn mock_session_cleanup_expired_removes_only_expired() {
    common::session_cleanup_expired_removes_only_expired(&MockSessionRepository::new()).await;
}

#[tokio::test]
async fn mock_session_count_active_for_user_is_accurate() {
    common::session_count_active_for_user_is_accurate(&MockSessionRepository::new()).await;
}
