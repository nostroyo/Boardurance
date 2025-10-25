use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, header::COOKIE, StatusCode},
    middleware::Next,
    response::Response,
};

use futures_util::future::BoxFuture;
use serde_json::json;
use std::sync::Arc;
use tower::{Layer, Service};
use uuid::Uuid;

use crate::domain::UserRole;
use crate::services::{JwtService, SessionManager};

/// User context extracted from valid JWT token
#[derive(Debug, Clone)]
pub struct UserContext {
    pub user_uuid: Uuid,
    pub email: String,
    pub role: UserRole,
    pub token_id: String,
}

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authentication token")]
    MissingToken,
    #[error("Invalid token format")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token is blacklisted")]
    BlacklistedToken,
    #[error("Internal authentication error: {0}")]
    InternalError(String),
}

impl From<AuthError> for StatusCode {
    fn from(error: AuthError) -> Self {
        match error {
            AuthError::MissingToken => StatusCode::UNAUTHORIZED,
            AuthError::InvalidToken => StatusCode::UNAUTHORIZED,
            AuthError::TokenExpired => StatusCode::UNAUTHORIZED,
            AuthError::BlacklistedToken => StatusCode::UNAUTHORIZED,
            AuthError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Authentication middleware layer
#[derive(Clone)]
pub struct AuthMiddleware {
    jwt_service: Arc<JwtService>,
    session_manager: Arc<SessionManager>,
}

impl AuthMiddleware {
    /// Create a new authentication middleware
    pub fn new(jwt_service: Arc<JwtService>, session_manager: Arc<SessionManager>) -> Self {
        Self {
            jwt_service,
            session_manager,
        }
    }

    /// Extract token from request (Authorization header or cookie)
    fn extract_token_from_request(request: &Request) -> Option<String> {
        // Try Authorization header first (for API clients)
        if let Some(auth_header) = request.headers().get(AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    return Some(auth_str[7..].to_string());
                }
            }
        }

        // Fall back to cookie (for web clients)
        if let Some(cookie_header) = request.headers().get(COOKIE) {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if cookie.starts_with("access_token=") {
                        return Some(cookie[13..].to_string());
                    }
                }
            }
        }

        None
    }

    /// Validate token and create user context
    async fn validate_and_create_context(&self, token: &str) -> Result<UserContext, AuthError> {
        // Validate JWT token
        let claims = self.jwt_service
            .validate_token(token)
            .map_err(|e| match e {
                crate::services::jwt::JwtError::TokenExpired => AuthError::TokenExpired,
                crate::services::jwt::JwtError::InvalidToken => AuthError::InvalidToken,
                _ => AuthError::InternalError(e.to_string()),
            })?;

        // Check if token is blacklisted
        let is_blacklisted = self.session_manager
            .is_token_blacklisted(&claims.jti)
            .await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;

        if is_blacklisted {
            return Err(AuthError::BlacklistedToken);
        }

        // Validate session
        let is_session_valid = self.session_manager
            .validate_session(&claims.jti)
            .await
            .map_err(|e| match e {
                crate::services::session::SessionError::TokenBlacklisted => AuthError::BlacklistedToken,
                crate::services::session::SessionError::SessionExpired => AuthError::TokenExpired,
                crate::services::session::SessionError::SessionNotFound => AuthError::InvalidToken,
                _ => AuthError::InternalError(e.to_string()),
            })?;

        if !is_session_valid {
            return Err(AuthError::InvalidToken);
        }

        // Parse user UUID
        let user_uuid = Uuid::parse_str(&claims.sub)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(UserContext {
            user_uuid,
            email: claims.email,
            role: claims.role,
            token_id: claims.jti,
        })
    }
}

impl<S> Layer<S> for AuthMiddleware {
    type Service = AuthService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            jwt_service: self.jwt_service.clone(),
            session_manager: self.session_manager.clone(),
        }
    }
}

/// Authentication service that wraps the inner service
#[derive(Clone)]
pub struct AuthService<S> {
    inner: S,
    jwt_service: Arc<JwtService>,
    session_manager: Arc<SessionManager>,
}

impl<S> Service<Request> for AuthService<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        let jwt_service = self.jwt_service.clone();
        let session_manager = self.session_manager.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract token from request
            let token = match AuthMiddleware::extract_token_from_request(&request) {
                Some(token) => token,
                None => {
                    let error_response = Response::builder()
                        .status(StatusCode::UNAUTHORIZED)
                        .header("content-type", "application/json")
                        .body(
                            json!({
                                "error": "authentication_required",
                                "message": "Authentication token is required"
                            })
                            .to_string()
                            .into(),
                        )
                        .unwrap();
                    return Ok(error_response);
                }
            };

            // Create auth middleware instance for validation
            let auth_middleware = AuthMiddleware {
                jwt_service,
                session_manager,
            };

            // Validate token and create user context
            let user_context = match auth_middleware.validate_and_create_context(&token).await {
                Ok(context) => context,
                Err(error) => {
                    let (status, error_code, message) = match error {
                        AuthError::MissingToken => (
                            StatusCode::UNAUTHORIZED,
                            "authentication_required",
                            "Authentication token is required",
                        ),
                        AuthError::InvalidToken => (
                            StatusCode::UNAUTHORIZED,
                            "invalid_token",
                            "The provided token is invalid",
                        ),
                        AuthError::TokenExpired => (
                            StatusCode::UNAUTHORIZED,
                            "token_expired",
                            "The provided token has expired",
                        ),
                        AuthError::BlacklistedToken => (
                            StatusCode::UNAUTHORIZED,
                            "token_revoked",
                            "The provided token has been revoked",
                        ),
                        AuthError::InternalError(_) => (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "internal_error",
                            "An internal authentication error occurred",
                        ),
                    };

                    let error_response = Response::builder()
                        .status(status)
                        .header("content-type", "application/json")
                        .body(
                            json!({
                                "error": error_code,
                                "message": message
                            })
                            .to_string()
                            .into(),
                        )
                        .unwrap();
                    return Ok(error_response);
                }
            };

            // Add user context to request extensions
            request.extensions_mut().insert(user_context);

            // Continue to the next middleware/handler
            inner.call(request).await
        })
    }
}

