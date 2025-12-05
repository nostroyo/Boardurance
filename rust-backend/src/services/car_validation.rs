use mongodb::{bson::doc, Database};
use uuid::Uuid;

use crate::domain::{Body, Car, Engine, Pilot, Player};

/// Service for validating cars and their components for race participation
pub struct CarValidationService;

/// Validated car data containing all required components
#[derive(Debug, Clone)]
pub struct ValidatedCarData {
    pub car: Car,
    pub engine: Engine,
    pub body: Body,
    pub pilot: Pilot,
}

/// Errors that can occur during car validation
#[derive(Debug, thiserror::Error)]
pub enum CarValidationError {
    #[error("Car not found: {0}")]
    CarNotFound(Uuid),
    #[error("Car does not belong to player {player_uuid}")]
    InvalidOwnership { player_uuid: Uuid },
    #[error("Player not found: {0}")]
    PlayerNotFound(Uuid),
    #[error("Car missing engine component")]
    MissingEngine,
    #[error("Car missing body component")]
    MissingBody,
    #[error("Car missing pilot component")]
    MissingPilot,
    #[error("Engine not found: {0}")]
    EngineNotFound(Uuid),
    #[error("Body not found: {0}")]
    BodyNotFound(Uuid),
    #[error("Pilot not found: {0}")]
    PilotNotFound(Uuid),
    #[error("Car is not complete - missing required components")]
    IncompleteCarConfiguration,
    #[error("Component ownership mismatch - {component_type} {component_uuid} does not belong to player {player_uuid}")]
    ComponentOwnershipMismatch {
        component_type: String,
        component_uuid: Uuid,
        player_uuid: Uuid,
    },
    #[error("Database connection error: {0}")]
    DatabaseConnectionError(String),
    #[error("Database query error: {0}")]
    DatabaseQueryError(String),
    #[error("Invalid car configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Database serialization error: {0}")]
    DatabaseSerializationError(String),
}

