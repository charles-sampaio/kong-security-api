use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub jwt: JwtConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub uri: String,
    pub database_name: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: i64,
    pub audience: String,
    pub issuer: String,
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub enable_file_logging: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(AppConfig {
            database: DatabaseConfig {
                uri: env::var("MONGODB_URI")?,
                database_name: env::var("MONGODB_DB").unwrap_or_else(|_| "kong_security_api".to_string()),
            },
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET")?,
                expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .unwrap_or(2),
                audience: env::var("JWT_AUDIENCE").unwrap_or_else(|_| "kong_security_api".to_string()),
                issuer: env::var("JWT_ISSUER").unwrap_or_else(|_| "kong_security_api".to_string()),
            },
            logging: LoggingConfig {
                level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
                enable_file_logging: env::var("ENABLE_FILE_LOGGING")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()
                    .unwrap_or(false),
            },
        })
    }
}