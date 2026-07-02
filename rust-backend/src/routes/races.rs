use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::boost_hand_manager::{
    BoostAvailability, BoostCardErrorResponse, BoostHandManager,
};
use crate::domain::{
    BoostCycleSummary, BoostUsageRecord, LapAction, LapCharacteristic, LapResult,
    MovementProbability, MovementType, PerformanceCalculation, Race, RaceParticipant, RaceStatus,
    Sector, SectorType, Track, TyreType,
};
use crate::services::car_validation::{CarValidationService, ValidatedCarData};

use crate::app_state::AppState;
use crate::repositories::{PlayerRepository, RaceRepository, RepositoryError, RepositoryResult};

/// Resolve a participant's full car data (car + engine + body + pilot) from the
/// in-memory player aggregate. Returns `None` if any component cannot be found.
fn resolve_car_data(
    player: &crate::domain::Player,
    participant: &crate::domain::RaceParticipant,
) -> Option<ValidatedCarData> {
    let car = player
        .cars
        .iter()
        .find(|c| c.uuid == participant.car_uuid)?;
    let engine_uuid = car.engine_uuid?;
    let body_uuid = car.body_uuid?;
    let engine = player.engines.iter().find(|e| e.uuid == engine_uuid)?;
    let body = player.bodies.iter().find(|b| b.uuid == body_uuid)?;
    let pilot = player
        .pilots
        .iter()
        .find(|p| p.uuid == participant.pilot_uuid)?;

    Some(ValidatedCarData {
        car: car.clone(),
        engine: engine.clone(),
        body: body.clone(),
        pilot: pilot.clone(),
    })
}

/// Build a map of `player_uuid` -> resolved car data for all non-finished
/// participants, reading from the in-memory player repository. Failures to
/// resolve an individual participant are silently skipped.
async fn build_car_data_map(
    repo: &dyn PlayerRepository,
    race: &Race,
) -> HashMap<Uuid, ValidatedCarData> {
    let mut map = HashMap::new();
    for participant in &race.participants {
        if participant.is_finished {
            continue;
        }
        if let Ok(Some(player)) = repo.find_by_uuid(participant.player_uuid).await {
            if let Some(car_data) = resolve_car_data(&player, participant) {
                map.insert(participant.player_uuid, car_data);
            }
        }
    }
    map
}

// ===================== Solo mode (AI opponents) =====================

/// Number of AI opponents seeded for solo races.
const SOLO_BOT_COUNT: usize = 2;

/// A seeded AI opponent's racing identity (player + completed car + primary pilot).
#[derive(Debug, Clone)]
// The `_uuid` suffixes mirror the domain field names and read clearly here.
#[allow(clippy::struct_field_names)]
struct BotIdentity {
    player_uuid: Uuid,
    car_uuid: Uuid,
    pilot_uuid: Uuid,
}

/// Registry of seeded AI opponents, populated once at startup by
/// [`seed_solo_bots`] and read when bootstrapping a solo race.
static SOLO_BOTS: LazyLock<Mutex<Vec<BotIdentity>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Deterministic namespace for deriving stable AI bot player UUIDs, so the
/// same bot index always maps to the same UUID across processes/backends.
const BOT_UUID_NAMESPACE: Uuid = Uuid::from_bytes([
    0xb0, 0x70, 0x00, 0x00, 0xaa, 0xaa, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11,
]);

/// Derive the stable player UUID for AI bot `index`. Using a fixed UUID v5
/// (rather than the random uuid `Player::new_with_assets` would otherwise
/// assign) is what makes re-seeding idempotent against a persistent backend:
/// the same bot always resolves to the same document, so a redeploy upserts
/// in place instead of inserting a duplicate.
fn bot_player_uuid(index: usize) -> Uuid {
    Uuid::new_v5(&BOT_UUID_NAMESPACE, format!("bot{index}").as_bytes())
}

/// Seed AI opponent players (each with a complete, pre-equipped car) into the
/// player repository and record their racing identities.
///
/// Idempotent across process restarts and against a persistent (Mongo)
/// backend: each bot is assigned a stable UUID derived from its index
/// ([`bot_player_uuid`]), and an existing player at that UUID is reused
/// in-place (its identity is recorded, nothing is re-inserted) rather than
/// duplicated. The `SOLO_BOTS` in-process cache still short-circuits repeat
/// calls within the same process.
///
/// Reuses [`crate::routes::auth::create_starter_assets`] so AI cars resolve
/// through the same `build_car_data_map` pipeline as human players.
pub async fn seed_solo_bots(repo: &dyn PlayerRepository) {
    use crate::domain::{Email, Password, Player, TeamName};

    if !SOLO_BOTS.lock().unwrap().is_empty() {
        return;
    }

    let mut identities = Vec::new();

    for i in 0..SOLO_BOT_COUNT {
        let bot_uuid = bot_player_uuid(i);

        // Reuse an already-seeded bot (e.g. from a previous deploy against the
        // same Mongo database) instead of inserting a duplicate.
        match repo.find_by_uuid(bot_uuid).await {
            Ok(Some(existing)) => {
                let Some(car) = existing.cars.first() else {
                    tracing::error!("Existing AI bot {i} has no car; skipping");
                    continue;
                };
                let pilot_count = car.pilot_uuids.len().max(1);
                let pilot_uuid = car
                    .pilot_uuids
                    .get(i % pilot_count)
                    .or_else(|| car.pilot_uuids.first())
                    .copied();
                let Some(pilot_uuid) = pilot_uuid else {
                    tracing::error!("Existing AI bot {i} car has no pilots; skipping");
                    continue;
                };
                identities.push(BotIdentity {
                    player_uuid: existing.uuid,
                    car_uuid: car.uuid,
                    pilot_uuid,
                });
                continue;
            }
            Ok(None) => {} // Not seeded yet; fall through and create it.
            Err(e) => {
                tracing::error!("Failed to look up AI bot {i}: {e:?}");
                continue;
            }
        }

        let (cars, pilots, engines, bodies) = match crate::routes::auth::create_starter_assets() {
            Ok(assets) => assets,
            Err(e) => {
                tracing::error!("Failed to build assets for AI bot {i}: {e}");
                continue;
            }
        };

        let email = match Email::parse(&format!("bot{i}@boardurance.ai")) {
            Ok(email) => email,
            Err(e) => {
                tracing::error!("Invalid bot email for bot {i}: {e}");
                continue;
            }
        };
        let password_hash = match Password::new(format!("BotPassword{i}!")).and_then(|p| p.hash()) {
            Ok(hash) => hash,
            Err(e) => {
                tracing::error!("Failed to hash bot password for bot {i}: {e}");
                continue;
            }
        };
        let team_name = match TeamName::parse(&format!("AI Racer {}", i + 1)) {
            Ok(name) => name,
            Err(e) => {
                tracing::error!("Invalid bot team name for bot {i}: {e}");
                continue;
            }
        };

        let mut player = match Player::new_with_assets(
            email,
            password_hash,
            team_name,
            cars,
            pilots,
            engines,
            bodies,
        ) {
            Ok(player) => player,
            Err(e) => {
                tracing::error!("Failed to build AI bot player {i}: {e}");
                continue;
            }
        };
        // Override the randomly-generated uuid with the bot's stable identity.
        player.uuid = bot_uuid;

        // The first car is completed by `new_with_assets`. Vary opponents by
        // assigning each a different primary pilot from that car's roster.
        let Some(car) = player.cars.first() else {
            continue;
        };
        let pilot_count = car.pilot_uuids.len().max(1);
        let pilot_uuid = car
            .pilot_uuids
            .get(i % pilot_count)
            .or_else(|| car.pilot_uuids.first())
            .copied();
        let (Some(pilot_uuid), car_uuid) = (pilot_uuid, car.uuid) else {
            tracing::error!("AI bot {i} car has no pilots; skipping");
            continue;
        };

        // Another process/replica may have just won the race to create this
        // bot; a Conflict on the (unique) email means it already exists, so
        // fetch it instead of treating this as a hard failure.
        let created_uuid = match repo.create(&player).await {
            Ok(created) => created.uuid,
            Err(crate::repositories::RepositoryError::Conflict(_)) => {
                if let Ok(Some(existing)) = repo.find_by_email(player.email.as_ref()).await {
                    existing.uuid
                } else {
                    tracing::error!("AI bot {i} conflicted on create but could not be re-fetched");
                    continue;
                }
            }
            Err(e) => {
                tracing::error!("Failed to seed AI bot player {i}: {e:?}");
                continue;
            }
        };

        identities.push(BotIdentity {
            player_uuid: created_uuid,
            car_uuid,
            pilot_uuid,
        });
    }

    tracing::info!("Seeded {} AI opponent(s) for solo mode", identities.len());
    *SOLO_BOTS.lock().unwrap() = identities;
}

/// Build the standard solo-mode track. Sector ceilings are tuned so a
/// starter-grade car (base ~17-20) advances one sector with any boost >= 1 and
/// never drops back, so a solo race always reaches completion.
fn build_solo_track() -> Result<Track, String> {
    let sectors = vec![
        Sector {
            id: 0,
            name: "Start".to_string(),
            min_value: 0,
            max_value: 10,
            slot_capacity: None,
            sector_type: SectorType::Start,
        },
        Sector {
            id: 1,
            name: "Straight".to_string(),
            min_value: 5,
            max_value: 14,
            slot_capacity: None,
            sector_type: SectorType::Straight,
        },
        Sector {
            id: 2,
            name: "Curve".to_string(),
            min_value: 5,
            max_value: 14,
            slot_capacity: None,
            sector_type: SectorType::Curve,
        },
        Sector {
            id: 3,
            name: "Finish".to_string(),
            min_value: 8,
            max_value: 16,
            slot_capacity: None,
            sector_type: SectorType::Finish,
        },
    ];

    Track::new("Solo Circuit".to_string(), sectors)
}

/// Request to start a solo race. Only the human player's UUID is required; the
/// backend picks the player's first complete car and fills the rest of the grid
/// with seeded AI opponents.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSoloRaceRequest {
    pub player_uuid: String,
    /// Starting tyre for the human player, which defines the boost pool.
    /// Defaults to Medium.
    #[serde(default)]
    pub tyre_type: TyreType,
}

