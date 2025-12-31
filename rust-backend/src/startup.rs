#![allow(clippy::needless_for_each)]

use crate::app_state::AppState;
use crate::configuration::{DatabaseSettings, Settings};
use crate::middleware::{AuthMiddleware, RequireRole};
use crate::routes::{auth, health_check, players, races};
use crate::services::{JwtConfig, JwtService, SessionConfig, SessionManager};
use axum::{routing::get, Router};
use mongodb::{Client, Database};
use std::sync::Arc;

use axum::http::Method;
use tokio::net::TcpListener as TokioTcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub struct Application {
    port: u16,
    server: axum::serve::Serve<Router, Router>,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        let connection_pool = match get_connection_pool(&configuration.database).await {
            Ok(pool) => {
                tracing::info!("Successfully connected to MongoDB");
                pool
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to connect to MongoDB: {}. Server will run in degraded mode.",
                    e
                );
                // Create a mock database for testing
                let client = mongodb::Client::with_uri_str("mongodb://localhost:27017")
                    .await
                    .unwrap();
                client.database("mock_database")
            }
        };

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TokioTcpListener::bind(&address).await?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            connection_pool,
            configuration.application.base_url,
        )
        .await?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health_check,
        crate::routes::players::get_all_players,
        crate::routes::players::get_player_by_uuid,
        crate::routes::players::get_player_by_wallet,
        crate::routes::players::get_player_by_email,
        crate::routes::players::connect_wallet,
        crate::routes::players::disconnect_wallet,
        crate::routes::players::update_player_team_name,
        crate::routes::players::delete_player,
        crate::routes::players::add_car_to_player,
        crate::routes::players::remove_car_from_player,
        crate::routes::players::add_pilot_to_player,
        crate::routes::players::remove_pilot_from_player,
        crate::routes::races::create_race,
        crate::routes::races::get_all_races,
        crate::routes::races::get_race,
        crate::routes::races::join_race,
        crate::routes::races::start_race,
        crate::routes::races::process_turn,
        crate::routes::races::get_race_status,
        crate::routes::races::register_player,
        crate::routes::races::get_race_status_detailed,
        crate::routes::races::apply_lap_action,
        crate::routes::races::get_car_data,
        crate::routes::races::get_performance_preview,
        crate::routes::races::get_turn_phase,
        crate::routes::races::get_local_view,
        crate::routes::races::get_boost_availability,
        crate::routes::races::get_lap_history,
        crate::routes::races::submit_turn_action,
        crate::routes::auth::register_user,
        crate::routes::auth::login_user,
    ),
    components(
        schemas(
            crate::domain::Player,
            crate::domain::Car,
            crate::domain::Pilot,
            crate::domain::Engine,
            crate::domain::Body,
            crate::domain::ComponentRarity,
            crate::domain::PilotClass,
            crate::domain::PilotRarity,
            crate::domain::PilotSkills,
            crate::domain::PilotClassBonus,
            crate::domain::Race,
            crate::domain::Track,
            crate::domain::Sector,
            crate::domain::SectorType,
            crate::domain::RaceParticipant,
            crate::domain::RaceStatus,
            crate::domain::LapAction,
            crate::domain::LapResult,
            crate::domain::ParticipantMovement,
            crate::domain::MovementType,
            // Domain value objects
            crate::domain::Email,
            crate::domain::TeamName,
            crate::domain::WalletAddress,
            crate::domain::CarName,
            crate::domain::PilotName,
            crate::domain::EngineName,
            crate::domain::BodyName,
            crate::domain::PilotPerformance,
            // Route DTOs
            crate::routes::players::ConnectWalletRequest,
            crate::routes::players::UpdateTeamNameRequest,
            crate::routes::players::AddCarRequest,
            crate::routes::players::AddPilotRequest,
            crate::routes::players::PilotSkillsRequest,
            crate::routes::players::PlayerResponse,
            crate::routes::races::CreateRaceRequest,
            crate::routes::races::CreateSectorRequest,
            crate::routes::races::JoinRaceRequest,
            crate::routes::races::ProcessLapRequest,
            crate::routes::races::LapActionRequest,
            crate::routes::races::SubmitTurnActionRequest,
            crate::routes::races::SubmitTurnActionResponse,
            crate::routes::races::RaceResponse,
            crate::routes::races::LapResultResponse,
            // New API response models
            crate::routes::races::RegisterPlayerRequest,
            crate::routes::races::RegisterPlayerResponse,
            crate::routes::races::PlayerRacePosition,
            crate::routes::races::DetailedRaceStatusResponse,
            crate::routes::races::RaceProgressStatus,
            crate::routes::races::RaceStatusType,
            crate::routes::races::TurnPhase,
            crate::routes::races::TrackSituationData,
            crate::routes::races::SectorSituation,
            crate::routes::races::SectorCapacityInfo,
            crate::routes::races::SectorParticipant,
            crate::routes::races::PerformanceThresholds,
            crate::routes::races::ParticipantMovement,
            crate::routes::races::LeaderboardEntry,
            crate::routes::races::PlayerSpecificData,
            crate::routes::races::PerformancePreview,
            crate::routes::races::CurrentPlayerPosition,
            crate::routes::races::LapPerformanceRecord,
            crate::routes::races::RaceMetadata,
            crate::routes::races::ApplyLapRequest,
            crate::routes::races::CarDataResponse,
            crate::routes::races::CarInfo,
            crate::routes::races::PilotInfo,
            crate::routes::races::PilotSkills,
            crate::routes::races::PilotPerformance,
            crate::routes::races::EngineInfo,
            crate::routes::races::BodyInfo,
            crate::routes::races::PerformancePreviewResponse,
            crate::routes::races::BasePerformance,
            crate::routes::races::BoostOption,
            crate::routes::races::BoostCycleInfo,
            crate::routes::races::TurnPhaseResponse,
            crate::routes::races::LocalViewResponse,
            crate::routes::races::SectorInfo,
            crate::routes::races::ParticipantInfo,
            crate::routes::races::BoostAvailabilityResponse,
            crate::routes::races::LapHistoryResponse,
            crate::routes::races::LapRecord,
            crate::routes::races::CycleSummary,
            crate::routes::races::ErrorResponse,
            crate::routes::HealthResponse,
            crate::domain::UserRegistration,
            crate::domain::UserCredentials,
            crate::domain::HashedPassword,
            // Boost Hand System schemas
            crate::domain::BoostHand,
            crate::domain::BoostUsageRecord,
            crate::domain::BoostCycleSummary,
            crate::domain::boost_hand_manager::BoostCardError,
            crate::domain::boost_hand_manager::BoostUsageResult,
            crate::domain::boost_hand_manager::BoostAvailability,
            crate::domain::boost_hand_manager::BoostImpactOption,
            crate::domain::boost_hand_manager::BoostCardErrorResponse
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "test", description = "Test endpoints"),
        (name = "players", description = "Player management endpoints"),
        (name = "races", description = "Race management and gameplay endpoints"),
        (name = "boost-cards", description = "Boost card system endpoints with strategic resource management"),
        (name = "Authentication", description = "User authentication endpoints")
    )
)]
struct ApiDoc;

