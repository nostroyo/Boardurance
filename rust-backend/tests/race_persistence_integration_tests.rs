//! HTTP-level integration tests proving race persistence flows through
//! `state.race_repository` end-to-end.
//!
//! Before Phase 2 of the mongo-persistence feature, every race route read and
//! wrote a process-global `RACE_STORE` static instead of the repository
//! abstraction, so `state.race_repository` was never actually exercised by a
//! live request even though the trait/mock/Mongo implementations were already
//! tested in isolation (see `repository_conformance_mock.rs` /
//! `repository_conformance_mongo.rs`). These tests drive the real router
//! (built the same way `main.rs` does, via `startup::run`) over real HTTP —
//! create a race, join it, process a turn, and re-fetch it — to prove the
//! mutation actually persisted through the repository rather than some
//! now-removed in-memory shortcut.
//!
//! Follows the harness pattern of `tests/auth_integration_tests.rs`. That
//! harness's `get_connection_pool` call eagerly pings `MongoDB` even when
//! `StorageBackend::Mock` is selected for the repositories themselves, so
//! (like `auth_integration_tests.rs`) this file requires `MongoDB` reachable
//! at test time and runs under `cargo test-integration`, not `cargo test-fast`.

use rust_backend::configuration::get_configuration;
use rust_backend::startup::{get_connection_pool, run};
use rust_backend::telemetry::{get_subscriber, init_subscriber};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use uuid::Uuid;

static TRACING: std::sync::Once = std::sync::Once::new();

struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
}

impl TestApp {
    async fn post(&self, path: &str, body: &Value) -> reqwest::Response {
        self.client
            .post(format!("{}{}", self.address, path))
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", self.address, path))
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

async fn spawn_app() -> TestApp {
    TRACING.call_once(|| {
        let default_filter_level = "info".to_string();
        let subscriber_name = "test".to_string();
        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        }
    });

    std::env::set_var("APP_ENVIRONMENT", "test");

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c
    };

    let database = get_connection_pool(&configuration.database)
        .await
        .expect("Failed to connect to database");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let server = run(
        listener,
        database,
        configuration.application.base_url,
        rust_backend::configuration::StorageBackend::Mock,
    )
    .await
    .expect("Failed to build application.");
    #[allow(clippy::let_underscore_future)]
    let _ = tokio::spawn(async move { server.await.expect("Server failed to start") });

    let client = reqwest::Client::new();

    TestApp { address, client }
}

fn create_race_body() -> Value {
    json!({
        "name": "Persistence Test Race",
        "track_name": "Persistence Test Track",
        "total_laps": 3,
        "sectors": [
            {
                "id": 0,
                "name": "Start",
                "min_value": 0,
                "max_value": 10,
                "slot_capacity": null,
                "sector_type": "Start"
            },
            {
                "id": 1,
                "name": "Straight",
                "min_value": 5,
                "max_value": 14,
                "slot_capacity": null,
                "sector_type": "Straight"
            },
            {
                "id": 2,
                "name": "Finish",
                "min_value": 8,
                "max_value": 16,
                "slot_capacity": null,
                "sector_type": "Finish"
            }
        ]
    })
}

