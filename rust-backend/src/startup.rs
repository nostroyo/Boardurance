use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{health_check, test_items};
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
    ),
    components(
        schemas(
            crate::domain::TestItem,
            crate::routes::test_items::CreateTestItemRequest,
            crate::routes::HealthResponse
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "test", description = "Test endpoints")
    )
)]
struct ApiDoc;

pub async fn run(
    listener: TokioTcpListener,
    db_pool: Database,
    _base_url: String,
) -> Result<axum::serve::Serve<Router, Router>, anyhow::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .nest("/api/v1", test_items::routes())
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