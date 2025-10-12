#![allow(clippy::needless_for_each)]

use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{health_check, test_items, players, races};
use axum::{routing::get, Router};
use mongodb::{Client, Database};

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
            tracing::warn!("Failed to connect to MongoDB: {}. Server will run in degraded mode.", e);
            // Create a mock database for testing
            let client = mongodb::Client::with_uri_str("mongodb://localhost:27017").await.unwrap();
            client.database("mock_database")
        }
    };

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TokioTcpListener::bind(&address).await?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, connection_pool, configuration.application.base_url).await?;

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
        crate::routes::test_items::create_test_item,
        crate::routes::test_items::get_test_items,
        crate::routes::players::create_player,
        crate::routes::players::get_all_players,
        crate::routes::players::get_player_by_uuid,
        crate::routes::players::get_player_by_wallet,
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
    ),
    components(
        schemas(
            crate::domain::TestItem,
            crate::domain::Player,
            crate::domain::Car,
            crate::domain::Pilot,
            crate::domain::CarType,
            crate::domain::CarRarity,
            crate::domain::CarStats,
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
            crate::routes::test_items::CreateTestItemRequest,
            crate::routes::players::CreatePlayerRequest,
            crate::routes::players::ConnectWalletRequest,
            crate::routes::players::UpdateTeamNameRequest,
            crate::routes::players::AddCarRequest,
            crate::routes::players::CarStatsRequest,
            crate::routes::players::AddPilotRequest,
            crate::routes::players::PilotSkillsRequest,
            crate::routes::players::PlayerResponse,
            crate::routes::races::CreateRaceRequest,
            crate::routes::races::CreateSectorRequest,
            crate::routes::races::JoinRaceRequest,
            crate::routes::races::ProcessLapRequest,
            crate::routes::races::LapActionRequest,
            crate::routes::races::RaceResponse,
            crate::routes::races::LapResultResponse,
            crate::routes::HealthResponse
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "test", description = "Test endpoints"),
        (name = "players", description = "Player management endpoints"),
        (name = "races", description = "Race management and gameplay endpoints")
    )
)]
struct ApiDoc;

#[allow(clippy::unused_async)]
pub async fn run(
    listener: TokioTcpListener,
    db_pool: Database,
    _base_url: String,
) -> Result<axum::serve::Serve<Router, Router>, anyhow::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .nest("/api/v1", test_items::routes())
        .nest("/api/v1", players::routes())
        .nest("/api/v1", races::routes())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(db_pool);

    let server = axum::serve(listener, app);
    Ok(server)
}

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> Result<Database, mongodb::error::Error> {
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