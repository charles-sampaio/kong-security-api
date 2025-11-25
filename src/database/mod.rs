use mongodb::{Client, Database, options::ClientOptions};
use std::env;
use std::time::Duration;

pub mod indexes;

/// Conecta ao MongoDB com pool otimizado para performance e baixo uso de recursos
pub async fn connect_to_database() -> Result<Database, mongodb::error::Error> {
    let uri = env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    
    // Configurar opções otimizadas
    let mut client_options = ClientOptions::parse(&uri).await?;
    
    // Pool de conexões otimizado (mínimo de recursos)
    client_options.max_pool_size = Some(10); // Máximo 10 conexões simultâneas
    client_options.min_pool_size = Some(2);  // Mínimo 2 conexões mantidas
    
    // Timeouts agressivos para liberar recursos rapidamente
    client_options.connect_timeout = Some(Duration::from_secs(5));
    client_options.server_selection_timeout = Some(Duration::from_secs(5));
    
    // Heartbeat reduzido para economizar recursos
    client_options.heartbeat_freq = Some(Duration::from_secs(60));
    
    // Retry otimizado
    client_options.retry_reads = Some(true);
    client_options.retry_writes = Some(true);
    
    let client = Client::with_options(client_options)?;
    let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| "kong_security".to_string());
    
    Ok(client.database(&database_name))
}