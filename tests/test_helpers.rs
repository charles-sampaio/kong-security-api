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
