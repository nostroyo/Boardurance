use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use mongodb::{bson::{doc, DateTime as BsonDateTime}, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{
    Player, WalletAddress, TeamName, Car, CarName, CarType, CarRarity, CarStats,
    Pilot, PilotName, PilotClass, PilotRarity, PilotSkills,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePlayerRequest {
    pub wallet_address: String,
    pub team_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTeamNameRequest {
    pub team_name: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddCarRequest {
    pub name: String,
    pub car_type: CarType,
    pub rarity: CarRarity,
    pub stats: CarStatsRequest,
    pub nft_mint_address: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CarStatsRequest {
    pub speed: u8,
    pub acceleration: u8,
    pub handling: u8,
    pub durability: u8,
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
        .route("/players", post(create_player))
        .route("/players", get(get_all_players))
        .route("/players/:wallet_address", get(get_player_by_wallet))
        .route("/players/:wallet_address", put(update_player_team_name))
        .route("/players/:wallet_address", delete(delete_player))
        .route("/players/:wallet_address/cars", post(add_car_to_player))
        .route("/players/:wallet_address/cars/:car_uuid", delete(remove_car_from_player))
        .route("/players/:wallet_address/pilots", post(add_pilot_to_player))
        .route("/players/:wallet_address/pilots/:pilot_uuid", delete(remove_pilot_from_player))
}

/// Create a new player
#[utoipa::path(
    post,
    path = "/api/v1/players",
    request_body = CreatePlayerRequest,
    responses(
        (status = 201, description = "Player created successfully", body = PlayerResponse),
        (status = 400, description = "Bad request"),
        (status = 409, description = "Player already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "players"
)]
#[tracing::instrument(
    name = "Creating a new player",
    skip(database, payload),
    fields(
        wallet_address = %payload.wallet_address,
        team_name = %payload.team_name
    )
)]
pub async fn create_player(
    State(database): State<Database>,
    Json(payload): Json<CreatePlayerRequest>,
) -> Result<(StatusCode, Json<PlayerResponse>), StatusCode> {
    let wallet_address = match WalletAddress::parse(payload.wallet_address) {
        Ok(addr) => addr,
        Err(e) => {
            tracing::warn!("Invalid wallet address: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let team_name = match TeamName::parse(payload.team_name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid team name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Check if player already exists
    if let Ok(existing) = get_player_by_wallet_address(&database, wallet_address.as_ref()).await {
        if existing.is_some() {
            tracing::warn!("Player with wallet address {} already exists", wallet_address.as_ref());
            return Err(StatusCode::CONFLICT);
        }
    }

    // Create player with empty cars and pilots initially
    let player = match Player::new(wallet_address, team_name, vec![], vec![]) {
        Ok(mut p) => {
            // Allow empty cars and pilots for initial creation
            p.cars = vec![];
            p.pilots = vec![];
            p
        }
        Err(e) => {
            tracing::error!("Failed to create player: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match insert_player(&database, &player).await {
        Ok(created_player) => {
            tracing::info!("Player created successfully");
            Ok((
                StatusCode::CREATED,
                Json(PlayerResponse {
                    player: created_player,
                    message: "Player created successfully".to_string(),
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

/// Get player by wallet address
#[utoipa::path(
    get,
    path = "/api/v1/players/{wallet_address}",
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

/// Update player team name
#[utoipa::path(
    put,
    path = "/api/v1/players/{wallet_address}",
    params(
        ("wallet_address" = String, Path, description = "Player's wallet address")
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
    Path(wallet_address): Path<String>,
    Json(payload): Json<UpdateTeamNameRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let new_team_name = match TeamName::parse(payload.team_name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid team name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match update_player_team_name_in_db(&database, &wallet_address, new_team_name).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Team name updated successfully for wallet: {}", wallet_address);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Team name updated successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for wallet address: {}", wallet_address);
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
    path = "/api/v1/players/{wallet_address}",
    params(
        ("wallet_address" = String, Path, description = "Player's wallet address")
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
    Path(wallet_address): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match delete_player_from_db(&database, &wallet_address).await {
        Ok(true) => {
            tracing::info!("Player deleted successfully: {}", wallet_address);
            Ok(StatusCode::OK)
        }
        Ok(false) => {
            tracing::warn!("Player not found for deletion: {}", wallet_address);
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
    path = "/api/v1/players/{wallet_address}/cars",
    params(
        ("wallet_address" = String, Path, description = "Player's wallet address")
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
    Path(wallet_address): Path<String>,
    Json(payload): Json<AddCarRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let car_name = match CarName::parse(payload.name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid car name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let car_stats = match CarStats::new(
        payload.stats.speed,
        payload.stats.acceleration,
        payload.stats.handling,
        payload.stats.durability,
    ) {
        Ok(stats) => stats,
        Err(e) => {
            tracing::warn!("Invalid car stats: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let car = match Car::new(
        car_name,
        payload.car_type,
        payload.rarity,
        car_stats,
        payload.nft_mint_address,
    ) {
        Ok(car) => car,
        Err(e) => {
            tracing::warn!("Failed to create car: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match add_car_to_player_in_db(&database, &wallet_address, car).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Car added successfully to player: {}", wallet_address);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Car added successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for wallet address: {}", wallet_address);
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
    path = "/api/v1/players/{wallet_address}/cars/{car_uuid}",
    params(
        ("wallet_address" = String, Path, description = "Player's wallet address"),
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
    Path((wallet_address, car_uuid_str)): Path<(String, String)>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let car_uuid = match Uuid::parse_str(&car_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid car UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match remove_car_from_player_in_db(&database, &wallet_address, car_uuid).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Car removed successfully from player: {}", wallet_address);
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
    path = "/api/v1/players/{wallet_address}/pilots",
    params(
        ("wallet_address" = String, Path, description = "Player's wallet address")
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
    Path(wallet_address): Path<String>,
    Json(payload): Json<AddPilotRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let pilot_name = match PilotName::parse(payload.name) {
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

    let pilot = match Pilot::new(
        pilot_name,
        payload.pilot_class,
        payload.rarity,
        pilot_skills,
        payload.nft_mint_address,
    ) {
        Ok(pilot) => pilot,
        Err(e) => {
            tracing::warn!("Failed to create pilot: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match add_pilot_to_player_in_db(&database, &wallet_address, pilot).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Pilot added successfully to player: {}", wallet_address);
            Ok(Json(PlayerResponse {
                player: updated_player,
                message: "Pilot added successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Player not found for wallet address: {}", wallet_address);
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
    path = "/api/v1/players/{wallet_address}/pilots/{pilot_uuid}",
    params(
        ("wallet_address" = String, Path, description = "Player's wallet address"),
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
    Path((wallet_address, pilot_uuid_str)): Path<(String, String)>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let pilot_uuid = match Uuid::parse_str(&pilot_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid pilot UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match remove_pilot_from_player_in_db(&database, &wallet_address, pilot_uuid).await {
        Ok(Some(updated_player)) => {
            tracing::info!("Pilot removed successfully from player: {}", wallet_address);
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