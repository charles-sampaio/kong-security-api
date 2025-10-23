use actix_web::{post, get, web, HttpResponse, HttpRequest, Responder};
use bcrypt::{hash, verify};
use mongodb::Database;
use serde::Deserialize;
use crate::models::User;
use crate::auth::{generate_jwt, generate_refresh_token, verify_jwt, Claims};
use mongodb::bson::doc;

#[derive(Deserialize)]
pub struct RegisterData { 
    pub email: String, 
    pub password: String 
}

#[derive(Deserialize)]
pub struct LoginData { 
    pub email: String, 
    pub password: String 
}

#[post("/register")]
pub async fn register(data: web::Json<RegisterData>, db: web::Data<Database>) -> impl Responder {
    let users = db.collection::<User>("users");

    if users.find_one(doc! { "email": &data.email }).await.unwrap().is_some() {
        return HttpResponse::Conflict().body("User already exists");
    }

    let hashed = hash(&data.password, 12).unwrap();
    let new_user = User::new(data.email.clone(), hashed);

    users.insert_one(new_user).await.unwrap();
    HttpResponse::Created().body("User created successfully")
}

#[post("/login")]
pub async fn login(data: web::Json<LoginData>, db: web::Data<Database>) -> impl Responder {
    let users = db.collection::<User>("users");
    if let Some(user) = users.find_one(doc! { "email": &data.email }).await.unwrap() {
        if !user.is_active { 
            return HttpResponse::Forbidden().body("Account is deactivated"); 
        }

        if verify(&data.password, &user.password).unwrap() {
            let user_id = user._id.as_ref().unwrap().to_string();

            let access_token = generate_jwt(
                &user_id,
                &user.email,
                user.roles.clone().unwrap_or_else(|| vec!["user".to_string()]),
                user.is_active,
                "my_audience",
                "my_issuer"
            );

            let refresh_token = generate_refresh_token(&user_id);

            let mut tokens = user.refresh_tokens.clone().unwrap_or_else(|| vec![]);
            tokens.push(refresh_token.clone());

            users.update_one(
                doc! {"_id": &user._id.unwrap()},
                doc! {"$set": {"refresh_tokens": tokens}}
            ).await.unwrap();

            return HttpResponse::Ok().json(serde_json::json!({
                "access_token": access_token,
                "refresh_token": refresh_token
            }));
        }
    }

    HttpResponse::Unauthorized().body("Invalid credentials")
}

fn extract_user_from_token(req: &HttpRequest) -> Option<Claims> {
    let token = req.headers()
        .get("Authorization")?
        .to_str().ok()?
        .trim_start_matches("Bearer ")
        .to_string();

    verify_jwt(&token, "my_audience", "my_issuer")
}

#[get("/profile")]
pub async fn profile(req: HttpRequest) -> impl Responder {
    if let Some(claims) = extract_user_from_token(&req) {
        HttpResponse::Ok().json(serde_json::json!({
            "id": claims.sub,
            "email": claims.email,
            "roles": claims.roles,
            "is_active": claims.is_active,
            "iat": claims.iat,
            "exp": claims.exp
        }))
    } else {
        HttpResponse::Unauthorized().body("Invalid or missing token")
    }
}