impl CarValidationError {
    /// Returns the error code for API responses
    #[must_use] 
    pub fn error_code(&self) -> &'static str {
        match self {
            CarValidationError::CarNotFound(_) => "CAR_NOT_FOUND",
            CarValidationError::InvalidOwnership { .. } => "INVALID_CAR_OWNERSHIP",
            CarValidationError::PlayerNotFound(_) => "PLAYER_NOT_FOUND",
            CarValidationError::MissingEngine => "MISSING_ENGINE_COMPONENT",
            CarValidationError::MissingBody => "MISSING_BODY_COMPONENT",
            CarValidationError::MissingPilot => "MISSING_PILOT_COMPONENT",
            CarValidationError::EngineNotFound(_) => "ENGINE_NOT_FOUND",
            CarValidationError::BodyNotFound(_) => "BODY_NOT_FOUND",
            CarValidationError::PilotNotFound(_) => "PILOT_NOT_FOUND",
            CarValidationError::IncompleteCarConfiguration => "INCOMPLETE_CAR_CONFIGURATION",
            CarValidationError::ComponentOwnershipMismatch { .. } => "COMPONENT_OWNERSHIP_MISMATCH",
            CarValidationError::DatabaseConnectionError(_) => "DATABASE_CONNECTION_ERROR",
            CarValidationError::DatabaseQueryError(_) => "DATABASE_QUERY_ERROR",
            CarValidationError::DatabaseSerializationError(_) => "DATABASE_SERIALIZATION_ERROR",
            CarValidationError::InvalidConfiguration(_) => "INVALID_CAR_CONFIGURATION",
        }
    }

    /// Returns a user-friendly message for API responses
    #[must_use] 
    pub fn user_message(&self) -> String {
        match self {
            CarValidationError::CarNotFound(uuid) => {
                format!("The specified car ({uuid}) was not found in your inventory")
            }
            CarValidationError::InvalidOwnership { player_uuid } => {
                format!("You do not own this car. Player UUID: {player_uuid}")
            }
            CarValidationError::PlayerNotFound(uuid) => {
                format!("Player not found: {uuid}")
            }
            CarValidationError::MissingEngine => {
                "This car is missing an engine. Please equip an engine before racing.".to_string()
            }
            CarValidationError::MissingBody => {
                "This car is missing a body. Please equip a body before racing.".to_string()
            }
            CarValidationError::MissingPilot => {
                "This car is missing a pilot. Please assign a pilot before racing.".to_string()
            }
            CarValidationError::EngineNotFound(uuid) => {
                format!("The engine ({uuid}) assigned to this car was not found")
            }
            CarValidationError::BodyNotFound(uuid) => {
                format!("The body ({uuid}) assigned to this car was not found")
            }
            CarValidationError::PilotNotFound(uuid) => {
                format!("The pilot ({uuid}) assigned to this car was not found")
            }
            CarValidationError::IncompleteCarConfiguration => {
                "This car is not properly configured. Please ensure it has an engine, body, and pilot.".to_string()
            }
            CarValidationError::ComponentOwnershipMismatch { component_type, component_uuid, .. } => {
                format!("The {component_type} ({component_uuid}) does not belong to you")
            }
            CarValidationError::DatabaseConnectionError(_) => {
                "Unable to connect to the database. Please try again later.".to_string()
            }
            CarValidationError::DatabaseQueryError(_) => {
                "Database query failed. Please try again later.".to_string()
            }
            CarValidationError::DatabaseSerializationError(_) => {
                "Data processing error. Please try again later.".to_string()
            }
            CarValidationError::InvalidConfiguration(msg) => {
                format!("Invalid car configuration: {msg}")
            }
        }
    }

    /// Returns suggested actions for resolving the error
    #[must_use] 
    pub fn suggested_action(&self) -> Option<String> {
        match self {
            CarValidationError::CarNotFound(_) => Some(
                "Check your car inventory and ensure you're using the correct car UUID."
                    .to_string(),
            ),
            CarValidationError::InvalidOwnership { .. } => Some(
                "Verify that you own this car and are using the correct player credentials."
                    .to_string(),
            ),
            CarValidationError::PlayerNotFound(_) => {
                Some("Verify your player credentials and try logging in again.".to_string())
            }
            CarValidationError::MissingEngine => Some(
                "Go to your inventory, select an engine, and equip it to this car.".to_string(),
            ),
            CarValidationError::MissingBody => {
                Some("Go to your inventory, select a body, and equip it to this car.".to_string())
            }
            CarValidationError::MissingPilot => Some(
                "Go to your inventory, select a pilot, and assign them to this car.".to_string(),
            ),
            CarValidationError::EngineNotFound(_) => Some(
                "The engine may have been removed. Please equip a different engine to this car."
                    .to_string(),
            ),
            CarValidationError::BodyNotFound(_) => Some(
                "The body may have been removed. Please equip a different body to this car."
                    .to_string(),
            ),
            CarValidationError::PilotNotFound(_) => Some(
                "The pilot may have been removed. Please assign a different pilot to this car."
                    .to_string(),
            ),
            CarValidationError::IncompleteCarConfiguration => Some(
                "Complete your car setup by equipping an engine, body, and assigning a pilot."
                    .to_string(),
            ),
            CarValidationError::ComponentOwnershipMismatch { .. } => {
                Some("Ensure all car components belong to your account.".to_string())
            }
            CarValidationError::DatabaseConnectionError(_)
            | CarValidationError::DatabaseQueryError(_)
            | CarValidationError::DatabaseSerializationError(_) => Some(
                "Please try again in a few moments. If the problem persists, contact support."
                    .to_string(),
            ),
            CarValidationError::InvalidConfiguration(_) => Some(
                "Please check your car configuration and ensure all pilots are properly assigned."
                    .to_string(),
            ),
        }
    }
}

