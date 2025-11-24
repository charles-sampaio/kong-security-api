use mongodb::{Client, Database};
use std::env;

pub mod indexes;

pub async fn connect_to_database() -> Result<Database, mongodb::error::Error> {
    let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let client = Client::with_uri_str(&uri).await?;
    let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| "kong_security".to_string());
    Ok(client.database(&database_name))
}