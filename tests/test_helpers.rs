use serde::{Deserialize, Serialize};
use sled::Db;
use std::sync::Arc;

/// Estrutura do usuário para testes com Sled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub password: String,
    pub name: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub created_at: i64,
}

/// Estrutura do token de reset de senha para testes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPasswordResetToken {
    pub email: String,
    pub token: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub used: bool,
    pub ip_address: Option<String>,
}

impl TestPasswordResetToken {
    /// Verifica se o token é válido (não expirado e não usado)
    pub fn is_valid(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        !self.used && self.expires_at > now
    }
}

/// Banco de dados Sled em memória para testes
pub struct TestDatabase {
    db: Arc<Db>,
}

impl TestDatabase {
    /// Cria um novo banco Sled em memória
    pub fn new() -> Self {
        let config = sled::Config::new().temporary(true);
        let db = config.open().expect("Failed to create test database");
        
        TestDatabase {
            db: Arc::new(db),
        }
    }

    /// Insere um usuário no banco de testes
    pub fn insert_user(&self, user: &TestUser) -> Result<(), String> {
        let key = format!("user:{}", user.email);
        let value = bincode::serialize(user)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        self.db
            .insert(key.as_bytes(), value)
            .map_err(|e| format!("Insert error: {}", e))?;
        
        Ok(())
    }

    /// Busca um usuário por email
    pub fn get_user_by_email(&self, email: &str) -> Result<Option<TestUser>, String> {
        let key = format!("user:{}", email);
        
        match self.db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let user: TestUser = bincode::deserialize(&bytes)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
                Ok(Some(user))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Verifica se um usuário existe
    pub fn user_exists(&self, email: &str) -> bool {
        let key = format!("user:{}", email);
        self.db.get(key.as_bytes()).ok().flatten().is_some()
    }

    /// Remove um usuário
    pub fn delete_user(&self, email: &str) -> Result<(), String> {
        let key = format!("user:{}", email);
        self.db
            .remove(key.as_bytes())
            .map_err(|e| format!("Delete error: {}", e))?;
        Ok(())
    }

    /// Limpa todos os dados (útil entre testes)
    pub fn clear(&self) -> Result<(), String> {
        self.db.clear().map_err(|e| format!("Clear error: {}", e))?;
        Ok(())
    }

    /// Retorna o número de usuários
    pub fn count_users(&self) -> usize {
        self.db
            .scan_prefix(b"user:")
            .count()
    }

    /// Lista todos os usuários
    pub fn list_users(&self) -> Result<Vec<TestUser>, String> {
        let mut users = Vec::new();
        
        for item in self.db.scan_prefix(b"user:") {
            match item {
                Ok((_, value)) => {
                    let user: TestUser = bincode::deserialize(&value)
                        .map_err(|e| format!("Deserialization error: {}", e))?;
                    users.push(user);
                }
                Err(e) => return Err(format!("Scan error: {}", e)),
            }
        }
        
        Ok(users)
    }

    // ========================================================================
    // PASSWORD RESET FUNCTIONS
    // ========================================================================

    /// Cria um token de reset de senha
    pub fn create_password_reset_token(&self, email: &str, ip_address: Option<String>) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let token = uuid::Uuid::new_v4().to_string();
        let expires_at = now + 3600; // 1 hora
        
        let reset_token = TestPasswordResetToken {
            email: email.to_string(),
            token: token.clone(),
            created_at: now,
            expires_at,
            used: false,
            ip_address,
        };
        
        let key = format!("reset_token:{}", token);
        let value = bincode::serialize(&reset_token).unwrap();
        self.db.insert(key.as_bytes(), value).unwrap();
        
        token
    }

    /// Cria um token expirado (para testes)
    pub fn create_expired_password_reset_token(&self, email: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let token = uuid::Uuid::new_v4().to_string();
        let expires_at = now - 3600; // Expirado (1 hora atrás)
        
        let reset_token = TestPasswordResetToken {
            email: email.to_string(),
            token: token.clone(),
            created_at: now - 7200, // Criado 2 horas atrás
            expires_at,
            used: false,
            ip_address: None,
        };
        
        let key = format!("reset_token:{}", token);
        let value = bincode::serialize(&reset_token).unwrap();
        self.db.insert(key.as_bytes(), value).unwrap();
        
        token
    }

    /// Valida um token de reset
    pub fn validate_password_reset_token(&self, token: &str) -> Result<TestPasswordResetToken, String> {
        let key = format!("reset_token:{}", token);
        
        match self.db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let reset_token: TestPasswordResetToken = bincode::deserialize(&bytes)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
                
                if reset_token.is_valid() {
                    Ok(reset_token)
                } else {
                    Err("Invalid or expired token".to_string())
                }
            }
            Ok(None) => Err("Token not found".to_string()),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Marca um token como usado
    pub fn mark_token_as_used(&self, token: &str) -> Result<(), String> {
        let key = format!("reset_token:{}", token);
        
        match self.db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let mut reset_token: TestPasswordResetToken = bincode::deserialize(&bytes)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
                
                reset_token.used = true;
                
                let value = bincode::serialize(&reset_token)
                    .map_err(|e| format!("Serialization error: {}", e))?;
                
                self.db
                    .insert(key.as_bytes(), value)
                    .map_err(|e| format!("Insert error: {}", e))?;
                
                Ok(())
            }
            Ok(None) => Err("Token not found".to_string()),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Atualiza a senha de um usuário
    pub fn update_user_password(&self, email: &str, new_password_hash: &str) -> Result<(), String> {
        let key = format!("user:{}", email);
        
        match self.db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let mut user: TestUser = bincode::deserialize(&bytes)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
                
                user.password = new_password_hash.to_string();
                
                let value = bincode::serialize(&user)
                    .map_err(|e| format!("Serialization error: {}", e))?;
                
                self.db
                    .insert(key.as_bytes(), value)
                    .map_err(|e| format!("Insert error: {}", e))?;
                
                Ok(())
            }
            Ok(None) => Err("User not found".to_string()),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Invalida todos os tokens de um email
    pub fn invalidate_all_tokens_for_email(&self, email: &str) -> Result<(), String> {
        let tokens = self.get_tokens_by_email(email)?;
        
        for token in tokens {
            self.mark_token_as_used(&token.token)?;
        }
        
        Ok(())
    }

