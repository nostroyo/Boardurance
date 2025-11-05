# Race API Refinement Design Document

## Overview

The Race API Refinement enhances the existing Rust backend race system to provide streamlined, player-centric API endpoints that support real-time racing interactions. This design builds upon the current race domain model and routes to implement specific patterns for player registration, comprehensive race status tracking, and individual lap processing with consistent JSON responses.

## Architecture

### API Endpoint Structure
```
/api/v1/races/{race_uuid}/register     POST   - Player registration
/api/v1/races/{race_uuid}/status       GET    - Race status with track situation
/api/v1/races/{race_uuid}/apply-lap    POST   - Individual lap action processing
/api/v1/races/{race_uuid}/history      GET    - Turn-by-turn race history
```

### Data Flow Architecture
```
Frontend Request → API Validation → Domain Logic → Database Update → JSON Response
     ↓                ↓                ↓              ↓              ↓
Player Action → Car Validation → Race Processing → State Persistence → Status Data
```

### Integration with Existing System
- **Extends**: Current `rust-backend/src/routes/races.rs` with new endpoints
- **Utilizes**: Existing `Race`, `RaceParticipant`, and domain models from `rust-backend/src/domain/race.rs`
- **Enhances**: Current race processing logic with individual action handling
- **Maintains**: Existing authentication and middleware patterns

## Components and Interfaces

### 1. Enhanced Race Routes (`races.rs`)

**New Endpoint Functions**:
```rust
// Player registration endpoint
pub async fn register_player(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<RegisterPlayerRequest>,
) -> Result<Json<RegisterPlayerResponse>, StatusCode>

// Enhanced race status endpoint
pub async fn get_race_status_detailed(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Query(params): Query<StatusQueryParams>,
) -> Result<Json<DetailedRaceStatusResponse>, StatusCode>

// Individual lap processing endpoint
pub async fn apply_lap_action(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<ApplyLapRequest>,
) -> Result<Json<DetailedRaceStatusResponse>, StatusCode>

// Turn history retrieval endpoint
pub async fn get_race_history(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Query(params): Query<HistoryQueryParams>,
) -> Result<Json<RaceHistoryResponse>, StatusCode>
```

### 2. Request/Response Data Models

**Registration Models**:
```rust
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
```

**Status Query Models**:
```rust
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
```

**Lap Action Models**:
```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplyLapRequest {
    pub player_uuid: String,
    pub car_uuid: String,
    pub boost_value: u32, // 0-5
}
```

**History Query Models**:
```rust
#[derive(Debug, Deserialize)]
pub struct HistoryQueryParams {
    pub from_turn: Option<u32>,
    pub to_turn: Option<u32>,
    pub lap_number: Option<u32>,
    pub player_uuid: Option<String>, // Filter for specific player
    pub limit: Option<u32>, // Pagination limit
    pub offset: Option<u32>, // Pagination offset
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RaceHistoryResponse {
    pub race_uuid: String,
    pub total_turns: u32,
    pub turn_snapshots: Vec<TurnSnapshot>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationInfo {
    pub total_count: u32,
    pub limit: u32,
    pub offset: u32,
    pub has_more: bool,
}
```

### 3. Enhanced Response Data Structures

**Race Progress Status**:
```rust
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
```

**Track Situation Data**:
```rust
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
```

