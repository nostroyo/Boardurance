//! Integration tests for boost card system
//! These tests verify the boost card hand management, validation, and persistence
//! across the full race workflow including API endpoints and database operations.

use rust_backend::configuration::get_configuration;
use rust_backend::startup::{get_connection_pool, run};
use rust_backend::telemetry::{get_subscriber, init_subscriber};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use uuid::Uuid;

// Ensure that the `tracing` stack is only initialised once using `std::sync::Once`
static TRACING: std::sync::Once = std::sync::Once::new();

struct TestApp {
    pub address: String,
    pub _db_name: String,
    pub client: reqwest::Client,
}

impl TestApp {
    // Helper to create a test user and return their UUID and cookies
    pub async fn create_test_user(
        &self,
        email: &str,
        password: &str,
        team_name: &str,
    ) -> (String, String) {
        let register_body = json!({
            "email": email,
            "password": password,
            "team_name": team_name
        });

        let response = self
            .client
            .post(format!("{}/api/v1/auth/register", &self.address))
            .header("Content-Type", "application/json")
            .json(&register_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(201, response.status().as_u16());

        let cookies = TestApp::extract_cookies(&response);
        let response_body: Value = response.json().await.expect("Failed to parse response");
        let user_uuid = response_body["user"]["uuid"].as_str().unwrap().to_string();

        (user_uuid, cookies)
    }

    // Helper to extract cookies from response headers
    pub fn extract_cookies(response: &reqwest::Response) -> String {
        response
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|h| h.to_str().unwrap())
            .collect::<Vec<_>>()
            .join("; ")
    }

    // Helper to create a race
    pub async fn create_race(&self, cookies: &str) -> String {
        let race_body = json!({
            "name": "Test Race",
            "track_name": "Test Track",
            "sectors": [
                {
                    "id": 0,
                    "name": "Sector 1",
                    "min_value": 10,
                    "max_value": 20,
                    "slot_capacity": null,
                    "sector_type": "Straight"
                },
                {
                    "id": 1,
                    "name": "Sector 2",
                    "min_value": 15,
                    "max_value": 25,
                    "slot_capacity": null,
                    "sector_type": "Curve"
                }
            ],
            "total_laps": 3
        });

        let response = self
            .client
            .post(format!("{}/api/v1/races", &self.address))
            .header("Cookie", cookies)
            .json(&race_body)
            .send()
            .await
            .expect("Failed to create race");

        assert_eq!(201, response.status().as_u16());

        let response_body: Value = response.json().await.expect("Failed to parse response");
        response_body["race"]["uuid"].as_str().unwrap().to_string()
    }

    // Helper to register player for race
    pub async fn register_for_race(
        &self,
        race_uuid: &str,
        player_uuid: &str,
        car_uuid: &str,
        cookies: &str,
    ) -> reqwest::Response {
        let register_body = json!({
            "player_uuid": player_uuid,
            "car_uuid": car_uuid
        });

        self.client
            .post(format!(
                "{}/api/v1/races/{}/register",
                &self.address, race_uuid
            ))
            .header("Cookie", cookies)
            .json(&register_body)
            .send()
            .await
            .expect("Failed to register for race")
    }

    // Helper to start race
    pub async fn start_race(&self, race_uuid: &str, cookies: &str) -> reqwest::Response {
        self.client
            .post(format!(
                "{}/api/v1/races/{}/start",
                &self.address, race_uuid
            ))
            .header("Cookie", cookies)
            .send()
            .await
            .expect("Failed to start race")
    }

    // Helper to apply lap action
    pub async fn apply_lap_action(
        &self,
        race_uuid: &str,
        player_uuid: &str,
        car_uuid: &str,
        boost_value: u8,
        cookies: &str,
    ) -> reqwest::Response {
        let lap_body = json!({
            "player_uuid": player_uuid,
            "car_uuid": car_uuid,
            "boost_value": boost_value
        });

        self.client
            .post(format!(
                "{}/api/v1/races/{}/apply-lap",
                &self.address, race_uuid
            ))
            .header("Cookie", cookies)
            .json(&lap_body)
            .send()
            .await
            .expect("Failed to apply lap action")
    }