/// Start a solo race: the human player plus seeded AI opponents, auto-started
/// and ready to play. The backend selects the player's first complete car and
/// its primary pilot, then fills the remaining grid with AI bots.
#[utoipa::path(
    post,
    path = "/races/solo",
    request_body = CreateSoloRaceRequest,
    responses(
        (status = 201, description = "Solo race created and started", body = RaceResponse),
        (status = 400, description = "Invalid player UUID"),
        (status = 404, description = "Player or a complete car not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "races"
)]
#[tracing::instrument(name = "Creating a solo race", skip(state, payload))]
pub async fn create_solo_race(
    State(state): State<AppState>,
    Json(payload): Json<CreateSoloRaceRequest>,
) -> Result<(StatusCode, Json<RaceResponse>), StatusCode> {
    let player_uuid = Uuid::parse_str(&payload.player_uuid).map_err(|e| {
        tracing::warn!("Invalid player UUID: {e}");
        StatusCode::BAD_REQUEST
    })?;

    // Resolve the human's first complete car and its primary pilot.
    let player = match state.player_repository.find_by_uuid(player_uuid).await {
        Ok(Some(player)) => player,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to load player {player_uuid}: {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let Some(car) = player.cars.iter().find(|c| c.is_complete()) else {
        tracing::warn!("Player {player_uuid} has no complete car for a solo race");
        return Err(StatusCode::NOT_FOUND);
    };
    let Some(&pilot_uuid) = car.pilot_uuids.first() else {
        return Err(StatusCode::NOT_FOUND);
    };
    let human_car_uuid = car.uuid;

    // Build the track and the auto-started race.
    let track = build_solo_track().map_err(|e| {
        tracing::error!("Invalid solo track: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    // One turn = one lap; the race runs for this many turns/laps.
    let mut race = Race::new("Solo Race".to_string(), track, 5);
    race.status = RaceStatus::InProgress;
    race.lap_characteristic = LapCharacteristic::Straight;
    race.current_lap = 1;

    // Add the human (with their chosen tyre), then the seeded AI opponents.
    if let Err(e) =
        race.add_participant_with_tyre(player_uuid, human_car_uuid, pilot_uuid, payload.tyre_type)
    {
        tracing::error!("Failed to add human to solo race: {e}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let bots = SOLO_BOTS.lock().unwrap().clone();
    // Give AI opponents varied tyre strategies (cycling Soft/Medium/Hard) so the
    // grid isn't homogeneous.
    let ai_tyres = [TyreType::Soft, TyreType::Medium, TyreType::Hard];
    for (i, bot) in bots.into_iter().enumerate() {
        let tyre = ai_tyres[i % ai_tyres.len()];
        if let Err(e) =
            race.add_ai_participant_with_tyre(bot.player_uuid, bot.car_uuid, bot.pilot_uuid, tyre)
        {
            tracing::warn!("Failed to add AI opponent to solo race: {e}");
        }
    }

    // Normalize the grid: everyone starts at the line in a stable order.
    // Positions are 0-based (matching `sort_participants_in_sectors` and the UI's
    // slot 1 = position 0), so the front slot is filled from the very first turn.
    for (index, participant) in race.participants.iter_mut().enumerate() {
        participant.current_sector = 0;
        participant.current_position_in_sector = index as u32;
    }

    if let Err(e) = state.race_repository.save(&race).await {
        tracing::error!("Failed to persist new solo race {}: {e:?}", race.uuid);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    tracing::info!(
        "Solo race {} started with {} participants",
        race.uuid,
        race.participants.len()
    );

    Ok((
        StatusCode::CREATED,
        Json(RaceResponse {
            race,
            message: "Solo race created and started".to_string(),
        }),
    ))
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

/// Request to submit a single player's turn action
#[derive(Debug, Deserialize, ToSchema)]
pub struct SubmitTurnActionRequest {
    pub player_uuid: String,
    pub boost_value: u32,
}

/// Response after submitting a turn action
#[derive(Debug, Serialize, ToSchema)]
pub struct SubmitTurnActionResponse {
    pub success: bool,
    pub message: String,
    pub turn_phase: String, // "WaitingForPlayers", "Processing", "TurnProcessed"
    pub players_submitted: u32,
    pub total_players: u32,
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
    /// Starting tyre, which defines the boost card pool. Defaults to Medium.
    #[serde(default)]
    pub tyre_type: TyreType,
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
    pub player_uuid: Option<String>,   // For player-specific data
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
/// Final performance = `min(base_performance, sector.max_value)` + `boost_value`
/// (additive model, matching lap resolution). `boost_value` ranges 0-4.
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

/// Request to perform a pit stop.
///
/// A pit stop refills the player's boost card pool and consumes the turn as a
/// free boost-0 lap. Optionally switches to a new tyre type (which defines the
/// refilled pool); if omitted, the current tyre is kept.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PitStopRequest {
    /// UUID of the player pitting
    pub player_uuid: String,
    /// UUID of the car being used
    pub car_uuid: String,
    /// Optional new tyre to fit during the pit stop. Defaults to the current tyre.
    #[serde(default)]
    pub new_tyre: Option<TyreType>,
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
    TurnProcessed,
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
    pub boost_usage_history: Vec<BoostUsageRecord>,
    pub boost_cycle_summaries: Vec<BoostCycleSummary>,
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

// Car Data Endpoint Response Models

#[derive(Debug, Serialize, ToSchema)]
pub struct CarDataResponse {
    pub car: CarInfo,
    pub pilot: PilotInfo,
    pub engine: EngineInfo,
    pub body: BodyInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CarInfo {
    pub uuid: String,
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PilotInfo {
    pub uuid: String,
    pub name: String,
    pub pilot_class: String,
    pub rarity: String,
    pub skills: PilotSkills,
    pub performance: PilotPerformance,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PilotSkills {
    pub reaction_time: u8,
    pub precision: u8,
    pub focus: u8,
    pub stamina: u8,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PilotPerformance {
    pub straight_value: u8,
    pub curve_value: u8,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EngineInfo {
    pub uuid: String,
    pub name: String,
    pub rarity: String,
    pub straight_value: u8,
    pub curve_value: u8,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BodyInfo {
    pub uuid: String,
    pub name: String,
    pub rarity: String,
    pub straight_value: u8,
    pub curve_value: u8,
}

// Turn Phase Endpoint Response Models

#[derive(Debug, Serialize, ToSchema)]
pub struct TurnPhaseResponse {
    pub turn_phase: String, // "WaitingForPlayers", "AllSubmitted", "Processing", "TurnProcessed", "Complete"
    pub current_lap: u32,
    pub total_laps: u32,
    pub lap_characteristic: String,
    pub submitted_players: Vec<String>, // UUIDs
    pub pending_players: Vec<String>,   // UUIDs
    pub total_active_players: u32,
}

// Local View Endpoint Response Models

#[derive(Debug, Serialize, ToSchema)]
pub struct LocalViewResponse {
    pub center_sector: u32,
    /// Total number of sectors on the track. Lets the UI number sectors with
    /// the leading (highest-index) sector as "Sector 1".
    pub total_sectors: u32,
    pub visible_sectors: Vec<SectorInfo>,
    pub visible_participants: Vec<ParticipantInfo>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SectorInfo {
    pub id: u32,
    pub name: String,
    pub min_value: u32,
    pub max_value: u32,
    pub slot_capacity: Option<u32>,
    pub sector_type: String,
    pub current_occupancy: u32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ParticipantInfo {
    pub player_uuid: String,
    pub player_name: Option<String>,
    pub car_name: String,
    pub current_sector: u32,
    pub position_in_sector: u32,
    pub total_value: u32,
    pub current_lap: u32,
    pub is_finished: bool,
}

// Performance Preview Endpoint Response Models

#[derive(Debug, Serialize, ToSchema)]
pub struct PerformancePreviewResponse {
    pub base_performance: BasePerformance,
    pub boost_options: Vec<BoostOption>,
    pub boost_cycle_info: BoostCycleInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BasePerformance {
    pub engine_contribution: u32,
    pub body_contribution: u32,
    pub pilot_contribution: u32,
    pub base_value: u32,
    pub sector_ceiling: u32,
    pub capped_base_value: u32,
    pub lap_characteristic: String, // "Straight" or "Curve"
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoostOption {
    pub boost_value: u8,
    pub is_available: bool,
    pub final_value: u32,
    pub movement_probability: String, // "MoveUp", "Stay", "MoveDown"
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoostCycleInfo {
    pub tyre_type: TyreType,
    pub pit_stops_completed: u32,
    pub cards_remaining: u32,
    pub available_cards: Vec<u8>,
}

// Boost Availability Endpoint Response Models

#[derive(Debug, Serialize, ToSchema)]
pub struct BoostAvailabilityResponse {
    pub available_cards: Vec<u8>,
    /// Remaining count per boost card value (keys "1".."4").
    pub hand_state: std::collections::HashMap<String, u32>,
    pub tyre_type: TyreType,
    pub pit_stops_completed: u32,
    pub cards_remaining: u32,
}

// Lap History Endpoint Response Models

#[derive(Debug, Serialize, ToSchema)]
pub struct LapHistoryResponse {
    pub laps: Vec<LapRecord>,
    pub cycle_summaries: Vec<CycleSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LapRecord {
    pub lap_number: u32,
    pub lap_characteristic: String,
    pub boost_used: u8,
    pub boost_cycle: u32,
    pub base_value: u32,
    pub final_value: u32,
    pub from_sector: u32,
    pub to_sector: u32,
    pub movement_type: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CycleSummary {
    pub cycle_number: u32,
    pub cards_used: Vec<u8>,
    pub laps_in_cycle: Vec<u32>,
    pub average_boost: f32,
}

// Error Response Model

/// Standard error response format used across all endpoints
///
/// This struct provides a consistent error response format with:
/// - error: A short error code or category
/// - message: A human-readable error message
/// - details: Optional additional information about the error
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

// Helper Functions

/// Calculate movement probability based on performance value and sector thresholds.
///
/// Delegates to [`crate::domain::ai_player::classify_movement`] so the preview
/// shares a single source of truth with lap resolution and the AI brain:
/// `MoveUp` if `final > max`, `MoveDown` if `final < min`, otherwise `Stay`.
///
/// # Arguments
/// * `final_value` - The final performance value after boost
/// * `sector` - The current sector with min/max thresholds
fn calculate_movement_probability(final_value: u32, sector: &Sector) -> MovementProbability {
    crate::domain::ai_player::classify_movement(final_value, sector)
}

/// Calculate visible sector IDs for local view (center Â±2 sectors)
///
/// This function handles circular track wrapping by using modulo arithmetic.
/// For a track with N sectors, it returns 5 sector IDs centered on the given sector.
///
/// # Arguments
/// * `center` - The center sector ID (player's current sector)
/// * `total_sectors` - Total number of sectors in the track
///
/// # Returns
/// * `Vec<u32>` - Vector of 5 sector IDs in order (center-2, center-1, center, center+1, center+2)
///
/// # Examples
/// ```
/// // For a track with 10 sectors, player at sector 1:
/// // Returns [9, 0, 1, 2, 3] (wraps around at track boundaries)
/// let visible = get_visible_sector_ids(1, 10);
/// ```
fn get_visible_sector_ids(center: u32, total_sectors: usize) -> Vec<u32> {
    let mut ids = Vec::new();
    #[allow(clippy::cast_possible_truncation)]
    let total = total_sectors as i32;

    // Calculate center Â±2 sectors with proper wrapping
    for offset in -2..=2 {
        let sector_id = (center as i32 + offset).rem_euclid(total) as u32;
        ids.push(sector_id);
    }

    ids
}

pub fn routes() -> Router<AppState> {
    Router::new()
        // Public routes (no authentication required)
        .route("/races", get(get_all_races))
        .route("/races/:race_uuid", get(get_race))
        .route("/races/:race_uuid/status", get(get_race_status))
        // Enhanced API endpoints
        .route("/races/:race_uuid/register", post(register_player))
        .route(
            "/races/:race_uuid/status-detailed",
            get(get_race_status_detailed),
        )
        .route("/races/:race_uuid/apply-lap", post(apply_lap_action))
        // New player-specific endpoints
        .route(
            "/races/:race_uuid/players/:player_uuid/local-view",
            get(get_local_view),
        )
        .route(
            "/races/:race_uuid/players/:player_uuid/boost-availability",
            get(get_boost_availability),
        )
        .route(
            "/races/:race_uuid/players/:player_uuid/lap-history",
            get(get_lap_history),
        )
        // Race-level endpoint
        .route("/races/:race_uuid/turn-phase", get(get_turn_phase))
        // Protected routes - These should be protected with AuthMiddleware
        // TODO: Apply middleware layers in startup.rs:
        // 1. AuthMiddleware to validate JWT tokens and extract UserContext
        // 2. Custom validation for race participation/ownership
        // Routes that require authentication:
        .route("/races", post(create_race)) // Any authenticated user can create
        .route("/races/:race_uuid/join", post(join_race)) // Any authenticated user can join
        // Routes that require race ownership or admin role:
        .route("/races/:race_uuid/start", post(start_race)) // Race creator or admin
}

/// Turn-processing routes that need access to the in-memory player repository
/// (via `AppState`) to compute real movement from each car's stats.
pub fn turn_routes() -> Router<AppState> {
    Router::new()
        .route("/races/solo", post(create_solo_race))
        .route("/races/:race_uuid/submit-action", post(submit_turn_action))
        .route("/races/:race_uuid/pit", post(pit_stop_action))
        .route("/races/:race_uuid/turn", post(process_turn))
        .route(
            "/races/:race_uuid/players/:player_uuid/car-data",
            get(get_car_data),
        )
        .route(
            "/races/:race_uuid/players/:player_uuid/performance-preview",
            get(get_performance_preview),
        )
}

// Helper Functions for Enhanced API

async fn register_player_in_race(
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
    player_uuid: Uuid,
    car_uuid: Uuid,
    pilot_uuid: Uuid,
    tyre_type: TyreType,
) -> RepositoryResult<Option<Race>> {
    // Get the race first
    let Some(mut race) = get_race_by_uuid(race_repository, race_uuid).await? else {
        return Ok(None);
    };

    // Try to add participant with their chosen starting tyre
    if let Err(e) = race.add_participant_with_tyre(player_uuid, car_uuid, pilot_uuid, tyre_type) {
        return Err(RepositoryError::Validation(e));
    }

    // Persist the mutated race and return it.
    race_repository.save(&race).await?;
    Ok(Some(race))
}

fn get_player_race_position(race: &Race, player_uuid: Uuid) -> Result<PlayerRacePosition, String> {
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or("Player not found in race")?;

    // Calculate qualification rank based on starting sector and position
    let mut qualification_rank = 1u32;
    for other_participant in &race.participants {
        if other_participant.player_uuid != player_uuid
            && (other_participant.current_sector > participant.current_sector
                || (other_participant.current_sector == participant.current_sector
                    && other_participant.current_position_in_sector
                        < participant.current_position_in_sector))
        {
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
            message: "Race was cancelled".to_string(),
        },
    };

    #[allow(clippy::cast_possible_truncation)]
    let finished_participants = race.participants.iter().filter(|p| p.is_finished).count() as u32;

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
    race: &Race,
) -> Result<TrackSituationData, mongodb::error::Error> {
    let mut sectors = Vec::new();

    // Build sector situation for each sector
    for sector in &race.track.sectors {
        let participants_in_sector: Vec<_> = race
            .participants
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
            available_slots: sector
                .slot_capacity
                .map(|cap| cap.saturating_sub(sector_participants.len() as u32)),
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
        b.current_sector
            .cmp(&a.current_sector)
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
        start_time: if race.status == RaceStatus::InProgress || race.status == RaceStatus::Finished
        {
            Some(
                DateTime::from_timestamp(race.created_at.timestamp_millis() / 1000, 0)
                    .unwrap_or_default(),
            )
        } else {
            None
        },
        estimated_completion: None, // TODO: Calculate based on current progress
        total_turns: 0,             // TODO: Implement turn tracking
    }
}

#[allow(clippy::unused_async)]
async fn build_player_specific_data(
    race: &Race,
    player_uuid: Uuid,
) -> Result<PlayerSpecificData, mongodb::error::Error> {
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or_else(|| mongodb::error::Error::custom("Player not found in race"))?;

    if participant.is_finished {
        return Err(mongodb::error::Error::custom(
            "Player has finished the race",
        ));
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
        if other_participant.player_uuid != player_uuid
            && !other_participant.is_finished
            && (other_participant.current_sector > participant.current_sector
                || (other_participant.current_sector == participant.current_sector
                    && other_participant.current_position_in_sector
                        < participant.current_position_in_sector))
        {
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
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
    player_uuid: Uuid,
    boost_value: u32,
    car_data: &ValidatedCarData,
) -> RepositoryResult<Option<Race>> {
    // Get the race first
    let Some(mut race) = get_race_by_uuid(race_repository, race_uuid).await? else {
        return Ok(None);
    };

    // Process individual lap action using the new method
    match race.process_individual_lap_action(player_uuid, boost_value, car_data) {
        Ok(_individual_result) => {
            // Persist the mutated race and return it.
            race_repository.save(&race).await?;
            Ok(Some(race))
        }
        Err(e) => Err(RepositoryError::Validation(e)),
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
    skip(state, payload),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %payload.player_uuid,
        car_uuid = %payload.car_uuid
    )
)]
pub async fn register_player(
    State(state): State<AppState>,
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
        state.player_repository.as_ref(),
        player_uuid,
        car_uuid,
    )
    .await
    {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("Car validation failed: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // 3. Register player in race
    let updated_race = match register_player_in_race(
        &state.race_repository,
        race_uuid,
        player_uuid,
        car_uuid,
        car_data.pilot.uuid,
        payload.tyre_type,
    )
    .await
    {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to register player: {:?}", e);
            if e.to_string().contains("already participating")
                || e.to_string().contains("already started")
            {
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

    tracing::info!(
        "Player {} successfully registered for race {}",
        player_uuid,
        race_uuid
    );

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
    skip(state),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = ?params.player_uuid
    )
)]
pub async fn get_race_status_detailed(
    State(state): State<AppState>,
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

    let race = match get_race_by_uuid(&state.race_repository, race_uuid).await {
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
    let track_situation = match build_track_situation_data(&race).await {
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

        match build_player_specific_data(&race, player_uuid).await {
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
/// - Boost cards add to performance: `min(base_value, sector.max_value)` + `boost_value`
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
    skip(state, payload),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %payload.player_uuid,
        boost_value = payload.boost_value
    )
)]
pub async fn apply_lap_action(
    State(state): State<AppState>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<ApplyLapRequest>,
) -> Result<Json<DetailedRaceStatusResponse>, (StatusCode, Json<BoostCardErrorResponse>)> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(BoostCardErrorResponse {
                    error_code: "INVALID_UUID".to_string(),
                    message: format!("Invalid race UUID: {e}"),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
    };

    let player_uuid = match Uuid::parse_str(&payload.player_uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid player UUID: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(BoostCardErrorResponse {
                    error_code: "INVALID_UUID".to_string(),
                    message: format!("Invalid player UUID: {e}"),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
    };

    let car_uuid = match Uuid::parse_str(&payload.car_uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid car UUID: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(BoostCardErrorResponse {
                    error_code: "INVALID_UUID".to_string(),
                    message: format!("Invalid car UUID: {e}"),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
    };

    // Validate car data
    let car_data = match CarValidationService::validate_car_for_race(
        state.player_repository.as_ref(),
        player_uuid,
        car_uuid,
    )
    .await
    {
        Ok(data) => data,
        Err(e) => {
            tracing::warn!("Car validation failed: {}", e);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(BoostCardErrorResponse {
                    error_code: "CAR_VALIDATION_FAILED".to_string(),
                    message: format!("Car validation failed: {e}"),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
    };

    // Get race to validate boost card before processing
    let race = match get_race_by_uuid(&state.race_repository, race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((
                StatusCode::NOT_FOUND,
                Json(BoostCardErrorResponse {
                    error_code: "RACE_NOT_FOUND".to_string(),
                    message: "Race not found".to_string(),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to fetch race: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoostCardErrorResponse {
                    error_code: "DATABASE_ERROR".to_string(),
                    message: "Failed to fetch race".to_string(),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
    };

    // Find participant and validate boost card
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid);

    if let Some(participant) = participant {
        // Validate boost card selection before processing
        #[allow(clippy::cast_possible_truncation)]
        let boost_value_u8 = payload.boost_value as u8;

        if let Err(boost_error) =
            BoostHandManager::validate_boost_selection(&participant.boost_hand, boost_value_u8)
        {
            tracing::warn!("Boost card validation failed: {}", boost_error);
            let error_response =
                BoostCardErrorResponse::from_error(&boost_error, &participant.boost_hand);
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    }

    // Process individual lap action
    let updated_race = match process_individual_lap_action(
        &state.race_repository,
        race_uuid,
        player_uuid,
        payload.boost_value,
        &car_data,
    )
    .await
    {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((
                StatusCode::NOT_FOUND,
                Json(BoostCardErrorResponse {
                    error_code: "RACE_NOT_FOUND".to_string(),
                    message: "Race not found".to_string(),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to process lap action: {:?}", e);
            let error_msg = e.to_string();

            // Check for boost card errors in the error message
            if error_msg.contains("not available") || error_msg.contains("Invalid boost") {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(BoostCardErrorResponse {
                        error_code: "BOOST_CARD_ERROR".to_string(),
                        message: error_msg,
                        available_cards: vec![],
                        pit_stops_completed: 0,
                        cards_remaining: 0,
                    }),
                ));
            }

            if error_msg.contains("not in progress") || error_msg.contains("already submitted") {
                return Err((
                    StatusCode::CONFLICT,
                    Json(BoostCardErrorResponse {
                        error_code: "RACE_STATE_ERROR".to_string(),
                        message: error_msg,
                        available_cards: vec![],
                        pit_stops_completed: 0,
                        cards_remaining: 0,
                    }),
                ));
            }

            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoostCardErrorResponse {
                    error_code: "INTERNAL_ERROR".to_string(),
                    message: "Failed to process lap action".to_string(),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
    };

    // Return same format as status endpoint with updated boost hand state
    let race_progress = build_race_progress_status(&updated_race);
    let track_situation = match build_track_situation_data(&updated_race).await {
        Ok(data) => data,
        Err(e) => {
            tracing::error!("Failed to build track situation: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoostCardErrorResponse {
                    error_code: "INTERNAL_ERROR".to_string(),
                    message: "Failed to build track situation".to_string(),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
    };
    let race_metadata = build_race_metadata(&updated_race);
    let player_data = match build_player_specific_data(&updated_race, player_uuid).await {
        Ok(data) => Some(data),
        Err(e) => {
            tracing::error!("Failed to build player specific data: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BoostCardErrorResponse {
                    error_code: "INTERNAL_ERROR".to_string(),
                    message: "Failed to build player specific data".to_string(),
                    available_cards: vec![],
                    pit_stops_completed: 0,
                    cards_remaining: 0,
                }),
            ));
        }
    };

    tracing::info!(
        "Lap action processed for player {} in race {}",
        player_uuid,
        race_uuid
    );

    Ok(Json(DetailedRaceStatusResponse {
        race_progress,
        track_situation,
        player_data,
        race_metadata,
    }))
}

/// Perform a pit stop for a player
///
/// Refills the player's boost card pool (optionally switching tyre) and
/// consumes the turn as a free boost-0 lap.
#[utoipa::path(
    post,
    path = "/api/v1/races/{race_uuid}/pit",
    request_body = PitStopRequest,
    params(("race_uuid" = String, Path, description = "Race UUID")),
    responses(
        (status = 200, description = "Pit stop applied", body = DetailedRaceStatusResponse),
        (status = 400, description = "Invalid request", body = BoostCardErrorResponse),
        (status = 404, description = "Race not found", body = BoostCardErrorResponse),
        (status = 409, description = "Invalid race state", body = BoostCardErrorResponse)
    ),
    tag = "races"
)]
pub async fn pit_stop_action(
    State(state): State<AppState>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<PitStopRequest>,
) -> Result<Json<DetailedRaceStatusResponse>, (StatusCode, Json<BoostCardErrorResponse>)> {
    fn err(
        status: StatusCode,
        code: &str,
        message: String,
    ) -> (StatusCode, Json<BoostCardErrorResponse>) {
        (
            status,
            Json(BoostCardErrorResponse {
                error_code: code.to_string(),
                message,
                available_cards: vec![],
                pit_stops_completed: 0,
                cards_remaining: 0,
            }),
        )
    }

    let race_uuid = Uuid::parse_str(&race_uuid_str).map_err(|e| {
        err(
            StatusCode::BAD_REQUEST,
            "INVALID_UUID",
            format!("Invalid race UUID: {e}"),
        )
    })?;
    let player_uuid = Uuid::parse_str(&payload.player_uuid).map_err(|e| {
        err(
            StatusCode::BAD_REQUEST,
            "INVALID_UUID",
            format!("Invalid player UUID: {e}"),
        )
    })?;

    // Build the car-data map for ALL participants from the in-memory player
    // repository — the same source `/submit-action` uses. This lets the pit path
    // enqueue the AI opponents (and resolve the human's car stats) without a DB
    // round-trip. Previously the pit path never enqueued AI, so a solo turn
    // deadlocked waiting on opponents that never submitted.
    let car_data_map = match state.race_repository.find_by_uuid(race_uuid).await {
        Ok(Some(race)) => build_car_data_map(state.player_repository.as_ref(), &race).await,
        Ok(None) => {
            return Err(err(
                StatusCode::NOT_FOUND,
                "RACE_NOT_FOUND",
                "Race not found".to_string(),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to fetch race {race_uuid}: {e:?}");
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Failed to fetch race".to_string(),
            ));
        }
    };

    let updated_race = match resolve_human_turn(
        &state.race_repository,
        race_uuid,
        player_uuid,
        TurnIntent::Pit(payload.new_tyre),
        &car_data_map,
    )
    .await
    {
        Ok(Some(race)) => race,
        Ok(None) => {
            return Err(err(
                StatusCode::NOT_FOUND,
                "RACE_NOT_FOUND",
                "Race not found".to_string(),
            ))
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("not in progress")
                || error_msg.contains("already submitted")
                || error_msg.contains("already finished")
            {
                return Err(err(StatusCode::CONFLICT, "RACE_STATE_ERROR", error_msg));
            }
            if error_msg.contains("not found") {
                return Err(err(StatusCode::NOT_FOUND, "PLAYER_NOT_FOUND", error_msg));
            }
            return Err(err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Failed to process pit stop".to_string(),
            ));
        }
    };

    let race_progress = build_race_progress_status(&updated_race);
    let track_situation = build_track_situation_data(&updated_race)
        .await
        .map_err(|e| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                format!("Failed to build track situation: {e}"),
            )
        })?;
    let race_metadata = build_race_metadata(&updated_race);
    let player_data = build_player_specific_data(&updated_race, player_uuid)
        .await
        .ok();

    tracing::info!(
        "Pit stop processed for player {} in race {}",
        player_uuid,
        race_uuid
    );

    Ok(Json(DetailedRaceStatusResponse {
        race_progress,
        track_situation,
        player_data,
        race_metadata,
    }))
}

// New Car Data Endpoint Implementation

/// Get complete car data for a player in a race
///
/// This endpoint returns comprehensive car information including:
/// - Car details (name, UUID)
/// - Pilot information (skills, performance, class, rarity)
/// - Engine specifications (straight/curve values, rarity)
/// - Body specifications (straight/curve values, rarity)
///
/// The data is retrieved using the `CarValidationService` which ensures
/// the car belongs to the player and has all required components.
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}/players/{player_uuid}/car-data",
    params(
        ("race_uuid" = String, Path, description = "Race UUID"),
        ("player_uuid" = String, Path, description = "Player UUID")
    ),
    responses(
        (
            status = 200,
            description = "Car data retrieved successfully",
            body = CarDataResponse,
            example = json!({
                "car": {
                    "uuid": "550e8400-e29b-41d4-a716-446655440001",
                    "name": "Lightning Bolt"
                },
                "pilot": {
                    "uuid": "550e8400-e29b-41d4-a716-446655440002",
                    "name": "Max Speed",
                    "pilot_class": "SpeedDemon",
                    "rarity": "Professional",
                    "skills": {
                        "reaction_time": 8,
                        "precision": 7,
                        "focus": 6,
                        "stamina": 7
                    },
                    "performance": {
                        "straight_value": 5,
                        "curve_value": 4
                    }
                },
                "engine": {
                    "uuid": "550e8400-e29b-41d4-a716-446655440003",
                    "name": "V8 Turbo",
                    "rarity": "Rare",
                    "straight_value": 8,
                    "curve_value": 6
                },
                "body": {
                    "uuid": "550e8400-e29b-41d4-a716-446655440004",
                    "name": "Aerodynamic Frame",
                    "rarity": "Rare",
                    "straight_value": 7,
                    "curve_value": 8
                }
            })
        ),
        (
            status = 400,
            description = "Invalid UUID format",
            body = ErrorResponse,
            example = json!({
                "error": "INVALID_UUID",
                "message": "Invalid UUID format",
                "details": null
            })
        ),
        (
            status = 404,
            description = "Player not found in race",
            body = ErrorResponse,
            example = json!({
                "error": "PLAYER_NOT_FOUND",
                "message": "Player not found in race",
                "details": null
            })
        ),
        (
            status = 500,
            description = "Internal server error",
            body = ErrorResponse,
            example = json!({
                "error": "DATABASE_ERROR",
                "message": "Internal server error",
                "details": "Failed to fetch race: connection timeout"
            })
        )
    ),
    tag = "races"
)]
#[tracing::instrument(
    name = "Getting car data for player in race",
    skip(state),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %player_uuid_str
    )
)]
pub async fn get_car_data(
    State(state): State<AppState>,
    Path((race_uuid_str, player_uuid_str)): Path<(String, String)>,
) -> Result<Json<CarDataResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Parse and validate UUIDs
    let race_uuid = if let Ok(uuid) = Uuid::parse_str(&race_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid race UUID format: {}", race_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    let player_uuid = if let Ok(uuid) = Uuid::parse_str(&player_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid player UUID format: {}", player_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    // 2. Fetch race and find participant
    let race = match state.race_repository.find_by_uuid(race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "RACE_NOT_FOUND".to_string(),
                    message: "Race not found".to_string(),
                    details: None,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to fetch race {race_uuid}: {e:?}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Internal server error".to_string(),
                    details: Some(format!("Failed to fetch race: {e}")),
                }),
            ));
        }
    };

    // 3. Find participant by player_uuid
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or_else(|| {
            tracing::warn!("Player {} not found in race {}", player_uuid, race_uuid);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "PLAYER_NOT_FOUND".to_string(),
                    message: "Player not found in race".to_string(),
                    details: None,
                }),
            )
        })?;

    // 4. Resolve car data from the in-memory player repository
    let player = if let Ok(Some(p)) = state.player_repository.find_by_uuid(player_uuid).await {
        p
    } else {
        tracing::warn!("Player {} not found in repository", player_uuid);
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "PLAYER_NOT_FOUND".to_string(),
                message: "Player not found in race".to_string(),
                details: None,
            }),
        ));
    };

    let car_data = if let Some(cd) = resolve_car_data(&player, participant) {
        cd
    } else {
        tracing::warn!("Car components not found for player {}", player_uuid);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "CAR_VALIDATION_FAILED".to_string(),
                message: "Car components not found".to_string(),
                details: None,
            }),
        ));
    };

    // 5. Transform domain models to API response models
    let response = CarDataResponse {
        car: CarInfo {
            uuid: car_data.car.uuid.to_string(),
            name: car_data.car.name.as_ref().to_string(),
        },
        pilot: PilotInfo {
            uuid: car_data.pilot.uuid.to_string(),
            name: car_data.pilot.name.as_ref().to_string(),
            pilot_class: format!("{:?}", car_data.pilot.pilot_class),
            rarity: format!("{:?}", car_data.pilot.rarity),
            skills: PilotSkills {
                reaction_time: car_data.pilot.skills.reaction_time,
                precision: car_data.pilot.skills.precision,
                focus: car_data.pilot.skills.focus,
                stamina: car_data.pilot.skills.stamina,
            },
            performance: PilotPerformance {
                straight_value: car_data.pilot.performance.straight_value,
                curve_value: car_data.pilot.performance.curve_value,
            },
        },
        engine: EngineInfo {
            uuid: car_data.engine.uuid.to_string(),
            name: car_data.engine.name.as_ref().to_string(),
            rarity: format!("{:?}", car_data.engine.rarity),
            straight_value: car_data.engine.straight_value,
            curve_value: car_data.engine.curve_value,
        },
        body: BodyInfo {
            uuid: car_data.body.uuid.to_string(),
            name: car_data.body.name.as_ref().to_string(),
            rarity: format!("{:?}", car_data.body.rarity),
            straight_value: car_data.body.straight_value,
            curve_value: car_data.body.curve_value,
        },
    };

    tracing::info!(
        "Car data retrieved for player {} in race {}",
        player_uuid,
        race_uuid
    );
    Ok(Json(response))
}

/// Get performance preview for all boost options
///
/// This endpoint calculates and returns performance predictions for all boost card options (0-4).
/// It provides:
/// - Base performance breakdown (engine, body, pilot contributions)
/// - Sector ceiling application
/// - Final performance values for each boost option
/// - Movement probability for each boost option
/// - Boost cycle information (available cards, cycle status)
///
/// The performance calculation follows the additive model used by lap resolution:
/// `final_value = min(base_value, sector.max_value) + boost_value`
///
/// Movement probabilities are determined by comparing final values to sector thresholds:
/// - `MoveUp`: `final_value` > `sector.max_value`
/// - Stay: `sector.min_value` <= `final_value` <= `sector.max_value`
/// - `MoveDown`: `final_value` < `sector.min_value`
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}/players/{player_uuid}/performance-preview",
    params(
        ("race_uuid" = String, Path, description = "Race UUID"),
        ("player_uuid" = String, Path, description = "Player UUID")
    ),
    responses(
        (
            status = 200,
            description = "Performance preview calculated successfully",
            body = PerformancePreviewResponse,
            example = json!({
                "base_performance": {
                    "engine_contribution": 8,
                    "body_contribution": 7,
                    "pilot_contribution": 5,
                    "base_value": 20,
                    "sector_ceiling": 25,
                    "capped_base_value": 20,
                    "lap_characteristic": "Straight"
                },
                "boost_options": [
                    {
                        "boost_value": 0,
                        "is_available": false,
                        "final_value": 20,
                        "movement_probability": "Stay"
                    },
                    {
                        "boost_value": 1,
                        "is_available": true,
                        "final_value": 22,
                        "movement_probability": "Stay"
                    },
                    {
                        "boost_value": 2,
                        "is_available": true,
                        "final_value": 23,
                        "movement_probability": "Stay"
                    },
                    {
                        "boost_value": 3,
                        "is_available": true,
                        "final_value": 25,
                        "movement_probability": "MoveUp"
                    },
                    {
                        "boost_value": 4,
                        "is_available": true,
                        "final_value": 26,
                        "movement_probability": "MoveUp"
                    }
                ],
                "boost_cycle_info": {
                    "current_cycle": 1,
                    "cycles_completed": 0,
                    "cards_remaining": 4,
                    "available_cards": [1, 2, 3, 4]
                }
            })
        ),
        (
            status = 400,
            description = "Invalid UUID format",
            body = ErrorResponse,
            example = json!({
                "error": "INVALID_UUID",
                "message": "Invalid UUID format",
                "details": null
            })
        ),
        (
            status = 404,
            description = "Player not found in race or race not found",
            body = ErrorResponse,
            example = json!({
                "error": "PLAYER_NOT_FOUND",
                "message": "Player not found in race",
                "details": null
            })
        ),
        (
            status = 409,
            description = "Race not in progress or player already finished",
            body = ErrorResponse,
            example = json!({
                "error": "RACE_NOT_IN_PROGRESS",
                "message": "Race is not in progress",
                "details": null
            })
        ),
        (
            status = 500,
            description = "Internal server error",
            body = ErrorResponse,
            example = json!({
                "error": "DATABASE_ERROR",
                "message": "Internal server error",
                "details": "Failed to fetch race: connection timeout"
            })
        )
    ),
    tag = "races"
)]
#[tracing::instrument(
    name = "Getting performance preview for player in race",
    skip(state),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %player_uuid_str
    )
)]
pub async fn get_performance_preview(
    State(state): State<AppState>,
    Path((race_uuid_str, player_uuid_str)): Path<(String, String)>,
) -> Result<Json<PerformancePreviewResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Parse and validate UUIDs
    let race_uuid = if let Ok(uuid) = Uuid::parse_str(&race_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid race UUID format: {}", race_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    let player_uuid = if let Ok(uuid) = Uuid::parse_str(&player_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid player UUID format: {}", player_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    // 2. Fetch race
    let race = match state.race_repository.find_by_uuid(race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "RACE_NOT_FOUND".to_string(),
                    message: "Race not found".to_string(),
                    details: None,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to fetch race {race_uuid}: {e:?}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Internal server error".to_string(),
                    details: Some(format!("Failed to fetch race: {e}")),
                }),
            ));
        }
    };

    // 3. Validate race is in progress
    if race.status != RaceStatus::InProgress {
        tracing::warn!("Race {} is not in progress", race_uuid);
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "RACE_NOT_IN_PROGRESS".to_string(),
                message: "Race is not in progress".to_string(),
                details: None,
            }),
        ));
    }

    // 4. Find participant by player_uuid
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or_else(|| {
            tracing::warn!("Player {} not found in race {}", player_uuid, race_uuid);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "PLAYER_NOT_FOUND".to_string(),
                    message: "Player not found in race".to_string(),
                    details: None,
                }),
            )
        })?;

    // 5. Check if player has already finished
    if participant.is_finished {
        tracing::warn!("Player {} has already finished the race", player_uuid);
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "PLAYER_FINISHED".to_string(),
                message: "Player has already finished the race".to_string(),
                details: None,
            }),
        ));
    }

    // 6. Resolve car data from the in-memory player repository
    let player = if let Ok(Some(p)) = state.player_repository.find_by_uuid(player_uuid).await {
        p
    } else {
        tracing::warn!("Player {} not found in repository", player_uuid);
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "PLAYER_NOT_FOUND".to_string(),
                message: "Player not found in race".to_string(),
                details: None,
            }),
        ));
    };

    let car_data = if let Some(cd) = resolve_car_data(&player, participant) {
        cd
    } else {
        tracing::warn!("Car components not found for player {}", player_uuid);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "CAR_VALIDATION_FAILED".to_string(),
                message: "Car components not found".to_string(),
                details: None,
            }),
        ));
    };

    // 7. Calculate base performance using Race::calculate_performance_with_car_data()
    // We need to create a temporary PerformanceCalculation with boost 0 to get base values
    let current_sector = &race.track.sectors[participant.current_sector as usize];

    // Get performance values based on lap characteristic
    let (engine_contribution, body_contribution, pilot_contribution) = match race.lap_characteristic
    {
        LapCharacteristic::Straight => (
            u32::from(car_data.engine.straight_value),
            u32::from(car_data.body.straight_value),
            u32::from(car_data.pilot.performance.straight_value),
        ),
        LapCharacteristic::Curve => (
            u32::from(car_data.engine.curve_value),
            u32::from(car_data.body.curve_value),
            u32::from(car_data.pilot.performance.curve_value),
        ),
    };

    let base_value = engine_contribution + body_contribution + pilot_contribution;
    let capped_base_value = std::cmp::min(base_value, current_sector.max_value);

    // 8. Build base performance response
    let base_performance = BasePerformance {
        engine_contribution,
        body_contribution,
        pilot_contribution,
        base_value,
        sector_ceiling: current_sector.max_value,
        capped_base_value,
        lap_characteristic: format!("{:?}", race.lap_characteristic),
    };

    // 9. Calculate boost options for each boost card (0-4)
    let mut boost_options = Vec::new();

    for boost_value in 0..=4 {
        let is_available = participant.boost_hand.is_card_available(boost_value);

        // Calculate final value using the additive model that lap resolution
        // uses: final = min(base, max) + boost.
        let final_value = capped_base_value + u32::from(boost_value);

        // Determine movement probability
        let movement_probability = calculate_movement_probability(final_value, current_sector);

        boost_options.push(BoostOption {
            boost_value,
            is_available,
            final_value,
            movement_probability: format!("{movement_probability:?}"),
        });
    }

    // 10. Get boost pool info from participant's boost_hand
    let boost_cycle_info = BoostCycleInfo {
        tyre_type: participant.boost_hand.tyre_type,
        pit_stops_completed: participant.boost_hand.pit_stops_completed,
        cards_remaining: participant.boost_hand.cards_remaining,
        available_cards: participant.boost_hand.get_available_cards(),
    };

    // 11. Return complete preview
    let response = PerformancePreviewResponse {
        base_performance,
        boost_options,
        boost_cycle_info,
    };

    tracing::info!(
        "Performance preview calculated for player {} in race {}",
        player_uuid,
        race_uuid
    );
    Ok(Json(response))
}

