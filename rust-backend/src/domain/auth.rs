use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// User roles for authorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum UserRole {
    Player,
    Admin,
    SuperAdmin,
}

impl UserRole {
    /// Check if this role has admin privileges
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::SuperAdmin)
    }
    
    /// Check if this role can access any resource
    pub fn can_access_any_resource(&self) -> bool {
        self.is_admin()
    }
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Player
    }
}

/// A secure password wrapper that prevents accidental exposure
#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

/// A hashed password stored in the database
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct HashedPassword(String);

/// User credentials for authentication
#[derive(Debug, Deserialize, ToSchema)]
pub struct UserCredentials {
    pub email: String,
    pub password: String,
}

/// Registration data for new users
#[derive(Debug, Deserialize, ToSchema)]
pub struct UserRegistration {
    pub email: String,
    pub password: String,
    pub team_name: String,
}

impl Password {
    /// Create a new password from a string
    pub fn new(password: String) -> Result<Self, String> {
        if password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }
        
        if password.len() < 8 {
            return Err("Password must be at least 8 characters long".to_string());
        }
        
        if password.len() > 128 {
            return Err("Password cannot be longer than 128 characters".to_string());
        }
        
        // Check for at least one uppercase, one lowercase, and one digit
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        
        if !has_uppercase {
            return Err("Password must contain at least one uppercase letter".to_string());
        }
        
        if !has_lowercase {
            return Err("Password must contain at least one lowercase letter".to_string());
        }
        
        if !has_digit {
            return Err("Password must contain at least one digit".to_string());
        }
        
        Ok(Self(Secret::new(password)))
    }
    
    /// Hash the password using Argon2
    pub fn hash(&self) -> Result<HashedPassword, String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(self.0.expose_secret().as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash password: {}", e))?;
        
        Ok(HashedPassword(password_hash.to_string()))
    }
}

impl HashedPassword {
    /// Verify a password against this hash
    pub fn verify(&self, password: &Password) -> Result<bool, String> {
        let parsed_hash = PasswordHash::new(&self.0)
            .map_err(|e| format!("Failed to parse password hash: {}", e))?;
        
        let argon2 = Argon2::default();
        
        match argon2.verify_password(password.0.expose_secret().as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(format!("Password verification failed: {}", e)),
        }
    }
    
    /// Get the hash string for database storage
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Create from a stored hash string (for loading from database)
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_validation_works() {
        // Valid password
        assert!(Password::new("ValidPass123".to_string()).is_ok());
        
        // Too short
        assert!(Password::new("short".to_string()).is_err());
        
        // No uppercase
        assert!(Password::new("lowercase123".to_string()).is_err());
        
        // No lowercase
        assert!(Password::new("UPPERCASE123".to_string()).is_err());
        
        // No digit
        assert!(Password::new("NoDigitPass".to_string()).is_err());
        
        // Empty
        assert!(Password::new("".to_string()).is_err());
    }

    #[test]
    fn password_hashing_and_verification_works() {
        let password = Password::new("TestPassword123".to_string()).unwrap();
        let hashed = password.hash().unwrap();
        
        // Correct password should verify
        assert!(hashed.verify(&password).unwrap());
        
        // Wrong password should not verify
        let wrong_password = Password::new("WrongPassword123".to_string()).unwrap();
        assert!(!hashed.verify(&wrong_password).unwrap());
    }

    #[test]
    fn different_hashes_for_same_password() {
        let password = Password::new("TestPassword123".to_string()).unwrap();
        let hash1 = password.hash().unwrap();
        let hash2 = password.hash().unwrap();
        
        // Hashes should be different due to random salt
        assert_ne!(hash1.as_str(), hash2.as_str());
        
        // But both should verify the same password
        assert!(hash1.verify(&password).unwrap());
        assert!(hash2.verify(&password).unwrap());
    }
}