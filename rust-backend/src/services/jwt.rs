use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;
use uuid::Uuid;

use crate::domain::{Player, UserRole};

/// JWT service configuration
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry: StdDuration,
    pub refresh_token_expiry: StdDuration,
    pub issuer: String,
    pub audience: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "your-super-secret-key-here".to_string(),
            access_token_expiry: StdDuration::from_secs(30 * 60), // 30 minutes
            refresh_token_expiry: StdDuration::from_secs(30 * 24 * 60 * 60), // 30 days
            issuer: "racing-game-api".to_string(),
            audience: "racing-game-client".to_string(),
        }
    }
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // User UUID
    pub email: String,      // User email
    pub role: UserRole,     // User role
    pub exp: usize,         // Expiration
    pub iat: usize,         // Issued at
    pub iss: String,        // Issuer
    pub aud: String,        // Audience
    pub jti: String,        // JWT ID for blacklisting
}

/// Token pair for access and refresh tokens
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

/// JWT service for token generation and validation
#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    config: JwtConfig,
}

impl std::fmt::Debug for JwtService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtService")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// JWT service errors
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Token generation failed: {0}")]
    TokenGeneration(String),
    #[error("Token validation failed: {0}")]
    TokenValidation(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Missing claims")]
    MissingClaims,
}

impl JwtService {
    /// Create a new JWT service with the given configuration
    #[must_use] 
    pub fn new(config: JwtConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());
        
