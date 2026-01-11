//! Simple Mock Repository Test
//! This test demonstrates the basic mock repository functionality
//! without requiring complex domain models or external dependencies.

use rust_backend::repositories::{MockPlayerRepository, PlayerRepository};
use rust_backend::domain::{Player, TeamName, WalletAddress};
use uuid::Uuid;

#[tokio::test]
async fn test_mock_player_repository_basic_operations() {
    // Arrange
    let repo = MockPlayerRepository::new();
    
    // Create a simple test player
    let player = Player {
        uuid: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        team_name: TeamName::new("Test Team".to_string()).unwrap(),
        wallet_address: WalletAddress::new("test_wallet_123".to_string()).unwrap(),
        cars: Vec::new(),
        pilots: Vec::new(),
    };

    // Act & Assert - Create player
    let created_player = repo.create(&player).await.unwrap();
    assert_eq!(created_player.uuid, player.uuid);
    assert_eq!(created_player.email, player.email);

    // Act & Assert - Find by wallet address
    let found_player = repo.find_by_wallet_address(&player.wallet_address.to_string()).await.unwrap();
    assert!(found_player.is_some());
    assert_eq!(found_player.unwrap().uuid, player.uuid);

    // Act & Assert - Find by email
    let found_by_email = repo.find_by_email(&player.email).await.unwrap();
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
    let wallet_address = "duplicate_wallet".to_string();
    
    let player1 = Player {
        uuid: Uuid::new_v4(),
        email: "player1@example.com".to_string(),
        team_name: TeamName::new("Team 1".to_string()).unwrap(),
        wallet_address: WalletAddress::new(wallet_address.clone()).unwrap(),
        cars: Vec::new(),
        pilots: Vec::new(),
    };
    
    let player2 = Player {
        uuid: Uuid::new_v4(),
        email: "player2@example.com".to_string(),
        team_name: TeamName::new("Team 2".to_string()).unwrap(),
        wallet_address: WalletAddress::new(wallet_address).unwrap(),
        cars: Vec::new(),
        pilots: Vec::new(),
    };

    // Act - Create first player (should succeed)
    let result1 = repo.create(&player1).await;
    assert!(result1.is_ok());

    // Act - Try to create second player with same wallet (should fail)
    let result2 = repo.create(&player2).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_mock_player_repository_isolation() {
    // This test verifies that different repository instances are isolated
    
    // Arrange
    let repo1 = MockPlayerRepository::new();
    let repo2 = MockPlayerRepository::new();
    
    let player = Player {
        uuid: Uuid::new_v4(),
        email: "isolation@example.com".to_string(),
        team_name: TeamName::new("Isolation Team".to_string()).unwrap(),
        wallet_address: WalletAddress::new("isolation_wallet".to_string()).unwrap(),
        cars: Vec::new(),
        pilots: Vec::new(),
    };

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
async fn test_mock_player_repository_performance() {
    // This test verifies that mock operations are fast
    
    let start_time = std::time::Instant::now();
    
    // Arrange
    let repo = MockPlayerRepository::new();
    
    // Act - Perform many operations
    for i in 0..100 {
        let player = Player {
            uuid: Uuid::new_v4(),
            email: format!("user{}@example.com", i),
            team_name: TeamName::new(format!("Team {}", i)).unwrap(),
            wallet_address: WalletAddress::new(format!("wallet_{}", i)).unwrap(),
            cars: Vec::new(),
            pilots: Vec::new(),
        };
        
        repo.create(&player).await.unwrap();
        
        let found = repo.find_by_wallet_address(&player.wallet_address.to_string()).await.unwrap();
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
        Player {
            uuid: Uuid::new_v4(),
            email: "preloaded1@example.com".to_string(),
            team_name: TeamName::new("Preloaded Team 1".to_string()).unwrap(),
            wallet_address: WalletAddress::new("preloaded_wallet_1".to_string()).unwrap(),
            cars: Vec::new(),
            pilots: Vec::new(),
        },
        Player {
            uuid: Uuid::new_v4(),
            email: "preloaded2@example.com".to_string(),
            team_name: TeamName::new("Preloaded Team 2".to_string()).unwrap(),
            wallet_address: WalletAddress::new("preloaded_wallet_2".to_string()).unwrap(),
            cars: Vec::new(),
            pilots: Vec::new(),
        },
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