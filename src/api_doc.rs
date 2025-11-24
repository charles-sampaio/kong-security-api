use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Kong Security API",
        version = "1.0.0",
        description = "Secure API with JWT authentication, comprehensive logging, and advanced security features",
        contact(
            name = "API Support",
            email = "support@kongsecurity.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
        (url = "https://api.kongsecurity.com", description = "Production server")
    ),
    components(schemas(
        LoginRequest,
        RegisterRequest,
        AuthResponse,
        UserResponse,
        LoginStats,
        ErrorResponse,
        HealthResponse,
    )),
    tags(
        (name = "Authentication", description = "User authentication and registration endpoints"),
        (name = "Logs", description = "Login audit log endpoints"),
        (name = "Health", description = "API health check endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

/// Security scheme for JWT Bearer authentication
pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("Enter JWT token obtained from /auth/login"))
                        .build(),
                ),
            )
        }
    }
}

// ========== Request/Response Schemas ==========

use utoipa::ToSchema;
use serde::{Serialize, Deserialize};

/// Login request payload
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoginRequest {
    /// User's email address
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// User's password (minimum 8 characters)
    #[schema(example = "SecurePass123!", min_length = 8)]
    pub password: String,
}

/// Registration request payload
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RegisterRequest {
    /// User's email address
    #[schema(example = "newuser@example.com")]
    pub email: String,
    
    /// Strong password (min 8 chars, must contain uppercase, lowercase, number, and special char)
    #[schema(example = "MySecure@Pass123")]
    pub password: String,
    
    /// User's full name
    #[schema(example = "John Doe")]
    pub name: String,
}

/// Authentication response with JWT token
#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    /// JWT authentication token
    #[schema(example = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    
    /// User information
    pub user: UserResponse,
}

/// User information in responses
#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    /// User's unique identifier
    #[schema(example = "507f1f77bcf86cd799439011")]
    pub id: String,
    
    /// User's email address
    #[schema(example = "user@example.com")]
    pub email: String,
    
    /// User's display name
    #[schema(example = "John Doe")]
    pub name: String,
    
    /// User's roles
    #[schema(example = json!(["user", "admin"]))]
    pub roles: Vec<String>,
}

/// Login statistics
#[derive(Serialize, ToSchema)]
pub struct LoginStats {
    /// Total number of login attempts
    pub total_attempts: i64,
    
    /// Number of successful logins
    pub successful_logins: i64,
    
    /// Number of failed logins
    pub failed_logins: i64,
    
    /// Success rate as percentage
    pub success_rate: f64,
    
    /// Most common device type
    pub most_common_device: String,
    
    /// Most common browser
    pub most_common_browser: String,
}

/// Generic error response
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error type/category
    #[schema(example = "Validation failed")]
    pub error: String,
    
    /// Detailed error message
    #[schema(example = "Password must contain at least one uppercase letter")]
    pub message: String,
}

/// Health check response
#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    /// API status
    #[schema(example = "healthy")]
    pub status: String,
    
    /// API version
    #[schema(example = "1.0.0")]
    pub version: String,
    
    /// Database connection status
    #[schema(example = "connected")]
    pub database: String,
}
