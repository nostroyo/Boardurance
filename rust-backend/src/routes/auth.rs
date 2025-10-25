use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{post},
    Router,
};
use mongodb::{bson::doc, Database};
use serde_json::{json, Value};
use utoipa::OpenApi;

use crate::domain::{
    Email, TeamName, Player, Password, UserRegistration, UserCredentials
};

#[derive(OpenApi)]
#[openapi(
    paths(register_user, login_user),
    components(schemas(UserRegistration, UserCredentials))
)]
pub struct AuthApiDoc;

pub fn auth_routes() -> Router<Database> {
    Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
}

/// Register a new user
#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = UserRegistration,
    responses(
        (status = 201, description = "User registered successfully", body = Value),
        (status = 400, description = "Invalid input data", body = Value),
        (status = 409, description = "User already exists", body = Value),
        (status = 500, description = "Internal server error", body = Value)
    ),
    tag = "Authentication"
)]
pub async fn register_user(
    State(db): State<Database>,
    Json(registration): Json<UserRegistration>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let collection = db.collection::<Player>("players");
    // Validate email format
    let email = Email::parse(&registration.email)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": e}))))?;

    // Validate team name
    let team_name = TeamName::parse(&registration.team_name)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": e}))))?;

    // Validate and hash password
    let password = Password::new(registration.password)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": e}))))?;
    
    let password_hash = password.hash()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))))?;

    // Check if user already exists
    let existing_user = collection
        .find_one(doc! {"email": email.as_ref()}, None)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking existing user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database error"})))
        })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({"error": "User with this email already exists"}))
        ));
    }

    // Create new player with empty assets
    let player = Player::new(email, password_hash, team_name, vec![], vec![])
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": e}))))?;

    // Insert into database
    let result = collection.insert_one(&player, None).await
        .map_err(|e| {
            tracing::error!("Database error inserting user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Failed to create user"})))
        })?;

    tracing::info!("User registered successfully: {}", player.uuid);

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "User registered successfully",
            "user_id": result.inserted_id,
            "uuid": player.uuid
        }))
    ))
}

/// Login user
#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = UserCredentials,
    responses(
        (status = 200, description = "Login successful", body = Value),
        (status = 400, description = "Invalid input data", body = Value),
        (status = 401, description = "Invalid credentials", body = Value),
        (status = 500, description = "Internal server error", body = Value)
    ),
    tag = "Authentication"
)]
pub async fn login_user(
    State(db): State<Database>,
    Json(credentials): Json<UserCredentials>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let collection = db.collection::<Player>("players");
    // Validate email format
    let email = Email::parse(&credentials.email)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": e}))))?;

    // Validate password format
    let password = Password::new(credentials.password)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({"error": e}))))?;

    // Find user by email
    let user = collection
        .find_one(doc! {"email": email.as_ref()}, None)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Database error"})))
        })?;

    let user = user.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, Json(json!({"error": "Invalid credentials"})))
    })?;

    // Verify password
    let is_valid = user.verify_password(&password)
        .map_err(|e| {
            tracing::error!("Password verification error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Authentication error"})))
        })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid credentials"}))
        ));
    }

    tracing::info!("User logged in successfully: {}", user.uuid);

    // TODO: Generate and return JWT token or session
    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Login successful",
            "user_id": user.id,
            "uuid": user.uuid,
            "email": user.email.as_ref(),
            "team_name": user.team_name.as_ref()
        }))
    ))
}