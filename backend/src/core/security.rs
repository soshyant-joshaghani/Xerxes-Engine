use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
}

/// bcrypt only looks at the first 72 bytes.
fn prepare(password: &str) -> Vec<u8> {
    let bytes = password.as_bytes();
    let end = bytes.len().min(72);
    bytes[..end].to_vec()
}

/// Produces `$2b$...` hashes.
pub fn hash_password(password: &str, cost: u32) -> Result<String, String> {
    bcrypt::hash(prepare(password), cost).map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hashed: &str) -> bool {
    bcrypt::verify(prepare(password), hashed).unwrap_or(false)
}

pub fn create_access_token(
    subject: &str,
    secret: &str,
    expires_minutes: i64,
) -> Result<String, String> {
    create_token_with_exp(
        subject,
        secret,
        Utc::now().timestamp() + expires_minutes * 60,
    )
}

/// Explicit expiry (unix seconds); used by tests for expired tokens.
pub fn create_token_with_exp(subject: &str, secret: &str, exp: i64) -> Result<String, String> {
    let claims = Claims {
        sub: subject.to_string(),
        exp,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| e.to_string())
}

/// Returns the `sub` claim of a valid, unexpired HS256 token.
pub fn decode_access_token(token: &str, secret: &str) -> Result<String, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 0;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims.sub)
    .map_err(|e| e.to_string())
}
