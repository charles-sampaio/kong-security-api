use actix_web::HttpRequest;
use crate::models::User;
use crate::auth::jwt::verify_jwt;
use mongodb::bson::oid::ObjectId;
use std::str::FromStr;

pub fn create_jwt_token(user: &User) -> Result<String, String> {
    let user_id = user._id.as_ref()
        .ok_or("User ID is required")?
        .to_hex();
    
    let aud = "kong-security-api";
    let iss = "kong-security-service";
    
    let token = crate::auth::jwt::generate_jwt(
        &user_id,
        &user.email,
        user.roles.clone().unwrap_or_default(),
        user.is_active,
        aud,
        iss
    );
    
    Ok(token)
}

pub fn verify_jwt_token(req: &HttpRequest) -> Result<User, String> {
    let auth_header = req.headers()
        .get("Authorization")
        .ok_or("Authorization header missing")?
        .to_str()
        .map_err(|_| "Invalid authorization header")?;

    if !auth_header.starts_with("Bearer ") {
        return Err("Invalid authorization format".to_string());
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix
    let aud = "kong-security-api";
    let iss = "kong-security-service";

    let claims = verify_jwt(token, aud, iss)
        .ok_or("Invalid token")?;

    // Convert claims back to User struct
    let user_id = ObjectId::from_str(&claims.sub)
        .map_err(|_| "Invalid user ID in token")?;

    let user = User {
        _id: Some(user_id),
        tenant_id: String::from(""), // TODO: Extract tenant_id from token claims
        email: claims.email,
        password: None, // OAuth users don't have password
        oauth_provider: None,
        oauth_id: None,
        name: None,
        picture: None,
        roles: Some(claims.roles),
        created_at: Some(mongodb::bson::DateTime::now()),
        updated_at: Some(mongodb::bson::DateTime::now()),
        is_active: claims.is_active,
        last_login: None,
        email_verified: false,
        password_reset_token: None,
        password_reset_expiry: None,
        refresh_tokens: None,
    };

    Ok(user)
}