    // Helper to get detailed race status
    pub async fn get_race_status_detailed(
        &self,
        race_uuid: &str,
        player_uuid: Option<&str>,
        cookies: &str,
    ) -> reqwest::Response {
        let url = if let Some(player_uuid) = player_uuid {
            format!(
                "{}/api/v1/races/{}/status-detailed?player_uuid={}",
                &self.address, race_uuid, player_uuid
            )
        } else {
            format!(
                "{}/api/v1/races/{}/status-detailed",
                &self.address, race_uuid
            )
        };

        self.client
            .get(&url)
            .header("Cookie", cookies)
            .send()
            .await
            .expect("Failed to get race status")
    }

    // Helper to get player's first car UUID
    pub async fn get_player_first_car(&self, player_uuid: &str, cookies: &str) -> String {
        let response = self
            .client
            .get(format!("{}/api/v1/players/{}", &self.address, player_uuid))
            .header("Cookie", cookies)
            .send()
            .await
            .expect("Failed to get player data");

        let player_data: Value = response.json().await.expect("Failed to parse player data");
        player_data["cars"][0]["uuid"].as_str().unwrap().to_string()
    }
}

async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
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

    // Set test environment to use test configuration
    std::env::set_var("APP_ENVIRONMENT", "test");

    // Randomise configuration to ensure test isolation
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // Use a random OS port
        c.application.port = 0;
        c
    };

    // Create and migrate the database
    let database = get_connection_pool(&configuration.database)
        .await
        .expect("Failed to connect to database");

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{port}");

    let server = run(listener, database, configuration.application.base_url)
        .await
        .expect("Failed to build application.");
    #[allow(clippy::let_underscore_future)]
    let _ = tokio::spawn(async move { server.await.expect("Server failed to start") });

    let client = reqwest::Client::new();

    TestApp {
        address,
        _db_name: configuration.database.database_name,
        client,
    }
}

// ============================================================================
// BOOST CARD INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_boost_hand_initializes_with_all_cards_available() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    // Act - Register for race
    let register_response = app
        .register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    assert_eq!(200, register_response.status().as_u16());

    // Start race
    let start_response = app.start_race(&race_uuid, &cookies).await;
    assert_eq!(200, start_response.status().as_u16());

    // Get detailed status
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    assert_eq!(200, status_response.status().as_u16());

    let status_data: Value = status_response
        .json()
        .await
        .expect("Failed to parse status");

    // Assert - Verify boost hand is initialized correctly
    let boost_availability = &status_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["cards_remaining"], 5);
    assert_eq!(boost_availability["current_cycle"], 1);
    assert_eq!(boost_availability["cycles_completed"], 0);

    let available_cards = boost_availability["available_cards"].as_array().unwrap();
    assert_eq!(available_cards.len(), 5);

    // Verify all cards 0-4 are available
    for i in 0..=4 {
        assert!(available_cards.contains(&json!(i)));
    }
}

#[tokio::test]
async fn test_using_boost_card_marks_it_unavailable() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Use boost card 2
    let lap_response = app
        .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 2, &cookies)
        .await;
    assert_eq!(200, lap_response.status().as_u16());

    let lap_data: Value = lap_response
        .json()
        .await
        .expect("Failed to parse lap response");

    // Assert - Verify boost hand state updated
    let boost_availability = &lap_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["cards_remaining"], 4);

    let available_cards = boost_availability["available_cards"].as_array().unwrap();
    assert_eq!(available_cards.len(), 4);
    assert!(!available_cards.contains(&json!(2)));

    // Verify hand state shows card 2 as unavailable
    let hand_state = &boost_availability["hand_state"];
    assert_eq!(hand_state["2"], false);
}

#[tokio::test]
async fn test_cannot_use_same_boost_card_twice() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Use boost card 3
    let lap1_response = app
        .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 3, &cookies)
        .await;
    assert_eq!(200, lap1_response.status().as_u16());

    // Try to use boost card 3 again
    let lap2_response = app
        .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 3, &cookies)
        .await;

    // Assert - Should return 400 with boost card error
    assert_eq!(400, lap2_response.status().as_u16());

    let error_data: Value = lap2_response
        .json()
        .await
        .expect("Failed to parse error response");
    assert_eq!(error_data["error_code"], "BOOST_CARD_NOT_AVAILABLE");
    assert!(error_data["message"]
        .as_str()
        .unwrap()
        .contains("not available"));

    let available_cards = error_data["available_cards"].as_array().unwrap();
    assert!(!available_cards.contains(&json!(3)));
}