/// Convenience function to create auth middleware as a function
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // This is a placeholder for function-style middleware
    // The actual implementation should use the Layer-based approach above
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Email, HashedPassword, Player, TeamName};
    use crate::services::{JwtConfig, SessionConfig};
    use axum::body::Body;
    use axum::http::{HeaderValue, Method};
    use mongodb::Database;
    use std::sync::Arc;

    fn create_test_player() -> Player {
        Player::new_with_assets(
            Email::parse("test@example.com").unwrap(),
            HashedPassword::from_hash("test_hash".to_string()),
            TeamName::parse("Test Team").unwrap(),
            vec![], // cars
            vec![], // pilots
            vec![], // engines
            vec![], // bodies
        ).unwrap()
    }

    async fn create_mock_database() -> Database {
        let client = mongodb::Client::with_uri_str("mongodb://mock:27017").await.unwrap();
        client.database("mock_test")
    }

    #[test]
    fn user_context_creation_works() {
        let user_uuid = Uuid::new_v4();
        let context = UserContext {
            user_uuid,
            email: "test@example.com".to_string(),
            role: UserRole::Player,
            token_id: "test_token_id".to_string(),
        };

        assert_eq!(context.user_uuid, user_uuid);
        assert_eq!(context.email, "test@example.com");
        assert_eq!(context.role, UserRole::Player);
        assert_eq!(context.token_id, "test_token_id");
    }

    #[test]
    fn auth_error_to_status_code_conversion_works() {
        assert_eq!(StatusCode::from(AuthError::MissingToken), StatusCode::UNAUTHORIZED);
        assert_eq!(StatusCode::from(AuthError::InvalidToken), StatusCode::UNAUTHORIZED);
        assert_eq!(StatusCode::from(AuthError::TokenExpired), StatusCode::UNAUTHORIZED);
        assert_eq!(StatusCode::from(AuthError::BlacklistedToken), StatusCode::UNAUTHORIZED);
        assert_eq!(
            StatusCode::from(AuthError::InternalError("test".to_string())),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[tokio::test]
    async fn auth_middleware_creation_works() {
        let jwt_config = JwtConfig::default();
        let jwt_service = Arc::new(JwtService::new(jwt_config));
        
        let session_config = SessionConfig::default();
        let db = Arc::new(create_mock_database().await);
        let session_manager = Arc::new(SessionManager::new(db, session_config));

        let auth_middleware = AuthMiddleware::new(jwt_service, session_manager);
        
        // Should not panic and should be created successfully
        assert!(Arc::strong_count(&auth_middleware.jwt_service) >= 1);
        assert!(Arc::strong_count(&auth_middleware.session_manager) >= 1);
    }

    #[test]
    fn extract_token_from_authorization_header_works() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Add Authorization header
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer test_token_123"),
        );

        let token = AuthMiddleware::extract_token_from_request(&request);
        assert_eq!(token, Some("test_token_123".to_string()));
    }

    #[test]
    fn extract_token_from_cookie_works() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Add Cookie header
        request.headers_mut().insert(
            COOKIE,
            HeaderValue::from_static("access_token=cookie_token_456; other=value"),
        );

        let token = AuthMiddleware::extract_token_from_request(&request);
        assert_eq!(token, Some("cookie_token_456".to_string()));
    }

    #[test]
    fn extract_token_prefers_authorization_header() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Add both Authorization header and Cookie
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer header_token"),
        );
        request.headers_mut().insert(
            COOKIE,
            HeaderValue::from_static("access_token=cookie_token"),
        );

        let token = AuthMiddleware::extract_token_from_request(&request);
        assert_eq!(token, Some("header_token".to_string()));
    }

    #[test]
    fn extract_token_returns_none_when_no_token() {
        let request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let token = AuthMiddleware::extract_token_from_request(&request);
        assert_eq!(token, None);
    }

    #[test]
    fn extract_token_handles_malformed_authorization_header() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Add malformed Authorization header
        request.headers_mut().insert(
            AUTHORIZATION,
            HeaderValue::from_static("NotBearer token"),
        );

        let token = AuthMiddleware::extract_token_from_request(&request);
        assert_eq!(token, None);
    }

    #[test]
    fn extract_token_handles_malformed_cookie() {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        // Add cookie without access_token
        request.headers_mut().insert(
            COOKIE,
            HeaderValue::from_static("other_cookie=value; session=123"),
        );

        let token = AuthMiddleware::extract_token_from_request(&request);
        assert_eq!(token, None);
    }
}