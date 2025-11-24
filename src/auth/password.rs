use bcrypt::{DEFAULT_COST, hash, verify};
use std::error::Error;

/// Hash de senha usando bcrypt
pub fn hash_password(password: &str) -> Result<String, Box<dyn Error>> {
    Ok(hash(password, DEFAULT_COST)?)
}

/// Verifica se a senha corresponde ao hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, Box<dyn Error>> {
    Ok(verify(password, hash)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_and_verify() {
        let password = "MySecurePassword123!";
        let hashed = hash_password(password).expect("Failed to hash");
        
        assert!(verify_password(password, &hashed).expect("Failed to verify"));
        assert!(!verify_password("WrongPassword", &hashed).expect("Failed to verify"));
    }
}
