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

    // Helper to perform a pit stop (refills the boost pool, optionally swaps tyre)
    pub async fn apply_pit_action(
        &self,
        race_uuid: &str,
        player_uuid: &str,
        car_uuid: &str,
        new_tyre: Option<&str>,
        cookies: &str,
    ) -> reqwest::Response {
        let mut pit_body = json!({
            "player_uuid": player_uuid,
            "car_uuid": car_uuid,
        });
        if let Some(tyre) = new_tyre {
            pit_body["new_tyre"] = json!(tyre);
        }

        self.client
            .post(format!("{}/api/v1/races/{}/pit", &self.address, race_uuid))
            .header("Cookie", cookies)
            .json(&pit_body)
            .send()
            .await
            .expect("Failed to apply pit action")
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

    // Assert - Verify boost hand is initialized correctly (default Medium pool
    // [2, 2, 3, 3, 4] -> 5 cards; available distinct values are 0, 2, 3, 4).
    let boost_availability = &status_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["cards_remaining"], 5);
    assert_eq!(boost_availability["tyre_type"], "Medium");
    assert_eq!(boost_availability["pit_stops_completed"], 0);

    let available_cards = boost_availability["available_cards"].as_array().unwrap();
    assert_eq!(available_cards.len(), 4);

    // Free move 0 plus the Medium-pool values 2, 3, 4 are available; 1 is not.
    for i in [0, 2, 3, 4] {
        assert!(available_cards.contains(&json!(i)));
    }
    assert!(!available_cards.contains(&json!(1)));
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

    // Act - Use boost card 4 (the single value-4 card in the Medium pool)
    let lap_response = app
        .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 4, &cookies)
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
    // 0, 2, 3 remain (the lone 4 is spent).
    assert_eq!(available_cards.len(), 3);
    assert!(!available_cards.contains(&json!(4)));

    // hand_state now holds remaining counts per value: the value-4 count is 0.
    let hand_state = &boost_availability["hand_state"];
    assert_eq!(hand_state["4"], 0);
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

    // Act - Use boost card 4 (single copy in the Medium pool)
    let lap1_response = app
        .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 4, &cookies)
        .await;
    assert_eq!(200, lap1_response.status().as_u16());

    // Try to use boost card 4 again (now depleted)
    let lap2_response = app
        .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 4, &cookies)
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
    assert!(!available_cards.contains(&json!(4)));
}

#[tokio::test]
async fn test_pit_stop_refills_boost_pool() {
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

    // Act - Spend the entire Medium pool [2, 2, 3, 3, 4]. There is NO
    // auto-replenish: once spent, only the free boost 0 remains.
    for boost_value in [2, 2, 3, 3, 4] {
        let lap_response = app
            .apply_lap_action(&race_uuid, &player_uuid, &car_uuid, boost_value, &cookies)
            .await;
        assert_eq!(
            200,
            lap_response.status().as_u16(),
            "Failed to use boost card {boost_value}"
        );
    }

    // Confirm the pool is empty (only the free 0 remains, no refill yet).
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    let status_data: Value = status_response.json().await.expect("Failed to parse");
    let boost_availability = &status_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["cards_remaining"], 0);
    assert_eq!(boost_availability["pit_stops_completed"], 0);
    let available_cards = boost_availability["available_cards"].as_array().unwrap();
    assert_eq!(available_cards, &[json!(0)]);

    // Pit-stop onto Soft tyres refills the pool ([3, 4, 4] -> 3 cards).
    let pit_response = app
        .apply_pit_action(&race_uuid, &player_uuid, &car_uuid, Some("Soft"), &cookies)
        .await;
    assert_eq!(200, pit_response.status().as_u16());

    // Assert - Verify the pool was refilled from the new tyre.
    let status_response = app
        .get_race_status_detailed(&race_uuid, Some(&player_uuid), &cookies)
        .await;
    let status_data: Value = status_response.json().await.expect("Failed to parse");
    let boost_availability = &status_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["cards_remaining"], 3);
    assert_eq!(boost_availability["tyre_type"], "Soft");
    assert_eq!(boost_availability["pit_stops_completed"], 1);

    let available_cards = boost_availability["available_cards"].as_array().unwrap();
    // Soft pool [3, 4, 4] -> distinct available values 0, 3, 4.
    for i in [0, 3, 4] {
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

    // Act - Use some boost cards. Medium pool is [2, 2, 3, 3, 4]; spend one 2
    // and the lone 4, leaving one 2 and two 3s (3 cards).
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 2, &cookies)
        .await;
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 4, &cookies)
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
    // 0 (free), 2 (one left), 3 (two left) available; 4 spent; 1 never in pool.
    assert_eq!(available_cards.len(), 3);
    assert!(available_cards.contains(&json!(0)));
    assert!(available_cards.contains(&json!(2)));
    assert!(available_cards.contains(&json!(3)));
    assert!(!available_cards.contains(&json!(4)));
    assert!(!available_cards.contains(&json!(1)));
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
        // cycle_number is the pit segment: 0 before any pit stop.
        assert_eq!(usage_history[i]["cycle_number"], 0);
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

    // Act - Spend the lone value-4 card and both value-2 cards. Boost 0 is the
    // free no-op (it never depletes); boost 1 is not in the Medium pool at all.
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 4, &cookies)
        .await;
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 2, &cookies)
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

    // Assert - Verify boost impact preview always covers 0-4.
    let boost_impact_preview = status_data["player_data"]["boost_availability"]
        ["boost_impact_preview"]
        .as_array()
        .unwrap();
    assert_eq!(boost_impact_preview.len(), 5);

    for option in boost_impact_preview {
        let boost_value = option["boost_value"].as_u64().unwrap();
        let is_available = option["is_available"].as_bool().unwrap();

        match boost_value {
            // 0 is always free/available; 3 still has both copies left.
            0 | 3 => assert!(is_available, "boost {boost_value} should be available"),
            // 1 never in Medium pool; 2 and 4 are now depleted.
            1 | 2 | 4 => assert!(!is_available, "boost {boost_value} should be unavailable"),
            _ => unreachable!(),
        }
    }
}