/// Full create -> join -> turn -> `get_race` flow over real HTTP, asserting the
/// race's mutated state (participants, `turns_taken`) is exactly what's expected
/// at each step. This can only pass if every handler along the way actually
/// reads/writes through the same backing store (`state.race_repository`); the
/// old `RACE_STORE` global would have made this trivially true even with a
/// broken repository wiring, which is precisely the gap this test closes.
#[tokio::test]
async fn race_persists_through_create_join_turn_and_refetch() {
    let app = spawn_app().await;

    // 1. Create a race. `create_race` auto-starts it (status InProgress,
    // current_lap 1) for UX reasons, so it's immediately joinable per the
    // domain rule that allows joining while still on lap 1.
    let create_response = app.post("/api/v1/races", &create_race_body()).await;
    assert_eq!(201, create_response.status().as_u16());
    let create_body: Value = create_response.json().await.expect("valid JSON");
    let race_uuid = create_body["race"]["uuid"]
        .as_str()
        .expect("race uuid present")
        .to_string();
    assert_eq!(create_body["race"]["status"], "InProgress");
    assert_eq!(
        create_body["race"]["participants"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    // 2. Join two players over HTTP. Each join must be visible to the next
    // request, which only holds if `join_race` persists through the shared
    // repository rather than a per-request/in-memory copy.
    let player_a = Uuid::new_v4();
    let car_a = Uuid::new_v4();
    let pilot_a = Uuid::new_v4();
    let join_a = app
        .post(
            &format!("/api/v1/races/{race_uuid}/join"),
            &json!({
                "player_uuid": player_a.to_string(),
                "car_uuid": car_a.to_string(),
                "pilot_uuid": pilot_a.to_string(),
            }),
        )
        .await;
    assert_eq!(200, join_a.status().as_u16());
    let body_a: Value = join_a.json().await.expect("valid JSON");
    assert_eq!(body_a["race"]["participants"].as_array().unwrap().len(), 1);

    let player_b = Uuid::new_v4();
    let car_b = Uuid::new_v4();
    let pilot_b = Uuid::new_v4();
    let join_b = app
        .post(
            &format!("/api/v1/races/{race_uuid}/join"),
            &json!({
                "player_uuid": player_b.to_string(),
                "car_uuid": car_b.to_string(),
                "pilot_uuid": pilot_b.to_string(),
            }),
        )
        .await;
    assert_eq!(200, join_b.status().as_u16());
    let body_b: Value = join_b.json().await.expect("valid JSON");
    // Both the previously-joined player_a AND the newly-joined player_b must
    // be present: proves join_b's handler loaded the race state that join_a's
    // handler wrote, not a stale/local copy.
    assert_eq!(body_b["race"]["participants"].as_array().unwrap().len(), 2);

    // 3. Independently re-fetch the race and confirm both joins persisted.
    let get_after_joins = app.get(&format!("/api/v1/races/{race_uuid}")).await;
    assert_eq!(200, get_after_joins.status().as_u16());
    let race_after_joins: Value = get_after_joins.json().await.expect("valid JSON");
    let participants_after_joins = race_after_joins["participants"].as_array().unwrap();
    assert_eq!(participants_after_joins.len(), 2);
    let participant_uuids: Vec<&str> = participants_after_joins
        .iter()
        .map(|p| p["player_uuid"].as_str().unwrap())
        .collect();
    assert!(participant_uuids.contains(&player_a.to_string().as_str()));
    assert!(participant_uuids.contains(&player_b.to_string().as_str()));
    assert_eq!(race_after_joins["turns_taken"].as_u64().unwrap(), 0);

    // 4. Process a batch turn for both players (neither is a registered
    // player, so the real per-car performance lookup misses and the handler
    // falls back to placeholder performance — this is existing, unchanged
    // behavior; the point of this test is the persistence path, not the
    // performance model).
    let turn_response = app
        .post(
            &format!("/api/v1/races/{race_uuid}/turn"),
            &json!({
                "actions": [
                    {"player_uuid": player_a.to_string(), "boost_value": 2},
                    {"player_uuid": player_b.to_string(), "boost_value": 1},
                ]
            }),
        )
        .await;
    assert_eq!(200, turn_response.status().as_u16());

    // 5. Re-fetch via a brand-new request and confirm the turn's mutation
    // (turns_taken incremented) survived — the thing `RACE_STORE` masked
    // before, since every handler previously shared one process-global
    // HashMap regardless of whether `state.race_repository` was wired up
    // correctly.
    let get_after_turn = app.get(&format!("/api/v1/races/{race_uuid}")).await;
    assert_eq!(200, get_after_turn.status().as_u16());
    let race_after_turn: Value = get_after_turn.json().await.expect("valid JSON");
    assert_eq!(
        race_after_turn["turns_taken"].as_u64().unwrap(),
        1,
        "the processed turn must be persisted and visible on refetch"
    );
    assert!(race_after_turn["pending_actions"]
        .as_array()
        .unwrap()
        .is_empty());

    // 6. `get_all_races` (a separate handler/read path) must also see the
    // same race with the same persisted state.
    let all_races_response = app.get("/api/v1/races").await;
    assert_eq!(200, all_races_response.status().as_u16());
    let all_races: Value = all_races_response.json().await.expect("valid JSON");
    let found = all_races
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["uuid"].as_str() == Some(race_uuid.as_str()))
        .expect("created race must appear in get_all_races");
    assert_eq!(found["turns_taken"].as_u64().unwrap(), 1);
}

/// The solo-mode turn-orchestration path (`/races/solo` + `/submit-action`)
/// exercises a different set of previously-migrated helpers
/// (`resolve_human_turn`, `drive_ai_only_turns`, `submit_player_action_in_db`)
/// than the batch `/turn` path above, each of which used to read/write the
/// removed `RACE_STORE` global directly with no state param at all. This
/// proves that path also persists through `state.race_repository`: a solo
/// race created for a freshly-registered player (registration seeds a
/// complete starter car, per `Player::new_with_assets`) must still show the
/// submitted turn's effects on refetch.
#[tokio::test]
async fn solo_race_turn_persists_through_submit_action() {
    let app = spawn_app().await;

    // Register a real player; registration seeds a complete starter car, so
    // `create_solo_race` (which requires `car.is_complete()`) can use it.
    let register_response = app
        .post(
            "/api/v1/auth/register",
            &json!({
                "email": "race-persistence-solo@example.com",
                "password": "Password123",
                "team_name": "Persistence Test Team",
            }),
        )
        .await;
    assert_eq!(201, register_response.status().as_u16());
    let register_body: Value = register_response.json().await.expect("valid JSON");
    let player_uuid = register_body["user"]["uuid"]
        .as_str()
        .expect("player uuid present")
        .to_string();

    // Create + auto-start the solo race (human + seeded AI opponents).
    let solo_response = app
        .post("/api/v1/races/solo", &json!({ "player_uuid": player_uuid }))
        .await;
    assert_eq!(201, solo_response.status().as_u16());
    let solo_body: Value = solo_response.json().await.expect("valid JSON");
    let race_uuid = solo_body["race"]["uuid"]
        .as_str()
        .expect("race uuid present")
        .to_string();
    assert_eq!(solo_body["race"]["turns_taken"].as_u64().unwrap(), 0);
    let participants_at_start = solo_body["race"]["participants"].as_array().unwrap().len();
    assert!(
        participants_at_start >= 2,
        "solo race must include the human plus at least one AI opponent"
    );

    // Submit the human's boost action; `resolve_human_turn` should enqueue
    // the AI opponents and process the lap once everyone has acted.
    let submit_response = app
        .post(
            &format!("/api/v1/races/{race_uuid}/submit-action"),
            &json!({ "player_uuid": player_uuid, "boost_value": 1 }),
        )
        .await;
    assert_eq!(200, submit_response.status().as_u16());
    let submit_body: Value = submit_response.json().await.expect("valid JSON");
    assert_eq!(submit_body["turn_phase"], "TurnProcessed");

    // Re-fetch via a brand-new request: the turn must have been persisted by
    // `submit_player_action_in_db`/`resolve_human_turn`/`process_lap_in_db`
    // through `state.race_repository`, not lost with the removed global.
    let get_response = app.get(&format!("/api/v1/races/{race_uuid}")).await;
    assert_eq!(200, get_response.status().as_u16());
    let race_after_turn: Value = get_response.json().await.expect("valid JSON");
    assert_eq!(
        race_after_turn["turns_taken"].as_u64().unwrap(),
        1,
        "the submitted turn must be persisted and visible on refetch"
    );
    assert!(race_after_turn["pending_actions"]
        .as_array()
        .unwrap()
        .is_empty());
}
