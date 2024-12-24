use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    middleware::Role,
    models::auth::Claims,
};

fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set")
        .into_bytes()
}

pub fn generate_token(email: &str, role: &Role) -> jsonwebtoken::errors::Result<String> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 24 * 3600; // 24 hours from now

    let claims = Claims {
        sub: email.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&get_jwt_secret()),
    )
}

pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&get_jwt_secret()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
} 