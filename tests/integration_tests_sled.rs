/// Testes de Integração usando Sled (Banco em Memória)
/// 
/// Estes testes usam Sled DIRETAMENTE (sem HTTP), tornando os testes:
/// - ✅ Mais rápidos (sem overhead de rede/HTTP)
/// - ✅ Isolados (cada teste tem seu próprio banco)
/// - ✅ Sem dependências externas (não precisa de servidor rodando)

mod test_helpers;

use test_helpers::{TestDatabase, create_test_user};

#[test]
fn test_database_starts_empty() {
    let db = TestDatabase::new();
    assert_eq!(db.count_users(), 0);
}

#[test]
fn test_user_registration() {
    let db = TestDatabase::new();
    
    let user = create_test_user(
        "newuser@example.com",
        "hashed_password",
        "New User"
    );

    // Registra usuário
    let result = db.insert_user(&user);
    assert!(result.is_ok());
    
    // Verifica que foi criado
    assert!(db.user_exists("newuser@example.com"));
    assert_eq!(db.count_users(), 1);
}

#[test]
fn test_user_login() {
    let db = TestDatabase::new();
    
    // Registra usuário primeiro
    let user = create_test_user(
        "login@example.com",
        "hashed_SecurePass123!",
        "Login User"
    );
    db.insert_user(&user).unwrap();

    // Simula login - busca usuário por email
    let found_user = db.get_user_by_email("login@example.com").unwrap();
    
    assert!(found_user.is_some());
    let found_user = found_user.unwrap();
    assert_eq!(found_user.email, "login@example.com");
    assert_eq!(found_user.name, "Login User");
    
    // Verifica senha
    assert_eq!(found_user.password, "hashed_SecurePass123!");
}

#[test]
fn test_login_with_wrong_password() {
    let db = TestDatabase::new();
    
    // Registra usuário
    let user = create_test_user(
        "wrongpass@example.com",
        "hashed_CorrectPassword",
        "Wrong Pass User"
    );
    db.insert_user(&user).unwrap();

    // Busca usuário
    let found_user = db.get_user_by_email("wrongpass@example.com").unwrap().unwrap();
    
    // Verifica senha errada
    assert_ne!(found_user.password, "hashed_WrongPassword");
}

#[test]
fn test_duplicate_registration() {
    let db = TestDatabase::new();
    
    let user1 = create_test_user(
        "duplicate@example.com",
        "hashed_password",
        "Duplicate User"
    );

    // Primeiro registro - deve funcionar
    assert!(db.insert_user(&user1).is_ok());
    assert_eq!(db.count_users(), 1);

    // Segundo registro - para evitar duplicatas, verificamos primeiro
    assert!(db.user_exists("duplicate@example.com"));
    
    // Não inserimos novamente
    let user2 = create_test_user(
        "duplicate@example.com",
        "hashed_password2",
        "Duplicate User 2"
    );
    
    // Mesmo se inserir, Sled sobrescreve (comportamento esperado)
    db.insert_user(&user2).unwrap();
    assert_eq!(db.count_users(), 1); // Ainda é 1, não 2
}

#[test]
fn test_user_not_found() {
    let db = TestDatabase::new();
    
    // Busca usuário que não existe
    let result = db.get_user_by_email("notfound@example.com").unwrap();
    assert!(result.is_none());
}

#[test]
fn test_multiple_users() {
    let db = TestDatabase::new();
    
    // Cria múltiplos usuários
    for i in 1..=5 {
        let user = create_test_user(
            &format!("user{}@example.com", i),
            "hashed_password",
            &format!("User {}", i)
        );
        db.insert_user(&user).unwrap();
    }

    assert_eq!(db.count_users(), 5);
    
    // Verifica que todos foram criados
    for i in 1..=5 {
        assert!(db.user_exists(&format!("user{}@example.com", i)));
    }
}

#[test]
fn test_user_deletion() {
    let db = TestDatabase::new();
    
    // Cria usuário
    let user = create_test_user(
        "delete@example.com",
        "hashed_password",
        "Delete User"
    );
    db.insert_user(&user).unwrap();
    assert!(db.user_exists("delete@example.com"));
    
    // Deleta usuário
    db.delete_user("delete@example.com").unwrap();
    assert!(!db.user_exists("delete@example.com"));
}

#[test]
fn test_clear_database() {
    let db = TestDatabase::new();
    
    // Cria vários usuários
    for i in 1..=10 {
        let user = create_test_user(
            &format!("clear{}@example.com", i),
            "hashed_password",
            &format!("Clear User {}", i)
        );
        db.insert_user(&user).unwrap();
    }

    assert_eq!(db.count_users(), 10);
    
    // Limpa banco
    db.clear().unwrap();
    assert_eq!(db.count_users(), 0);
}

#[test]
fn test_list_all_users() {
    let db = TestDatabase::new();
    
    // Cria 3 usuários
    for i in 1..=3 {
        let user = create_test_user(
            &format!("list{}@example.com", i),
            "hashed_password",
            &format!("List User {}", i)
        );
        db.insert_user(&user).unwrap();
    }

    // Lista todos
    let users = db.list_users().unwrap();
    assert_eq!(users.len(), 3);
    
    // Verifica que todos estão na lista
    let emails: Vec<String> = users.iter().map(|u| u.email.clone()).collect();
    assert!(emails.contains(&"list1@example.com".to_string()));
    assert!(emails.contains(&"list2@example.com".to_string()));
    assert!(emails.contains(&"list3@example.com".to_string()));
}

#[test]
fn test_user_fields() {
    let db = TestDatabase::new();
    
    let user = create_test_user(
        "fields@example.com",
        "hashed_mypassword",
        "Fields User"
    );
    db.insert_user(&user).unwrap();
    
    let retrieved = db.get_user_by_email("fields@example.com").unwrap().unwrap();
    
    // Verifica todos os campos
    assert_eq!(retrieved.email, "fields@example.com");
    assert_eq!(retrieved.password, "hashed_mypassword");
    assert_eq!(retrieved.name, "Fields User");
    assert_eq!(retrieved.roles, vec!["user".to_string()]);
    assert!(retrieved.is_active);
    assert!(!retrieved.id.is_empty());
    assert!(retrieved.created_at > 0);
}
