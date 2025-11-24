use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use regex::Regex;

/// Estrutura de validação para login
#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 100, message = "Email too long"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

/// Estrutura de validação para registro
#[derive(Debug, Deserialize, Validate, Serialize)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    #[validate(length(max = 100, message = "Email too long"))]
    pub email: String,
    
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
    
    #[validate(length(min = 2, max = 50, message = "Name must be between 2 and 50 characters"))]
    #[validate(custom(function = "validate_name"))]
    pub name: String,
}

/// Validar força da senha
/// Requisitos:
/// - Mínimo 8 caracteres
/// - Pelo menos uma letra maiúscula
/// - Pelo menos uma letra minúscula
/// - Pelo menos um número
/// - Pelo menos um caractere especial
fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new("Password must be at least 8 characters"));
    }

    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err(ValidationError::new("Password must contain at least one uppercase letter"));
    }
    if !has_lowercase {
        return Err(ValidationError::new("Password must contain at least one lowercase letter"));
    }
    if !has_digit {
        return Err(ValidationError::new("Password must contain at least one number"));
    }
    if !has_special {
        return Err(ValidationError::new("Password must contain at least one special character"));
    }

    Ok(())
}

/// Validar nome (sem caracteres especiais perigosos)
fn validate_name(name: &str) -> Result<(), ValidationError> {
    // Permitir apenas letras, espaços, hífens e apóstrofos
    let re = Regex::new(r"^[a-zA-ZÀ-ÿ\s\-']+$").unwrap();
    
    if !re.is_match(name) {
        return Err(ValidationError::new("Name contains invalid characters"));
    }

    // Verificar SQL injection patterns
    let lower = name.to_lowercase();
    if lower.contains("select") || lower.contains("union") || 
       lower.contains("drop") || lower.contains("insert") ||
       lower.contains("<script>") || lower.contains("javascript:") {
        return Err(ValidationError::new("Name contains forbidden patterns"));
    }

    Ok(())
}

/// Sanitizar string removendo caracteres perigosos
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            c.is_alphanumeric() || 
            c.is_whitespace() || 
            *c == '-' || 
            *c == '_' || 
            *c == '@' || 
            *c == '.' ||
            *c == '\''
        })
        .collect()
}

/// Validar ObjectId do MongoDB
pub fn validate_object_id(id: &str) -> bool {
    // ObjectId deve ter exatamente 24 caracteres hexadecimais
    if id.len() != 24 {
        return false;
    }
    id.chars().all(|c| c.is_ascii_hexdigit())
}

/// Helper para validar e retornar erros de validação formatados
pub fn format_validation_errors(errors: validator::ValidationErrors) -> String {
    let mut messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = if let Some(msg) = &error.message {
                msg.to_string()
            } else {
                format!("Validation error in field: {}", field)
            };
            messages.push(message);
        }
    }
    
    messages.join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength() {
        // Senha fraca
        assert!(validate_password_strength("12345678").is_err());
        assert!(validate_password_strength("password").is_err());
        assert!(validate_password_strength("PASSWORD").is_err());
        assert!(validate_password_strength("Pass1234").is_err());
        
        // Senha forte
        assert!(validate_password_strength("Pass123!@").is_ok());
        assert!(validate_password_strength("MyP@ssw0rd").is_ok());
    }

    #[test]
    fn test_name_validation() {
        // Nomes válidos
        assert!(validate_name("John Doe").is_ok());
        assert!(validate_name("María García").is_ok());
        assert!(validate_name("O'Brien").is_ok());
        
        // Nomes inválidos
        assert!(validate_name("John<script>").is_err());
        assert!(validate_name("DROP TABLE users").is_err());
        assert!(validate_name("User123!@#").is_err());
    }

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("Hello<script>"), "Helloscript");
        assert_eq!(sanitize_string("test@example.com"), "test@example.com");
        assert_eq!(sanitize_string("Valid Name 123"), "Valid Name 123");
    }

    #[test]
    fn test_validate_object_id() {
        assert!(validate_object_id("507f1f77bcf86cd799439011"));
        assert!(!validate_object_id("invalid"));
        assert!(!validate_object_id("507f1f77bcf86cd799439")); // muito curto
        assert!(!validate_object_id("507f1f77bcf86cd799439011zzz")); // caracteres inválidos
    }
}
