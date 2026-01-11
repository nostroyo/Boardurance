//! Mock Repository Integration Tests
//! These tests demonstrate how to use mock repositories instead of real MongoDB
//! for fast, isolated testing without external dependencies.

use chrono::Utc;
use rust_backend::domain::{Car, Player, Race, RaceStatus, TeamName, WalletAddress};
use rust_backend::repositories::{MockPlayerRepository, MockRaceRepository, MockSessionRepository, PlayerRepository, RaceRepository, SessionRepository};
use rust_backend::services::session::{Session, SessionConfig, SessionManager, SessionMetadata};
use rust_backend::services::{JwtConfig, JwtService};
use rust_backend::test_utils::{TestApp, TestAppState};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// MOCK REPOSITORY UNIT TESTS
// ============================================================================

#[tokio::test]
async fn mock_player_repository_create_and_find_works() {
    // Arrange
    let repo = MockPlayerRepository::new();
    let player = create_test_player("test@example.com", "Test Team");

    // Act - Create player
    let created_player = repo.create(&player).await.unwrap();
    
    // Assert - Player was created
    assert_eq!(created_player.uuid, player.uuid);
    assert_eq!(created_player.email, player.email);
    
    // Act - Find by wallet address
    let found_player = repo.find_by_wallet_address(&player.wallet_address.to_string()).await.unwrap();
    
    // Assert - Player was found
    assert!(found_player.is_some());
    assert_eq!(found_player.unwrap().uuid, player.uuid);
}

#[tokio::test]
async fn mock_player_repository_prevents_duplicate_wallet_addresses() {
    // Arrange
    let repo = MockPlayerRepository::new();
    let player1 = create_test_player("test1@example.com", "Team 1");
    let mut player2 = create_test_player("test2@example.com", "Team 2");
    player2.wallet_address = player1.wallet_address.clone(); // Same wallet address

    // Act - Create first player
    let result1 = repo.create(&player1).await;
    assert!(result1.is_ok());

    // Act - Try to create second player with same wallet
    let result2 = repo.create(&player2).await;

    // Assert - Second creation should fail
    assert!(result2.is_err());
}

#[tokio::test]
async fn mock_race_repository_create_and_join_works() {
    // Arrange
    let race_repo = MockRaceRepository::new();
    let player_repo = MockPlayerRepository::new();
    
    let race = create_test_race();
    let player = create_test_player("racer@example.com", "Racing Team");
    
    // Create player first
    player_repo.create(&player).await.unwrap();

    // Act - Create race
    let created_race = race_repo.create(&race).await.unwrap();
    assert_eq!(created_race.uuid, race.uuid);

    // Act - Join race (this would normally require ValidatedCarData)
    // For now, we'll test the basic repository functionality
    let found_race = race_repo.find_by_uuid(race.uuid).await.unwrap();
    assert!(found_race.is_some());
    assert_eq!(found_race.unwrap().status, RaceStatus::WaitingForPlayers);
}

#[tokio::test]
async fn mock_session_repository_create_and_validate_works() {
    // Arrange
    let repo = MockSessionRepository::new();
    let user_uuid = Uuid::new_v4();
    let token = "test_session_token".to_string();
    let now = Utc::now();
    
    let session = Session {
        id: None,
        user_uuid,
        token: token.clone(),
        created_at: now,
        last_activity: now,
        expires_at: now + chrono::Duration::hours(24),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        is_active: true,
        updated_at: now,
    };

    // Act - Create session
    repo.create(&session).await.unwrap();

    // Act - Find session
    let found_session = repo.find_by_token(&token).await.unwrap();

    // Assert - Session was found and is valid
    assert!(found_session.is_some());
    let found = found_session.unwrap();
    assert_eq!(found.user_uuid, user_uuid);
    assert_eq!(found.token, token);
    assert!(found.is_active);
}

#[tokio::test]
async fn mock_session_repository_expired_sessions_not_returned() {
    // Arrange
    let repo = MockSessionRepository::new();
    let user_uuid = Uuid::new_v4();
    let token = "expired_token".to_string();
    let now = Utc::now();
    
    let expired_session = Session {
        id: None,
        user_uuid,
        token: token.clone(),
        created_at: now - chrono::Duration::hours(25),
        last_activity: now - chrono::Duration::hours(25),
        expires_at: now - chrono::Duration::hours(1), // Expired 1 hour ago
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-agent".to_string()),
        is_active: true,
        updated_at: now - chrono::Duration::hours(25),
    };

    // Act - Create expired session
    repo.create(&expired_session).await.unwrap();

    // Act - Try to find expired session
    let found_session = repo.find_by_token(&token).await.unwrap();

    // Assert - Expired session should not be returned
    assert!(found_session.is_none());
}

// ============================================================================
// SESSION MANAGER WITH MOCK REPOSITORY TESTS
// ============================================================================