#[allow(clippy::unused_async)]
pub async fn run(
    listener: TokioTcpListener,
    db_pool: Database,
    _base_url: String,
) -> Result<axum::serve::Serve<Router, Router>, anyhow::Error> {
    // Initialize JWT service
    let jwt_config = JwtConfig {
        secret: std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-super-secret-jwt-key-change-this-in-production".to_string()),
        access_token_expiry: std::time::Duration::from_secs(30 * 60), // 30 minutes
        refresh_token_expiry: std::time::Duration::from_secs(30 * 24 * 60 * 60), // 30 days
        issuer: "racing-game-api".to_string(),
        audience: "racing-game-client".to_string(),
    };
    let jwt_service = Arc::new(JwtService::new(jwt_config));

    // Initialize session manager
    let session_config = SessionConfig::default();
    let session_manager = Arc::new(SessionManager::new(
        Arc::new(db_pool.clone()),
        session_config,
    ));

    // Create application state for auth routes
    let app_state = AppState::new(db_pool.clone(), jwt_service, session_manager);

    // Create auth routes with AppState
    let auth_routes = auth::routes().with_state(app_state.clone());

    // Create admin-protected routes with AppState and middleware
    let admin_routes = players::admin_routes()
        .layer(RequireRole::admin())
        .layer(AuthMiddleware::new(
            app_state.jwt_service.clone(),
            app_state.session_manager.clone(),
        ))
        .with_state(app_state.clone());

    // Create main app with Database state for other routes
    let app = Router::new()
        .route("/health_check", get(health_check))
        .nest("/api/v1", players::routes())
        .nest("/api/v1", races::routes())
        .merge(auth_routes) // Merge the auth routes that already have their state
        .nest("/api/v1/admin", admin_routes) // Nest the admin routes with middleware
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:5173".parse().unwrap(),
                    "http://localhost:5174".parse().unwrap(),
                    "http://localhost:5175".parse().unwrap(),
                ])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                ])
                .allow_credentials(true),
        )
        .with_state(db_pool);

    // TODO: Add admin-only routes with proper authentication middleware
    // Examples:
    // - GET /api/v1/admin/system/stats (system statistics)
    // - POST /api/v1/admin/races/:uuid/force-start (force start any race)
    // - DELETE /api/v1/admin/players/:uuid (delete any player)
    // - GET /api/v1/admin/dashboard (administrative dashboard with sensitive data)

    let server = axum::serve(listener, app);
    Ok(server)
}

pub async fn get_connection_pool(
    configuration: &DatabaseSettings,
) -> Result<Database, mongodb::error::Error> {
    // Try with authentication first, fallback to no auth for local development
    let connection_string = if configuration.username.is_empty() {
        configuration.connection_string_without_auth()
    } else {
        configuration.with_db()
    };

    let client = Client::with_uri_str(&connection_string).await?;
    let database = client.database(&configuration.database_name);

    // Test the connection
    client
        .database("admin")
        .run_command(mongodb::bson::doc! {"ping": 1}, None)
        .await?;

    tracing::info!("Successfully connected to MongoDB");
    Ok(database)
}