**Player-Specific Data**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct PlayerSpecificData {
    pub boost_availability: BoostAvailability,
    pub performance_preview: PerformancePreview,
    pub current_position: CurrentPlayerPosition,
    pub lap_history: Option<Vec<LapPerformanceRecord>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoostAvailability {
    pub available_range: (u32, u32), // (min, max) - typically (0, 5)
    pub current_sector_ceiling: u32,
    pub base_performance: u32,
    pub boost_impact_preview: Vec<BoostImpactOption>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoostImpactOption {
    pub boost_value: u32,
    pub predicted_final_value: u32,
    pub movement_probability: MovementProbability,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MovementProbability {
    pub can_move_up: bool,
    pub can_move_down: bool,
    pub will_stay: bool,
    pub target_sector: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentPlayerPosition {
    pub current_sector: u32,
    pub position_in_sector: u32,
    pub sector_rank: u32,
    pub overall_rank: u32,
    pub distance_to_leader: u32, // In sectors + positions
}
```

### 4. Car Validation Service

**Car Validation Logic**:
```rust
pub struct CarValidationService;

impl CarValidationService {
    pub async fn validate_car_for_race(
        database: &Database,
        player_uuid: Uuid,
        car_uuid: Uuid,
    ) -> Result<ValidatedCarData, CarValidationError> {
        // 1. Verify car exists and belongs to player
        let car = Self::get_car_by_uuid(database, car_uuid).await?;
        Self::verify_car_ownership(player_uuid, &car)?;
        
        // 2. Validate car has all required components
        let engine = Self::get_car_engine(database, &car).await?;
        let body = Self::get_car_body(database, &car).await?;
        let pilot = Self::get_car_pilot(database, &car).await?;
        
        // 3. Return validated car data
        Ok(ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        })
    }
    
    fn verify_car_ownership(player_uuid: Uuid, car: &Car) -> Result<(), CarValidationError> {
        // Implementation for ownership verification
    }
    
    async fn get_car_engine(database: &Database, car: &Car) -> Result<Engine, CarValidationError> {
        // Implementation for engine retrieval and validation
    }
    
    async fn get_car_body(database: &Database, car: &Car) -> Result<Body, CarValidationError> {
        // Implementation for body retrieval and validation
    }
    
    async fn get_car_pilot(database: &Database, car: &Car) -> Result<Pilot, CarValidationError> {
        // Implementation for pilot retrieval and validation
    }
}

#[derive(Debug)]
pub struct ValidatedCarData {
    pub car: Car,
    pub engine: Engine,
    pub body: Body,
    pub pilot: Pilot,
}

#[derive(Debug, thiserror::Error)]
pub enum CarValidationError {
    #[error("Car not found: {0}")]
    CarNotFound(Uuid),
    #[error("Car does not belong to player")]
    InvalidOwnership,
    #[error("Car missing engine component")]
    MissingEngine,
    #[error("Car missing body component")]
    MissingBody,
    #[error("Car missing pilot component")]
    MissingPilot,
    #[error("Database error: {0}")]
    DatabaseError(String),
}
```

### 5. Enhanced Race Processing Logic

**Individual Lap Processing with Turn History**:
```rust
impl Race {
    pub fn process_individual_lap_action(
        &mut self,
        player_uuid: Uuid,
        boost_value: u32,
        car_data: &ValidatedCarData,
    ) -> Result<IndividualLapResult, String> {
        // 1. Validate player is in race and not finished
        let participant = self.get_participant_mut(player_uuid)?;
        if participant.is_finished {
            return Err("Player has already finished the race".to_string());
        }
        
        // 2. Calculate performance using validated car data
        let performance = self.calculate_performance(
            participant,
            boost_value,
            car_data,
            &self.lap_characteristic,
        )?;
        
        // 3. Store action for batch processing
        self.pending_actions.push(LapAction {
            player_uuid,
            boost_value,
        });
        
        // 4. Check if all participants have submitted actions
        if self.all_actions_submitted() {
            // Create turn snapshot BEFORE processing
            let pre_turn_snapshot = self.create_turn_snapshot_before_processing();
            
            // Process all actions simultaneously
            let lap_result = self.process_lap(&self.pending_actions)?;
            
            // Create turn snapshot AFTER processing with actions
            let post_turn_snapshot = self.create_turn_snapshot_after_processing(&self.pending_actions);
            
            // Store both snapshots in turn history
            self.turn_history.push(pre_turn_snapshot);
            self.turn_history.push(post_turn_snapshot);
            self.current_turn += 1;
            
            self.pending_actions.clear();
            
            Ok(IndividualLapResult::LapProcessed(lap_result))
        } else {
            // Return current state with action recorded
            Ok(IndividualLapResult::ActionRecorded {
                predicted_performance: performance,
                waiting_for_players: self.get_pending_players(),
            })
        }
    }
    
    fn create_turn_snapshot_before_processing(&self) -> TurnSnapshot {
        TurnSnapshot {
            turn_number: self.current_turn,
            lap_number: self.current_lap,
            lap_characteristic: self.lap_characteristic.clone(),
            sector_states: self.create_sector_snapshots(),
            participant_actions: Vec::new(), // No actions yet
            timestamp: Utc::now(),
        }
    }
    
    fn create_turn_snapshot_after_processing(&self, actions: &[LapAction]) -> TurnSnapshot {
        TurnSnapshot {
            turn_number: self.current_turn,
            lap_number: self.current_lap,
            lap_characteristic: self.lap_characteristic.clone(),
            sector_states: self.create_sector_snapshots(),
            participant_actions: actions.to_vec(),
            timestamp: Utc::now(),
        }
    }
    
    fn create_sector_snapshots(&self) -> Vec<SectorSnapshot> {
        let mut sector_snapshots = Vec::new();
        
        for sector in &self.track.sectors {
            let participants_in_sector: Vec<ParticipantSnapshot> = self.participants
                .iter()
                .filter(|p| p.current_sector == sector.id && !p.is_finished)
                .map(|p| ParticipantSnapshot {
                    player_uuid: p.player_uuid,
                    car_uuid: p.car_uuid,
                    pilot_uuid: p.pilot_uuid,
                    position_in_sector: p.current_position_in_sector,
                    total_value: p.total_value,
                    current_lap: p.current_lap,
                    performance_this_turn: 0, // Will be updated after processing
                    boost_used: 0, // Will be updated after processing
                })
                .collect();
            
            sector_snapshots.push(SectorSnapshot {
                sector_id: sector.id,
                participants: participants_in_sector,
                sector_info: sector.clone(),
            });
        }
        
        sector_snapshots
    }
}
    
    fn calculate_performance(
        &self,
        participant: &RaceParticipant,
        boost_value: u32,
        car_data: &ValidatedCarData,
        lap_characteristic: &LapCharacteristic,
    ) -> Result<PerformanceCalculation, String> {
        // Enhanced performance calculation using actual car stats
        let engine_value = match lap_characteristic {
            LapCharacteristic::Straight => car_data.engine.straight_value,
            LapCharacteristic::Curve => car_data.engine.curve_value,
        };
        
        let body_value = match lap_characteristic {
            LapCharacteristic::Straight => car_data.body.straight_value,
            LapCharacteristic::Curve => car_data.body.curve_value,
        };
        
        let pilot_value = match lap_characteristic {
            LapCharacteristic::Straight => car_data.pilot.performance.straight_value,
            LapCharacteristic::Curve => car_data.pilot.performance.curve_value,
        };
        
        let base_value = engine_value + body_value + pilot_value;
        let current_sector = &self.track.sectors[participant.current_sector as usize];
        let capped_base_value = std::cmp::min(base_value, current_sector.max_value);
        let final_value = capped_base_value + boost_value;
        
        Ok(PerformanceCalculation {
            engine_contribution: engine_value,
            body_contribution: body_value,
            pilot_contribution: pilot_value,
            base_value,
            sector_ceiling: current_sector.max_value,
            capped_base_value,
            boost_value,
            final_value,
        })
    }
}

#[derive(Debug)]
pub enum IndividualLapResult {
    ActionRecorded {
        predicted_performance: PerformanceCalculation,
        waiting_for_players: Vec<Uuid>,
    },
    LapProcessed(LapResult),
}

#[derive(Debug)]
pub struct PerformanceCalculation {
    pub engine_contribution: u32,
    pub body_contribution: u32,
    pub pilot_contribution: u32,
    pub base_value: u32,
    pub sector_ceiling: u32,
    pub capped_base_value: u32,
    pub boost_value: u32,
    pub final_value: u32,
}
```

## Data Models

### Enhanced Race Domain Extensions

**Pending Actions Tracking**:
```rust
// Add to existing Race struct
impl Race {
    pub pending_actions: Vec<LapAction>,
    pub action_submissions: HashMap<Uuid, DateTime<Utc>>, // Track submission times
    
    pub fn all_actions_submitted(&self) -> bool {
        let active_participants: HashSet<Uuid> = self.participants
            .iter()
            .filter(|p| !p.is_finished)
            .map(|p| p.player_uuid)
            .collect();
            
        let submitted_actions: HashSet<Uuid> = self.pending_actions
            .iter()
            .map(|a| a.player_uuid)
            .collect();
            
        active_participants == submitted_actions
    }
    
    pub fn get_pending_players(&self) -> Vec<Uuid> {
        let submitted: HashSet<Uuid> = self.pending_actions
            .iter()
            .map(|a| a.player_uuid)
            .collect();
            
        self.participants
            .iter()
            .filter(|p| !p.is_finished && !submitted.contains(&p.player_uuid))
            .map(|p| p.player_uuid)
            .collect()
    }
}
```

### Database Schema Extensions

**Race Status Tracking**:
```rust
// Additional fields for race document
#[derive(Debug, Serialize, Deserialize)]
pub struct RaceStatusTracking {
    pub turn_phase: TurnPhase,
    pub last_action_timestamp: DateTime<Utc>,
    pub pending_action_count: u32,
    pub total_active_participants: u32,
}

// Turn History Storage
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TurnSnapshot {
    pub turn_number: u32,
    pub lap_number: u32,
    pub lap_characteristic: LapCharacteristic,
    pub sector_states: Vec<SectorSnapshot>,
    pub participant_actions: Vec<LapAction>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SectorSnapshot {
    pub sector_id: u32,
    pub participants: Vec<ParticipantSnapshot>,
    pub sector_info: Sector, // Complete sector configuration
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParticipantSnapshot {
    pub player_uuid: Uuid,
    pub car_uuid: Uuid,
    pub pilot_uuid: Uuid,
    pub position_in_sector: u32,
    pub total_value: u32,
    pub current_lap: u32,
    pub performance_this_turn: u32,
    pub boost_used: u32,
}

// Add to Race struct
impl Race {
    pub status_tracking: RaceStatusTracking,
    pub turn_history: Vec<TurnSnapshot>, // Complete history of all turns
    pub current_turn: u32,
}
```

## API Integration

### Endpoint Implementation Strategy

**1. Register Player Endpoint**:
```rust
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
pub async fn register_player(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<RegisterPlayerRequest>,
) -> Result<Json<RegisterPlayerResponse>, StatusCode> {
    // 1. Parse and validate UUIDs
    let race_uuid = parse_uuid(&race_uuid_str)?;
    let player_uuid = parse_uuid(&payload.player_uuid)?;
    let car_uuid = parse_uuid(&payload.car_uuid)?;
    
    // 2. Validate car and get components
    let car_data = CarValidationService::validate_car_for_race(
        &database, 
        player_uuid, 
        car_uuid
    ).await.map_err(|e| {
        tracing::warn!("Car validation failed: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    
    // 3. Register player in race
    let updated_race = register_player_in_race(
        &database,
        race_uuid,
        player_uuid,
        car_uuid,
        car_data.pilot.uuid,
    ).await?;
    
    // 4. Build response
    let player_position = get_player_race_position(&updated_race, player_uuid)?;
    let race_status = build_race_progress_status(&updated_race);
    
    Ok(Json(RegisterPlayerResponse {
        success: true,
        message: "Successfully registered for race".to_string(),
        race_status,
        player_position,
    }))
}
```

**2. Enhanced Status Endpoint**:
```rust
pub async fn get_race_status_detailed(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Query(params): Query<StatusQueryParams>,
) -> Result<Json<DetailedRaceStatusResponse>, StatusCode> {
    let race_uuid = parse_uuid(&race_uuid_str)?;
    let race = get_race_by_uuid(&database, race_uuid).await?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    // Build comprehensive status response
    let race_progress = build_race_progress_status(&race);
    let track_situation = build_track_situation_data(&database, &race).await?;
    let race_metadata = build_race_metadata(&race);
    
    // Include player-specific data if requested
    let player_data = if let Some(player_uuid_str) = params.player_uuid {
        let player_uuid = parse_uuid(&player_uuid_str)?;
        Some(build_player_specific_data(&database, &race, player_uuid).await?)
    } else {
        None
    };
    
    Ok(Json(DetailedRaceStatusResponse {
        race_progress,
        track_situation,
        player_data,
        race_metadata,
    }))
}
```

**3. Apply Lap Action Endpoint**:
```rust
pub async fn apply_lap_action(
    State(database): State<Database>,
    Path(race_uuid_str): Path<String>,
    Json(payload): Json<ApplyLapRequest>,
) -> Result<Json<DetailedRaceStatusResponse>, StatusCode> {
    let race_uuid = parse_uuid(&race_uuid_str)?;
    let player_uuid = parse_uuid(&payload.player_uuid)?;
    let car_uuid = parse_uuid(&payload.car_uuid)?;
    
    // Validate boost value
    if payload.boost_value > 5 {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Validate car data
    let car_data = CarValidationService::validate_car_for_race(
        &database,
        player_uuid,
        car_uuid,
    ).await.map_err(|_| StatusCode::BAD_REQUEST)?;
    
    // Process individual lap action
    let updated_race = process_individual_lap_action(
        &database,
        race_uuid,
        player_uuid,
        payload.boost_value,
        &car_data,
    ).await?;
    
    // Return same format as status endpoint
    let race_progress = build_race_progress_status(&updated_race);
    let track_situation = build_track_situation_data(&database, &updated_race).await?;
    let race_metadata = build_race_metadata(&updated_race);
    let player_data = Some(build_player_specific_data(&database, &updated_race, player_uuid).await?);
    
    Ok(Json(DetailedRaceStatusResponse {
        race_progress,
        track_situation,
        player_data,
        race_metadata,
    }))
}
```

## Error Handling

### Comprehensive Error Response System

**Error Response Format**:
```rust
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub error: ApiError,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub suggested_action: Option<String>,
}

// Error code constants
pub mod error_codes {
    pub const RACE_NOT_FOUND: &str = "RACE_NOT_FOUND";
    pub const RACE_ALREADY_STARTED: &str = "RACE_ALREADY_STARTED";
    pub const PLAYER_ALREADY_REGISTERED: &str = "PLAYER_ALREADY_REGISTERED";
    pub const CAR_VALIDATION_FAILED: &str = "CAR_VALIDATION_FAILED";
    pub const INVALID_BOOST_VALUE: &str = "INVALID_BOOST_VALUE";
    pub const PLAYER_NOT_IN_RACE: &str = "PLAYER_NOT_IN_RACE";
    pub const RACE_NOT_IN_PROGRESS: &str = "RACE_NOT_IN_PROGRESS";
    pub const ACTION_ALREADY_SUBMITTED: &str = "ACTION_ALREADY_SUBMITTED";
}
```

**Error Handling Middleware**:
```rust
pub fn create_error_response(
    error_code: &str,
    message: &str,
    details: Option<serde_json::Value>,
    suggested_action: Option<&str>,
) -> ApiErrorResponse {
    ApiErrorResponse {
        error: ApiError {
            code: error_code.to_string(),
            message: message.to_string(),
            details,
            suggested_action: suggested_action.map(|s| s.to_string()),
        },
        timestamp: Utc::now(),
        request_id: generate_request_id(),
    }
}
```

## Testing Strategy

### Unit Testing Approach

**Car Validation Tests**:
```rust
#[cfg(test)]
mod car_validation_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_valid_car_registration() {
        // Test complete car with all components
    }
    
    #[tokio::test]
    async fn test_car_missing_engine() {
        // Test car validation failure for missing engine
    }
    
    #[tokio::test]
    async fn test_car_ownership_validation() {
        // Test car ownership verification
    }
}
```

**API Endpoint Tests**:
```rust
#[cfg(test)]
mod api_tests {
    #[tokio::test]
    async fn test_register_player_success() {
        // Test successful player registration
    }
    
    #[tokio::test]
    async fn test_register_player_race_started() {
        // Test registration failure when race already started
    }
    
    #[tokio::test]
    async fn test_status_endpoint_comprehensive() {
        // Test detailed status response format
    }
    
    #[tokio::test]
    async fn test_apply_lap_individual_action() {
        // Test individual lap action processing
    }
}
```

### Integration Testing

**End-to-End Race Flow**:
```rust
#[tokio::test]
async fn test_complete_race_flow() {
    // 1. Create race
    // 2. Register multiple players
    // 3. Start race
    // 4. Process individual lap actions
    // 5. Verify status consistency
    // 6. Complete race
}
```

## Performance Considerations

### Optimization Strategies

**Database Query Optimization**:
- Index race UUID for fast lookups
- Cache frequently accessed race data
- Batch participant data retrieval
- Optimize sector position calculations
- Index turn_history by turn_number and lap_number for efficient historical queries
- Implement turn history pagination for large races
- Use MongoDB aggregation pipelines for turn history analysis

**Response Caching**:
- Cache track situation data for active races
- Invalidate cache on race state changes
- Use Redis for distributed caching
- Implement cache warming for popular races

**Concurrent Action Processing**:
- Use database transactions for atomic updates
- Implement optimistic locking for race state
- Queue individual actions for batch processing
- Handle concurrent status requests efficiently

## Security Considerations

### Authentication and Authorization

**Player Verification**:
- Validate JWT tokens for all endpoints
- Verify player ownership of cars
- Implement rate limiting per player
- Log all race interactions for audit

**Data Validation**:
- Sanitize all input parameters
- Validate UUID formats strictly
- Enforce boost value constraints
- Prevent race state manipulation

**API Security**:
- Use HTTPS for all communications
- Implement CORS policies
- Add request signing for critical operations
- Monitor for suspicious activity patterns

This design provides a comprehensive foundation for implementing the refined race API system while maintaining compatibility with the existing codebase and ensuring robust, scalable race management capabilities.