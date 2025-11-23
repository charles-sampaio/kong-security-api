use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey, Algorithm, TokenData};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub iat: usize,
    pub exp: usize,
    pub jti: String,
    pub aud: String,
    pub iss: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,
    pub jti: String,
    pub iat: usize,
    pub exp: usize,
}

fn get_private_key() -> Vec<u8> {
    include_bytes!("../../private.pem").to_vec()
}

fn get_public_key() -> Vec<u8> {
    include_bytes!("../../public.pem").to_vec()
}

// Gera JWT com RS256
pub fn generate_jwt(
    user_id: &str,
    email: &str,
    roles: Vec<String>,
    is_active: bool,
    aud: &str,
    iss: &str
) -> String {
    let iat = Utc::now().timestamp() as usize;
    let exp = (Utc::now() + Duration::hours(2)).timestamp() as usize;
    let jti = Uuid::new_v4().to_string();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        roles,
        is_active,
        iat,
        exp,
        jti,
        aud: aud.to_string(),
        iss: iss.to_string(),
    };

    encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(&get_private_key()).expect("Invalid private key")
    ).expect("Failed to generate JWT token")
}

pub fn generate_refresh_token(user_id: &str) -> String {
    let iat = Utc::now().timestamp() as usize;
    let exp = (Utc::now() + Duration::days(30)).timestamp() as usize;
    let jti = Uuid::new_v4().to_string();

    let claims = RefreshTokenClaims { sub: user_id.to_string(), jti, iat, exp };

    encode(
        &Header::new(Algorithm::RS256),
        &claims,
        &EncodingKey::from_rsa_pem(&get_private_key()).expect("Invalid private key")
    ).expect("Failed to generate JWT token")
}

pub fn verify_jwt(token: &str, aud: &str, iss: &str) -> Option<Claims> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[aud]);

    let mut issuers = HashSet::new();
    issuers.insert(iss.to_string());
    validation.iss = Some(issuers);

    decode::<Claims>(
        token,
        &DecodingKey::from_rsa_pem(&get_public_key()).ok()?,
        &validation
    ).ok().map(|TokenData { claims, .. }| claims)
}

pub fn verify_refresh_token(token: &str) -> Option<RefreshTokenClaims> {
    let validation = Validation::new(Algorithm::RS256);

    decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_rsa_pem(&get_public_key()).ok()?,
        &validation
    ).ok().map(|TokenData { claims, .. }| claims)
}