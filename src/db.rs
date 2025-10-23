use mongodb::{Client, Database};
use std::env;

pub async fn get_db() -> Database {
    let uri = env::var("MONGODB_URI").expect("MONGODB_URI not set");
    let client = Client::with_uri_str(uri)
        .await
        .expect("Failed to connect to MongoDB");
    client.database("kong_security_api")
}
