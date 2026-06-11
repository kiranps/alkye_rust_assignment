use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::auth::AuthError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub email: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

fn secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key_change_in_prod".into())
}

pub fn create_token(user_id: i32, email: &str, role: &str) -> Result<String, AuthError> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        iat: now,
        exp: now + 86400,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret().as_bytes()),
    )
    .map_err(|_| AuthError::TokenCreation)
}

pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret().as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AuthError::Unauthorized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let token = create_token(1, "admin@example.com", "admin").unwrap();
        let claims = validate_token(&token).unwrap();
        assert_eq!(claims.sub, 1);
        assert_eq!(claims.email, "admin@example.com");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn test_validate_token_rejects_garbage() {
        let result = validate_token("not.a.real.token");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_token_rejects_empty() {
        let result = validate_token("");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_claims_have_future_expiry() {
        let token = create_token(42, "test@test.com", "staff").unwrap();
        let claims = validate_token(&token).unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        assert!(claims.exp > now, "token should not be expired");
        assert!(claims.exp <= now + 86400, "exp should be at most 24h from now");
        assert_eq!(claims.iat, claims.exp - 86400);
    }

    #[test]
    fn test_different_users_get_different_tokens() {
        let t1 = create_token(1, "a@a.com", "admin").unwrap();
        let t2 = create_token(2, "b@b.com", "staff").unwrap();
        assert_ne!(t1, t2);
    }
}