        Self {
            encoding_key,
            decoding_key,
            config,
        }
    }
    
    /// Generate an access token for the given user
    pub fn generate_access_token(&self, user: &Player) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + Duration::from_std(self.config.access_token_expiry)
            .map_err(|e| JwtError::TokenGeneration(e.to_string()))?;
        
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let claims = Claims {
            sub: user.uuid.to_string(),
            email: user.email.as_ref().to_string(),
            role: user.role.clone(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenGeneration(e.to_string()))
    }
    
    /// Generate a refresh token for the given user
    pub fn generate_refresh_token(&self, user: &Player) -> Result<String, JwtError> {
        let now = Utc::now();
        let exp = now + Duration::from_std(self.config.refresh_token_expiry)
            .map_err(|e| JwtError::TokenGeneration(e.to_string()))?;
        
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let claims = Claims {
            sub: user.uuid.to_string(),
            email: user.email.as_ref().to_string(),
            role: user.role.clone(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            jti: Uuid::new_v4().to_string(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenGeneration(e.to_string()))
    }
    
    /// Validate a token and return the claims
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);
        
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                _ => JwtError::TokenValidation(e.to_string()),
            })?;
        
        Ok(token_data.claims)
    }
    
    /// Generate both access and refresh tokens
    pub fn generate_token_pair(&self, user: &Player) -> Result<TokenPair, JwtError> {
        let access_token = self.generate_access_token(user)?;
        let refresh_token = self.generate_refresh_token(user)?;
        
        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.config.access_token_expiry.as_secs(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Email, TeamName, HashedPassword, Car, CarName, Pilot, PilotName, PilotClass, PilotRarity, PilotSkills, PilotPerformance, Engine, EngineName, Body, BodyName, ComponentRarity};
    
    fn create_test_player() -> Player {
        {
            let skills = PilotSkills::new(6, 6, 7, 5).unwrap();
            let performance = PilotPerformance::new(3, 3).unwrap();
            
            let pilot1 = Pilot::new(
                PilotName::parse("Pilot 1").unwrap(),
                PilotClass::AllRounder,
                PilotRarity::Professional,
                skills.clone(),
                performance.clone(),
                None,
            ).unwrap();
            
            let pilot2 = Pilot::new(
                PilotName::parse("Pilot 2").unwrap(),
                PilotClass::AllRounder,
                PilotRarity::Professional,
                skills.clone(),
                performance.clone(),
                None,
            ).unwrap();
            
            let pilot3 = Pilot::new(
                PilotName::parse("Pilot 3").unwrap(),
                PilotClass::AllRounder,
                PilotRarity::Professional,
                skills,
                performance,
                None,
            ).unwrap();
            
            let engine = Engine::new(
                EngineName::parse("Test Engine").unwrap(),
                ComponentRarity::Common,
                5,
                4,
                None,
            ).unwrap();
            
            let body = Body::new(
                BodyName::parse("Test Body").unwrap(),
                ComponentRarity::Common,
                4,
                5,
                None,
            ).unwrap();
            
            let mut car = Car::new(CarName::parse("Test Car").unwrap(), None).unwrap();
            car.assign_pilots(vec![pilot1.uuid, pilot2.uuid, pilot3.uuid]).unwrap();
            car.assign_engine(engine.uuid);
            car.assign_body(body.uuid);
            
            Player::new_with_assets(
                Email::parse("test@example.com").unwrap(),
                HashedPassword::from_hash("test_hash".to_string()),
                TeamName::parse("Test Team").unwrap(),
                vec![car], // cars
                vec![pilot1, pilot2, pilot3], // pilots
                vec![engine], // engines
                vec![body], // bodies
            ).unwrap()
        }
    }
    
    #[test]
    fn jwt_service_creation_works() {
        let config = JwtConfig::default();
        let jwt_service = JwtService::new(config);
        
        // Should not panic and should be created successfully
        assert!(!jwt_service.config.secret.is_empty());
    }
    
    #[test]
    fn access_token_generation_works() {
        let config = JwtConfig::default();
        let jwt_service = JwtService::new(config);
        let player = create_test_player();
        
        let token = jwt_service.generate_access_token(&player);
        assert!(token.is_ok());
        
        let token_str = token.unwrap();
        assert!(!token_str.is_empty());
        assert!(token_str.contains('.'));  // JWT format check
    }
    
    #[test]
    fn refresh_token_generation_works() {
        let config = JwtConfig::default();
        let jwt_service = JwtService::new(config);
        let player = create_test_player();
        
        let token = jwt_service.generate_refresh_token(&player);
        assert!(token.is_ok());
        
        let token_str = token.unwrap();
        assert!(!token_str.is_empty());
        assert!(token_str.contains('.'));  // JWT format check
    }
    
    #[test]
    fn token_validation_works_with_valid_token() {
        let config = JwtConfig::default();
        let jwt_service = JwtService::new(config);
        let player = create_test_player();
        
        let token = jwt_service.generate_access_token(&player).unwrap();
        let claims = jwt_service.validate_token(&token);
        
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.sub, player.uuid.to_string());
        assert_eq!(claims.email, player.email.as_ref());
        assert_eq!(claims.role, player.role);
    }
    
    #[test]
    fn token_validation_fails_with_invalid_token() {
        let config = JwtConfig::default();
        let jwt_service = JwtService::new(config);
        
        let invalid_token = "invalid.token.here";
        let result = jwt_service.validate_token(invalid_token);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JwtError::TokenValidation(_)));
    }
    
    #[test]
    fn token_validation_fails_with_wrong_secret() {
        let config1 = JwtConfig {
            secret: "secret1".to_string(),
            ..JwtConfig::default()
        };
        let config2 = JwtConfig {
            secret: "secret2".to_string(),
            ..JwtConfig::default()
        };
        
        let jwt_service1 = JwtService::new(config1);
        let jwt_service2 = JwtService::new(config2);
        let player = create_test_player();
        
        let token = jwt_service1.generate_access_token(&player).unwrap();
        let result = jwt_service2.validate_token(&token);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn token_pair_generation_works() {
        let config = JwtConfig::default();
        let jwt_service = JwtService::new(config);
        let player = create_test_player();
        
        let token_pair = jwt_service.generate_token_pair(&player);
        assert!(token_pair.is_ok());
        
        let pair = token_pair.unwrap();
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert!(pair.expires_in > 0);
        
        // Both tokens should be valid
        assert!(jwt_service.validate_token(&pair.access_token).is_ok());
        assert!(jwt_service.validate_token(&pair.refresh_token).is_ok());
    }
    
    #[test]
    fn tokens_have_different_expiry_times() {
        let config = JwtConfig {
            access_token_expiry: StdDuration::from_secs(60),
            refresh_token_expiry: StdDuration::from_secs(3600),
            ..JwtConfig::default()
        };
        let jwt_service = JwtService::new(config);
        let player = create_test_player();
        
        let access_token = jwt_service.generate_access_token(&player).unwrap();
        let refresh_token = jwt_service.generate_refresh_token(&player).unwrap();
        
        let access_claims = jwt_service.validate_token(&access_token).unwrap();
        let refresh_claims = jwt_service.validate_token(&refresh_token).unwrap();
        
        // Refresh token should expire later than access token
        assert!(refresh_claims.exp > access_claims.exp);
    }
    
    #[test]
    fn tokens_have_unique_jti() {
        let config = JwtConfig::default();
        let jwt_service = JwtService::new(config);
        let player = create_test_player();
        
        let token1 = jwt_service.generate_access_token(&player).unwrap();
        let token2 = jwt_service.generate_access_token(&player).unwrap();
        
        let claims1 = jwt_service.validate_token(&token1).unwrap();
        let claims2 = jwt_service.validate_token(&token2).unwrap();
        
        // Each token should have a unique JWT ID
        assert_ne!(claims1.jti, claims2.jti);
    }
}