#[tokio::test]
async fn test_pit_segments_track_correctly() {
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

    // Act - Spend the whole Medium pool (pit segment 0).
    for boost_value in [2, 2, 3, 3, 4] {
        app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, boost_value, &cookies)
            .await;
    }

    // Pit-stop (consumes a turn as a free boost-0 lap and refills the pool onto
    // a fresh Medium tyre), then use a couple of cards in pit segment 1.
    app.apply_pit_action(
        &race_uuid,
        &player_uuid,
        &car_uuid,
        Some("Medium"),
        &cookies,
    )
    .await;
    app.apply_lap_action(&race_uuid, &player_uuid, &car_uuid, 2, &cookies)
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

    // Assert - Verify pit/segment tracking. After the refill, the new pool of 5
    // had its free pit-0 lap, then a 2 and a 4 spent -> 3 cards remaining.
    let boost_availability = &status_data["player_data"]["boost_availability"];
    assert_eq!(boost_availability["pit_stops_completed"], 1);
    assert_eq!(boost_availability["cards_remaining"], 3);

    // Verify usage history spans both segments. 5 (segment 0) + 1 free pit-0
    // lap + 2 (segment 1) = 8 records.
    let usage_history = status_data["player_data"]["boost_usage_history"]
        .as_array()
        .unwrap();
    assert_eq!(usage_history.len(), 8);

    // First 5 are in pit segment 0.
    for i in 0..5 {
        assert_eq!(usage_history[i]["cycle_number"], 0);
    }

    // The pit's free boost-0 lap and the two subsequent uses are in segment 1.
    for i in 5..8 {
        assert_eq!(usage_history[i]["cycle_number"], 1);
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

    // Act - Spend the whole Medium pool in a specific order, plus a free 0 move.
    // All uses are in pit segment 0 (no pit stop), so they form one summary.
    let boost_sequence = vec![2, 0, 4, 3, 2, 3];
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

    // Assert - Verify cycle summary (now grouped by pit segment).
    let cycle_summaries = status_data["player_data"]["boost_cycle_summaries"]
        .as_array()
        .unwrap();
    assert_eq!(cycle_summaries.len(), 1);

    let cycle1 = &cycle_summaries[0];
    assert_eq!(cycle1["cycle_number"], 0);

    let cards_used = cycle1["cards_used"].as_array().unwrap();
    assert_eq!(cards_used.len(), 6);

    // Verify average boost
    let average_boost = cycle1["average_boost"].as_f64().unwrap();
    let expected_average = (2.0 + 0.0 + 4.0 + 3.0 + 2.0 + 3.0) / 6.0;
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
