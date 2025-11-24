use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

/// Password reset token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    /// Tenant ID
    pub tenant_id: String,
    
    /// Email of the user who requested the reset
    pub email: String,
    
    /// Unique generated token (UUID)
    pub token: String,
    
    /// Token creation date
    pub created_at: DateTime,
    
    /// Token expiration date (e.g., 1 hour after creation)
    pub expires_at: DateTime,
    
    /// Whether the token has already been used
    pub used: bool,
    
    /// IP address from which it was requested (optional, for auditing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,
}

impl PasswordResetToken {
    /// Creates a new password reset token
    pub fn new(tenant_id: String, email: String, token: String, expiration_hours: i64, ip_address: Option<String>) -> Self {
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::hours(expiration_hours);
        
        Self {
            id: None,
            tenant_id,
            email,
            token,
            created_at: DateTime::from_millis(now.timestamp_millis()),
            expires_at: DateTime::from_millis(expires_at.timestamp_millis()),
            used: false,
            ip_address,
        }
    }
    
    /// Checks if the token is still valid
    pub fn is_valid(&self) -> bool {
        if self.used {
            return false;
        }
        
        let now = chrono::Utc::now();
        let expires_at_chrono = chrono::DateTime::from_timestamp(
            self.expires_at.timestamp_millis() / 1000, 
            0
        ).unwrap();
        
        now < expires_at_chrono
    }
    
    /// Marks the token as used
    pub fn mark_as_used(&mut self) {
        self.used = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_token() {
        let token = PasswordResetToken::new(
            "test@example.com".to_string(),
            "test-token-123".to_string(),
            1,
            Some("127.0.0.1".to_string())
        );
        
        assert_eq!(token.email, "test@example.com");
        assert_eq!(token.token, "test-token-123");
        assert!(!token.used);
        assert_eq!(token.ip_address, Some("127.0.0.1".to_string()));
    }
    
    #[test]
    fn test_token_is_valid() {
        let token = PasswordResetToken::new(
            "test@example.com".to_string(),
            "test-token-123".to_string(),
            1,
            None
        );
        
        assert!(token.is_valid());
    }
    
    #[test]
    fn test_used_token_is_invalid() {
        let mut token = PasswordResetToken::new(
            "test@example.com".to_string(),
            "test-token-123".to_string(),
            1,
            None
        );
        
        token.mark_as_used();
        assert!(!token.is_valid());
    }
}