#[tokio::test]
async fn test_boost_hand_replenishes_after_all_cards_used() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Use all 5 boost cards
    for boost_value in 0..=4 {
        let lap_response = app
            .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, boost_value, &cookies)
            .await;
        assert_eq!(
            200,
            lap_response.status().as_u16(),
            "Failed to use boost card {boost_value}"
        );
    }

    // Get status after using all cards
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    assert_eq!(200, status_response.status().as_u16());

    let status_data: Value = status_response
        .json()
        .await
        .expect("Failed to parse status");

    // Assert - Verify replenishment occurred
    let boost_availability = &status_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["cards_remaining"], 5);
    assert_eq!(boost_availability["current_cycle"], 2);
    assert_eq!(boost_availability["cycles_completed"], 1);

    let available_cards = boost_availability["available_cards"].as_array().unwrap();
    assert_eq!(available_cards.len(), 5);

    // All cards should be available again
    for i in 0..=4 {
        assert!(available_cards.contains(&json!(i)));
    }
}

#[tokio::test]
async fn test_boost_hand_state_persists_in_database() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Use some boost cards
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 1, &cookies)
        .await;
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 3, &cookies)
        .await;

    // Get status (which reads from database)
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    assert_eq!(200, status_response.status().as_u16());

    let status_data: Value = status_response
        .json()
        .await
        .expect("Failed to parse status");

    // Assert - Verify persisted state is correct
    let boost_availability = &status_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["cards_remaining"], 3);

    let available_cards = boost_availability["available_cards"].as_array().unwrap();
    assert_eq!(available_cards.len(), 3);
    assert!(available_cards.contains(&json!(0)));
    assert!(available_cards.contains(&json!(2)));
    assert!(available_cards.contains(&json!(4)));
    assert!(!available_cards.contains(&json!(1)));
    assert!(!available_cards.contains(&json!(3)));
}

#[tokio::test]
async fn test_boost_usage_history_tracks_all_usages() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Use several boost cards
    let boost_sequence = vec![2, 0, 4];
    for boost_value in &boost_sequence {
        app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, *boost_value, &cookies)
            .await;
    }

    // Get status
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    let status_data: Value = status_response
        .json()
        .await
        .expect("Failed to parse status");

    // Assert - Verify usage history
    let usage_history = status_data["player_data"]["boost_usage_history"]
        .as_array()
        .unwrap();
    assert_eq!(usage_history.len(), 3);

    for (i, boost_value) in boost_sequence.iter().enumerate() {
        assert_eq!(usage_history[i]["boost_value"], *boost_value);
        assert_eq!(usage_history[i]["cycle_number"], 1);
        assert_eq!(usage_history[i]["lap_number"], (i + 1) as u64);
    }
}

#[tokio::test]
async fn test_invalid_boost_value_returns_error() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Try to use invalid boost value (5)
    let lap_response = app
        .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 5, &cookies)
        .await;

    // Assert - Should return 400 with error
    assert_eq!(400, lap_response.status().as_u16());

    let error_data: Value = lap_response
        .json()
        .await
        .expect("Failed to parse error response");
    assert_eq!(error_data["error_code"], "INVALID_BOOST_VALUE");
    assert!(error_data["message"]
        .as_str()
        .unwrap()
        .contains("Invalid boost value"));
}

#[tokio::test]
async fn test_boost_impact_preview_shows_only_available_cards() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Use some cards
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 0, &cookies)
        .await;
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 2, &cookies)
        .await;

    // Get status
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    let status_data: Value = status_response
        .json()
        .await
        .expect("Failed to parse status");

    // Assert - Verify boost impact preview
    let boost_impact_preview = status_data["player_data"]["boost_availability"]
        ["boost_impact_preview"]
        .as_array()
        .unwrap();
    assert_eq!(boost_impact_preview.len(), 5);

    for option in boost_impact_preview {
        let boost_value = option["boost_value"].as_u64().unwrap();
        let is_available = option["is_available"].as_bool().unwrap();

        if boost_value == 0 || boost_value == 2 {
            assert!(!is_available, "Used cards should not be available");
        } else {
            assert!(is_available, "Unused cards should be available");
        }
    }
}