#[tokio::test]
async fn session_manager_with_mock_repository_works() {
    // Arrange
    let mock_repo = Arc::new(MockSessionRepository::new());
    let config = SessionConfig::default();
    let session_manager = SessionManager::new(mock_repo.clone(), config);
    
    let user_uuid = Uuid::new_v4();
    let token_id = "test_token_123".to_string();
    let metadata = SessionMetadata {
        ip_address: Some("192.168.1.1".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
    };

    // Act - Create session through manager
    let result = session_manager.create_session(user_uuid, token_id.clone(), metadata).await;
    
    // Assert - Session creation succeeded
    assert!(result.is_ok());

    // Act - Validate session
    let validation_result = session_manager.validate_session(&token_id).await;
    
    // Assert - Session validation succeeded
    assert!(validation_result.is_ok());
    assert!(validation_result.unwrap());

    // Act - Invalidate session
    let invalidation_result = session_manager.invalidate_session(&token_id, "test logout").await;
    
    // Assert - Session invalidation succeeded
    assert!(invalidation_result.is_ok());

    // Act - Try to validate invalidated session
    let validation_after_invalidation = session_manager.validate_session(&token_id).await;
    
    // Assert - Validation should fail for invalidated session
    assert!(validation_after_invalidation.is_err());
}

// ============================================================================
// INTEGRATION TEST WITH TEST APP
// ============================================================================

#[tokio::test]
async fn test_app_with_mocks_health_check_works() {
    // Arrange
    let app = TestApp::new().await;

    // Act - Call health check endpoint
    let response = app.client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert - Health check should return 200
    assert_eq!(200, response.status().as_u16());
    
    let response_body: Value = response.json().await.expect("Failed to parse response");
    assert_eq!(response_body["status"], "ok");
}

#[tokio::test]
async fn test_app_with_preloaded_data_works() {
    // Arrange - Create test data
    let player = create_test_player("preloaded@example.com", "Preloaded Team");
    let race = create_test_race();
    let session = create_test_session();
    
    let app = TestApp::with_test_data(
        vec![player.clone()],
        vec![race.clone()],
        vec![session.clone()],
    ).await;

    // Assert - Test data should be accessible through the app state
    let found_player = app.state.player_repo
        .find_by_wallet_address(&player.wallet_address.to_string())
        .await
        .unwrap();
    assert!(found_player.is_some());
    assert_eq!(found_player.unwrap().email, player.email);

    let found_race = app.state.race_repo
        .find_by_uuid(race.uuid)
        .await
        .unwrap();
    assert!(found_race.is_some());
    assert_eq!(found_race.unwrap().uuid, race.uuid);

    let found_session = app.state.session_repo
        .find_by_token(&session.token)
        .await
        .unwrap();
    assert!(found_session.is_some());
    assert_eq!(found_session.unwrap().user_uuid, session.user_uuid);
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_test_player(email: &str, team_name: &str) -> Player {
    Player {
        uuid: Uuid::new_v4(),
        email: email.to_string(),
        team_name: TeamName::new(team_name.to_string()).unwrap(),
        wallet_address: WalletAddress::new(format!("wallet_{}", Uuid::new_v4())).unwrap(),
        cars: Vec::new(),
        pilots: Vec::new(),
    }
}

fn create_test_race() -> Race {
    use rust_backend::domain::{Track, Sector, SectorType};
    
    Race {
        uuid: Uuid::new_v4(),
        track: Track {
            name: "Test Track".to_string(),
            sectors: vec![
                Sector {
                    sector_number: 1,
                    sector_type: SectorType::Straight,
                    length: 1000,
                    difficulty: 5,
                },
                Sector {
                    sector_number: 2,
                    sector_type: SectorType::Corner,
                    length: 500,
                    difficulty: 8,
                },
                Sector {
                    sector_number: 3,
                    sector_type: SectorType::Straight,
                    length: 800,
                    difficulty: 3,
                },
            ],
        },
        pilots: Vec::new(),
        status: RaceStatus::WaitingForPlayers,
        total_laps: 3,
        current_lap: 0,
        max_participants: 8,
        created_at: Utc::now(),
        started_at: None,
        finished_at: None,
    }
}

fn create_test_session() -> Session {
    let now = Utc::now();
    Session {
        id: None,
        user_uuid: Uuid::new_v4(),
        token: format!("session_token_{}", Uuid::new_v4()),
        created_at: now,
        last_activity: now,
        expires_at: now + chrono::Duration::hours(24),
        ip_address: Some("127.0.0.1".to_string()),
        user_agent: Some("test-user-agent".to_string()),
        is_active: true,
        updated_at: now,
    }
}

// ============================================================================
// PERFORMANCE AND ISOLATION TESTS
// ============================================================================

#[tokio::test]
async fn mock_repositories_are_isolated_between_tests() {
    // This test verifies that each test gets a fresh mock repository
    // and data doesn't leak between tests
    
    // Arrange
    let repo1 = MockPlayerRepository::new();
    let repo2 = MockPlayerRepository::new();
    
    let player = create_test_player("isolation@example.com", "Isolation Team");

    // Act - Add player to first repository
    repo1.create(&player).await.unwrap();

    // Assert - Second repository should not have the player
    let found_in_repo2 = repo2.find_by_wallet_address(&player.wallet_address.to_string()).await.unwrap();
    assert!(found_in_repo2.is_none());

    // Assert - First repository should still have the player
    let found_in_repo1 = repo1.find_by_wallet_address(&player.wallet_address.to_string()).await.unwrap();
    assert!(found_in_repo1.is_some());
}

#[tokio::test]
async fn mock_repositories_perform_fast_operations() {
    // This test verifies that mock operations are fast (no network/disk I/O)
    
    let start_time = std::time::Instant::now();
    
    // Arrange
    let repo = MockPlayerRepository::new();
    
    // Act - Perform many operations
    for i in 0..1000 {
        let player = create_test_player(&format!("user{}@example.com", i), &format!("Team {}", i));
        repo.create(&player).await.unwrap();
        
        let found = repo.find_by_wallet_address(&player.wallet_address.to_string()).await.unwrap();
        assert!(found.is_some());
    }
    
    let elapsed = start_time.elapsed();
    
    // Assert - Operations should complete very quickly (under 100ms for 1000 operations)
    assert!(elapsed.as_millis() < 100, "Mock operations took too long: {:?}", elapsed);
}