/// Get turn phase information for a race
///
/// This endpoint returns the current turn phase state for simultaneous turn resolution.
/// It provides information about:
/// - Current turn phase (`WaitingForPlayers`, `AllSubmitted`, Processing, Complete)
/// - Current lap number and lap characteristic
/// - List of players who have submitted actions
/// - List of players who are still pending action submission
/// - Total number of active players
///
/// Turn phases are determined by:
/// - Complete: Race status is not `InProgress`
/// - `AllSubmitted`: All active participants have submitted actions
/// - `WaitingForPlayers`: Some participants haven't submitted actions yet
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}/turn-phase",
    params(
        ("race_uuid" = String, Path, description = "Race UUID")
    ),
    responses(
        (
            status = 200,
            description = "Turn phase information retrieved successfully",
            body = TurnPhaseResponse,
            example = json!({
                "turn_phase": "WaitingForPlayers",
                "current_lap": 2,
                "lap_characteristic": "Straight",
                "submitted_players": [
                    "550e8400-e29b-41d4-a716-446655440000",
                    "550e8400-e29b-41d4-a716-446655440001"
                ],
                "pending_players": [
                    "550e8400-e29b-41d4-a716-446655440002"
                ],
                "total_active_players": 3
            })
        ),
        (
            status = 400,
            description = "Invalid UUID format",
            body = ErrorResponse,
            example = json!({
                "error": "INVALID_UUID",
                "message": "Invalid UUID format",
                "details": null
            })
        ),
        (
            status = 404,
            description = "Race not found",
            body = ErrorResponse,
            example = json!({
                "error": "RACE_NOT_FOUND",
                "message": "Race not found",
                "details": null
            })
        ),
        (
            status = 500,
            description = "Internal server error",
            body = ErrorResponse,
            example = json!({
                "error": "DATABASE_ERROR",
                "message": "Internal server error",
                "details": "Failed to fetch race: connection timeout"
            })
        )
    ),
    tag = "races"
)]
#[tracing::instrument(
    name = "Getting turn phase for race",
    skip(state),
    fields(
        race_uuid = %race_uuid_str
    )
)]
pub async fn get_turn_phase(
    State(state): State<AppState>,
    Path(race_uuid_str): Path<String>,
) -> Result<Json<TurnPhaseResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Parse and validate UUID
    let race_uuid = if let Ok(uuid) = Uuid::parse_str(&race_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid race UUID format: {}", race_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    // 2. Fetch race from database
    let race = match get_race_by_uuid(&state.race_repository, race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "RACE_NOT_FOUND".to_string(),
                    message: "Race not found".to_string(),
                    details: None,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to fetch race: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Internal server error".to_string(),
                    details: Some(format!("Failed to fetch race: {e}")),
                }),
            ));
        }
    };

    // 3. Determine turn phase using race.all_actions_submitted() and race status
    let turn_phase = if race.status != RaceStatus::InProgress {
        "Complete".to_string()
    } else if race.all_actions_submitted() {
        "AllSubmitted".to_string()
    } else {
        "WaitingForPlayers".to_string()
    };

    // 4. Get submitted players from race.pending_actions
    let submitted_players: Vec<String> = race
        .pending_actions
        .iter()
        .map(|action| action.player_uuid.to_string())
        .collect();

    // 5. Get pending players using race.get_pending_players()
    let pending_players: Vec<String> = race
        .get_pending_players()
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    // 6. Calculate total active players (not finished)
    #[allow(clippy::cast_possible_truncation)]
    let total_active_players = race.participants.iter().filter(|p| !p.is_finished).count() as u32;

    // 7. Return phase information with player lists
    let response = TurnPhaseResponse {
        turn_phase,
        current_lap: race.current_lap,
        total_laps: race.total_laps,
        lap_characteristic: format!("{:?}", race.lap_characteristic),
        submitted_players,
        pending_players,
        total_active_players,
    };

    tracing::info!(
        "Turn phase retrieved for race {}: {}",
        race_uuid,
        response.turn_phase
    );
    Ok(Json(response))
}