impl CarValidationService {
    /// Validates a car for race participation
    ///
    /// This method performs comprehensive validation:
    /// 1. Verifies the car exists and belongs to the specified player
    /// 2. Validates the car has all required components (engine, body, pilot)
    /// 3. Returns validated car data with all components
    ///
    /// # Arguments
    /// * `database` - `MongoDB` database connection
    /// * `player_uuid` - UUID of the player who owns the car
    /// * `car_uuid` - UUID of the car to validate
    ///
    /// # Returns
    /// * `Ok(ValidatedCarData)` - Car and all components if validation passes
    /// * `Err(CarValidationError)` - Specific error if validation fails
    pub async fn validate_car_for_race(
        database: &Database,
        player_uuid: Uuid,
        car_uuid: Uuid,
    ) -> Result<ValidatedCarData, CarValidationError> {
        // 1. Get the player and verify car ownership
        let player = Self::get_player_by_uuid(database, player_uuid).await?;
        let car = Self::verify_car_ownership(&player, car_uuid)?;

        // 2. Validate car has all required components
        let engine = Self::get_car_engine(&car, &player)?;
        let body = Self::get_car_body(&car, &player)?;
        let pilot = Self::get_car_pilot(&car, &player)?;

        // 3. Return validated car data
        Ok(ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        })
    }

    /// Gets a player by UUID from the database
    async fn get_player_by_uuid(
        database: &Database,
        player_uuid: Uuid,
    ) -> Result<Player, CarValidationError> {
        let collection = database.collection::<Player>("players");
        let filter = doc! { "uuid": player_uuid.to_string() };

        match collection.find_one(filter, None).await {
            Ok(Some(player)) => Ok(player),
            Ok(None) => Err(CarValidationError::PlayerNotFound(player_uuid)),
            Err(e) => {
                if e.to_string().contains("connection") {
                    Err(CarValidationError::DatabaseConnectionError(e.to_string()))
                } else {
                    Err(CarValidationError::DatabaseQueryError(e.to_string()))
                }
            }
        }
    }

    /// Verifies that the car belongs to the specified player
    fn verify_car_ownership(player: &Player, car_uuid: Uuid) -> Result<Car, CarValidationError> {
        let car = player
            .cars
            .iter()
            .find(|car| car.uuid == car_uuid)
            .cloned()
            .ok_or(CarValidationError::CarNotFound(car_uuid))?;

        // Additional validation: check if car is complete
        if !car.is_complete() {
            return Err(CarValidationError::IncompleteCarConfiguration);
        }

        Ok(car)
    }

    /// Gets the engine component for the car
    fn get_car_engine(car: &Car, player: &Player) -> Result<Engine, CarValidationError> {
        let engine_uuid = car.engine_uuid.ok_or(CarValidationError::MissingEngine)?;

        // First try to find the engine in the player's inventory
        if let Some(engine) = player.engines.iter().find(|e| e.uuid == engine_uuid) {
            return Ok(engine.clone());
        }

        // If not found in player inventory, this is an ownership issue
        // The car references an engine that doesn't belong to the player
        Err(CarValidationError::ComponentOwnershipMismatch {
            component_type: "engine".to_string(),
            component_uuid: engine_uuid,
            player_uuid: player.uuid,
        })
    }

    /// Gets the body component for the car
    fn get_car_body(car: &Car, player: &Player) -> Result<Body, CarValidationError> {
        let body_uuid = car.body_uuid.ok_or(CarValidationError::MissingBody)?;

        // First try to find the body in the player's inventory
        if let Some(body) = player.bodies.iter().find(|b| b.uuid == body_uuid) {
            return Ok(body.clone());
        }

        // If not found in player inventory, this is an ownership issue
        // The car references a body that doesn't belong to the player
        Err(CarValidationError::ComponentOwnershipMismatch {
            component_type: "body".to_string(),
            component_uuid: body_uuid,
            player_uuid: player.uuid,
        })
    }

    /// Gets the primary pilot component for the car (first pilot in the list)
    fn get_car_pilot(car: &Car, player: &Player) -> Result<Pilot, CarValidationError> {
        // Validate that car has exactly 3 pilots
        car.validate_pilots().map_err(|e| CarValidationError::InvalidConfiguration(e))?;
        
        // Use the first pilot as the primary pilot for validation
        let pilot_uuid = car.pilot_uuids.first().ok_or(CarValidationError::MissingPilot)?;

        // First try to find the pilot in the player's inventory
        if let Some(pilot) = player.pilots.iter().find(|p| p.uuid == *pilot_uuid) {
            return Ok(pilot.clone());
        }

        // If not found in player inventory, this is an ownership issue
        // The car references a pilot that doesn't belong to the player
        Err(CarValidationError::ComponentOwnershipMismatch {
            component_type: "pilot".to_string(),
            component_uuid: *pilot_uuid,
            player_uuid: player.uuid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        BodyName, CarName, ComponentRarity, Email, EngineName, Password, PilotClass, PilotName,
        PilotPerformance, PilotRarity, PilotSkills, TeamName,
    };

    fn create_test_engine() -> Engine {
        Engine::new(
            EngineName::parse("Test Engine").unwrap(),
            ComponentRarity::Common,
            5,
            4,
            None,
        )
        .unwrap()
    }

    fn create_test_body() -> Body {
        Body::new(
            BodyName::parse("Test Body").unwrap(),
            ComponentRarity::Common,
            4,
            5,
            None,
        )
        .unwrap()
    }

    fn create_test_pilot() -> Pilot {
        let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
        let performance = PilotPerformance::new(3, 3).unwrap();

        Pilot::new(
            PilotName::parse("Test Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Professional,
            skills,
            performance,
            None,
        )
        .unwrap()
    }

    fn create_test_car_with_components(engine: &Engine, body: &Body, pilots: &[Pilot; 3]) -> Car {
        let mut car = Car::new(CarName::parse("Test Car").unwrap(), None).unwrap();

        car.assign_engine(engine.uuid);
        car.assign_body(body.uuid);
        car.assign_pilots(vec![pilots[0].uuid, pilots[1].uuid, pilots[2].uuid]).unwrap();

        car
    }

    fn create_test_player_with_assets(
        car: Car,
        engine: Engine,
        body: Body,
        pilots: [Pilot; 3],
    ) -> Player {
        let email = Email::parse("test@example.com").unwrap();
        let password_hash = Password::new("TestPassword123".to_string())
            .unwrap()
            .hash()
            .unwrap();
        let team_name = TeamName::parse("Test Team").unwrap();

        Player::new_with_assets(
            email,
            password_hash,
            team_name,
            vec![car],
            vec![pilots[0].clone(), pilots[1].clone(), pilots[2].clone()],
            vec![engine],
            vec![body],
        )
        .unwrap()
    }

    #[test]
    fn test_verify_car_ownership_success() {
        let engine = create_test_engine();
        let body = create_test_body();
        let pilots = [create_test_pilot(), create_test_pilot(), create_test_pilot()];
        let car = create_test_car_with_components(&engine, &body, &pilots);
        let player = create_test_player_with_assets(car.clone(), engine, body, pilots);

        let result = CarValidationService::verify_car_ownership(&player, car.uuid);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().uuid, car.uuid);
    }

    #[test]
    fn test_verify_car_ownership_car_not_found() {
        let engine = create_test_engine();
        let body = create_test_body();
        let pilots = [create_test_pilot(), create_test_pilot(), create_test_pilot()];
        let car = create_test_car_with_components(&engine, &body, &pilots);
        let player = create_test_player_with_assets(car, engine, body, pilots);

        let non_existent_car_uuid = Uuid::new_v4();
        let result = CarValidationService::verify_car_ownership(&player, non_existent_car_uuid);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CarValidationError::CarNotFound(_)
        ));
    }

    #[test]
    fn test_car_missing_components() {
        let engine = create_test_engine();
        let body = create_test_body();
        let pilots = [create_test_pilot(), create_test_pilot(), create_test_pilot()];

        // Create car without components
        let car = Car::new(CarName::parse("Incomplete Car").unwrap(), None).unwrap();
        let _player = create_test_player_with_assets(car.clone(), engine, body, pilots);

        // Test missing engine
        assert!(car.engine_uuid.is_none());

        // Test missing body
        assert!(car.body_uuid.is_none());

        // Test missing pilots
        assert!(car.pilot_uuids.is_empty());

        // Verify car is not complete
        assert!(!car.is_complete());
    }

    #[test]
    #[test]
    fn test_incomplete_car_configuration_error() {
        let engine = create_test_engine();
        let body = create_test_body();
        let pilot1 = create_test_pilot();
        let pilot2 = create_test_pilot();
        let pilot3 = create_test_pilot();

        // Create a complete car first (required by new_with_assets)
        let complete_car = Car::new(CarName::parse("Complete Car").unwrap(), None).unwrap();
        
        // Create an incomplete second car
        let incomplete_car = Car::new(CarName::parse("Incomplete Car").unwrap(), None).unwrap();

        // Create a player with both cars - new_with_assets will complete the first car
        let email = Email::parse("test@example.com").unwrap();
        let password_hash = Password::new("TestPassword123".to_string())
            .unwrap()
            .hash()
            .unwrap();
        let team_name = TeamName::parse("Test Team").unwrap();
        let player_with_incomplete_car = Player::new_with_assets(
            email,
            password_hash,
            team_name,
            vec![complete_car, incomplete_car.clone()],
            vec![pilot1, pilot2, pilot3],
            vec![engine],
            vec![body],
        )
        .unwrap();

        // Try to verify ownership of the incomplete car (second car)
        let result =
            CarValidationService::verify_car_ownership(&player_with_incomplete_car, incomplete_car.uuid);
        
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CarValidationError::IncompleteCarConfiguration
        ));
    }

    #[test]
    fn test_error_codes() {
        let car_uuid = Uuid::new_v4();
        let player_uuid = Uuid::new_v4();
        let engine_uuid = Uuid::new_v4();

        // Test error codes
        assert_eq!(
            CarValidationError::CarNotFound(car_uuid).error_code(),
            "CAR_NOT_FOUND"
        );

        assert_eq!(
            CarValidationError::InvalidOwnership { player_uuid }.error_code(),
            "INVALID_CAR_OWNERSHIP"
        );

        assert_eq!(
            CarValidationError::MissingEngine.error_code(),
            "MISSING_ENGINE_COMPONENT"
        );

        assert_eq!(
            CarValidationError::ComponentOwnershipMismatch {
                component_type: "engine".to_string(),
                component_uuid: engine_uuid,
                player_uuid,
            }
            .error_code(),
            "COMPONENT_OWNERSHIP_MISMATCH"
        );
    }

    #[test]
    fn test_user_messages() {
        let car_uuid = Uuid::new_v4();
        let _player_uuid = Uuid::new_v4();

        let error = CarValidationError::CarNotFound(car_uuid);
        let message = error.user_message();
        assert!(message.contains(&car_uuid.to_string()));
        assert!(message.contains("not found in your inventory"));

        let error = CarValidationError::MissingEngine;
        let message = error.user_message();
        assert!(message.contains("missing an engine"));
        assert!(message.contains("Please equip an engine"));
    }

    #[test]
    fn test_suggested_actions() {
        let error = CarValidationError::MissingEngine;
        let action = error.suggested_action();
        assert!(action.is_some());
        assert!(action.unwrap().contains("Go to your inventory"));

        let error = CarValidationError::DatabaseConnectionError("test".to_string());
        let action = error.suggested_action();
        assert!(action.is_some());
        assert!(action.unwrap().contains("try again"));
    }
}
