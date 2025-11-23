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
use uuid;

use crate::app_state::AppState;
use crate::domain::{
    Email, TeamName, Player, Password, UserRegistration, UserCredentials,
    Car, CarName, Engine, EngineName, Body, BodyName, ComponentRarity,
    Pilot, PilotName, PilotClass, PilotRarity, PilotSkills
};
use crate::services::session::SessionMetadata;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register_user))
        .route("/auth/login", post(login_user))
        .route("/auth/logout", post(logout_user))
        .route("/auth/refresh", post(refresh_token))
}

// Helper function to create starter assets for new players
#[allow(clippy::type_complexity)]
fn create_starter_assets() -> Result<(Vec<Car>, Vec<Pilot>, Vec<Engine>, Vec<Body>), String> {
    // Create 6 pilots with different classes and rarities
    let pilot1 = Pilot::new(
        PilotName::parse("Speedster Ace").unwrap(),
        PilotClass::Speedster,
        PilotRarity::Rookie,
        PilotSkills::new(7, 5, 6, 4).unwrap(),
        crate::domain::PilotPerformance::new(8, 5).unwrap(),
        None,
    ).map_err(|e| format!("Failed to create pilot 1: {e}"))?;

    let pilot2 = Pilot::new(
        PilotName::parse("Tech Master").unwrap(),
        PilotClass::Technician,
        PilotRarity::Rookie,
        PilotSkills::new(5, 8, 7, 5).unwrap(),
        crate::domain::PilotPerformance::new(5, 8).unwrap(),
        None,
    ).map_err(|e| format!("Failed to create pilot 2: {e}"))?;

    let pilot3 = Pilot::new(
        PilotName::parse("Endurance Pro").unwrap(),
        PilotClass::Endurance,
        PilotRarity::Rookie,
        PilotSkills::new(4, 6, 8, 9).unwrap(),
        crate::domain::PilotPerformance::new(6, 7).unwrap(),
        None,
    ).map_err(|e| format!("Failed to create pilot 3: {e}"))?;

    let pilot4 = Pilot::new(
        PilotName::parse("All-Round Rookie").unwrap(),
        PilotClass::AllRounder,
        PilotRarity::Rookie,
        PilotSkills::new(6, 6, 6, 6).unwrap(),
        crate::domain::PilotPerformance::new(6, 6).unwrap(),
        None,
    ).map_err(|e| format!("Failed to create pilot 4: {e}"))?;

    let pilot5 = Pilot::new(
        PilotName::parse("Speed Demon").unwrap(),
        PilotClass::Speedster,
        PilotRarity::Professional,
        PilotSkills::new(8, 4, 5, 3).unwrap(),
        crate::domain::PilotPerformance::new(9, 4).unwrap(),
        None,
    ).map_err(|e| format!("Failed to create pilot 5: {e}"))?;

    let pilot6 = Pilot::new(
        PilotName::parse("Precision Driver").unwrap(),
        PilotClass::Technician,
        PilotRarity::Professional,
        PilotSkills::new(4, 9, 8, 6).unwrap(),
        crate::domain::PilotPerformance::new(4, 9).unwrap(),
        None,
    ).map_err(|e| format!("Failed to create pilot 6: {e}"))?;

    // Create 2 starter cars
    let mut car1 = Car::new(CarName::parse("Car 1").unwrap(), None)
        .map_err(|e| format!("Failed to create starter car 1: {e}"))?;
    
    let mut car2 = Car::new(CarName::parse("Car 2").unwrap(), None)
        .map_err(|e| format!("Failed to create starter car 2: {e}"))?;

    // Assign pilots to cars (3 pilots per car)
    car1.assign_pilots(vec![pilot1.uuid, pilot2.uuid, pilot3.uuid])
        .map_err(|e| format!("Failed to assign pilots to car 1: {e}"))?;
    
    car2.assign_pilots(vec![pilot4.uuid, pilot5.uuid, pilot6.uuid])
        .map_err(|e| format!("Failed to assign pilots to car 2: {e}"))?;

    // Create 2 starter engines with different characteristics
    let engine1 = Engine::new(
        EngineName::parse("Basic Engine 1").unwrap(),
        ComponentRarity::Common,
        7, // straight_value - good for straights (0-10 range)
        5, // curve_value
        None,
    ).map_err(|e| format!("Failed to create starter engine 1: {e}"))?;

    let engine2 = Engine::new(
        EngineName::parse("Basic Engine 2").unwrap(),
        ComponentRarity::Common,
        5, // straight_value
        7, // curve_value - good for curves
        None,
    ).map_err(|e| format!("Failed to create starter engine 2: {e}"))?;

    // Create 2 starter bodies with different characteristics
    let body1 = Body::new(
        BodyName::parse("Basic Body 1").unwrap(),
        ComponentRarity::Common,
        5, // straight_value (0-10 range)
        7, // curve_value - good for curves
        None,
    ).map_err(|e| format!("Failed to create starter body 1: {e}"))?;

    let body2 = Body::new(
        BodyName::parse("Basic Body 2").unwrap(),
        ComponentRarity::Common,
        7, // straight_value - good for straights
        5, // curve_value
        None,
    ).map_err(|e| format!("Failed to create starter body 2: {e}"))?;

    Ok((
        vec![car1, car2],
        vec![pilot1, pilot2, pilot3, pilot4, pilot5, pilot6],
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
#[allow(clippy::cast_possible_wrap)]
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

    // Create starter assets for new player (2 cars with 3 pilots each, engines, and bodies)
    let (starter_cars, starter_pilots, starter_engines, starter_bodies) = create_starter_assets()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": e}))))?;

    // Create new player with starter assets including 6 pilots assigned to 2 cars
    let player = Player::new_with_assets(
        email, 
        password_hash, 
        team_name, 
        starter_cars,
        starter_pilots,
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
            .map(std::string::ToString::to_string),
        user_agent: headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(std::string::ToString::to_string),
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
#[allow(clippy::cast_possible_wrap)]
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
            .map(std::string::ToString::to_string),
        user_agent: headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(std::string::ToString::to_string),
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

/// Logout user
#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logout successful", body = Value),
        (status = 401, description = "Not authenticated", body = Value),
        (status = 500, description = "Internal server error", body = Value)
    ),
    tag = "Authentication"
)]
pub async fn logout_user(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, [(String, String); 2], ResponseJson<Value>), (StatusCode, ResponseJson<Value>)> {
    // Extract token from cookie or header
    let token = extract_token_from_headers(&headers);
    
    if let Some(token) = token {
        // Try to invalidate the session
        if let Err(e) = app_state.session_manager.invalidate_session(&token, "user_logout").await {
            tracing::warn!("Failed to invalidate session during logout: {}", e);
            // Continue with logout even if session invalidation fails
        }
    }

    // Clear cookies regardless of session invalidation result
    let clear_access = Cookie::build(("access_token", ""))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::seconds(0))
        .path("/")
        .build();

    let clear_refresh = Cookie::build(("refresh_token", ""))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::seconds(0))
        .path("/auth/refresh")
        .build();

    Ok((
        StatusCode::OK,
        [
            (SET_COOKIE.to_string(), clear_access.to_string()),
            (SET_COOKIE.to_string(), clear_refresh.to_string()),
        ],
        ResponseJson(json!({
            "message": "Logout successful"
        }))
    ))
}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed successfully", body = Value),
        (status = 401, description = "Invalid refresh token", body = Value),
        (status = 500, description = "Internal server error", body = Value)
    ),
    tag = "Authentication"
)]
pub async fn refresh_token(
    State(app_state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, [(String, String); 1], ResponseJson<Value>), (StatusCode, ResponseJson<Value>)> {
    // Extract refresh token from cookie
    let refresh_token = extract_refresh_token_from_headers(&headers)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, ResponseJson(json!({"error": "Refresh token not found"}))))?;

    // Validate refresh token
    let claims = app_state.jwt_service.validate_token(&refresh_token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, ResponseJson(json!({"error": "Invalid refresh token"}))))?;

    // Check if token is blacklisted
    let is_blacklisted = app_state.session_manager.is_token_blacklisted(&claims.jti).await
        .map_err(|e| {
            tracing::error!("Failed to check token blacklist: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Authentication error"})))
        })?;

    if is_blacklisted {
        return Err((StatusCode::UNAUTHORIZED, ResponseJson(json!({"error": "Token has been revoked"}))));
    }

    // Get user from database to generate new token
    let collection = app_state.database.collection::<Player>("players");
    let user_uuid = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, ResponseJson(json!({"error": "Invalid token"}))))?;

    let user = collection
        .find_one(doc! {"uuid": user_uuid.to_string()}, None)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Database error"})))
        })?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, ResponseJson(json!({"error": "User not found"}))))?;

    // Generate new access token
    let new_access_token = app_state.jwt_service.generate_access_token(&user)
        .map_err(|e| {
            tracing::error!("Failed to generate new access token: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Token generation failed"})))
        })?;

    // Invalidate old refresh token and create new session
    if let Err(e) = app_state.session_manager.invalidate_session(&claims.jti, "token_refresh").await {
        tracing::warn!("Failed to invalidate old session during refresh: {}", e);
    }

    // Extract new token ID and create new session
    let new_claims = app_state.jwt_service.validate_token(&new_access_token)
        .map_err(|e| {
            tracing::error!("Failed to validate new token: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(json!({"error": "Token validation error"})))
        })?;

    let session_metadata = SessionMetadata {
        ip_address: headers
            .get("x-forwarded-for")
            .or_else(|| headers.get("x-real-ip"))
            .and_then(|h| h.to_str().ok())
            .map(std::string::ToString::to_string),
        user_agent: headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(std::string::ToString::to_string),
    };

    if let Err(e) = app_state.session_manager.create_session(user.uuid, new_claims.jti, session_metadata).await {
        tracing::error!("Failed to create new session: {}", e);
        // Continue anyway, as the token is still valid
    }

    // Create new access token cookie
    let access_cookie = Cookie::build(("access_token", new_access_token))
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .max_age(TimeDuration::seconds(30 * 60)) // 30 minutes
        .path("/")
        .build();

    Ok((
        StatusCode::OK,
        [(SET_COOKIE.to_string(), access_cookie.to_string())],
        ResponseJson(json!({
            "message": "Token refreshed successfully"
        }))
    ))
}

// Helper functions
fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    // Try Authorization header first
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(stripped) = auth_str.strip_prefix("Bearer ") {
                return Some(stripped.to_string());
            }
        }
    }

    // Try cookie
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(stripped) = cookie.strip_prefix("access_token=") {
                    return Some(stripped.to_string());
                }
            }
        }
    }

    None
}

fn extract_refresh_token_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(stripped) = cookie.strip_prefix("refresh_token=") {
                    return Some(stripped.to_string());
                }
            }
        }
    }
    None
}