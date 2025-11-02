use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use mongodb::{bson::{doc, DateTime as BsonDateTime}, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{
    Player, WalletAddress, TeamName, Email, Car, CarName,
    Pilot, PilotName, PilotClass, PilotRarity, PilotSkills,
    Engine, EngineName, Body, BodyName, ComponentRarity,
    Password,
};
use crate::middleware::{AuthMiddleware, RequireRole};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePlayerRequest {
    pub email: String,
    pub team_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnectWalletRequest {
    pub wallet_address: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTeamNameRequest {
    pub team_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePlayerConfigurationRequest {
    pub team_name: String,
    pub cars: Vec<Car>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddCarRequest {
    pub name: String,
    pub nft_mint_address: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddPilotRequest {
    pub name: String,
    pub pilot_class: PilotClass,
    pub rarity: PilotRarity,
    pub skills: PilotSkillsRequest,
    pub nft_mint_address: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PilotSkillsRequest {
    pub reaction_time: u8,
    pub precision: u8,
    pub focus: u8,
    pub stamina: u8,
}

#[derive(Serialize, ToSchema)]
pub struct PlayerResponse {
    pub player: Player,
    pub message: String,
}

pub fn routes() -> Router<Database> {
    Router::new()
        // Public routes (no authentication required)
        .route("/players", post(create_player))  // User registration
        
        // Protected routes - These should be protected with AuthMiddleware + RequireOwnership
        // TODO: Apply middleware layers in startup.rs:
        // 1. AuthMiddleware to validate JWT tokens and extract UserContext
        // 2. RequireOwnership::player("player_uuid") to validate ownership
        // Routes that require player ownership or admin role:
        .route("/players/:player_uuid", get(get_player_by_uuid))
        .route("/players/:player_uuid", put(update_player_team_name))
        .route("/players/:player_uuid/configuration", put(update_player_configuration))
        .route("/players/:player_uuid", delete(delete_player))
        .route("/players/:player_uuid/wallet", post(connect_wallet))
        .route("/players/:player_uuid/wallet", delete(disconnect_wallet))
        .route("/players/:player_uuid/cars", post(add_car_to_player))
        .route("/players/:player_uuid/cars/:car_uuid", delete(remove_car_from_player))
        .route("/players/:player_uuid/pilots", post(add_pilot_to_player))
        .route("/players/:player_uuid/pilots/:pilot_uuid", delete(remove_pilot_from_player))
}

/// Admin-only routes that require authentication and admin role
pub fn admin_routes() -> Router<crate::app_state::AppState> {
    use crate::app_state::AppState;
    
    Router::new()
        // Admin-only routes - Protected with AuthMiddleware + RequireRole::admin
        // SECURITY: These routes expose sensitive user information
        .route("/players", get(get_all_players_admin))                              // Admin: view all players
        .route("/players/by-wallet/:wallet_address", get(get_player_by_wallet_admin)) // Admin: lookup by wallet
        .route("/players/by-email/:email", get(get_player_by_email_admin))           // Admin: lookup by email
}

/// Create a new player with starter assets
#[utoipa::path(
    post,
    path = "/api/v1/players",
    request_body = CreatePlayerRequest,
    responses(
        (status = 201, description = "Player created successfully with assets", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 409, description = "Email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(
    name = "Creating a new player with assets",
    skip(database, payload),
    fields(
        email = %payload.email,
        team_name = %payload.team_name
    )
)]
pub async fn create_player(
    State(database): State<Database>,
    Json(payload): Json<CreatePlayerRequest>,
) -> Result<(StatusCode, Json<PlayerResponse>), StatusCode> {
    let email = match Email::parse(&payload.email) {
        Ok(email) => {
            // Check if email is already registered
            if let Ok(existing) = get_player_by_email_address(&database, email.as_ref()).await {
                if existing.is_some() {
                    tracing::warn!("Email {} is already registered", email.as_ref());
                    return Err(StatusCode::CONFLICT);
                }
            }
            email
        }
        Err(e) => {
            tracing::warn!("Invalid email address: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let team_name = match TeamName::parse(&payload.team_name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid team name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Create 2 empty cars
    let car1 = match Car::new(
        CarName::parse("Car 1").unwrap(),
        None,
    ) {
        Ok(car) => car,
        Err(e) => {
            tracing::error!("Failed to create car 1: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let car2 = match Car::new(
        CarName::parse("Car 2").unwrap(),
        None,
    ) {
        Ok(car) => car,
        Err(e) => {
            tracing::error!("Failed to create car 2: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create 2 basic engines
    let engine1 = match Engine::new(
        EngineName::parse("Basic Engine 1").unwrap(),
        ComponentRarity::Common,
        30, // straight_value
        25, // curve_value
        None,
    ) {
        Ok(engine) => engine,
        Err(e) => {
            tracing::error!("Failed to create engine 1: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let engine2 = match Engine::new(
        EngineName::parse("Basic Engine 2").unwrap(),
        ComponentRarity::Common,
        25, // straight_value
        30, // curve_value
        None,
    ) {
        Ok(engine) => engine,
        Err(e) => {
            tracing::error!("Failed to create engine 2: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create 2 basic bodies
    let body1 = match Body::new(
        BodyName::parse("Basic Body 1").unwrap(),
        ComponentRarity::Common,
        20, // straight_value
        30, // curve_value
        None,
    ) {
        Ok(body) => body,
        Err(e) => {
            tracing::error!("Failed to create body 1: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let body2 = match Body::new(
        BodyName::parse("Basic Body 2").unwrap(),
        ComponentRarity::Common,
        30, // straight_value
        20, // curve_value
        None,
    ) {
        Ok(body) => body,
        Err(e) => {
            tracing::error!("Failed to create body 2: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Create a default password hash for testing purposes
    // In production, users should register through the auth endpoints
    let default_password = Password::new("TempPassword123".to_string())
        .expect("Default password should be valid");
    let password_hash = default_password.hash()
        .expect("Password hashing should work");

    // Create player with assets
    let player = match Player::new_with_assets(
        email,
        password_hash,
        team_name,
        vec![car1, car2],
        vec![], // Empty pilots as requested
        vec![engine1, engine2],
        vec![body1, body2],
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to create player: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match insert_player(&database, &player).await {
        Ok(created_player) => {
            tracing::info!("Player created successfully with assets. UUID: {}", created_player.uuid);
            Ok((
                StatusCode::CREATED,
                Json(PlayerResponse {
                    player: created_player,
                    message: "Player created successfully with starter assets".to_string(),
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to create player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get all players
#[utoipa::path(
    get,
    path = "/api/v1/players",
    responses(
        (status = 200, description = "List of all players", body = Vec<Player>),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Fetching all players", skip(database))]
pub async fn get_all_players(State(database): State<Database>) -> Result<Json<Vec<Player>>, StatusCode> {
    match get_all_players_from_db(&database).await {
        Ok(players) => {
            tracing::info!("Successfully fetched {} players", players.len());
            Ok(Json(players))
        }
        Err(e) => {
            tracing::error!("Failed to fetch players: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get player by UUID
#[utoipa::path(
    get,
    path = "/api/v1/players/{player_uuid}",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID")
    ),
    responses(
        (status = 200, description = "Player found", body = Player),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Fetching player by UUID", skip(database))]
pub async fn get_player_by_uuid(
    State(database): State<Database>,
    Path(player_uuid_str): Path<String>,
) -> Result<Json<Player>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match get_player_by_uuid_from_db(&database, player_uuid).await {
        Ok(Some(player)) => {
            tracing::info!("Player found for UUID: {}", player_uuid);
            Ok(Json(player))
        }
        Ok(None) => {
            tracing::warn!("Player not found for UUID: {}", player_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to fetch player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get player by wallet address
#[utoipa::path(
    get,
    path = "/api/v1/players/by-wallet/{wallet_address}",
    params(
        ("wallet_address" = String, Path, description = "Player's wallet address")
    ),
    responses(
        (status = 200, description = "Player found", body = Player),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Fetching player by wallet address", skip(database))]
pub async fn get_player_by_wallet(
    State(database): State<Database>,
    Path(wallet_address): Path<String>,
) -> Result<Json<Player>, StatusCode> {
    match get_player_by_wallet_address(&database, &wallet_address).await {
        Ok(Some(player)) => {
            tracing::info!("Player found for wallet address: {}", wallet_address);
            Ok(Json(player))
        }
        Ok(None) => {
            tracing::warn!("Player not found for wallet address: {}", wallet_address);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to fetch player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get player by email address
#[utoipa::path(
    get,
    path = "/api/v1/players/by-email/{email}",
    params(
        ("email" = String, Path, description = "Player's email address")
    ),
    responses(
        (status = 200, description = "Player found", body = Player),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Fetching player by email address", skip(database))]
pub async fn get_player_by_email(
    State(database): State<Database>,
    Path(email): Path<String>,
) -> Result<Json<Player>, StatusCode> {
    match get_player_by_email_address(&database, &email).await {
        Ok(Some(player)) => {
            tracing::info!("Player found for email address: {}", email);
            Ok(Json(player))
        }
        Ok(None) => {
            tracing::warn!("Player not found for email address: {}", email);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to fetch player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Connect wallet to player
#[utoipa::path(
    post,
    path = "/api/v1/players/{player_uuid}/wallet",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID")
    ),
    request_body = ConnectWalletRequest,
    responses(
        (status = 200, description = "Wallet connected successfully", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Player not found"),
        (status = 409, description = "Wallet already connected"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Connecting wallet to player", skip(database, payload))]
pub async fn connect_wallet(
    State(database): State<Database>,
    Path(player_uuid_str): Path<String>,
    Json(payload): Json<ConnectWalletRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let wallet_address = match WalletAddress::parse(&payload.wallet_address) {
        Ok(addr) => addr,
        Err(e) => {
            tracing::warn!("Invalid wallet address: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Check if wallet is already connected to another player
    if let Ok(existing) = get_player_by_wallet_address(&database, wallet_address.as_ref()).await {
        if existing.is_some() {
            tracing::warn!("Wallet address {} is already connected to another player", wallet_address.as_ref());
            return Err(StatusCode::CONFLICT);
        }
    }

    match connect_wallet_to_player(&database, player_uuid, wallet_address).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Wallet connected successfully to player: {}", player_uuid);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Wallet connected successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for UUID: {}", player_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to connect wallet: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Disconnect wallet from player
#[utoipa::path(
    delete,
    path = "/api/v1/players/{player_uuid}/wallet",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID")
    ),
    responses(
        (status = 200, description = "Wallet disconnected successfully", body = PlayerResponse),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Disconnecting wallet from player", skip(database))]
pub async fn disconnect_wallet(
    State(database): State<Database>,
    Path(player_uuid_str): Path<String>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match disconnect_wallet_from_player(&database, player_uuid).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Wallet disconnected successfully from player: {}", player_uuid);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Wallet disconnected successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for UUID: {}", player_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to disconnect wallet: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update player configuration (cars and inventory)
#[utoipa::path(
    put,
    path = "/api/v1/players/{player_uuid}/configuration",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID")
    ),
    request_body = UpdatePlayerConfigurationRequest,
    responses(
        (status = 200, description = "Configuration updated successfully", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Updating player configuration", skip(database, payload))]
pub async fn update_player_configuration(
    State(database): State<Database>,
    Path(player_uuid_str): Path<String>,
    Json(payload): Json<UpdatePlayerConfigurationRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let new_team_name = match TeamName::parse(&payload.team_name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid team name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match update_player_configuration_by_uuid(&database, player_uuid, new_team_name, payload.cars).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Configuration updated successfully for player: {}", player_uuid);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Configuration updated successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for UUID: {}", player_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to update configuration: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update player team name
#[utoipa::path(
    put,
    path = "/api/v1/players/{player_uuid}",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID")
    ),
    request_body = UpdateTeamNameRequest,
    responses(
        (status = 200, description = "Team name updated successfully", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Updating player team name", skip(database, payload))]
pub async fn update_player_team_name(
    State(database): State<Database>,
    Path(player_uuid_str): Path<String>,
    Json(payload): Json<UpdateTeamNameRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let new_team_name = match TeamName::parse(&payload.team_name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid team name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match update_player_team_name_by_uuid(&database, player_uuid, new_team_name).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Team name updated successfully for player: {}", player_uuid);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Team name updated successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for UUID: {}", player_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to update team name: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete player
#[utoipa::path(
    delete,
    path = "/api/v1/players/{player_uuid}",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID")
    ),
    responses(
        (status = 200, description = "Player deleted successfully"),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Deleting player", skip(database))]
pub async fn delete_player(
    State(database): State<Database>,
    Path(player_uuid_str): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match delete_player_by_uuid(&database, player_uuid).await {
        Ok(true) => {
            tracing::info!("Player deleted successfully: {}", player_uuid);
            Ok(StatusCode::OK)
        }
        Ok(false) => {
            tracing::warn!("Player not found for deletion: {}", player_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to delete player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Add car to player
#[utoipa::path(
    post,
    path = "/api/v1/players/{player_uuid}/cars",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID")
    ),
    request_body = AddCarRequest,
    responses(
        (status = 200, description = "Car added successfully", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Adding car to player", skip(database, payload))]
pub async fn add_car_to_player(
    State(database): State<Database>,
    Path(player_uuid_str): Path<String>,
    Json(payload): Json<AddCarRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    let car_name = match CarName::parse(&payload.name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid car name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let car = match Car::new(
        car_name,
        payload.nft_mint_address,
    ) {
        Ok(car) => car,
        Err(e) => {
            tracing::warn!("Failed to create car: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match add_car_to_player_by_uuid(&database, player_uuid, car).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Car added successfully to player: {}", player_uuid);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Car added successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for UUID: {}", player_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to add car to player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Remove car from player
#[utoipa::path(
    delete,
    path = "/api/v1/players/{player_uuid}/cars/{car_uuid}",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID"),
        ("car_uuid" = String, Path, description = "Car UUID to remove")
    ),
    responses(
        (status = 200, description = "Car removed successfully", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Player or car not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Removing car from player", skip(database))]
pub async fn remove_car_from_player(
    State(database): State<Database>,
    Path((player_uuid_str, car_uuid_str)): Path<(String, String)>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let car_uuid = match Uuid::parse_str(&car_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid car UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match remove_car_from_player_by_uuid(&database, player_uuid, car_uuid).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Car removed successfully from player: {}", player_uuid);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Car removed successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player or car not found for removal");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to remove car from player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Add pilot to player
#[utoipa::path(
    post,
    path = "/api/v1/players/{player_uuid}/pilots",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID")
    ),
    request_body = AddPilotRequest,
    responses(
        (status = 200, description = "Pilot added successfully", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Player not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Adding pilot to player", skip(database, payload))]
pub async fn add_pilot_to_player(
    State(database): State<Database>,
    Path(player_uuid_str): Path<String>,
    Json(payload): Json<AddPilotRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    let pilot_name = match PilotName::parse(&payload.name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid pilot name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let pilot_skills = match PilotSkills::new(
        payload.skills.reaction_time,
        payload.skills.precision,
        payload.skills.focus,
        payload.skills.stamina,
    ) {
        Ok(skills) => skills,
        Err(e) => {
            tracing::warn!("Invalid pilot skills: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Create performance based on skills (temporary implementation)
    let pilot_performance = match crate::domain::PilotPerformance::new(
        u8::midpoint(pilot_skills.reaction_time, pilot_skills.focus),    // straight value
        u8::midpoint(pilot_skills.precision, pilot_skills.stamina),     // curve value
    ) {
        Ok(performance) => performance,
        Err(e) => {
            tracing::warn!("Failed to create pilot performance: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let pilot = match Pilot::new(
        pilot_name,
        payload.pilot_class,
        payload.rarity,
        pilot_skills,
        pilot_performance,
        payload.nft_mint_address,
    ) {
        Ok(pilot) => pilot,
        Err(e) => {
            tracing::warn!("Failed to create pilot: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match add_pilot_to_player_by_uuid(&database, player_uuid, pilot).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Pilot added successfully to player: {}", player_uuid);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Pilot added successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for UUID: {}", player_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to add pilot to player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Remove pilot from player
#[utoipa::path(
    delete,
    path = "/api/v1/players/{player_uuid}/pilots/{pilot_uuid}",
    params(
        ("player_uuid" = String, Path, description = "Player's UUID"),
        ("pilot_uuid" = String, Path, description = "Pilot UUID to remove")
    ),
    responses(
        (status = 200, description = "Pilot removed successfully", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Player or pilot not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(name = "Removing pilot from player", skip(database))]
pub async fn remove_pilot_from_player(
    State(database): State<Database>,
    Path((player_uuid_str, pilot_uuid_str)): Path<(String, String)>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = match Uuid::parse_str(&player_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let pilot_uuid = match Uuid::parse_str(&pilot_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid pilot UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match remove_pilot_from_player_by_uuid(&database, player_uuid, pilot_uuid).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Pilot removed successfully from player: {}", player_uuid);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Pilot removed successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player or pilot not found for removal");
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to remove pilot from player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Database operations
#[tracing::instrument(name = "Saving new player in the database", skip(database, player))]
pub async fn insert_player(
    database: &Database,
    player: &Player,
) -> Result<Player, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let result = collection.insert_one(player, None).await?;
    
    let mut created_player = player.clone();
    created_player.id = Some(result.inserted_id.as_object_id().unwrap());
    Ok(created_player)
}

#[tracing::instrument(name = "Getting all players from the database", skip(database))]
pub async fn get_all_players_from_db(database: &Database) -> Result<Vec<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let mut cursor = collection.find(None, None).await?;
    
    let mut players = Vec::new();
    while cursor.advance().await? {
        let player = cursor.deserialize_current()?;
        players.push(player);
    }
    
    Ok(players)
}



#[tracing::instrument(name = "Getting player by wallet address from the database", skip(database))]
pub async fn get_player_by_wallet_address(
    database: &Database,
    wallet_address: &str,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "wallet_address": wallet_address };
    collection.find_one(filter, None).await
}

#[tracing::instrument(name = "Getting player by email address from the database", skip(database))]
pub async fn get_player_by_email_address(
    database: &Database,
    email: &str,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "email": email };
    collection.find_one(filter, None).await
}

#[tracing::instrument(name = "Updating player team name in the database", skip(database, new_team_name))]
pub async fn update_player_team_name_in_db(
    database: &Database,
    wallet_address: &str,
    new_team_name: TeamName,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "wallet_address": wallet_address };
    let update = doc! { 
        "$set": { 
            "team_name": new_team_name.as_ref(),
            "updated_at": BsonDateTime::now()
        } 
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Deleting player from the database", skip(database))]
pub async fn delete_player_from_db(
    database: &Database,
    wallet_address: &str,
) -> Result<bool, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "wallet_address": wallet_address };
    let result = collection.delete_one(filter, None).await?;
    Ok(result.deleted_count > 0)
}

#[tracing::instrument(name = "Adding car to player in the database", skip(database, car))]
pub async fn add_car_to_player_in_db(
    database: &Database,
    wallet_address: &str,
    car: Car,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "wallet_address": wallet_address };
    let update = doc! { 
        "$push": { "cars": mongodb::bson::to_bson(&car).unwrap() },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Removing car from player in the database", skip(database))]
pub async fn remove_car_from_player_in_db(
    database: &Database,
    wallet_address: &str,
    car_uuid: Uuid,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "wallet_address": wallet_address };
    let update = doc! { 
        "$pull": { "cars": { "uuid": car_uuid.to_string() } },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Adding pilot to player in the database", skip(database, pilot))]
pub async fn add_pilot_to_player_in_db(
    database: &Database,
    wallet_address: &str,
    pilot: Pilot,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "wallet_address": wallet_address };
    let update = doc! { 
        "$push": { "pilots": mongodb::bson::to_bson(&pilot).unwrap() },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Removing pilot from player in the database", skip(database))]
pub async fn remove_pilot_from_player_in_db(
    database: &Database,
    wallet_address: &str,
    pilot_uuid: Uuid,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "wallet_address": wallet_address };
    let update = doc! { 
        "$pull": { "pilots": { "uuid": pilot_uuid.to_string() } },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}
#[
tracing::instrument(name = "Getting player by UUID from the database", skip(database))]
pub async fn get_player_by_uuid_from_db(
    database: &Database,
    player_uuid: Uuid,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    collection.find_one(filter, None).await
}

#[tracing::instrument(name = "Connecting wallet to player in the database", skip(database, wallet_address))]
pub async fn connect_wallet_to_player(
    database: &Database,
    player_uuid: Uuid,
    wallet_address: WalletAddress,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let update = doc! { 
        "$set": { 
            "wallet_address": wallet_address.as_ref(),
            "updated_at": BsonDateTime::now()
        } 
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Disconnecting wallet from player in the database", skip(database))]
pub async fn disconnect_wallet_from_player(
    database: &Database,
    player_uuid: Uuid,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let update = doc! { 
        "$unset": { "wallet_address": "" },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Updating player team name by UUID in the database", skip(database, new_team_name))]
pub async fn update_player_team_name_by_uuid(
    database: &Database,
    player_uuid: Uuid,
    new_team_name: TeamName,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let update = doc! { 
        "$set": { 
            "team_name": new_team_name.as_ref(),
            "updated_at": BsonDateTime::now()
        } 
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Deleting player by UUID from the database", skip(database))]
pub async fn delete_player_by_uuid(
    database: &Database,
    player_uuid: Uuid,
) -> Result<bool, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let result = collection.delete_one(filter, None).await?;
    Ok(result.deleted_count > 0)
}

#[tracing::instrument(name = "Adding car to player by UUID in the database", skip(database, car))]
pub async fn add_car_to_player_by_uuid(
    database: &Database,
    player_uuid: Uuid,
    car: Car,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let update = doc! { 
        "$push": { "cars": mongodb::bson::to_bson(&car).unwrap() },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Removing car from player by UUID in the database", skip(database))]
pub async fn remove_car_from_player_by_uuid(
    database: &Database,
    player_uuid: Uuid,
    car_uuid: Uuid,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let update = doc! { 
        "$pull": { "cars": { "uuid": car_uuid.to_string() } },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Adding pilot to player by UUID in the database", skip(database, pilot))]
pub async fn add_pilot_to_player_by_uuid(
    database: &Database,
    player_uuid: Uuid,
    pilot: Pilot,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let update = doc! { 
        "$push": { "pilots": mongodb::bson::to_bson(&pilot).unwrap() },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Removing pilot from player by UUID in the database", skip(database))]
pub async fn remove_pilot_from_player_by_uuid(
    database: &Database,
    player_uuid: Uuid,
    pilot_uuid: Uuid,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let update = doc! { 
        "$pull": { "pilots": { "uuid": pilot_uuid.to_string() } },
        "$set": { "updated_at": BsonDateTime::now() }
    };
    
    collection.find_one_and_update(filter, update, None).await
}
#[tracing::instrument(name = "Updating player configuration by UUID in the database", skip(database, new_team_name, cars))]
pub async fn update_player_configuration_by_uuid(
    database: &Database,
    player_uuid: Uuid,
    new_team_name: TeamName,
    cars: Vec<Car>,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    let update = doc! { 
        "$set": { 
            "team_name": new_team_name.as_ref(),
            "cars": mongodb::bson::to_bson(&cars)?,
            "updated_at": BsonDateTime::now()
        } 
    };
    
    let options = mongodb::options::FindOneAndUpdateOptions::builder()
        .return_document(mongodb::options::ReturnDocument::After)
        .build();
    
    collection.find_one_and_update(filter, update, options).await
}
// A
dmin-only handler functions that work with AppState
#[utoipa::path(
    get,
    path = "/api/v1/admin/players",
    responses(
        (status = 200, description = "List of all players", body = [PlayerResponse])
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "admin"
)]
pub async fn get_all_players_admin(
    State(app_state): State<crate::app_state::AppState>,
) -> Result<Json<Vec<PlayerResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let db = app_state.database();
    get_all_players_impl(db).await
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/players/by-wallet/{wallet_address}",
    responses(
        (status = 200, description = "Player found", body = PlayerResponse),
        (status = 404, description = "Player not found")
    ),
    params(
        ("wallet_address" = String, Path, description = "Wallet address to search for")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "admin"
)]
pub async fn get_player_by_wallet_admin(
    Path(wallet_address): Path<String>,
    State(app_state): State<crate::app_state::AppState>,
) -> Result<Json<PlayerResponse>, (StatusCode, Json<serde_json::Value>)> {
    let db = app_state.database();
    get_player_by_wallet_impl(db, wallet_address).await
}

#[utoipa::path(
    get,
    path = "/api/v1/admin/players/by-email/{email}",
    responses(
        (status = 200, description = "Player found", body = PlayerResponse),
        (status = 404, description = "Player not found")
    ),
    params(
        ("email" = String, Path, description = "Email address to search for")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "admin"
)]
pub async fn get_player_by_email_admin(
    Path(email): Path<String>,
    State(app_state): State<crate::app_state::AppState>,
) -> Result<Json<PlayerResponse>, (StatusCode, Json<serde_json::Value>)> {
    let db = app_state.database();
    get_player_by_email_impl(db, email).await
}

// Implementation functions that can be shared between regular and admin handlers
async fn get_all_players_impl(
    db: &mongodb::Database,
) -> Result<Json<Vec<PlayerResponse>>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("[FETCHING ALL PLAYERS - START]");
    let start_time = std::time::Instant::now();

    match get_all_players_from_database(db).await {
        Ok(players) => {
            let response: Vec<PlayerResponse> = players
                .into_iter()
                .map(|player| PlayerResponse {
                    player,
                    message: "Player retrieved successfully".to_string(),
                })
                .collect();

            tracing::info!(
                "[FETCHING ALL PLAYERS - END]",
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Ok(Json(response))
        }
        Err(e) => {
            tracing::error!("[FETCHING ALL PLAYERS - EVENT] Failed to fetch players: {}", e);
            tracing::info!(
                "[FETCHING ALL PLAYERS - END]",
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch players"})),
            ))
        }
    }
}

async fn get_player_by_wallet_impl(
    db: &mongodb::Database,
    wallet_address: String,
) -> Result<Json<PlayerResponse>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("[FETCHING PLAYER BY WALLET - START]", wallet_address = %wallet_address);
    let start_time = std::time::Instant::now();

    match get_player_by_wallet_from_database(db, &wallet_address).await {
        Ok(Some(player)) => {
            let response = PlayerResponse {
                player,
                message: "Player retrieved successfully".to_string(),
            };

            tracing::info!(
                "[FETCHING PLAYER BY WALLET - END]",
                wallet_address = %wallet_address,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Ok(Json(response))
        }
        Ok(None) => {
            tracing::info!(
                "[FETCHING PLAYER BY WALLET - EVENT] Player not found",
                wallet_address = %wallet_address
            );
            tracing::info!(
                "[FETCHING PLAYER BY WALLET - END]",
                wallet_address = %wallet_address,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Player not found"})),
            ))
        }
        Err(e) => {
            tracing::error!(
                "[FETCHING PLAYER BY WALLET - EVENT] Failed to fetch player: {}",
                e,
                wallet_address = %wallet_address
            );
            tracing::info!(
                "[FETCHING PLAYER BY WALLET - END]",
                wallet_address = %wallet_address,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch player"})),
            ))
        }
    }
}

async fn get_player_by_email_impl(
    db: &mongodb::Database,
    email: String,
) -> Result<Json<PlayerResponse>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("[FETCHING PLAYER BY EMAIL - START]", email = %email);
    let start_time = std::time::Instant::now();

    match get_player_by_email_from_database(db, &email).await {
        Ok(Some(player)) => {
            let response = PlayerResponse {
                player,
                message: "Player retrieved successfully".to_string(),
            };

            tracing::info!(
                "[FETCHING PLAYER BY EMAIL - END]",
                email = %email,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Ok(Json(response))
        }
        Ok(None) => {
            tracing::info!(
                "[FETCHING PLAYER BY EMAIL - EVENT] Player not found",
                email = %email
            );
            tracing::info!(
                "[FETCHING PLAYER BY EMAIL - END]",
                email = %email,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Player not found"})),
            ))
        }
        Err(e) => {
            tracing::error!(
                "[FETCHING PLAYER BY EMAIL - EVENT] Failed to fetch player: {}",
                e,
                email = %email
            );
            tracing::info!(
                "[FETCHING PLAYER BY EMAIL - END]",
                email = %email,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch player"})),
            ))
        }
    }
}