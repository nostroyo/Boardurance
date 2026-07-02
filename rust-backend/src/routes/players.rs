use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use mongodb::{
    bson::{doc, DateTime as BsonDateTime},
    Database,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::domain::{
    Car, CarName, Pilot, PilotClass, PilotName, PilotRarity, PilotSkills, Player, TeamName,
};

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
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddPilotRequest {
    pub name: String,
    pub pilot_class: PilotClass,
    pub rarity: PilotRarity,
    pub skills: PilotSkillsRequest,
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

/// Player routes backed by the in-memory `MockPlayerRepository` from `AppState`.
///
/// Registration (in `auth.rs`) stores new players in this same repository, so
/// these routes serve real registered players without needing a database —
/// which is what the frontend Team page relies on. This also covers the asset
/// mutators (cars / pilots / configuration): they previously queried a Mongo
/// `players` collection that registered players never populated, so they 404'd
/// for every real player. They now operate on the same store as registration.
///
/// TODO: protect these with `AuthMiddleware` + an ownership check — the routes
/// currently trust the path `player_uuid`.
pub fn team_routes() -> Router<AppState> {
    Router::new()
        .route("/players/:player_uuid", get(get_player_by_uuid_mock))
        .route("/players/:player_uuid", put(update_team_name_mock))
        .route("/players/:player_uuid", delete(delete_player_mock))
        .route(
            "/players/:player_uuid/configuration",
            put(update_player_configuration),
        )
        .route("/players/:player_uuid/cars", post(add_car_to_player))
        .route(
            "/players/:player_uuid/cars/:car_uuid",
            delete(remove_car_from_player),
        )
        .route("/players/:player_uuid/pilots", post(add_pilot_to_player))
        .route(
            "/players/:player_uuid/pilots/:pilot_uuid",
            delete(remove_pilot_from_player),
        )
}

/// Get a player by UUID from the in-memory repository.
#[tracing::instrument(name = "Fetching player by UUID (mock)", skip(state))]
pub async fn get_player_by_uuid_mock(
    State(state): State<AppState>,
    Path(player_uuid_str): Path<String>,
) -> Result<Json<Player>, StatusCode> {
    let player_uuid = Uuid::parse_str(&player_uuid_str).map_err(|e| {
        tracing::warn!("Invalid player UUID: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match state.player_repository.find_by_uuid(player_uuid).await {
        Ok(Some(player)) => Ok(Json(player)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to fetch player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update a player's team name in the in-memory repository.
#[tracing::instrument(name = "Updating team name (mock)", skip(state, payload))]
pub async fn update_team_name_mock(
    State(state): State<AppState>,
    Path(player_uuid_str): Path<String>,
    Json(payload): Json<UpdateTeamNameRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player_uuid = Uuid::parse_str(&player_uuid_str).map_err(|e| {
        tracing::warn!("Invalid player UUID: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    let new_team_name = TeamName::parse(&payload.team_name).map_err(|e| {
        tracing::warn!("Invalid team name: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match state
        .player_repository
        .update_team_name_by_uuid(player_uuid, new_team_name)
        .await
    {
        Ok(Some(player)) => Ok(Json(PlayerResponse {
            player,
            message: "Team name updated successfully".to_string(),
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to update team name: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a player from the in-memory repository.
#[tracing::instrument(name = "Deleting player (mock)", skip(state))]
pub async fn delete_player_mock(
    State(state): State<AppState>,
    Path(player_uuid_str): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let player_uuid = Uuid::parse_str(&player_uuid_str).map_err(|e| {
        tracing::warn!("Invalid player UUID: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    match state.player_repository.delete_by_uuid(player_uuid).await {
        Ok(true) => Ok(StatusCode::OK),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to delete player: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Admin-only routes that require authentication and admin role
pub fn admin_routes() -> Router<crate::app_state::AppState> {
    // Temporarily disabled due to tracing format issues in admin functions
    Router::new()
    // TODO: Re-enable admin routes after fixing tracing format issues
    // .route("/players", get(get_all_players_admin))
    // .route("/players/by-email/:email", get(get_player_by_email_admin))
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
pub async fn get_all_players(
    State(database): State<Database>,
) -> Result<Json<Vec<Player>>, StatusCode> {
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
#[tracing::instrument(name = "Updating player configuration", skip(state, payload))]
pub async fn update_player_configuration(
    State(state): State<AppState>,
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

    // Replace the player's cars, then update the team name. Both operate on the
    // in-memory repository where registration stores players (the same store the
    // GET/PUT/DELETE team routes use), so configuration changes target the player
    // that actually exists instead of an empty Mongo collection.
    match state
        .player_repository
        .set_cars_by_uuid(player_uuid, payload.cars)
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => {
            tracing::warn!("Player not found for UUID: {}", player_uuid);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to update cars: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    match state
        .player_repository
        .update_team_name_by_uuid(player_uuid, new_team_name)
        .await
    {
        Ok(Some(updated_player)) => {
            tracing::info!(
                "Configuration updated successfully for player: {}",
                player_uuid
            );
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
#[tracing::instrument(name = "Adding car to player", skip(state, payload))]
pub async fn add_car_to_player(
    State(state): State<AppState>,
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

    let car = match Car::new(car_name) {
        Ok(car) => car,
        Err(e) => {
            tracing::warn!("Failed to create car: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match state
        .player_repository
        .add_car_by_uuid(player_uuid, car)
        .await
    {
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
#[tracing::instrument(name = "Removing car from player", skip(state))]
pub async fn remove_car_from_player(
    State(state): State<AppState>,
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

    match state
        .player_repository
        .remove_car_by_uuid(player_uuid, car_uuid)
        .await
    {
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
#[tracing::instrument(name = "Adding pilot to player", skip(state, payload))]
pub async fn add_pilot_to_player(
    State(state): State<AppState>,
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
        u8::midpoint(pilot_skills.reaction_time, pilot_skills.focus), // straight value
        u8::midpoint(pilot_skills.precision, pilot_skills.stamina),   // curve value
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
    ) {
        Ok(pilot) => pilot,
        Err(e) => {
            tracing::warn!("Failed to create pilot: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match state
        .player_repository
        .add_pilot_by_uuid(player_uuid, pilot)
        .await
    {
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
#[tracing::instrument(name = "Removing pilot from player", skip(state))]
pub async fn remove_pilot_from_player(
    State(state): State<AppState>,
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

    match state
        .player_repository
        .remove_pilot_by_uuid(player_uuid, pilot_uuid)
        .await
    {
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
pub async fn get_all_players_from_db(
    database: &Database,
) -> Result<Vec<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let mut cursor = collection.find(None, None).await?;

    let mut players = Vec::new();
    while cursor.advance().await? {
        let player = cursor.deserialize_current()?;
        players.push(player);
    }

    Ok(players)
}

#[tracing::instrument(
    name = "Getting player by email address from the database",
    skip(database)
)]
pub async fn get_player_by_email_address(
    database: &Database,
    email: &str,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "email": email };
    collection.find_one(filter, None).await
}

#[tracing::instrument(name = "Getting player by UUID from the database", skip(database))]
pub async fn get_player_by_uuid_from_db(
    database: &Database,
    player_uuid: Uuid,
) -> Result<Option<Player>, mongodb::error::Error> {
    let collection = database.collection::<Player>("players");
    let filter = doc! { "uuid": player_uuid.to_string() };
    collection.find_one(filter, None).await
}

#[tracing::instrument(
    name = "Updating player team name by UUID in the database",
    skip(database, new_team_name)
)]
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

/* TEMPORARILY COMMENTED OUT - ADMIN FUNCTIONS HAVE TRACING FORMAT ISSUES
// Admin-only handler functions that work with AppState
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
    let db = &app_state.database;
    get_all_players_impl(db).await
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
    let db = &app_state.database;
    get_player_by_email_impl(db, email).await
}

// Implementation functions that can be shared between regular and admin handlers
async fn get_all_players_impl(
    db: &mongodb::Database,
) -> Result<Json<Vec<PlayerResponse>>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("[FETCHING ALL PLAYERS - START]");
    let start_time = std::time::Instant::now();

    match get_all_players_from_db(db).await {
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

async fn get_player_by_email_impl(
    db: &mongodb::Database,
    email: String,
) -> Result<Json<PlayerResponse>, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!("[FETCHING PLAYER BY EMAIL - START] email={}", email);
    let start_time = std::time::Instant::now();

    match get_player_by_email_address(db, &email).await {
        Ok(Some(player)) => {
            let response = PlayerResponse {
                player,
                message: "Player retrieved successfully".to_string(),
            };

            tracing::info!(
                "[FETCHING PLAYER BY EMAIL - END] email={}",\n                email,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Ok(Json(response))
        }
        Ok(None) => {
            tracing::info!(
                "[FETCHING PLAYER BY EMAIL - EVENT] Player not found email={}",\n                email
            );
            tracing::info!(
                "[FETCHING PLAYER BY EMAIL - END] email={}",\n                email,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Player not found"})),
            ))
        }
        Err(e) => {
            tracing::error!(
                "[FETCHING PLAYER BY EMAIL - EVENT] Failed to fetch player: {} email={}",\n                e,\n                email
            );
            tracing::info!(
                "[FETCHING PLAYER BY EMAIL - END] email={}",\n                email,
                elapsed_milliseconds = start_time.elapsed().as_millis() as u64
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch player"})),
            ))
        }
    }
}*/

#[cfg(test)]
mod player_asset_tests {
    use super::*;
    use crate::app_state::AppState;
    use crate::domain::{Email, Password, Player, TeamName};
    use crate::test_utils::TestAppState;
    use axum::extract::{Path, State};
    use axum::Json;

    /// Build an `AppState` (backed by mock repositories) whose in-memory
    /// repository holds a single player (mirroring the post-registration
    /// state), returning that player's uuid.
    async fn seeded_state_with_player() -> (AppState, uuid::Uuid) {
        let email = Email::parse("driver@example.com").unwrap();
        let password_hash = Password::new("Sup3rSecret!".to_string())
            .unwrap()
            .hash()
            .unwrap();
        let team_name = TeamName::parse("Test Team").unwrap();
        let player = Player::new(email, password_hash, team_name, vec![], vec![]).unwrap();
        let player_uuid = player.uuid;

        let parts = TestAppState::with_test_data(vec![player], vec![], vec![]);
        // The mongodb driver connects lazily, so constructing a `Client`/`Database`
        // handle here performs no I/O and needs no real MongoDB instance — this
        // stays a mock-only test.
        let database = mongodb::Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("client construction is lazy and does not connect")
            .database("test_database");
        let state = AppState::new(
            parts.player_repo,
            parts.race_repo,
            parts.session_repo,
            parts.jwt_service,
            parts.session_manager,
            database,
        );
        (state, player_uuid)
    }

    /// Regression for the two-store split (#3): `add_car` MUST act on the same
    /// in-memory repository where registration stores players. Previously the
    /// handler queried a Mongo `players` collection that registered players never
    /// populated, so every real player got a 404. Here the player exists only in
    /// the mock repo (as after registration); the car must be added, not 404'd.
    #[tokio::test]
    async fn add_car_targets_the_registration_store() {
        let (state, player_uuid) = seeded_state_with_player().await;

        let response = add_car_to_player(
            State(state.clone()),
            Path(player_uuid.to_string()),
            Json(AddCarRequest {
                name: "Test Car".to_string(),
            }),
        )
        .await
        .expect("adding a car to a registered player must succeed (not 404)");

        assert_eq!(
            response.0.player.cars.len(),
            1,
            "the car must be persisted to the player in the in-memory store"
        );

        // And it is observable on the same store the team routes read from.
        let stored = state
            .player_repository
            .find_by_uuid(player_uuid)
            .await
            .unwrap()
            .expect("player should still be present");
        assert_eq!(stored.cars.len(), 1);
    }
}
