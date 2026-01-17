//! Simple Mock Repository Test
//! This test demonstrates the basic mock repository functionality
//! without requiring complex domain models or external dependencies.

use rust_backend::repositories::{MockPlayerRepository, PlayerRepository};
use rust_backend::domain::{Player, TeamName, Email, HashedPassword, UserRole};
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_mock_player_repository_basic_operations() {
    // Arrange
    let repo = MockPlayerRepository::new();
    
    // Create a simple test player using the actual domain constructor
    let email = Email::parse("test@example.com").unwrap();
    let team_name = TeamName::parse("Test Team").unwrap();
    let password_hash = HashedPassword::parse("$argon2id$v=19$m=15000,t=2,p=1$test$test").unwrap();
    
    let player = Player::new(
        email,
        password_hash,
        team_name,
        Vec::new(), // cars
        Vec::new(), // pilots
    ).unwrap();

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

#[tokio::test]
async fn test_mock_player_repository_prevents_duplicates() {
    // Arrange
    let repo = MockPlayerRepository::new();
    let email = "duplicate@example.com";
    
    let player1 = Player::new(
        Email::parse(email).unwrap(),
        HashedPassword::parse("$argon2id$v=19$m=15000,t=2,p=1$test1$test1").unwrap(),
        TeamName::parse("Team 1").unwrap(),
        Vec::new(),
        Vec::new(),
    ).unwrap();
    
    let player2 = Player::new(
        Email::parse(email).unwrap(),
        HashedPassword::parse("$argon2id$v=19$m=15000,t=2,p=1$test2$test2").unwrap(),
        TeamName::parse("Team 2").unwrap(),
        Vec::new(),
        Vec::new(),
    ).unwrap();

    // Act - Create first player (should succeed)
    let result1 = repo.create(&player1).await;
    assert!(result1.is_ok());

    // Act - Try to create second player with same email (should fail)
    let result2 = repo.create(&player2).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_mock_player_repository_isolation() {
    // This test verifies that different repository instances are isolated
    
    // Arrange
    let repo1 = MockPlayerRepository::new();
    let repo2 = MockPlayerRepository::new();
    
    let player = Player::new(
        Email::parse("isolation@example.com").unwrap(),
        HashedPassword::parse("$argon2id$v=19$m=15000,t=2,p=1$test$test").unwrap(),
        TeamName::parse("Isolation Team").unwrap(),
        Vec::new(),
        Vec::new(),
    ).unwrap();

    // Act - Add player to first repository
    repo1.create(&player).await.unwrap();

    // Assert - Second repository should not have the player
    let found_in_repo2 = repo2.find_by_email("isolation@example.com").await.unwrap();
    assert!(found_in_repo2.is_none());

    // Assert - First repository should still have the player
    let found_in_repo1 = repo1.find_by_email("isolation@example.com").await.unwrap();
    assert!(found_in_repo1.is_some());
}

#[tokio::test]
async fn test_mock_player_repository_performance() {
    // This test verifies that mock operations are fast
    
    let start_time = std::time::Instant::now();
    
    // Arrange
    let repo = MockPlayerRepository::new();
    
    // Act - Perform many operations
    for i in 0..100 {
        let player = Player::new(
            Email::parse(&format!("user{}@example.com", i)).unwrap(),
            HashedPassword::parse("$argon2id$v=19$m=15000,t=2,p=1$test$test").unwrap(),
            TeamName::parse(&format!("Team {}", i)).unwrap(),
            Vec::new(),
            Vec::new(),
        ).unwrap();
        
        repo.create(&player).await.unwrap();
        
        let found = repo.find_by_email(&format!("user{}@example.com", i)).await.unwrap();
        assert!(found.is_some());
    }
    
    let elapsed = start_time.elapsed();
    
    // Assert - Operations should complete very quickly (under 50ms for 100 operations)
    assert!(elapsed.as_millis() < 50, "Mock operations took too long: {:?}", elapsed);
}

#[tokio::test]
async fn test_mock_player_repository_with_preloaded_data() {
    // Arrange - Create test data
    let players = vec![
        Player::new(
            Email::parse("preloaded1@example.com").unwrap(),
            HashedPassword::parse("$argon2id$v=19$m=15000,t=2,p=1$test1$test1").unwrap(),
            TeamName::parse("Preloaded Team 1").unwrap(),
            Vec::new(),
            Vec::new(),
        ).unwrap(),
        Player::new(
            Email::parse("preloaded2@example.com").unwrap(),
            HashedPassword::parse("$argon2id$v=19$m=15000,t=2,p=1$test2$test2").unwrap(),
            TeamName::parse("Preloaded Team 2").unwrap(),
            Vec::new(),
            Vec::new(),
        ).unwrap(),
    ];
    
    // Act - Create repository with preloaded data
    let repo = MockPlayerRepository::with_players(players.clone());
    
    // Assert - All preloaded players should be accessible
    let all_players = repo.find_all().await.unwrap();
    assert_eq!(all_players.len(), 2);
    
    let found_player1 = repo.find_by_email("preloaded1@example.com").await.unwrap();
    assert!(found_player1.is_some());
    assert_eq!(found_player1.unwrap().uuid, players[0].uuid);
    
    let found_player2 = repo.find_by_email("preloaded2@example.com").await.unwrap();
    assert!(found_player2.is_some());
    assert_eq!(found_player2.unwrap().uuid, players[1].uuid);
}