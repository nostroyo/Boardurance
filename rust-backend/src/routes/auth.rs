use axum::{
    extract::{Json, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::Json as ResponseJson,
    routing::post,
    Router,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use mongodb::bson::doc;
use serde_json::{json, Value};
use time::Duration as TimeDuration;

use crate::app_state::AppState;
use crate::domain::{
    Email, TeamName, Player, Password, UserRegistration, UserCredentials,
    Car, CarName, Engine, EngineName, Body, BodyName, ComponentRarity
};
use crate::services::session::SessionMetadata;

pub fn routes() -> Router<AppState> {
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
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(registration): Json<UserRegistration>,
) -> Result<(StatusCode, [(String, String); 2], ResponseJson<Value>), (StatusCode, ResponseJson<Value>)> {
    let collection = app_state.database.collection::<Player>("players");
    
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

    // Generate JWT tokens
    let token_pair = app_state.jwt_service.generate_token_pair(&player)
        .map_err(|e| {
            tracing::error!("Failed to generate tokens: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Failed to generate authentication tokens"})))
        })?;

    // Extract token ID from access token for session management
    let access_claims = app_state.jwt_service.validate_token(&token_pair.access_token)
        .map_err(|e| {
            tracing::error!("Failed to validate generated token: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Token generation error"})))
        })?;

    // Create session metadata
    let session_metadata = SessionMetadata {
        ip_address: headers
            .get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        user_agent: headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
    };

    // Create session
    app_state.session_manager.create_session(player.uuid, access_claims.jti.clone(), session_metadata).await
        .map_err(|e| {
            tracing::error!("Failed to create session: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Session creation failed"})))
        })?;

    // Create secure cookies
    let access_cookie = Cookie::build(("access_token", token_pair.access_token))
        .http_only(true)
        .secure(false) // TODO: Make this configurable for development
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::seconds(token_pair.expires_in as i64))
        .path("/")
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", token_pair.refresh_token))
        .http_only(true)
        .secure(false) // TODO: Make this configurable for development
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::seconds(30 * 24 * 60 * 60)) // 30 days
        .path("/auth/refresh")
        .build();

    tracing::info!("User registered successfully: {}", player.uuid);

    Ok((
        StatusCode::CREATED,
        [
            (SET_COOKIE.to_string(), access_cookie.to_string()),
            (SET_COOKIE.to_string(), refresh_cookie.to_string()),
        ],
        ResponseJson(json!({
            "message": "User registered successfully",
            "user": {
                "uuid": player.uuid,
                "email": player.email.as_ref(),
                "team_name": player.team_name.as_ref(),
                "role": player.role
            }
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
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(credentials): Json<UserCredentials>,
) -> Result<(StatusCode, [(String, String); 2], ResponseJson<Value>), (StatusCode, ResponseJson<Value>)> {
    let collection = app_state.database.collection::<Player>("players");
    
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

    // Generate JWT tokens
    let token_pair = app_state.jwt_service.generate_token_pair(&user)
        .map_err(|e| {
            tracing::error!("Failed to generate tokens: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Failed to generate authentication tokens"})))
        })?;

    // Extract token ID from access token for session management
    let access_claims = app_state.jwt_service.validate_token(&token_pair.access_token)
        .map_err(|e| {
            tracing::error!("Failed to validate generated token: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Token generation error"})))
        })?;

    // Create session metadata
    let session_metadata = SessionMetadata {
        ip_address: headers
            .get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
        user_agent: headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string()),
    };

    // Create session
    app_state.session_manager.create_session(user.uuid, access_claims.jti.clone(), session_metadata).await
        .map_err(|e| {
            tracing::error!("Failed to create session: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Session creation failed"})))
        })?;

    // Create secure cookies
    let access_cookie = Cookie::build(("access_token", token_pair.access_token))
        .http_only(true)
        .secure(false) // TODO: Make this configurable for development
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::seconds(token_pair.expires_in as i64))
        .path("/")
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", token_pair.refresh_token))
        .http_only(true)
        .secure(false) // TODO: Make this configurable for development
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::seconds(30 * 24 * 60 * 60)) // 30 days
        .path("/auth/refresh")
        .build();

    tracing::info!("User logged in successfully: {}", user.uuid);

    Ok((
        StatusCode::OK,
        [
            (SET_COOKIE.to_string(), access_cookie.to_string()),
            (SET_COOKIE.to_string(), refresh_cookie.to_string()),
        ],
        ResponseJson(json!({
            "message": "Login successful",
            "user": {
                "uuid": user.uuid,
                "email": user.email.as_ref(),
                "team_name": user.team_name.as_ref(),
                "role": user.role
            }
        }))
    ))
}