/// Get local view for a player in a race
///
/// This endpoint calculates and returns the player's 5-sector local view (current sector Â±2).
/// It provides:
/// - The player's current sector as the center
/// - 5 visible sectors with their details (id, name, thresholds, capacity, type)
/// - All participants within the visible range with their positions
/// - Current occupancy for each visible sector
///
/// The local view handles circular track wrapping automatically, so sectors at the
/// beginning/end of the track are properly included when the player is near track boundaries.
///
/// # Sector Range Calculation
/// For a player at sector N on a track with M sectors:
/// - Visible sectors: [N-2, N-1, N, N+1, N+2] (with modulo wrapping)
/// - Example: Player at sector 1 on 10-sector track sees [9, 0, 1, 2, 3]
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}/players/{player_uuid}/local-view",
    params(
        ("race_uuid" = String, Path, description = "Race UUID"),
        ("player_uuid" = String, Path, description = "Player UUID")
    ),
    responses(
        (
            status = 200,
            description = "Local view calculated successfully",
            body = LocalViewResponse,
            example = json!({
                "center_sector": 5,
                "visible_sectors": [
                    {
                        "id": 3,
                        "name": "Sector 3",
                        "min_value": 15,
                        "max_value": 25,
                        "slot_capacity": 3,
                        "sector_type": "Normal",
                        "current_occupancy": 1
                    },
                    {
                        "id": 4,
                        "name": "Sector 4",
                        "min_value": 20,
                        "max_value": 30,
                        "slot_capacity": 3,
                        "sector_type": "Normal",
                        "current_occupancy": 2
                    },
                    {
                        "id": 5,
                        "name": "Sector 5",
                        "min_value": 25,
                        "max_value": 35,
                        "slot_capacity": 3,
                        "sector_type": "Normal",
                        "current_occupancy": 1
                    },
                    {
                        "id": 6,
                        "name": "Sector 6",
                        "min_value": 30,
                        "max_value": 40,
                        "slot_capacity": 3,
                        "sector_type": "Normal",
                        "current_occupancy": 0
                    },
                    {
                        "id": 7,
                        "name": "Sector 7",
                        "min_value": 35,
                        "max_value": 45,
                        "slot_capacity": 3,
                        "sector_type": "Normal",
                        "current_occupancy": 1
                    }
                ],
                "visible_participants": [
                    {
                        "player_uuid": "550e8400-e29b-41d4-a716-446655440000",
                        "player_name": "Player 1",
                        "car_name": "Lightning Bolt",
                        "current_sector": 5,
                        "position_in_sector": 1,
                        "total_value": 45,
                        "current_lap": 2,
                        "is_finished": false
                    },
                    {
                        "player_uuid": "550e8400-e29b-41d4-a716-446655440001",
                        "player_name": "Player 2",
                        "car_name": "Thunder Strike",
                        "current_sector": 4,
                        "position_in_sector": 1,
                        "total_value": 42,
                        "current_lap": 2,
                        "is_finished": false
                    }
                ]
            })
        ),
        (
            status = 400,
            description = "Invalid UUID format",
            body = ErrorResponse,
            example = json!({
                "error": "INVALID_UUID",
                "message": "Invalid UUID format",
                "details": null
            })
        ),
        (
            status = 404,
            description = "Player not found in race or race not found",
            body = ErrorResponse,
            example = json!({
                "error": "PLAYER_NOT_FOUND",
                "message": "Player not found in race",
                "details": null
            })
        ),
        (
            status = 500,
            description = "Internal server error",
            body = ErrorResponse,
            example = json!({
                "error": "DATABASE_ERROR",
                "message": "Internal server error",
                "details": "Failed to fetch race: connection timeout"
            })
        )
    ),
    tag = "races"
)]
#[tracing::instrument(
    name = "Getting local view for player in race",
    skip(state),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %player_uuid_str
    )
)]
pub async fn get_local_view(
    State(state): State<AppState>,
    Path((race_uuid_str, player_uuid_str)): Path<(String, String)>,
) -> Result<Json<LocalViewResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Parse and validate UUIDs
    let race_uuid = if let Ok(uuid) = Uuid::parse_str(&race_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid race UUID format: {}", race_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    let player_uuid = if let Ok(uuid) = Uuid::parse_str(&player_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid player UUID format: {}", player_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    // 2. Fetch race from database
    let race = match get_race_by_uuid(&state.race_repository, race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "RACE_NOT_FOUND".to_string(),
                    message: "Race not found".to_string(),
                    details: None,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to fetch race: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Internal server error".to_string(),
                    details: Some(format!("Failed to fetch race: {e}")),
                }),
            ));
        }
    };

    // 3. Find participant by player_uuid
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or_else(|| {
            tracing::warn!("Player {} not found in race {}", player_uuid, race_uuid);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "PLAYER_NOT_FOUND".to_string(),
                    message: "Player not found in race".to_string(),
                    details: None,
                }),
            )
        })?;

    let center_sector = participant.current_sector;

    // 4. Calculate visible sector IDs using helper function
    let visible_sector_ids = get_visible_sector_ids(center_sector, race.track.sectors.len());

    // 5. Filter sectors to visible range
    let mut visible_sectors = Vec::new();
    for sector_id in &visible_sector_ids {
        if let Some(sector) = race.track.sectors.iter().find(|s| s.id == *sector_id) {
            // Count participants in this sector
            #[allow(clippy::cast_possible_truncation)]
            let current_occupancy = race
                .participants
                .iter()
                .filter(|p| p.current_sector == *sector_id && !p.is_finished)
                .count() as u32;

            visible_sectors.push(SectorInfo {
                id: sector.id,
                name: sector.name.clone(),
                min_value: sector.min_value,
                max_value: sector.max_value,
                slot_capacity: sector.slot_capacity,
                sector_type: format!("{:?}", sector.sector_type),
                current_occupancy,
            });
        }
    }

    // 6. Filter participants to visible range
    let mut visible_participants = Vec::new();
    for participant in &race.participants {
        if visible_sector_ids.contains(&participant.current_sector) && !participant.is_finished {
            // TODO: Optionally fetch player names from database
            let player_name = None; // Placeholder for now

            // TODO: Fetch car name from database
            let car_name = format!("Car {}", participant.car_uuid);

            visible_participants.push(ParticipantInfo {
                player_uuid: participant.player_uuid.to_string(),
                player_name,
                car_name,
                current_sector: participant.current_sector,
                position_in_sector: participant.current_position_in_sector,
                total_value: participant.total_value,
                current_lap: participant.current_lap,
                is_finished: participant.is_finished,
            });
        }
    }

    // Sort participants by sector (descending) then by position in sector (ascending)
    visible_participants.sort_by(|a, b| {
        b.current_sector
            .cmp(&a.current_sector)
            .then_with(|| a.position_in_sector.cmp(&b.position_in_sector))
    });

    // 7. Return local view data with 5 sectors
    #[allow(clippy::cast_possible_truncation)]
    let total_sectors = race.track.sectors.len() as u32;
    let response = LocalViewResponse {
        center_sector,
        total_sectors,
        visible_sectors,
        visible_participants,
    };

    tracing::info!(
        "Local view calculated for player {} in race {}",
        player_uuid,
        race_uuid
    );
    Ok(Json(response))
}

