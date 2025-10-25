use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::post,
    Router,
};
use mongodb::{bson::doc, Database};
use serde_json::{json, Value};

use crate::domain::{
    Email, TeamName, Player, Password, UserRegistration, UserCredentials,
    Car, CarName, Engine, EngineName, Body, BodyName, ComponentRarity
};

pub fn routes() -> Router<Database> {
    Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
}

// Helper function to create starter assets for new players
fn create_starter_assets() -> Result<(Vec<Car>, Vec<Engine>, Vec<Body>), String> {
    // Create 2 starter cars
    let car1 = Car::new(CarName::parse("Starter Car 1").unwrap(), None)
        .map_err(|e| format!("Failed to create starter car 1: {}", e))?;
    
    let car2 = Car::new(CarName::parse("Starter Car 2").unwrap(), None)
        .map_err(|e| format!("Failed to create starter car 2: {}", e))?;

    // Create 2 starter engines with different characteristics
    let engine1 = Engine::new(
        EngineName::parse("Rookie Engine").unwrap(),
        ComponentRarity::Common,
        35, // straight_value - good for straights
        25, // curve_value
        None,
    ).map_err(|e| format!("Failed to create starter engine 1: {}", e))?;

    let engine2 = Engine::new(
        EngineName::parse("Balanced Engine").unwrap(),
        ComponentRarity::Common,
        30, // straight_value
        30, // curve_value - balanced
        None,
    ).map_err(|e| format!("Failed to create starter engine 2: {}", e))?;

    // Create 2 starter bodies with different characteristics
    let body1 = Body::new(
        BodyName::parse("Lightweight Frame").unwrap(),
        ComponentRarity::Common,
        25, // straight_value
        35, // curve_value - good for curves
        None,
    ).map_err(|e| format!("Failed to create starter body 1: {}", e))?;

    let body2 = Body::new(
        BodyName::parse("Sturdy Frame").unwrap(),
        ComponentRarity::Common,
        35, // straight_value - good for straights
        25, // curve_value
        None,
    ).map_err(|e| format!("Failed to create starter body 2: {}", e))?;

    Ok((
        vec![car1, car2],
        vec![engine1, engine2],
        vec![body1, body2],
    ))
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
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<Value>)> {
    let collection = db.collection::<Player>("players");
    
    // Validate email format
    let email = Email::parse(&registration.email)
        .map_err(|e| (StatusCode::BAD_REQUEST, ResponseJson(json!({"error": e}))))?;

    // Validate team name
    let team_name = TeamName::parse(&registration.team_name)
        .map_err(|e| (StatusCode::BAD_REQUEST, ResponseJson(json!({"error": e}))))?;

    // Validate and hash password
    let password = Password::new(registration.password)
        .map_err(|e| (StatusCode::BAD_REQUEST, ResponseJson(json!({"error": e}))))?;
    
    let password_hash = password.hash()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": e}))))?;

    // Check if user already exists
    let existing_user = collection
        .find_one(doc! {"email": email.as_ref()}, None)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking existing user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Database error"})))
        })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::CONFLICT,
            ResponseJson(json!({"error": "User with this email already exists"}))
        ));
    }

    // Create starter assets for new player
    let (starter_cars, starter_engines, starter_bodies) = create_starter_assets()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": e}))))?;

    // Create new player with starter assets
    let player = Player::new_with_assets(
        email, 
        password_hash, 
        team_name, 
        starter_cars,
        vec![], // No pilots initially - players can recruit them later
        starter_engines,
        starter_bodies,
    ).map_err(|e| (StatusCode::BAD_REQUEST, ResponseJson(json!({"error": e}))))?;

    // Insert into database
    let _result = collection.insert_one(&player, None).await
        .map_err(|e| {
            tracing::error!("Database error inserting user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Failed to create user"})))
        })?;

    tracing::info!("User registered successfully: {}", player.uuid);

    Ok(ResponseJson(json!({
        "message": "User registered successfully",
        "user": {
            "uuid": player.uuid,
            "email": player.email.as_ref(),
            "team_name": player.team_name.as_ref(),
            "role": player.role
        }
    })))
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
) -> Result<ResponseJson<Value>, (StatusCode, ResponseJson<Value>)> {
    let collection = db.collection::<Player>("players");
    
    // Validate email format
    let email = Email::parse(&credentials.email)
        .map_err(|e| (StatusCode::BAD_REQUEST, ResponseJson(json!({"error": e}))))?;

    // Validate password format
    let password = Password::new(credentials.password)
        .map_err(|e| (StatusCode::BAD_REQUEST, ResponseJson(json!({"error": e}))))?;

    // Find user by email
    let user = collection
        .find_one(doc! {"email": email.as_ref()}, None)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Database error"})))
        })?;

    let user = user.ok_or_else(|| {
        (StatusCode::UNAUTHORIZED, ResponseJson(json!({"error": "Invalid credentials"})))
    })?;

    // Verify password
    let is_valid = user.verify_password(&password)
        .map_err(|e| {
            tracing::error!("Password verification error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Authentication error"})))
        })?;

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            ResponseJson(json!({"error": "Invalid credentials"}))
        ));
    }

    tracing::info!("User logged in successfully: {}", user.uuid);

    Ok(ResponseJson(json!({
        "message": "Login successful",
        "user": {
            "uuid": user.uuid,
            "email": user.email.as_ref(),
            "team_name": user.team_name.as_ref(),
            "role": user.role
        }
    })))
}