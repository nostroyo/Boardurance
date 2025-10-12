use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use mongodb::{bson::{doc, DateTime as BsonDateTime}, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::{
    Race, Track, Sector, SectorType, RaceStatus, LapAction, LapResult,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRaceRequest {
    pub name: String,
    pub track_name: String,
    pub sectors: Vec<CreateSectorRequest>,
    pub total_laps: u32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSectorRequest {
    pub id: u32,
    pub name: String,
    pub min_value: u32,
    pub max_value: u32,
    pub slot_capacity: Option<u32>,
    pub sector_type: SectorType,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct JoinRaceRequest {
    pub player_uuid: String,
    pub car_uuid: String,
    pub pilot_uuid: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProcessLapRequest {
    pub actions: Vec<LapActionRequest>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LapActionRequest {
    pub player_uuid: String,
    pub boost_value: u32,
}

#[derive(Serialize, ToSchema)]
pub struct RaceResponse {
    pub race: Race,
    pub message: String,
}

#[derive(Serialize, ToSchema)]
pub struct LapResultResponse {
    pub result: LapResult,
    pub race_status: RaceStatus,
}

pub fn routes() -> Router<Database> {
    Router::new()
        .route("/races", post(create_race))
        .route("/races", get(get_all_races))
        .route("/races/:race_uuid", get(get_race))
        .route("/races/:race_uuid/join", post(join_race))
        .route("/races/:race_uuid/start", post(start_race))
        .route("/races/:race_uuid/turn", post(process_turn))
        .route("/races/:race_uuid/status", get(get_race_status))
}

/// Create a new race
#[utoipa::path(
    post,
    path = "/api/v1/races",
    request_body = CreateRaceRequest,
    responses(
        (status = 201, description = "Race created successfully", body = RaceResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(
    name = "Creating a new race",
    skip(database, payload),
    fields(
        race_name = %payload.name,
        track_name = %payload.track_name,
        total_laps = payload.total_laps
    )
)]
pub async fn create_race(
    State(database): State<Database>,
    Json(payload): Json<CreateRaceRequest>,
) -> Result<(StatusCode, Json<RaceResponse>), StatusCode> {
    // Create sectors from request
    let sectors: Vec<Sector> = payload.sectors.into_iter().map(|s| Sector {
        id: s.id,
        name: s.name,
        min_value: s.min_value,
        max_value: s.max_value,
        slot_capacity: s.slot_capacity,
        sector_type: s.sector_type,
    }).collect();

    // Create track
    let track = match Track::new(payload.track_name, sectors) {
        Ok(track) => track,
        Err(e) => {
            tracing::warn!("Invalid track configuration: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Create race
    let race = Race::new(payload.name, track, payload.total_laps);

    match insert_race(&database, &race).await {
        Ok(created_race) => {
            tracing::info!("Race created successfully with UUID: {}", created_race.uuid);
            Ok((
                StatusCode::CREATED,
                Json(RaceResponse {
                    race: created_race,
                    message: "Race created successfully".to_string(),
                }),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to create race: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get all races
#[utoipa::path(
    get,
    path = "/api/v1/races",
    responses(
        (status = 200, description = "List of all races", body = Vec<Race>),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(name = "Fetching all races", skip(database))]
pub async fn get_all_races(State(database): State<Database>) -> Result<Json<Vec<Race>>, StatusCode> {
    match get_all_races_from_db(&database).await {
        Ok(races) => {
            tracing::info!("Successfully fetched {} races", races.len());
            Ok(Json(races))
        }
        Err(e) => {
            tracing::error!("Failed to fetch races: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get race by UUID
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}",
    params(
        ("race_uuid" = String, Path, description = "Race UUID")
    ),
    responses(
        (status = 200, description = "Race found", body = Race),
        (status = 404, description = "Race not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(name = "Fetching race by UUID", skip(database))]
pub async fn get_race(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
) -> Result<Json<Race>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match get_race_by_uuid(&database, race_uuid).await {
        Ok(Some(race)) => {
            tracing::info!("Race found for UUID: {}", race_uuid);
            Ok(Json(race))
        }
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to fetch race: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Join a race
#[utoipa::path(
    post,
    path = "/api/v1/races/{race_uuid}/join",
    params(
        ("race_uuid" = String, Path, description = "Race UUID")
    ),
    request_body = JoinRaceRequest,
    responses(
        (status = 200, description = "Successfully joined race", body = RaceResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Race not found"),
        (status = 409, description = "Cannot join race"),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(name = "Joining race", skip(database, payload))]
pub async fn join_race(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<JoinRaceRequest>,
) -> Result<Json<RaceResponse>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let player_uuid = match Uuid::parse_str(&payload.player_uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let car_uuid = match Uuid::parse_str(&payload.car_uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid car UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let pilot_uuid = match Uuid::parse_str(&payload.pilot_uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid pilot UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match join_race_in_db(&database, race_uuid, player_uuid, car_uuid, pilot_uuid).await {
        Ok(Some(updated_race)) => {
            tracing::info!("Player {} joined race {}", player_uuid, race_uuid);
            Ok(Json(RaceResponse {
                race: updated_race,
                message: "Successfully joined race".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to join race: {:?}", e);
            if e.to_string().contains("already participating") || e.to_string().contains("already started") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Start a race
#[utoipa::path(
    post,
    path = "/api/v1/races/{race_uuid}/start",
    params(
        ("race_uuid" = String, Path, description = "Race UUID")
    ),
    responses(
        (status = 200, description = "Race started successfully", body = RaceResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Race not found"),
        (status = 409, description = "Cannot start race"),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(name = "Starting race", skip(database))]
pub async fn start_race(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
) -> Result<Json<RaceResponse>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match start_race_in_db(&database, race_uuid).await {
        Ok(Some(updated_race)) => {
            tracing::info!("Race {} started successfully", race_uuid);
            Ok(Json(RaceResponse {
                race: updated_race,
                message: "Race started successfully".to_string(),
            }))
        }
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to start race: {:?}", e);
            if e.to_string().contains("already started") || e.to_string().contains("without participants") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Process a turn in the race
#[utoipa::path(
    post,
    path = "/api/v1/races/{race_uuid}/turn",
    params(
        ("race_uuid" = String, Path, description = "Race UUID")
    ),
    request_body = ProcessLapRequest,
    responses(
        (status = 200, description = "Lap processed successfully", body = LapResultResponse),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Race not found"),
        (status = 409, description = "Cannot process turn"),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(name = "Processing race turn", skip(database, payload))]
pub async fn process_turn(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<ProcessLapRequest>,
) -> Result<Json<LapResultResponse>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Convert request actions to domain actions
    let mut actions = Vec::new();
    for action_req in payload.actions {
        let player_uuid = match Uuid::parse_str(&action_req.player_uuid) {
            Ok(uuid) => uuid,
            Err(e) => {
                tracing::warn!("Invalid player UUID in action: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        actions.push(LapAction {
            player_uuid,
            boost_value: action_req.boost_value,
        });
    }

    match process_lap_in_db(&database, race_uuid, actions).await {
        Ok(Some((lap_result, race_status))) => {
            tracing::info!("Turn processed successfully for race {}", race_uuid);
            Ok(Json(LapResultResponse {
                result: lap_result,
                race_status,
            }))
        }
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to process turn: {:?}", e);
            if e.to_string().contains("not in progress") || e.to_string().contains("Missing action") {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Get race status
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}/status",
    params(
        ("race_uuid" = String, Path, description = "Race UUID")
    ),
    responses(
        (status = 200, description = "Race status", body = RaceStatus),
        (status = 404, description = "Race not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(name = "Getting race status", skip(database))]
pub async fn get_race_status(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
) -> Result<Json<RaceStatus>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match get_race_by_uuid(&database, race_uuid).await {
        Ok(Some(race)) => {
            tracing::info!("Race status retrieved for UUID: {}", race_uuid);
            Ok(Json(race.status))
        }
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to fetch race status: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Database operations
#[tracing::instrument(name = "Saving new race in the database", skip(database, race))]
pub async fn insert_race(
    database: &Database,
    race: &Race,
) -> Result<Race, mongodb::error::Error> {
    let collection = database.collection::<Race>("races");
    let result = collection.insert_one(race, None).await?;
    
    let mut created_race = race.clone();
    created_race.id = Some(result.inserted_id.as_object_id().unwrap());
    Ok(created_race)
}

#[tracing::instrument(name = "Getting all races from the database", skip(database))]
pub async fn get_all_races_from_db(database: &Database) -> Result<Vec<Race>, mongodb::error::Error> {
    let collection = database.collection::<Race>("races");
    let mut cursor = collection.find(None, None).await?;
    
    let mut races = Vec::new();
    while cursor.advance().await? {
        let race = cursor.deserialize_current()?;
        races.push(race);
    }
    
    Ok(races)
}

#[tracing::instrument(name = "Getting race by UUID from the database", skip(database))]
pub async fn get_race_by_uuid(
    database: &Database,
    race_uuid: Uuid,
) -> Result<Option<Race>, mongodb::error::Error> {
    let collection = database.collection::<Race>("races");
    let filter = doc! { "uuid": race_uuid.to_string() };
    collection.find_one(filter, None).await
}

#[tracing::instrument(name = "Joining race in the database", skip(database))]
pub async fn join_race_in_db(
    database: &Database,
    race_uuid: Uuid,
    player_uuid: Uuid,
    car_uuid: Uuid,
    pilot_uuid: Uuid,
) -> Result<Option<Race>, mongodb::error::Error> {
    let collection = database.collection::<Race>("races");
    
    // Get the race first
    let Some(mut race) = get_race_by_uuid(database, race_uuid).await? else {
        return Ok(None);
    };

    // Try to add participant
    if let Err(e) = race.add_participant(player_uuid, car_uuid, pilot_uuid) {
        return Err(mongodb::error::Error::custom(e));
    }

    // Update the race in database
    let filter = doc! { "uuid": race_uuid.to_string() };
    let update = doc! { 
        "$set": { 
            "participants": mongodb::bson::to_bson(&race.participants).unwrap(),
            "updated_at": BsonDateTime::now()
        } 
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Starting race in the database", skip(database))]
pub async fn start_race_in_db(
    database: &Database,
    race_uuid: Uuid,
) -> Result<Option<Race>, mongodb::error::Error> {
    let collection = database.collection::<Race>("races");
    
    // Get the race first
    let Some(mut race) = get_race_by_uuid(database, race_uuid).await? else {
        return Ok(None);
    };

    // Try to start race
    if let Err(e) = race.start_race() {
        return Err(mongodb::error::Error::custom(e));
    }

    // Update the race in database
    let filter = doc! { "uuid": race_uuid.to_string() };
    let update = doc! { 
        "$set": { 
            "status": mongodb::bson::to_bson(&race.status).unwrap(),
            "current_lap": race.current_lap,
            "lap_characteristic": mongodb::bson::to_bson(&race.lap_characteristic).unwrap(),
            "updated_at": BsonDateTime::now()
        } 
    };
    
    collection.find_one_and_update(filter, update, None).await
}

#[tracing::instrument(name = "Processing turn in the database", skip(database, actions))]
pub async fn process_lap_in_db(
    database: &Database,
    race_uuid: Uuid,
    actions: Vec<LapAction>,
) -> Result<Option<(LapResult, RaceStatus)>, mongodb::error::Error> {
    let collection = database.collection::<Race>("races");
    
    // Get the race first
    let Some(mut race) = get_race_by_uuid(database, race_uuid).await? else {
        return Ok(None);
    };

    // Process the lap
    let lap_result = match race.process_lap(&actions) {
        Ok(result) => result,
        Err(e) => return Err(mongodb::error::Error::custom(e)),
    };

    // Update the race in database
    let filter = doc! { "uuid": race_uuid.to_string() };
    let update = doc! { 
        "$set": { 
            "participants": mongodb::bson::to_bson(&race.participants).unwrap(),
            "current_lap": race.current_lap,
            "lap_characteristic": mongodb::bson::to_bson(&race.lap_characteristic).unwrap(),
            "status": mongodb::bson::to_bson(&race.status).unwrap(),
            "updated_at": BsonDateTime::now()
        } 
    };
    
    collection.find_one_and_update(filter, update, None).await?;
    
    Ok(Some((lap_result, race.status)))
}