/// Get boost card availability for a player in a race
///
/// This endpoint returns the current boost hand state for a player, including:
/// - Available boost cards (0-4) that can be used
/// - Complete hand state showing which cards are available/used
/// - Current cycle information (cycle number, cycles completed)
/// - Cards remaining before next replenishment
/// - Next replenishment lap number
///
/// The boost hand system manages 5 cards per cycle. When all cards are used,
/// the hand automatically replenishes for the next cycle. This endpoint helps
/// the frontend display which boost options are currently available to the player.
///
/// # Boost Hand Mechanics
/// - Each player has 5 boost cards (values 0-4) per cycle
/// - Cards can only be used once per cycle
/// - When all 5 cards are used, the hand replenishes automatically
/// - Next replenishment occurs when `cards_remaining` reaches 0
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}/players/{player_uuid}/boost-availability",
    params(
        ("race_uuid" = String, Path, description = "Race UUID"),
        ("player_uuid" = String, Path, description = "Player UUID")
    ),
    responses(
        (
            status = 200,
            description = "Boost availability retrieved successfully",
            body = BoostAvailabilityResponse,
            example = json!({
                "available_cards": [1, 2, 4],
                "hand_state": {
                    "0": false,
                    "1": true,
                    "2": true,
                    "3": false,
                    "4": true
                },
                "current_cycle": 1,
                "cycles_completed": 0,
                "cards_remaining": 3,
                "next_replenishment_at": 3
            })
        ),
        (
            status = 400,
            description = "Invalid UUID format",
            body = ErrorResponse,
            example = json!({
                "error": "INVALID_UUID",
                "message": "Invalid UUID format",
                "details": null
            })
        ),
        (
            status = 404,
            description = "Player not found in race or race not found",
            body = ErrorResponse,
            example = json!({
                "error": "PLAYER_NOT_FOUND",
                "message": "Player not found in race",
                "details": null
            })
        ),
        (
            status = 409,
            description = "Race not in progress or player already finished",
            body = ErrorResponse,
            example = json!({
                "error": "PLAYER_FINISHED",
                "message": "Player has already finished the race",
                "details": null
            })
        ),
        (
            status = 500,
            description = "Internal server error",
            body = ErrorResponse,
            example = json!({
                "error": "DATABASE_ERROR",
                "message": "Internal server error",
                "details": "Failed to fetch race: connection timeout"
            })
        )
    ),
    tag = "races"
)]
#[tracing::instrument(
    name = "Getting boost availability for player in race",
    skip(state),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %player_uuid_str
    )
)]
pub async fn get_boost_availability(
    State(state): State<AppState>,
    Path((race_uuid_str, player_uuid_str)): Path<(String, String)>,
) -> Result<Json<BoostAvailabilityResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Parse and validate UUIDs
    let race_uuid = if let Ok(uuid) = Uuid::parse_str(&race_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid race UUID format: {}", race_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    let player_uuid = if let Ok(uuid) = Uuid::parse_str(&player_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid player UUID format: {}", player_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    // 2. Fetch race from database
    let race = match get_race_by_uuid(&state.race_repository, race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "RACE_NOT_FOUND".to_string(),
                    message: "Race not found".to_string(),
                    details: None,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to fetch race: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Internal server error".to_string(),
                    details: Some(format!("Failed to fetch race: {e}")),
                }),
            ));
        }
    };

    // 3. Validate race is in progress
    if race.status != RaceStatus::InProgress {
        tracing::warn!("Race {} is not in progress", race_uuid);
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "RACE_NOT_IN_PROGRESS".to_string(),
                message: "Race is not in progress".to_string(),
                details: None,
            }),
        ));
    }

    // 4. Find participant by player_uuid
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or_else(|| {
            tracing::warn!("Player {} not found in race {}", player_uuid, race_uuid);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "PLAYER_NOT_FOUND".to_string(),
                    message: "Player not found in race".to_string(),
                    details: None,
                }),
            )
        })?;

    // 5. Check if player has already finished
    if participant.is_finished {
        tracing::warn!("Player {} has already finished the race", player_uuid);
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "PLAYER_FINISHED".to_string(),
                message: "Player has already finished the race".to_string(),
                details: None,
            }),
        ));
    }

    // 6. Get boost hand from participant
    let boost_hand = &participant.boost_hand;

    // 7. Extract availability information
    let available_cards = boost_hand.get_available_cards();
    let hand_state = boost_hand.cards.clone();

    // 8. Return availability data (pool refills only via pit stop)
    let response = BoostAvailabilityResponse {
        available_cards,
        hand_state,
        tyre_type: boost_hand.tyre_type,
        pit_stops_completed: boost_hand.pit_stops_completed,
        cards_remaining: boost_hand.cards_remaining,
    };

    tracing::info!(
        "Boost availability retrieved for player {} in race {}",
        player_uuid,
        race_uuid
    );
    Ok(Json(response))
}