    /// Busca todos os tokens de um email
    pub fn get_tokens_by_email(&self, email: &str) -> Result<Vec<TestPasswordResetToken>, String> {
        let mut tokens = Vec::new();
        
        for item in self.db.scan_prefix(b"reset_token:") {
            match item {
                Ok((_, value)) => {
                    let token: TestPasswordResetToken = bincode::deserialize(&value)
                        .map_err(|e| format!("Deserialization error: {}", e))?;
                    
                    if token.email == email {
                        tokens.push(token);
                    }
                }
                Err(e) => return Err(format!("Scan error: {}", e)),
            }
        }
        
        Ok(tokens)
    }

    /// Busca um token pelo valor (para testes)
    pub fn get_token_by_value(&self, token_value: &str) -> Result<TestPasswordResetToken, String> {
        let key = format!("reset_token:{}", token_value);
        
        match self.db.get(key.as_bytes()) {
            Ok(Some(bytes)) => {
                let token: TestPasswordResetToken = bincode::deserialize(&bytes)
                    .map_err(|e| format!("Deserialization error: {}", e))?;
                Ok(token)
            }
            Ok(None) => Err("Token not found".to_string()),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Limpa tokens expirados
    pub fn cleanup_expired_tokens(&self) -> Result<(), String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let mut keys_to_remove = Vec::new();
        
        for item in self.db.scan_prefix(b"reset_token:") {
            match item {
                Ok((key, value)) => {
                    let token: TestPasswordResetToken = bincode::deserialize(&value)
                        .map_err(|e| format!("Deserialization error: {}", e))?;
                    
                    if token.expires_at < now {
                        keys_to_remove.push(key);
                    }
                }
                Err(e) => return Err(format!("Scan error: {}", e)),
            }
        }
        
        for key in keys_to_remove {
            self.db
                .remove(&key)
                .map_err(|e| format!("Remove error: {}", e))?;
        }
        
        Ok(())
    }
}

impl Default for TestDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper para criar usuário de teste
pub fn create_test_user(email: &str, password_hash: &str, name: &str) -> TestUser {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    TestUser {
        id: uuid::Uuid::new_v4().to_string(),
        email: email.to_string(),
        password: password_hash.to_string(),
        name: name.to_string(),
        roles: vec!["user".to_string()],
        is_active: true,
        created_at: timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sled_database_creation() {
        let db = TestDatabase::new();
        assert_eq!(db.count_users(), 0);
    }

    #[test]
    fn test_insert_and_get_user() {
        let db = TestDatabase::new();
        let user = create_test_user(
            "test@example.com",
            "hashed_password",
            "Test User"
        );

        db.insert_user(&user).unwrap();
        
        let retrieved = db.get_user_by_email("test@example.com").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().email, "test@example.com");
    }

    #[test]
    fn test_user_exists() {
        let db = TestDatabase::new();
        let user = create_test_user(
            "exists@example.com",
            "hashed_password",
            "Exists User"
        );

        assert!(!db.user_exists("exists@example.com"));
        db.insert_user(&user).unwrap();
        assert!(db.user_exists("exists@example.com"));
    }

    #[test]
    fn test_delete_user() {
        let db = TestDatabase::new();
        let user = create_test_user(
            "delete@example.com",
            "hashed_password",
            "Delete User"
        );

        db.insert_user(&user).unwrap();
        assert!(db.user_exists("delete@example.com"));
        
        db.delete_user("delete@example.com").unwrap();
        assert!(!db.user_exists("delete@example.com"));
    }

    #[test]
    fn test_clear_database() {
        let db = TestDatabase::new();
        
        for i in 1..=5 {
            let user = create_test_user(
                &format!("user{}@example.com", i),
                "hashed_password",
                &format!("User {}", i)
            );
            db.insert_user(&user).unwrap();
        }

        assert_eq!(db.count_users(), 5);
        db.clear().unwrap();
        assert_eq!(db.count_users(), 0);
    }

    #[test]
    fn test_list_users() {
        let db = TestDatabase::new();
        
        for i in 1..=3 {
            let user = create_test_user(
                &format!("list{}@example.com", i),
                "hashed_password",
                &format!("List User {}", i)
            );
            db.insert_user(&user).unwrap();
        }

        let users = db.list_users().unwrap();
        assert_eq!(users.len(), 3);
    }
}
