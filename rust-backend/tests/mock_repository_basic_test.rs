//! Basic Mock Repository Test
//! This test demonstrates the basic mock repository functionality
//! without requiring complex domain models or external dependencies.

use rust_backend::domain::{Email, HashedPassword, Player, TeamName};
use rust_backend::repositories::{MockPlayerRepository, PlayerRepository};

#[tokio::test]
async fn test_mock_player_repository_basic_operations() {
    // Arrange
    let repo = MockPlayerRepository::new();

    // Create a simple test player using the actual domain constructor
    let email = Email::parse("test@example.com").unwrap();
    let team_name = TeamName::parse("Test Team").unwrap();
    let password_hash =
        HashedPassword::from_hash("$argon2id$v=19$m=15000,t=2,p=1$test$test".to_string());

    let player = Player::new(
        email,
        password_hash,
        team_name,
        Vec::new(), // cars
        Vec::new(), // pilots
    )
    .unwrap();

    // Act & Assert - Create player
    let created_player = repo.create(&player).await.unwrap();
    assert_eq!(created_player.uuid, player.uuid);
    assert_eq!(created_player.email.as_ref(), player.email.as_ref());

    // Act & Assert - Find by email
    let found_by_email = repo.find_by_email("test@example.com").await.unwrap();
    assert!(found_by_email.is_some());
    assert_eq!(found_by_email.unwrap().uuid, player.uuid);

    // Act & Assert - Find by UUID
    let found_by_uuid = repo.find_by_uuid(player.uuid).await.unwrap();
    assert!(found_by_uuid.is_some());
    assert_eq!(found_by_uuid.unwrap().uuid, player.uuid);

    // Act & Assert - Find all players
    let all_players = repo.find_all().await.unwrap();
    assert_eq!(all_players.len(), 1);
    assert_eq!(all_players[0].uuid, player.uuid);
}
