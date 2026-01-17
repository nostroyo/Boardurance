//! Mock Repository Integration Tests
//! These tests demonstrate how to use mock repositories instead of real MongoDB
//! for fast, isolated testing without external dependencies.

use rust_backend::domain::{Email, HashedPassword, Player, TeamName};
use rust_backend::repositories::{MockPlayerRepository, PlayerRepository};
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
    assert_eq!(created_player.email.as_ref(), player.email.as_ref());

    // Act - Find by email
    let found_player = repo.find_by_email(player.email.as_ref()).await.unwrap();

    // Assert - Player was found
    assert!(found_player.is_some());
    assert_eq!(found_player.unwrap().uuid, player.uuid);
}

#[tokio::test]
async fn mock_player_repository_find_by_uuid_works() {
    // Arrange
    let repo = MockPlayerRepository::new();
    let player = create_test_player("test@example.com", "Test Team");

    // Act - Create player
    repo.create(&player).await.unwrap();

    // Act - Find by UUID
    let found_player = repo.find_by_uuid(player.uuid).await.unwrap();

    // Assert - Player was found
    assert!(found_player.is_some());
    assert_eq!(found_player.unwrap().uuid, player.uuid);
}

#[tokio::test]
async fn mock_player_repository_find_all_works() {
    // Arrange
    let repo = MockPlayerRepository::new();
    let player1 = create_test_player("test1@example.com", "Team 1");
    let player2 = create_test_player("test2@example.com", "Team 2");

    // Act - Create players
    repo.create(&player1).await.unwrap();
    repo.create(&player2).await.unwrap();

    // Act - Find all players
    let all_players = repo.find_all().await.unwrap();

    // Assert - Both players were found
    assert_eq!(all_players.len(), 2);
    let uuids: Vec<Uuid> = all_players.iter().map(|p| p.uuid).collect();
    assert!(uuids.contains(&player1.uuid));
    assert!(uuids.contains(&player2.uuid));
}

#[tokio::test]
async fn mock_player_repository_prevents_duplicate_emails() {
    // Arrange
    let repo = MockPlayerRepository::new();
    let player1 = create_test_player("test@example.com", "Team 1");
    let player2 = create_test_player("test@example.com", "Team 2"); // Same email

    // Act - Create first player
    let result1 = repo.create(&player1).await;
    assert!(result1.is_ok());

    // Act - Try to create second player with same email
    let result2 = repo.create(&player2).await;

    // Assert - Second creation should fail
    assert!(result2.is_err());
}

#[tokio::test]
async fn mock_player_repository_update_works() {
    // Arrange
    let repo = MockPlayerRepository::new();
    let player = create_test_player("test@example.com", "Original Team");

    // Act - Create player
    repo.create(&player).await.unwrap();

    // Act - Update player team name
    let new_team_name = TeamName::parse("Updated Team").unwrap();
    let updated_player = repo
        .update_team_name_by_uuid(player.uuid, new_team_name)
        .await
        .unwrap();

    // Assert - Player was updated
    assert!(updated_player.is_some());
    assert_eq!(updated_player.unwrap().team_name.as_ref(), "Updated Team");
}

#[tokio::test]
async fn mock_player_repository_delete_works() {
    // Arrange
    let repo = MockPlayerRepository::new();
    let player = create_test_player("test@example.com", "Test Team");

    // Act - Create player
    repo.create(&player).await.unwrap();

    // Verify player exists
    let found_before = repo.find_by_uuid(player.uuid).await.unwrap();
    assert!(found_before.is_some());

    // Act - Delete player
    let delete_result = repo.delete_by_uuid(player.uuid).await;
    assert!(delete_result.is_ok());
    assert!(delete_result.unwrap()); // Should return true for successful deletion

    // Assert - Player no longer exists
    let found_after = repo.find_by_uuid(player.uuid).await.unwrap();
    assert!(found_after.is_none());
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_test_player(email: &str, team_name: &str) -> Player {
    let email = Email::parse(email).unwrap();
    let team_name = TeamName::parse(team_name).unwrap();
    let password_hash = HashedPassword::from_hash("test_hash".to_string());

    Player::new(
        email,
        password_hash,
        team_name,
        Vec::new(), // cars
        Vec::new(), // pilots
    )
    .unwrap()
}