/// Get lap history for a player in a race
///
/// This endpoint returns the player's lap-by-lap performance history for the current race.
/// It provides:
/// - Lap records with boost usage, performance values, and movement information
/// - Boost cycle summaries with aggregated statistics per cycle
///
/// The lap history helps players analyze their performance trends and boost card
/// usage patterns throughout the race. Each lap record includes:
/// - Lap number and lap characteristic (Straight/Curve)
/// - Boost card used and which cycle it was from
/// - Base and final performance values
/// - Movement information (from/to sectors, movement type)
///
/// Cycle summaries provide aggregated statistics:
/// - Cards used in each cycle
/// - Laps when cards were used
/// - Average boost value per cycle
///
/// # Note
/// Historical performance and movement data is currently limited to boost usage tracking.
/// Full lap-by-lap performance reconstruction would require storing additional historical data.
#[utoipa::path(
    get,
    path = "/api/v1/races/{race_uuid}/players/{player_uuid}/lap-history",
    params(
        ("race_uuid" = String, Path, description = "Race UUID"),
        ("player_uuid" = String, Path, description = "Player UUID")
    ),
    responses(
        (
            status = 200,
            description = "Lap history retrieved successfully",
            body = LapHistoryResponse,
            example = json!({
                "laps": [
                    {
                        "lap_number": 1,
                        "lap_characteristic": "Straight",
                        "boost_used": 2,
                        "boost_cycle": 1,
                        "base_value": 20,
                        "final_value": 23,
                        "from_sector": 0,
                        "to_sector": 3,
                        "movement_type": "MovedUp"
                    },
                    {
                        "lap_number": 2,
                        "lap_characteristic": "Curve",
                        "boost_used": 0,
                        "boost_cycle": 1,
                        "base_value": 18,
                        "final_value": 18,
                        "from_sector": 3,
                        "to_sector": 4,
                        "movement_type": "StayedInSector"
                    }
                ],
                "cycle_summaries": [
                    {
                        "cycle_number": 1,
                        "cards_used": [2, 0, 3],
                        "laps_in_cycle": [1, 2, 3],
                        "average_boost": 1.67
                    }
                ]
            })
        ),
        (
            status = 400,
            description = "Invalid UUID format",
            body = ErrorResponse,
            example = json!({
                "error": "INVALID_UUID",
                "message": "Invalid UUID format",
                "details": null
            })
        ),
        (
            status = 404,
            description = "Player not found in race or race not found",
            body = ErrorResponse,
            example = json!({
                "error": "PLAYER_NOT_FOUND",
                "message": "Player not found in race",
                "details": null
            })
        ),
        (
            status = 500,
            description = "Internal server error",
            body = ErrorResponse,
            example = json!({
                "error": "DATABASE_ERROR",
                "message": "Internal server error",
                "details": "Failed to fetch race: connection timeout"
            })
        )
    ),
    tag = "races"
)]
#[tracing::instrument(
    name = "Getting lap history for player in race",
    skip(state),
    fields(
        race_uuid = %race_uuid_str,
        player_uuid = %player_uuid_str
    )
)]
pub async fn get_lap_history(
    State(state): State<AppState>,
    Path((race_uuid_str, player_uuid_str)): Path<(String, String)>,
) -> Result<Json<LapHistoryResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Parse and validate UUIDs
    let race_uuid = if let Ok(uuid) = Uuid::parse_str(&race_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid race UUID format: {}", race_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    let player_uuid = if let Ok(uuid) = Uuid::parse_str(&player_uuid_str) {
        uuid
    } else {
        tracing::warn!("Invalid player UUID format: {}", player_uuid_str);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "INVALID_UUID".to_string(),
                message: "Invalid UUID format".to_string(),
                details: None,
            }),
        ));
    };

    // 2. Fetch race from database
    let race = match get_race_by_uuid(&state.race_repository, race_uuid).await {
        Ok(Some(race)) => race,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "RACE_NOT_FOUND".to_string(),
                    message: "Race not found".to_string(),
                    details: None,
                }),
            ));
        }
        Err(e) => {
            tracing::error!("Failed to fetch race: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "DATABASE_ERROR".to_string(),
                    message: "Internal server error".to_string(),
                    details: Some(format!("Failed to fetch race: {e}")),
                }),
            ));
        }
    };

    // 3. Find participant by player_uuid
    let participant = race
        .participants
        .iter()
        .find(|p| p.player_uuid == player_uuid)
        .ok_or_else(|| {
            tracing::warn!("Player {} not found in race {}", player_uuid, race_uuid);
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "PLAYER_NOT_FOUND".to_string(),
                    message: "Player not found in race".to_string(),
                    details: None,
                }),
            )
        })?;

    // 4. Get boost usage history from participant
    let boost_usage_history = &participant.boost_usage_history;

    // 5. Build lap records from usage history
    // Note: We only have boost usage data. Historical performance values and movements
    // are not currently stored, so we use placeholder values.
    let mut lap_records = Vec::new();

    for usage_record in boost_usage_history {
        // Determine lap characteristic based on lap number
        // This is a simplified approach - ideally we'd store historical lap characteristics
        let lap_characteristic = if usage_record.lap_number % 2 == 1 {
            "Straight"
        } else {
            "Curve"
        };

        // Create lap record with available data
        // Note: base_value, final_value, from_sector, to_sector, and movement_type
        // are not historically tracked, so we use placeholder values
        lap_records.push(LapRecord {
            lap_number: usage_record.lap_number,
            lap_characteristic: lap_characteristic.to_string(),
            boost_used: usage_record.boost_value,
            boost_cycle: usage_record.cycle_number,
            base_value: 0,                        // Historical base value not tracked
            final_value: 0,                       // Historical final value not tracked
            from_sector: 0,                       // Historical sector movement not tracked
            to_sector: 0,                         // Historical sector movement not tracked
            movement_type: "Unknown".to_string(), // Historical movement type not tracked
        });
    }

    // 6. Get cycle summaries using participant.get_boost_cycle_summaries()
    let cycle_summaries_domain = participant.get_boost_cycle_summaries();

    // 7. Convert domain cycle summaries to API response format
    let cycle_summaries: Vec<CycleSummary> = cycle_summaries_domain
        .into_iter()
        .map(|summary| CycleSummary {
            cycle_number: summary.cycle_number,
            cards_used: summary.cards_used,
            laps_in_cycle: summary.laps_in_cycle,
            average_boost: summary.average_boost,
        })
        .collect();

    // 8. Return history data with lap records and cycle summaries
    let response = LapHistoryResponse {
        laps: lap_records,
        cycle_summaries,
    };

    tracing::info!(
        "Lap history retrieved for player {} in race {}",
        player_uuid,
        race_uuid
    );
    Ok(Json(response))
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
    skip(state, payload),
    fields(
        race_name = %payload.name,
        track_name = %payload.track_name,
        total_laps = payload.total_laps
    )
)]
pub async fn create_race(
    State(state): State<AppState>,
    Json(payload): Json<CreateRaceRequest>,
) -> Result<(StatusCode, Json<RaceResponse>), StatusCode> {
    // Create sectors from request
    let sectors: Vec<Sector> = payload
        .sectors
        .into_iter()
        .map(|s| Sector {
            id: s.id,
            name: s.name,
            min_value: s.min_value,
            max_value: s.max_value,
            slot_capacity: s.slot_capacity,
            sector_type: s.sector_type,
        })
        .collect();

    // Create track
    let track = match Track::new(payload.track_name, sectors) {
        Ok(track) => track,
        Err(e) => {
            tracing::warn!("Invalid track configuration: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Create race
    let mut race = Race::new(payload.name, track, payload.total_laps);

    // Auto-start the race immediately for better UX
    // This eliminates the need for manual race starting
    race.status = RaceStatus::InProgress;
    race.lap_characteristic = LapCharacteristic::Straight; // Start with straight characteristic
    race.current_lap = 1;

    tracing::info!("Auto-starting race {} for improved UX", race.uuid);

    match insert_race(&state.race_repository, &race).await {
        Ok(created_race) => {
            tracing::info!(
                "Race created and auto-started successfully with UUID: {}",
                created_race.uuid
            );
            Ok((
                StatusCode::CREATED,
                Json(RaceResponse {
                    race: created_race,
                    message: "Race created and started successfully".to_string(),
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
#[tracing::instrument(name = "Fetching all races", skip(state))]
pub async fn get_all_races(State(state): State<AppState>) -> Result<Json<Vec<Race>>, StatusCode> {
    match get_all_races_from_db(&state.race_repository).await {
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
#[tracing::instrument(name = "Fetching race by UUID", skip(state))]
pub async fn get_race(
    State(state): State<AppState>,
    Path(race_uuid_str): Path<String>,
) -> Result<Json<Race>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match get_race_by_uuid(&state.race_repository, race_uuid).await {
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
#[tracing::instrument(name = "Joining race", skip(state, payload))]
pub async fn join_race(
    State(state): State<AppState>,
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

    match join_race_in_db(
        &state.race_repository,
        race_uuid,
        player_uuid,
        car_uuid,
        pilot_uuid,
    )
    .await
    {
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
            if e.to_string().contains("already participating")
                || e.to_string().contains("already started")
            {
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
#[tracing::instrument(name = "Starting race", skip(state))]
pub async fn start_race(
    State(state): State<AppState>,
    Path(race_uuid_str): Path<String>,
) -> Result<Json<RaceResponse>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match start_race_in_db(&state.race_repository, race_uuid).await {
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
            if e.to_string().contains("already started")
                || e.to_string().contains("without participants")
            {
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
#[tracing::instrument(name = "Processing race turn", skip(state, payload))]
pub async fn process_turn(
    State(state): State<AppState>,
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

    // Build the real car-data map from the in-memory player repository.
    let car_data_map = match state.race_repository.find_by_uuid(race_uuid).await {
        Ok(Some(race)) => build_car_data_map(state.player_repository.as_ref(), &race).await,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to fetch race {race_uuid}: {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match process_lap_in_db(&state.race_repository, race_uuid, actions, &car_data_map).await {
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
            if e.to_string().contains("not in progress") || e.to_string().contains("Missing action")
            {
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
#[tracing::instrument(name = "Getting race status", skip(state))]
pub async fn get_race_status(
    State(state): State<AppState>,
    Path(race_uuid_str): Path<String>,
) -> Result<Json<RaceStatus>, StatusCode> {
    let race_uuid = match Uuid::parse_str(&race_uuid_str) {
        Ok(uuid) => uuid,
        Err(e) => {
            tracing::warn!("Invalid race UUID: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match get_race_by_uuid(&state.race_repository, race_uuid).await {
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
#[tracing::instrument(name = "Saving new race in the database", skip(race_repository, race))]
pub async fn insert_race(
    race_repository: &Arc<dyn RaceRepository>,
    race: &Race,
) -> RepositoryResult<Race> {
    race_repository.save(race).await
}

#[tracing::instrument(name = "Getting all races from the database", skip(race_repository))]
pub async fn get_all_races_from_db(
    race_repository: &Arc<dyn RaceRepository>,
) -> RepositoryResult<Vec<Race>> {
    race_repository.find_all().await
}

#[tracing::instrument(name = "Getting race by UUID from the database", skip(race_repository))]
pub async fn get_race_by_uuid(
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
) -> RepositoryResult<Option<Race>> {
    race_repository.find_by_uuid(race_uuid).await
}

#[tracing::instrument(name = "Joining race in the database", skip(race_repository))]
pub async fn join_race_in_db(
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
    player_uuid: Uuid,
    car_uuid: Uuid,
    pilot_uuid: Uuid,
) -> RepositoryResult<Option<Race>> {
    // Get the race first
    let Some(mut race) = get_race_by_uuid(race_repository, race_uuid).await? else {
        return Ok(None);
    };

    // Try to add participant
    if let Err(e) = race.add_participant(player_uuid, car_uuid, pilot_uuid) {
        return Err(RepositoryError::Validation(e));
    }

    // Persist the mutated race and return it.
    race_repository.save(&race).await?;
    Ok(Some(race))
}

#[tracing::instrument(name = "Starting race in the database", skip(race_repository))]
pub async fn start_race_in_db(
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
) -> RepositoryResult<Option<Race>> {
    // Get the race first
    let mut race = if let Some(race) = get_race_by_uuid(race_repository, race_uuid).await? {
        race
    } else {
        tracing::warn!("Race not found: {}", race_uuid);
        return Ok(None);
    };

    // Validate race can be started
    if race.status != RaceStatus::Waiting {
        let error_msg = format!(
            "Race has already started or finished. Current status: {:?}",
            race.status
        );
        tracing::warn!("{}", error_msg);
        return Err(RepositoryError::Validation(error_msg));
    }

    if race.participants.is_empty() {
        let error_msg = "Cannot start race without participants";
        tracing::warn!("{}", error_msg);
        return Err(RepositoryError::Validation(error_msg.to_string()));
    }

    tracing::info!(
        "Starting race {} with {} participants",
        race_uuid,
        race.participants.len()
    );

    // Update race status and initialize lap characteristic
    race.status = RaceStatus::InProgress;
    race.lap_characteristic = LapCharacteristic::Straight; // Start with straight characteristic
    race.current_lap = 1;

    // Sort participants in their starting sectors. Positions are 0-based to match
    // `sort_participants_in_sectors` and the UI (slot 1 = position 0), so the
    // front grid slot is occupied from the start.
    for (index, participant) in race.participants.iter_mut().enumerate() {
        participant.current_position_in_sector = index as u32;
        tracing::debug!(
            "Participant {} positioned at sector {} position {}",
            participant.player_uuid,
            participant.current_sector,
            participant.current_position_in_sector
        );
    }

    // Persist the mutated race and return it.
    tracing::info!("Updating race {} in store", race_uuid);
    race_repository.save(&race).await?;
    tracing::info!("Successfully started race {}", race_uuid);
    Ok(Some(race))
}

// Takes the default-hasher map the whole turn pipeline uses; generalizing over
// the hasher would force the same on every domain method it calls downstream.
#[allow(clippy::implicit_hasher)]
#[tracing::instrument(
    name = "Processing turn in the database",
    skip(race_repository, actions, car_data_map)
)]
pub async fn process_lap_in_db(
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
    actions: Vec<LapAction>,
    car_data_map: &HashMap<Uuid, ValidatedCarData>,
) -> RepositoryResult<Option<(LapResult, RaceStatus)>> {
    // Get the race first
    let Some(mut race) = race_repository.find_by_uuid(race_uuid).await? else {
        return Ok(None);
    };

    // Compute REAL performances from each car's stats. If resolution failed for
    // any participant (Err), fall back to the placeholder map so nothing 500s.
    let performance_calculations = match race.calculate_all_performances(&actions, car_data_map) {
        Ok(perf) => perf,
        Err(e) => {
            tracing::warn!(
                "Real performance calculation failed for race {} ({}); falling back to placeholder",
                race_uuid,
                e
            );
            let mut placeholder = HashMap::new();
            for action in &actions {
                // Placeholder performance calculation with base value 10
                let performance = PerformanceCalculation {
                    engine_contribution: 5,
                    body_contribution: 3,
                    pilot_contribution: 2,
                    base_value: 10,
                    sector_ceiling: 30, // Default ceiling
                    capped_base_value: 10,
                    boost_value: action.boost_value,
                    final_value: 10 + action.boost_value,
                };
                placeholder.insert(action.player_uuid, performance);
            }
            placeholder
        }
    };

    // Process the lap using the new method with car data
    let lap_result = match race.process_lap_with_car_data(&actions, &performance_calculations) {
        Ok(result) => result,
        Err(e) => return Err(RepositoryError::Validation(e)),
    };

    // Clear pending actions after successful processing
    race.pending_actions.clear();
    race.action_submissions.clear();
    race.pending_performance_calculations.clear();

    // Persist the mutated race to the in-memory store.
    let race_status = race.status.clone();
    race_repository.save(&race).await?;

    tracing::info!(
        "Turn processing completed for race {}. Ready for next turn.",
        race_uuid
    );

    Ok(Some((lap_result, race_status)))
}

/// Drive AI-only turns to completion.
///
/// In solo mode the human submission triggers a lap; if afterwards the only
/// active (non-finished) participants are AI — e.g. the human has already
/// finished — there is no one left to advance the race. This synthesizes and
/// processes full AI turns until the race finishes or a human is again required.
/// The loop is bounded as a safety net; the race's own safety cap also
/// guarantees termination.
/// Hard caps for the AI-only auto-advance loop, so it can never run forever or
/// hold the request open indefinitely even if a turn somehow fails to make
/// progress. The race's own safety bound finishes a stuck race long before
/// either of these trips during normal play.
const AI_AUTOADVANCE_MAX_TURNS: u32 = 1000;
const AI_AUTOADVANCE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

async fn drive_ai_only_turns(
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
    car_data_map: &HashMap<Uuid, ValidatedCarData>,
) -> RepositoryResult<()> {
    let deadline = std::time::Instant::now() + AI_AUTOADVANCE_TIMEOUT;

    for turn in 0..AI_AUTOADVANCE_MAX_TURNS {
        // Wall-clock timeout: bound request latency regardless of iteration count.
        if std::time::Instant::now() >= deadline {
            tracing::warn!(
                "AI auto-advance for race {} hit the {}s timeout after {} turns; leaving it InProgress",
                race_uuid,
                AI_AUTOADVANCE_TIMEOUT.as_secs(),
                turn
            );
            return Ok(());
        }

        let Some(mut race) = race_repository.find_by_uuid(race_uuid).await? else {
            return Ok(());
        };

        if race.status != RaceStatus::InProgress {
            return Ok(());
        }

        let active: Vec<&RaceParticipant> = race
            .participants
            .iter()
            .filter(|p| !p.is_finished)
            .collect();

        // Nothing to drive, or a human still needs to act this turn.
        if active.is_empty() || active.iter().any(|p| !p.is_ai) {
            return Ok(());
        }

        race.enqueue_ai_actions(car_data_map);
        let actions = race.pending_actions.clone();
        if actions.is_empty() {
            return Ok(());
        }
        race_repository.save(&race).await?;

        process_lap_in_db(race_repository, race_uuid, actions, car_data_map).await?;
    }

    tracing::warn!(
        "AI auto-advance for race {} hit the {}-turn cap; leaving it InProgress",
        race_uuid,
        AI_AUTOADVANCE_MAX_TURNS
    );
    Ok(())
}

/// Submit a single player's turn action (boost selection)
///
/// This endpoint allows individual players to submit their boost selection for the current turn.
/// Unlike the batch `process_turn` endpoint, this handles one player at a time and stores
/// the action until all players have submitted their choices.
#[utoipa::path(
    post,
    path = "/races/{race_uuid}/submit-action",
    request_body = SubmitTurnActionRequest,
    responses(
        (status = 200, description = "Action submitted successfully", body = SubmitTurnActionResponse),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Race or player not found"),
        (status = 409, description = "Action already submitted or race not in progress")
    ),
    params(
        ("race_uuid" = String, Path, description = "Race UUID")
    )
)]
#[tracing::instrument(name = "Submitting turn action", skip(state, payload))]
pub async fn submit_turn_action(
    State(state): State<AppState>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<SubmitTurnActionRequest>,
) -> Result<Json<SubmitTurnActionResponse>, StatusCode> {
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

    // Validate boost value
    if payload.boost_value > 4 {
        tracing::warn!("Invalid boost value: {}", payload.boost_value);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Build the real car-data map from the in-memory player repository.
    let car_data_map = match state.race_repository.find_by_uuid(race_uuid).await {
        Ok(Some(race)) => build_car_data_map(state.player_repository.as_ref(), &race).await,
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            tracing::error!("Failed to fetch race {race_uuid}: {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match submit_player_action_in_db(
        &state.race_repository,
        race_uuid,
        player_uuid,
        payload.boost_value,
        &car_data_map,
    )
    .await
    {
        Ok(Some(response)) => {
            tracing::info!(
                "Action submitted successfully for player {} in race {}",
                player_uuid,
                race_uuid
            );
            Ok(Json(response))
        }
        Ok(None) => {
            tracing::warn!("Race not found for UUID: {}", race_uuid);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            tracing::error!("Failed to submit action: {:?}", e);
            if e.to_string().contains("not found") {
                Err(StatusCode::NOT_FOUND)
            } else if e.to_string().contains("already submitted")
                || e.to_string().contains("not in progress")
            {
                Err(StatusCode::CONFLICT)
            } else {
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// What a human player is doing this turn. Both `/submit-action` (boost) and
/// `/pit` (pit stop) funnel through [`resolve_human_turn`] so the turn resolves
/// identically: the human's card is consumed, every AI opponent is enqueued, the
/// lap is processed once everyone has acted, and any AI-only continuation is
/// driven to completion. Keeping this single path is what stops the two
/// endpoints from drifting apart (the prior solo-pit deadlock + inert-pool bugs).
#[derive(Debug, Clone, Copy)]
enum TurnIntent {
    /// Play a boost card of the given value (0-4; 0 is the free no-boost move).
    Boost(u32),
    /// Pit: refill the boost pool from `new_tyre` (or keep the current tyre when
    /// `None`) and spend this turn as a free boost-0 move.
    Pit(Option<TyreType>),
}

/// Resolve a single human turn against the in-memory race store.
///
/// 1. record the human's action via the card-consuming domain method
///    (`record_player_action` / `record_pit_action`) — the human's pool depletes;
/// 2. enqueue every active AI opponent so the turn isn't waiting on absent humans;
/// 3. process the lap once all active participants have acted;
/// 4. drive any AI-only turns to completion (e.g. after the human finishes).
///
/// Returns the updated race, or `Ok(None)` if the race no longer exists.
async fn resolve_human_turn(
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
    player_uuid: Uuid,
    intent: TurnIntent,
    car_data_map: &HashMap<Uuid, ValidatedCarData>,
) -> RepositoryResult<Option<Race>> {
    let Some(mut race) = race_repository.find_by_uuid(race_uuid).await? else {
        return Ok(None);
    };

    if race.status != RaceStatus::InProgress {
        return Err(RepositoryError::Validation(
            "Race is not in progress".to_string(),
        ));
    }

    let Some(car_data) = car_data_map.get(&player_uuid) else {
        return Err(RepositoryError::Validation(
            "Car data not found for player".to_string(),
        ));
    };

    // 1. Record the human's card-consuming action. The domain method validates
    //    participant / not-finished / not-already-acted / boost availability.
    let record_result = match intent {
        TurnIntent::Boost(boost_value) => {
            race.record_player_action(player_uuid, boost_value, car_data)
        }
        TurnIntent::Pit(new_tyre) => race.record_pit_action(player_uuid, new_tyre, car_data),
    };
    if let Err(e) = record_result {
        return Err(RepositoryError::Validation(e));
    }

    // 2. Solo mode: enqueue every AI opponent so the turn can resolve without
    //    waiting on (non-existent) human opponents.
    race.enqueue_ai_actions(car_data_map);
    race_repository.save(&race).await?;

    // 3. Process the lap once everyone active has acted.
    if race.all_actions_submitted() {
        let actions = race.pending_actions.clone();
        if process_lap_in_db(race_repository, race_uuid, actions, car_data_map)
            .await?
            .is_none()
        {
            return Err(RepositoryError::Validation(
                "Race not found during processing".to_string(),
            ));
        }

        // 4. Keep driving AI-only turns so the race always reaches completion
        //    instead of stalling (e.g. once the human has finished).
        drive_ai_only_turns(race_repository, race_uuid, car_data_map).await?;
    }

    race_repository.find_by_uuid(race_uuid).await
}

/// Submit a player's boost action and resolve the turn (solo mode).
async fn submit_player_action_in_db(
    race_repository: &Arc<dyn RaceRepository>,
    race_uuid: Uuid,
    player_uuid: Uuid,
    boost_value: u32,
    car_data_map: &HashMap<Uuid, ValidatedCarData>,
) -> RepositoryResult<Option<SubmitTurnActionResponse>> {
    let Some(race) = resolve_human_turn(
        race_repository,
        race_uuid,
        player_uuid,
        TurnIntent::Boost(boost_value),
        car_data_map,
    )
    .await?
    else {
        return Ok(None);
    };

    let total_players = race.participants.iter().filter(|p| !p.is_finished).count() as u32;

    // If the human's action is still pending, we are waiting on other (human)
    // players; otherwise the turn was processed and pending was cleared.
    let still_waiting = race
        .pending_actions
        .iter()
        .any(|a| a.player_uuid == player_uuid);

    if still_waiting {
        Ok(Some(SubmitTurnActionResponse {
            success: true,
            message: "Action submitted successfully".to_string(),
            turn_phase: "WaitingForPlayers".to_string(),
            players_submitted: race.pending_actions.len() as u32,
            total_players,
        }))
    } else {
        Ok(Some(SubmitTurnActionResponse {
            success: true,
            message: "Turn processed successfully. Ready for next turn.".to_string(),
            turn_phase: "TurnProcessed".to_string(),
            players_submitted: 0,
            total_players,
        }))
    }
}

#[cfg(test)]
mod turn_resolution_tests {
    use super::*;
    use crate::domain::{
        Body, BodyName, Car, CarName, ComponentRarity, Engine, EngineName, Pilot, PilotClass,
        PilotName, PilotPerformance, PilotRarity, PilotSkills, Race, RaceStatus, Sector,
        SectorType, Track,
    };
    use crate::repositories::MockRaceRepository;
    use crate::services::car_validation::ValidatedCarData;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_car_data() -> ValidatedCarData {
        let engine = Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            7,
            5,
        )
        .unwrap();
        let body = Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            5,
            7,
        )
        .unwrap();
        let pilot = Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Rookie,
            PilotSkills::new(6, 6, 6, 6).unwrap(),
            PilotPerformance::new(8, 5).unwrap(),
        )
        .unwrap();
        let car = Car::new(CarName::parse("Test Car").unwrap()).unwrap();
        ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        }
    }

    fn test_track() -> Track {
        let sectors = vec![
            Sector {
                id: 0,
                name: "Start".to_string(),
                min_value: 0,
                max_value: 10,
                slot_capacity: None,
                sector_type: SectorType::Start,
            },
            Sector {
                id: 1,
                name: "Straight".to_string(),
                min_value: 5,
                max_value: 14,
                slot_capacity: None,
                sector_type: SectorType::Straight,
            },
        ];
        Track::new("Test Track".to_string(), sectors).unwrap()
    }

    /// Seed a solo race (1 human + 2 AI) into a fresh mock repository; return
    /// the repository, its uuid, the human's uuid, and the full car-data map.
    async fn seed_solo_race() -> (
        Arc<dyn RaceRepository>,
        Uuid,
        Uuid,
        HashMap<Uuid, ValidatedCarData>,
    ) {
        let mut race = Race::new("Turn Resolution Test".to_string(), test_track(), 3);
        race.status = RaceStatus::InProgress;

        let human = Uuid::new_v4();
        let ai1 = Uuid::new_v4();
        let ai2 = Uuid::new_v4();
        race.add_participant(human, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        race.add_ai_participant(ai1, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        race.add_ai_participant(ai2, Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        for p in &mut race.participants {
            p.current_sector = 0;
        }

        let race_uuid = race.uuid;
        let race_repository: Arc<dyn RaceRepository> = Arc::new(MockRaceRepository::new());
        race_repository.save(&race).await.unwrap();

        let mut map = HashMap::new();
        map.insert(human, make_car_data());
        map.insert(ai1, make_car_data());
        map.insert(ai2, make_car_data());

        (race_repository, race_uuid, human, map)
    }

    async fn human_cards_remaining(
        race_repository: &Arc<dyn RaceRepository>,
        race_uuid: Uuid,
        human: Uuid,
    ) -> u32 {
        race_repository
            .find_by_uuid(race_uuid)
            .await
            .unwrap()
            .unwrap()
            .participants
            .iter()
            .find(|p| p.player_uuid == human)
            .unwrap()
            .boost_hand
            .cards_remaining
    }

    /// Regression for the inert tyre-pool bug (#2): a boost submitted via
    /// `/submit-action` MUST consume one of the human's boost cards. Before the
    /// fix the submit path pushed a bare `LapAction` without `use_boost_card`,
    /// so `cards_remaining` never changed and the pool was effectively infinite.
    #[tokio::test]
    async fn submit_action_consumes_human_boost_card() {
        let (race_repository, race_uuid, human, map) = seed_solo_race().await;
        let before = human_cards_remaining(&race_repository, race_uuid, human).await;

        let card = race_repository
            .find_by_uuid(race_uuid)
            .await
            .unwrap()
            .unwrap()
            .participants
            .iter()
            .find(|p| p.player_uuid == human)
            .unwrap()
            .boost_hand
            .get_available_cards()
            .into_iter()
            .find(|&c| c >= 1)
            .expect("a fresh medium pool has at least one boost card");

        submit_player_action_in_db(&race_repository, race_uuid, human, u32::from(card), &map)
            .await
            .expect("submit should succeed");

        let after = human_cards_remaining(&race_repository, race_uuid, human).await;
        assert_eq!(
            after,
            before - 1,
            "submitting a boost must consume exactly one of the human's cards"
        );
    }

    /// Regression for the solo pit deadlock (#1): a pit via `/pit` MUST resolve
    /// the turn (enqueue AI, process the lap) rather than hang. Before the fix
    /// the pit path never enqueued the AI opponents, so `all_actions_submitted()`
    /// stayed false and the turn never processed.
    #[tokio::test]
    async fn pit_resolves_turn_in_solo_mode() {
        let (race_repository, race_uuid, human, map) = seed_solo_race().await;
        assert_eq!(
            race_repository
                .find_by_uuid(race_uuid)
                .await
                .unwrap()
                .unwrap()
                .turns_taken,
            0
        );

        resolve_human_turn(
            &race_repository,
            race_uuid,
            human,
            TurnIntent::Pit(None),
            &map,
        )
        .await
        .expect("pit should succeed")
        .expect("race should still exist");

        let race = race_repository
            .find_by_uuid(race_uuid)
            .await
            .unwrap()
            .unwrap();
        assert!(
            race.pending_actions.is_empty(),
            "the turn must resolve (pending actions cleared), not deadlock"
        );
        assert_eq!(
            race.turns_taken, 1,
            "exactly one turn should have been processed"
        );
        assert_eq!(
            race.participants
                .iter()
                .find(|p| p.player_uuid == human)
                .unwrap()
                .boost_hand
                .pit_stops_completed,
            1,
            "the pit must have refilled the pool"
        );
    }
}
