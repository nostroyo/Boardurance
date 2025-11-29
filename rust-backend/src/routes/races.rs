use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use mongodb::{bson::{doc, DateTime as BsonDateTime}, Database};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::{
    Race, Track, Sector, SectorType, RaceStatus, LapAction, LapResult, LapCharacteristic,
    MovementType,
};
use crate::domain::boost_hand_manager::{BoostHandManager, BoostAvailability, BoostCardErrorResponse};
use crate::services::car_validation::{CarValidationService, ValidatedCarData};

// Helper function to convert to BSON with proper error handling
fn to_bson_safe<T: serde::Serialize>(value: &T, field_name: &str) -> Result<mongodb::bson::Bson, mongodb::error::Error> {
    mongodb::bson::to_bson(value)
        .map_err(|e| mongodb::error::Error::custom(format!("Failed to serialize {}: {}", field_name, e)))
}

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

// Enhanced API Data Models

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterPlayerRequest {
    pub player_uuid: String,
    pub car_uuid: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterPlayerResponse {
    pub success: bool,
    pub message: String,
    pub race_status: RaceProgressStatus,
    pub player_position: PlayerRacePosition,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerRacePosition {
    pub starting_sector: u32,
    pub position_in_sector: u32,
    pub qualification_rank: u32,
}

#[derive(Debug, Deserialize)]
pub struct StatusQueryParams {
    pub player_uuid: Option<String>, // For player-specific data
    pub include_history: Option<bool>, // Include lap history
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DetailedRaceStatusResponse {
    pub race_progress: RaceProgressStatus,
    pub track_situation: TrackSituationData,
    pub player_data: Option<PlayerSpecificData>, // Only if player_uuid provided
    pub race_metadata: RaceMetadata,
}

/// Request to apply a lap action with boost card selection
/// 
/// # Boost Card System
/// The `boost_value` field represents which boost card to use (0-4).
/// Each player has 5 boost cards per cycle that can be used once each.
/// When all cards are used, the hand automatically replenishes.
/// 
/// # Performance Calculation
/// Final performance = base_performance * (1 + boost_value * 0.08)
/// - boost_value 0: No boost (1.0x multiplier)
/// - boost_value 1: 8% boost (1.08x multiplier)  
/// - boost_value 2: 16% boost (1.16x multiplier)
/// - boost_value 3: 24% boost (1.24x multiplier)
/// - boost_value 4: 32% boost (1.32x multiplier)
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplyLapRequest {
    /// UUID of the player making the lap action
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub player_uuid: String,
    
    /// UUID of the car being used for this lap
    #[schema(example = "550e8400-e29b-41d4-a716-446655440001")]
    pub car_uuid: String,
    
    /// Boost card value to use (0-4). Must be available in current cycle.
    #[schema(example = 3, minimum = 0, maximum = 4)]
    pub boost_value: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RaceProgressStatus {
    pub status: RaceStatusType,
    pub current_lap: u32,
    pub total_laps: u32,
    pub lap_characteristic: LapCharacteristic,
    pub turn_phase: TurnPhase,
    pub participants_count: u32,
    pub finished_participants: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub enum RaceStatusType {
    Waiting,
    Ongoing,
    Finished,
    Error { message: String },
}

#[derive(Debug, Serialize, ToSchema)]
pub enum TurnPhase {
    WaitingForPlayers,
    AllSubmitted,
    Processing,
    Complete,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TrackSituationData {
    pub sectors: Vec<SectorSituation>,
    pub recent_movements: Vec<ParticipantMovement>,
    pub lap_leaderboard: Vec<LeaderboardEntry>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SectorSituation {
    pub sector_id: u32,
    pub sector_name: String,
    pub sector_type: SectorType,
    pub capacity_info: SectorCapacityInfo,
    pub participants: Vec<SectorParticipant>,
    pub performance_thresholds: PerformanceThresholds,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SectorCapacityInfo {
    pub max_capacity: Option<u32>, // None = infinite
    pub current_occupancy: u32,
    pub available_slots: Option<u32>, // None = infinite
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SectorParticipant {
    pub player_uuid: String,
    pub player_name: Option<String>, // From player lookup
    pub car_name: String,
    pub position_in_sector: u32,
    pub total_value: u32,
    pub current_lap: u32,
    pub is_finished: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PerformanceThresholds {
    pub min_value: u32,
    pub max_value: u32,
    pub move_up_threshold: u32,
    pub move_down_threshold: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ParticipantMovement {
    pub player_uuid: String,
    pub from_sector: u32,
    pub to_sector: u32,
    pub movement_type: MovementType,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LeaderboardEntry {
    pub player_uuid: String,
    pub player_name: Option<String>,
    pub car_name: String,
    pub current_sector: u32,
    pub position_in_sector: u32,
    pub total_value: u32,
    pub current_lap: u32,
    pub overall_rank: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerSpecificData {
    pub boost_availability: BoostAvailability,
    pub performance_preview: PerformancePreview,
    pub current_position: CurrentPlayerPosition,
    pub lap_history: Option<Vec<LapPerformanceRecord>>,
    pub boost_usage_history: Vec<crate::domain::BoostUsageRecord>,
    pub boost_cycle_summaries: Vec<crate::domain::BoostCycleSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PerformancePreview {
    pub engine_contribution: u32,
    pub body_contribution: u32,
    pub pilot_contribution: u32,
    pub base_value: u32,
    pub sector_ceiling: u32,
    pub capped_base_value: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentPlayerPosition {
    pub current_sector: u32,
    pub position_in_sector: u32,
    pub sector_rank: u32,
    pub overall_rank: u32,
    pub distance_to_leader: u32, // In sectors + positions
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LapPerformanceRecord {
    pub lap_number: u32,
    pub boost_used: u32,
    pub final_value: u32,
    pub movement_type: MovementType,
    pub from_sector: u32,
    pub to_sector: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RaceMetadata {
    pub race_uuid: String,
    pub race_name: String,
    pub track_name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub total_turns: u32,
}



pub fn routes() -> Router<Database> {
    Router::new()
        // Public routes (no authentication required)
        .route("/races", get(get_all_races))
        .route("/races/:race_uuid", get(get_race))
        .route("/races/:race_uuid/status", get(get_race_status))
        
        // Enhanced API endpoints
        .route("/races/:race_uuid/register", post(register_player))
        .route("/races/:race_uuid/status-detailed", get(get_race_status_detailed))
        .route("/races/:race_uuid/apply-lap", post(apply_lap_action))
        
        // Protected routes - These should be protected with AuthMiddleware
        // TODO: Apply middleware layers in startup.rs:
        // 1. AuthMiddleware to validate JWT tokens and extract UserContext
        // 2. Custom validation for race participation/ownership
        
        // Routes that require authentication:
        .route("/races", post(create_race))              // Any authenticated user can create
        .route("/races/:race_uuid/join", post(join_race)) // Any authenticated user can join
        
        // Routes that require race ownership or admin role:
        .route("/races/:race_uuid/start", post(start_race))    // Race creator or admin
        .route("/races/:race_uuid/turn", post(process_turn))   // Race participants or admin
}

// Helper Functions for Enhanced API

async fn register_player_in_race(
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
            "participants": to_bson_safe(&race.participants, "participants")?,
            "pending_actions": to_bson_safe(&race.pending_actions, "pending_actions")?,
            "action_submissions": to_bson_safe(&race.action_submissions, "action_submissions")?,
            "pending_performance_calculations": to_bson_safe(&race.pending_performance_calculations, "pending_performance_calculations")?,
            "updated_at": BsonDateTime::now()
        } 
    };
    
    collection.find_one_and_update(filter, update, None).await
}

fn get_player_race_position(race: &Race, player_uuid: Uuid) -> Result<PlayerRacePosition, String> {
    let participant = race.participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or("Player not found in race")?;
    
    // Calculate qualification rank based on starting sector and position
    let mut qualification_rank = 1u32;
    for other_participant in &race.participants {
        if other_participant.player_uuid != player_uuid
            && (other_participant.current_sector > participant.current_sector ||
               (other_participant.current_sector == participant.current_sector && 
                other_participant.current_position_in_sector < participant.current_position_in_sector)) {
                qualification_rank += 1;
            }
    }
    
    Ok(PlayerRacePosition {
        starting_sector: participant.current_sector,
        position_in_sector: participant.current_position_in_sector,
        qualification_rank,
    })
}

fn build_race_progress_status(race: &Race) -> RaceProgressStatus {
    let status = match race.status {
        RaceStatus::Waiting => RaceStatusType::Waiting,
        RaceStatus::InProgress => RaceStatusType::Ongoing,
        RaceStatus::Finished => RaceStatusType::Finished,
        RaceStatus::Cancelled => RaceStatusType::Error { 
            message: "Race was cancelled".to_string() 
        },
    };
    
    #[allow(clippy::cast_possible_truncation)]
    let finished_participants = race.participants
        .iter()
        .filter(|p| p.is_finished)
        .count() as u32;
    
    // Determine turn phase based on race state
    let turn_phase = if race.status == RaceStatus::InProgress {
        // TODO: Implement proper turn phase tracking
        TurnPhase::WaitingForPlayers
    } else {
        TurnPhase::Complete
    };
    
    #[allow(clippy::cast_possible_truncation)]
    RaceProgressStatus {
        status,
        current_lap: race.current_lap,
        total_laps: race.total_laps,
        lap_characteristic: race.lap_characteristic.clone(),
        turn_phase,
        participants_count: race.participants.len() as u32,
        finished_participants,
    }
}

#[allow(clippy::unused_async)]
async fn build_track_situation_data(
    _database: &Database,
    race: &Race,
) -> Result<TrackSituationData, mongodb::error::Error> {
    let mut sectors = Vec::new();
    
    // Build sector situation for each sector
    for sector in &race.track.sectors {
        let participants_in_sector: Vec<_> = race.participants
            .iter()
            .filter(|p| p.current_sector == sector.id && !p.is_finished)
            .collect();
        
        let mut sector_participants = Vec::new();
        for participant in participants_in_sector {
            // TODO: Fetch player name from database
            let player_name = None; // Placeholder
            
            // TODO: Fetch car name from database
            let car_name = format!("Car {}", participant.car_uuid);
            
            sector_participants.push(SectorParticipant {
                player_uuid: participant.player_uuid.to_string(),
                player_name,
                car_name,
                position_in_sector: participant.current_position_in_sector,
                total_value: participant.total_value,
                current_lap: participant.current_lap,
                is_finished: participant.is_finished,
            });
        }
        
        // Sort by position in sector
        sector_participants.sort_by_key(|p| p.position_in_sector);
        
        #[allow(clippy::cast_possible_truncation)]
        let capacity_info = SectorCapacityInfo {
            max_capacity: sector.slot_capacity,
            current_occupancy: sector_participants.len() as u32,
            available_slots: sector.slot_capacity.map(|cap| cap.saturating_sub(sector_participants.len() as u32)),
        };
        
        let performance_thresholds = PerformanceThresholds {
            min_value: sector.min_value,
            max_value: sector.max_value,
            move_up_threshold: sector.max_value,
            move_down_threshold: sector.min_value,
        };
        
        sectors.push(SectorSituation {
            sector_id: sector.id,
            sector_name: sector.name.clone(),
            sector_type: sector.sector_type.clone(),
            capacity_info,
            participants: sector_participants,
            performance_thresholds,
        });
    }
    
    // Build recent movements (placeholder for now)
    let recent_movements = Vec::new();
    
    // Build lap leaderboard
    let mut leaderboard_entries = Vec::new();
    for (index, participant) in race.participants.iter().enumerate() {
        if !participant.is_finished {
            // TODO: Fetch player and car names from database
            let player_name = None;
            let car_name = format!("Car {}", participant.car_uuid);
            
            leaderboard_entries.push(LeaderboardEntry {
                player_uuid: participant.player_uuid.to_string(),
                player_name,
                car_name,
                current_sector: participant.current_sector,
                position_in_sector: participant.current_position_in_sector,
                total_value: participant.total_value,
                current_lap: participant.current_lap,
                #[allow(clippy::cast_possible_truncation)]
                overall_rank: (index + 1) as u32,
            });
        }
    }
    
    // Sort leaderboard by sector (descending) then by position in sector (ascending)
    leaderboard_entries.sort_by(|a, b| {
        b.current_sector.cmp(&a.current_sector)
            .then_with(|| a.position_in_sector.cmp(&b.position_in_sector))
    });
    
    // Update ranks after sorting
    #[allow(clippy::cast_possible_truncation)]
    for (index, entry) in leaderboard_entries.iter_mut().enumerate() {
        entry.overall_rank = (index + 1) as u32;
    }
    
    Ok(TrackSituationData {
        sectors,
        recent_movements,
        lap_leaderboard: leaderboard_entries,
    })
}

fn build_race_metadata(race: &Race) -> RaceMetadata {
    RaceMetadata {
        race_uuid: race.uuid.to_string(),
        race_name: race.name.clone(),
        track_name: race.track.name.clone(),
        start_time: if race.status == RaceStatus::InProgress || race.status == RaceStatus::Finished {
            Some(DateTime::from_timestamp(race.created_at.timestamp_millis() / 1000, 0).unwrap_or_default())
        } else {
            None
        },
        estimated_completion: None, // TODO: Calculate based on current progress
        total_turns: 0, // TODO: Implement turn tracking
    }
}

#[allow(clippy::unused_async)]
async fn build_player_specific_data(
    _database: &Database,
    race: &Race,
    player_uuid: Uuid,
) -> Result<PlayerSpecificData, mongodb::error::Error> {
    let participant = race.participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or_else(|| mongodb::error::Error::custom("Player not found in race"))?;
    
    if participant.is_finished {
        return Err(mongodb::error::Error::custom("Player has finished the race"));
    }
    
    let current_sector = &race.track.sectors[participant.current_sector as usize];
    
    // Build boost availability using BoostHandManager
    let base_performance = 10u32; // TODO: Calculate from car components
    let boost_availability = BoostHandManager::get_boost_availability(
        &participant.boost_hand,
        current_sector,
        base_performance,
    );
    
    // Build performance preview
    let performance_preview = PerformancePreview {
        engine_contribution: 5, // TODO: Get from actual car components
        body_contribution: 3,   // TODO: Get from actual car components
        pilot_contribution: 2,  // TODO: Get from actual car components
        base_value: base_performance,
        sector_ceiling: current_sector.max_value,
        capped_base_value: std::cmp::min(base_performance, current_sector.max_value),
    };
    
    // Build current position
    let mut overall_rank = 1u32;
    for other_participant in &race.participants {
        if other_participant.player_uuid != player_uuid && !other_participant.is_finished
            && (other_participant.current_sector > participant.current_sector ||
               (other_participant.current_sector == participant.current_sector && 
                other_participant.current_position_in_sector < participant.current_position_in_sector)) {
                overall_rank += 1;
            }
    }
    
    let current_position = CurrentPlayerPosition {
        current_sector: participant.current_sector,
        position_in_sector: participant.current_position_in_sector,
        sector_rank: participant.current_position_in_sector + 1,
        overall_rank,
        distance_to_leader: 0, // TODO: Calculate distance to leader
    };
    
    // Build lap history (placeholder)
    let lap_history = None; // TODO: Implement lap history tracking
    
    // Get boost usage history and cycle summaries
    let boost_usage_history = participant.boost_usage_history.clone();
    let boost_cycle_summaries = participant.get_boost_cycle_summaries();
    
    Ok(PlayerSpecificData {
        boost_availability,
        performance_preview,
        current_position,
        lap_history,
        boost_usage_history,
        boost_cycle_summaries,
    })
}

async fn process_individual_lap_action(
    database: &Database,
    race_uuid: Uuid,
    player_uuid: Uuid,
    boost_value: u32,
    car_data: &ValidatedCarData,
) -> Result<Option<Race>, mongodb::error::Error> {
    let collection = database.collection::<Race>("races");
    
    // Get the race first
    let Some(mut race) = get_race_by_uuid(database, race_uuid).await? else {
        return Ok(None);
    };

    // Process individual lap action using the new method
    match race.process_individual_lap_action(player_uuid, boost_value, car_data) {
        Ok(_individual_result) => {
            // Update the race in database with new fields
            let filter = doc! { "uuid": race_uuid.to_string() };
            let update = doc! { 
                "$set": { 
                    "participants": to_bson_safe(&race.participants, "participants")?,
                    "current_lap": race.current_lap,
                    "lap_characteristic": to_bson_safe(&race.lap_characteristic, "lap_characteristic")?,
                    "status": to_bson_safe(&race.status, "status")?,
                    "pending_actions": to_bson_safe(&race.pending_actions, "pending_actions")?,
                    "action_submissions": to_bson_safe(&race.action_submissions, "action_submissions")?,
                    "pending_performance_calculations": to_bson_safe(&race.pending_performance_calculations, "pending_performance_calculations")?,
                    "updated_at": BsonDateTime::now()
                } 
            };
            
            collection.find_one_and_update(filter, update, None).await
        }
        Err(e) => Err(mongodb::error::Error::custom(e)),
    }
}



// Enhanced API Endpoint Implementations

/// Register a player for a race
#[utoipa::path(
    post,
    path = "/api/v1/races/{race_uuid}/register",
    params(("race_uuid" = String, Path, description = "Race UUID")),
    request_body = RegisterPlayerRequest,
    responses(
        (status = 200, description = "Successfully registered for race", body = RegisterPlayerResponse),
        (status = 400, description = "Invalid request or car validation failed"),
        (status = 404, description = "Race not found"),
        (status = 409, description = "Cannot register (race started, player already registered, etc.)"),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(
    name = "Registering player for race",
    skip(database, payload),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %payload.player_uuid,
        car_uuid = %payload.car_uuid
    )
)]
pub async fn register_player(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<RegisterPlayerRequest>,
) -> Result<Json<RegisterPlayerResponse>, StatusCode> {
    // 1. Parse and validate UUIDs
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
    
    // 2. Validate car and get components
    let car_data = match CarValidationService::validate_car_for_race(
        &database, 
        player_uuid, 
        car_uuid
    ).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("Car validation failed: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // 3. Register player in race
    let updated_race = match register_player_in_race(
        &database,
        race_uuid,
        player_uuid,
        car_uuid,
        car_data.pilot.uuid,
    ).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to register player: {:?}", e);
            if e.to_string().contains("already participating") || e.to_string().contains("already started") {
                return Err(StatusCode::CONFLICT);
            }
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // 4. Build response
    let player_position = match get_player_race_position(&updated_race, player_uuid) {
        Ok(position) => position,
        Err(e) => {
            tracing::error!("Failed to get player position: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    let race_status = build_race_progress_status(&updated_race);
    
    tracing::info!("Player {} successfully registered for race {}", player_uuid, race_uuid);
    
    Ok(Json(RegisterPlayerResponse {
        success: true,
        message: "Successfully registered for race".to_string(),
        race_status,
        player_position,
    }))
}

/// Get detailed race status with comprehensive boost hand information
/// 
/// This endpoint provides complete race status including boost card system state.
/// When `player_uuid` is provided, returns player-specific data including:
/// - Current boost hand state (available/used cards)
/// - Boost cycle information (current cycle, cycles completed)
/// - Boost usage history and cycle summaries
/// - Performance impact preview for each boost card
/// 
/// # Boost Hand State Information
/// - `available_cards`: List of boost card values (0-4) currently available
/// - `hand_state`: Detailed boolean map of each card's availability
/// - `current_cycle`: Current boost cycle number (starts at 1)
/// - `cycles_completed`: Number of complete cycles finished
/// - `cards_remaining`: Cards left before next replenishment
/// - `boost_impact_preview`: Performance prediction for each boost option
/// 
/// # Usage History
/// - `boost_usage_history`: Lap-by-lap record of boost card usage
/// - `boost_cycle_summaries`: Aggregated statistics per cycle
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}/status-detailed",
    params(
        ("race_uuid" = String, Path, description = "Race UUID"),
        ("player_uuid" = Option<String>, Query, description = "Player UUID for player-specific boost hand data"),
        ("include_history" = Option<bool>, Query, description = "Include detailed lap and boost usage history")
    ),
    responses(
        (
            status = 200, 
            description = "Detailed race status with boost hand information",
            body = DetailedRaceStatusResponse,
            example = json!({
                "race_progress": {
                    "status": "Ongoing",
                    "current_lap": 3,
                    "total_laps": 5,
                    "participants_count": 2,
                    "finished_participants": 0
                },
                "track_situation": {
                    "sectors": [
                        {
                            "sector_id": 1,
                            "sector_name": "Start/Finish",
                            "participants": [
                                {
                                    "player_uuid": "550e8400-e29b-41d4-a716-446655440000",
                                    "position_in_sector": 1,
                                    "total_value": 45,
                                    "current_lap": 3
                                }
                            ]
                        }
                    ]
                },
                "player_data": {
                    "boost_availability": {
                        "available_cards": [1, 4],
                        "hand_state": {
                            "0": false,
                            "1": true,
                            "2": false,
                            "3": false,
                            "4": true
                        },
                        "current_cycle": 1,
                        "cycles_completed": 0,
                        "cards_remaining": 2,
                        "next_replenishment_at": 2,
                        "boost_impact_preview": [
                            {
                                "boost_value": 0,
                                "is_available": false,
                                "predicted_final_value": 20,
                                "movement_probability": "Stay"
                            },
                            {
                                "boost_value": 1,
                                "is_available": true,
                                "predicted_final_value": 22,
                                "movement_probability": "MoveUp"
                            }
                        ]
                    },
                    "boost_usage_history": [
                        {
                            "lap_number": 1,
                            "boost_value": 2,
                            "cycle_number": 1,
                            "cards_remaining_after": 3,
                            "replenishment_occurred": false
                        },
                        {
                            "lap_number": 2,
                            "boost_value": 0,
                            "cycle_number": 1,
                            "cards_remaining_after": 2,
                            "replenishment_occurred": false
                        }
                    ],
                    "boost_cycle_summaries": [
                        {
                            "cycle_number": 1,
                            "cards_used": [2, 0, 3],
                            "laps_in_cycle": [1, 2, 3],
                            "average_boost": 1.67
                        }
                    ]
                }
            })
        ),
        (status = 404, description = "Race not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "boost-cards"
)]
#[tracing::instrument(
    name = "Getting detailed race status",
    skip(database),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = ?params.player_uuid
    )
)]
pub async fn get_race_status_detailed(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Query(params): Query<StatusQueryParams>,
) -> Result<Json<DetailedRaceStatusResponse>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    let race = match get_race_by_uuid(&database, race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to fetch race: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Build comprehensive status response
    let race_progress = build_race_progress_status(&race);
    let track_situation = match build_track_situation_data(&database, &race).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to build track situation: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let race_metadata = build_race_metadata(&race);
    
    // Include player-specific data if requested
    let player_data = if let Some(player_uuid_str) = params.player_uuid {
        let player_uuid = match Uuid::parse_str(&player_uuid_str) {
            Ok(uuid) => uuid,
            Err(e) => {
                tracing::warn!("Invalid player UUID: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        };
        
        match build_player_specific_data(&database, &race, player_uuid).await {
            Ok(data) => Some(data),
            Err(e) => {
                tracing::error!("Failed to build player specific data: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        None
    };
    
    tracing::info!("Detailed race status retrieved for UUID: {}", race_uuid);
    
    Ok(Json(DetailedRaceStatusResponse {
        race_progress,
        track_situation,
        player_data,
        race_metadata,
    }))
}

/// Apply individual lap action for a player with boost card validation
/// 
/// This endpoint processes a player's lap action including boost card selection.
/// The boost card system enforces strategic resource management:
/// - Players have 5 boost cards (values 0-4) available per cycle
/// - Each card can only be used once per cycle
/// - When all 5 cards are used, the hand automatically replenishes
/// - Boost cards multiply performance: base_value * (1 + boost_value * 0.08)
/// 
/// # Boost Card Usage Flow
/// 1. Player selects an available boost card (0-4)
/// 2. System validates card availability in current cycle
/// 3. Card is marked as used and performance is calculated
/// 4. If all cards used, hand replenishes for next cycle
/// 
/// # Error Handling
/// - `BOOST_CARD_NOT_AVAILABLE`: Selected card already used in current cycle
/// - `INVALID_BOOST_VALUE`: Boost value outside 0-4 range
/// - `CAR_VALIDATION_FAILED`: Invalid car/player combination
#[utoipa::path(
    post,
    path = "/api/v1/races/{race_uuid}/apply-lap",
    params(("race_uuid" = String, Path, description = "Race UUID")),
    request_body(
        content = ApplyLapRequest,
        description = "Lap action request with boost card selection",
        example = json!({
            "player_uuid": "550e8400-e29b-41d4-a716-446655440000",
            "car_uuid": "550e8400-e29b-41d4-a716-446655440001", 
            "boost_value": 3
        })
    ),
    responses(
        (
            status = 200, 
            description = "Lap action processed successfully. Returns updated race status with boost hand state.",
            body = DetailedRaceStatusResponse,
            example = json!({
                "race_progress": {
                    "status": "Ongoing",
                    "current_lap": 2,
                    "total_laps": 5,
                    "participants_count": 4,
                    "finished_participants": 0
                },
                "player_data": {
                    "boost_availability": {
                        "available_cards": [0, 1, 2, 4],
                        "hand_state": {
                            "0": true,
                            "1": true, 
                            "2": true,
                            "3": false,
                            "4": true
                        },
                        "current_cycle": 1,
                        "cycles_completed": 0,
                        "cards_remaining": 4,
                        "next_replenishment_at": 4
                    }
                }
            })
        ),
        (
            status = 400, 
            description = "Boost card validation error or invalid request",
            body = BoostCardErrorResponse,
            examples(
                ("card_not_available" = (
                    summary = "Boost card already used",
                    description = "Player tried to use a boost card that was already used in the current cycle",
                    value = json!({
                        "error_code": "BOOST_CARD_NOT_AVAILABLE",
                        "message": "Boost card 3 is not available. Available cards: [0, 1, 2, 4]",
                        "available_cards": [0, 1, 2, 4],
                        "current_cycle": 1,
                        "cards_remaining": 4
                    })
                )),
                ("invalid_boost_value" = (
                    summary = "Invalid boost value",
                    description = "Player provided boost value outside the valid range (0-4)",
                    value = json!({
                        "error_code": "INVALID_BOOST_VALUE", 
                        "message": "Invalid boost value: 5. Must be between 0 and 4",
                        "available_cards": [0, 1, 2, 3, 4],
                        "current_cycle": 1,
                        "cards_remaining": 5
                    })
                ))
            )
        ),
        (status = 404, description = "Race not found"),
        (status = 409, description = "Cannot process action (race not in progress, etc.)"),
        (status = 500, description = "Internal server error")
    ),
    tag = "boost-cards"
)]
#[tracing::instrument(
    name = "Applying lap action",
    skip(database, payload),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %payload.player_uuid,
        boost_value = payload.boost_value
    )
)]
pub async fn apply_lap_action(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<ApplyLapRequest>,
) -> Result<Json<DetailedRaceStatusResponse>, (StatusCode, Json<BoostCardErrorResponse>)> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err((StatusCode::BAD_REQUEST, Json(BoostCardErrorResponse {
                error_code: "INVALID_UUID".to_string(),
                message: format!("Invalid race UUID: {e}"),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
    };
    
    let player_uuid = match Uuid::parse_str(&payload.player_uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err((StatusCode::BAD_REQUEST, Json(BoostCardErrorResponse {
                error_code: "INVALID_UUID".to_string(),
                message: format!("Invalid player UUID: {e}"),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
    };
    
    let car_uuid = match Uuid::parse_str(&payload.car_uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid car UUID: {}", e);
            return Err((StatusCode::BAD_REQUEST, Json(BoostCardErrorResponse {
                error_code: "INVALID_UUID".to_string(),
                message: format!("Invalid car UUID: {e}"),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
    };
    
    // Validate car data
    let car_data = match CarValidationService::validate_car_for_race(
        &database,
        player_uuid,
        car_uuid,
    ).await {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("Car validation failed: {}", e);
            return Err((StatusCode::BAD_REQUEST, Json(BoostCardErrorResponse {
                error_code: "CAR_VALIDATION_FAILED".to_string(),
                message: format!("Car validation failed: {e}"),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
    };
    
    // Get race to validate boost card before processing
    let race = match get_race_by_uuid(&database, race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((StatusCode::NOT_FOUND, Json(BoostCardErrorResponse {
                error_code: "RACE_NOT_FOUND".to_string(),
                message: "Race not found".to_string(),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
        Err(e) => {
            tracing::error!("Failed to fetch race: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(BoostCardErrorResponse {
                error_code: "DATABASE_ERROR".to_string(),
                message: "Failed to fetch race".to_string(),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
    };
    
    // Find participant and validate boost card
    let participant = race.participants
        .iter()
        .find(|p| p.player_uuid == player_uuid);
    
    if let Some(participant) = participant {
        // Validate boost card selection before processing
        #[allow(clippy::cast_possible_truncation)]
        let boost_value_u8 = payload.boost_value as u8;
        
        if let Err(boost_error) = BoostHandManager::validate_boost_selection(
            &participant.boost_hand,
            boost_value_u8,
        ) {
            tracing::warn!("Boost card validation failed: {}", boost_error);
            let error_response = BoostCardErrorResponse::from_error(
                &boost_error,
                &participant.boost_hand,
            );
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }
    
    // Process individual lap action
    let updated_race = match process_individual_lap_action(
        &database,
        race_uuid,
        player_uuid,
        payload.boost_value,
        &car_data,
    ).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((StatusCode::NOT_FOUND, Json(BoostCardErrorResponse {
                error_code: "RACE_NOT_FOUND".to_string(),
                message: "Race not found".to_string(),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
        Err(e) => {
            tracing::error!("Failed to process lap action: {:?}", e);
            let error_msg = e.to_string();
            
            // Check for boost card errors in the error message
            if error_msg.contains("not available") || error_msg.contains("Invalid boost") {
                return Err((StatusCode::BAD_REQUEST, Json(BoostCardErrorResponse {
                    error_code: "BOOST_CARD_ERROR".to_string(),
                    message: error_msg,
                    available_cards: vec![],
                    current_cycle: 0,
                    cards_remaining: 0,
                })));
            }
            
            if error_msg.contains("not in progress") || error_msg.contains("already submitted") {
                return Err((StatusCode::CONFLICT, Json(BoostCardErrorResponse {
                    error_code: "RACE_STATE_ERROR".to_string(),
                    message: error_msg,
                    available_cards: vec![],
                    current_cycle: 0,
                    cards_remaining: 0,
                })));
            }
            
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(BoostCardErrorResponse {
                error_code: "INTERNAL_ERROR".to_string(),
                message: "Failed to process lap action".to_string(),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
    };
    
    // Return same format as status endpoint with updated boost hand state
    let race_progress = build_race_progress_status(&updated_race);
    let track_situation = match build_track_situation_data(&database, &updated_race).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to build track situation: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(BoostCardErrorResponse {
                error_code: "INTERNAL_ERROR".to_string(),
                message: "Failed to build track situation".to_string(),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
    };
    let race_metadata = build_race_metadata(&updated_race);
    let player_data = match build_player_specific_data(&database, &updated_race, player_uuid).await {
        Ok(data) => Some(data),
        Err(e) => {
            tracing::error!("Failed to build player specific data: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(BoostCardErrorResponse {
                error_code: "INTERNAL_ERROR".to_string(),
                message: "Failed to build player specific data".to_string(),
                available_cards: vec![],
                current_cycle: 0,
                cards_remaining: 0,
            })));
        }
    };
    
    tracing::info!("Lap action processed for player {} in race {}", player_uuid, race_uuid);
    
    Ok(Json(DetailedRaceStatusResponse {
        race_progress,
        track_situation,
        player_data,
        race_metadata,
    }))
}

// Existing endpoint implementations...

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
    created_race.id = result.inserted_id.as_object_id();
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
            "participants": to_bson_safe(&race.participants, "participants")?,
            "pending_actions": to_bson_safe(&race.pending_actions, "pending_actions")?,
            "action_submissions": to_bson_safe(&race.action_submissions, "action_submissions")?,
            "pending_performance_calculations": to_bson_safe(&race.pending_performance_calculations, "pending_performance_calculations")?,
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
            "status": to_bson_safe(&race.status, "status")?,
            "current_lap": race.current_lap,
            "lap_characteristic": to_bson_safe(&race.lap_characteristic, "lap_characteristic")?,
            "pending_actions": to_bson_safe(&race.pending_actions, "pending_actions")?,
            "action_submissions": to_bson_safe(&race.action_submissions, "action_submissions")?,
            "pending_performance_calculations": to_bson_safe(&race.pending_performance_calculations, "pending_performance_calculations")?,
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
            "participants": to_bson_safe(&race.participants, "participants")?,
            "current_lap": race.current_lap,
            "lap_characteristic": to_bson_safe(&race.lap_characteristic, "lap_characteristic")?,
            "status": to_bson_safe(&race.status, "status")?,
            "pending_actions": to_bson_safe(&race.pending_actions, "pending_actions")?,
            "action_submissions": to_bson_safe(&race.action_submissions, "action_submissions")?,
            "pending_performance_calculations": to_bson_safe(&race.pending_performance_calculations, "pending_performance_calculations")?,
            "updated_at": BsonDateTime::now()
        } 
    };
    
    collection.find_one_and_update(filter, update, None).await?;
    
    Ok(Some((lap_result, race.status)))
}