#[tokio::test]
async fn test_multiple_cycles_track_correctly() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Complete first cycle
    for boost_value in 0..=4 {
        app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, boost_value, &cookies)
            .await;
    }

    // Use some cards from second cycle
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 1, &cookies)
        .await;
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 4, &cookies)
        .await;

    // Get status
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    let status_data: Value = status_response
        .json()
        .await
        .expect("Failed to parse status");

    // Assert - Verify cycle tracking
    let boost_availability = &status_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["current_cycle"], 2);
    assert_eq!(boost_availability["cycles_completed"], 1);
    assert_eq!(boost_availability["cards_remaining"], 3);

    // Verify usage history spans both cycles
    let usage_history = status_data["player_data"]["boost_usage_history"]
        .as_array()
        .unwrap();
    assert_eq!(usage_history.len(), 7); // 5 from first cycle + 2 from second

    // First 5 should be cycle 1
    for i in 0..5 {
        assert_eq!(usage_history[i]["cycle_number"], 1);
    }

    // Last 2 should be cycle 2
    for i in 5..7 {
        assert_eq!(usage_history[i]["cycle_number"], 2);
    }
}

#[tokio::test]
async fn test_boost_cycle_summaries_calculated_correctly() {
    // Arrange
    let app = spawn_app().await;
    let (player_uuid, cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let race_uuid = app.create_race(&cookies).await;
    let car_uuid = app.get_player_first_car(&player_uuid, &cookies).await;

    app.register_for_race(&race_uuid, &player_uuid, &car_uuid, &cookies)
        .await;
    app.start_race(&race_uuid, &cookies).await;

    // Act - Complete first cycle with specific sequence
    let boost_sequence = vec![2, 0, 4, 1, 3];
    for boost_value in &boost_sequence {
        app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, *boost_value, &cookies)
            .await;
    }

    // Get status
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    let status_data: Value = status_response
        .json()
        .await
        .expect("Failed to parse status");

    // Assert - Verify cycle summary
    let cycle_summaries = status_data["player_data"]["boost_cycle_summaries"]
        .as_array()
        .unwrap();
    assert_eq!(cycle_summaries.len(), 1);

    let cycle1 = &cycle_summaries[0];
    assert_eq!(cycle1["cycle_number"], 1);

    let cards_used = cycle1["cards_used"].as_array().unwrap();
    assert_eq!(cards_used.len(), 5);

    // Verify average boost
    let average_boost = cycle1["average_boost"].as_f64().unwrap();
    let expected_average = (2.0 + 0.0 + 4.0 + 1.0 + 3.0) / 5.0;
    assert!((average_boost - expected_average).abs() < 0.01);
}

#[tokio::test]
async fn test_concurrent_lap_submissions_handle_boost_cards_correctly() {
    // Arrange
    let app = spawn_app().await;

    // Create two players
    let (player1_uuid, player1_cookies) = app
        .create_test_user("player1@test.com", "Password123", "Player 1")
        .await;
    let (player2_uuid, player2_cookies) = app
        .create_test_user("player2@test.com", "Password123", "Player 2")
        .await;

    let race_uuid = app.create_race(&player1_cookies).await;

    let car1_uuid = app
        .get_player_first_car(&player1_uuid, &player1_cookies)
        .await;
    let car2_uuid = app
        .get_player_first_car(&player2_uuid, &player2_cookies)
        .await;

    // Register both players
    app.register_for_race(&race_uuid, &player1_uuid, &car1_uuid, &player1_cookies)
        .await;
    app.register_for_race(&race_uuid, &player2_uuid, &car2_uuid, &player2_cookies)
        .await;
    app.start_race(&race_uuid, &player1_cookies).await;

    // Act - Both players use boost card 2
    let lap1_response = app
        .apply_lap_action(&race_uuid, &player1_uuid, &car1_uuid, 2, &player1_cookies)
        .await;
    let lap2_response = app
        .apply_lap_action(&race_uuid, &player2_uuid, &car2_uuid, 2, &player2_cookies)
        .await;

    // Assert - Both should succeed (separate boost hands)
    assert_eq!(200, lap1_response.status().as_u16());
    assert_eq!(200, lap2_response.status().as_u16());

    // Verify each player's boost hand is independent
    let status1_response = app
        .get_race_status_detailed(&race_uuid, Some(&player1_uuid), &player1_cookies)
        .await;
    let status1_data: Value = status1_response
        .json()
        .await
        .expect("Failed to parse status");
    let boost1 = &status1_data["player_data"]["boost_availability"];
    assert_eq!(boost1["cards_remaining"], 4);

    let status2_response = app
        .get_race_status_detailed(&race_uuid, Some(&player2_uuid), &player2_cookies)
        .await;
    let status2_data: Value = status2_response
        .json()
        .await
        .expect("Failed to parse status");
    let boost2 = &status2_data["player_data"]["boost_availability"];
    assert_eq!(boost2["cards_remaining"], 4);
}
