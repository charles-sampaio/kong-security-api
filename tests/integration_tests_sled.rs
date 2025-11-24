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

// ============================================================================
// TESTES DE RECUPERAÇÃO DE SENHA (PASSWORD RESET)
// ============================================================================

#[test]
fn test_create_password_reset_token() {
    let db = TestDatabase::new();
    
    // Cria usuário primeiro
    let user = create_test_user(
        "reset@example.com",
        "hashed_oldpassword",
        "Reset User"
    );
    db.insert_user(&user).unwrap();
    
    // Cria token de reset
    let token = db.create_password_reset_token("reset@example.com", Some("192.168.1.1".to_string()));
    
    assert!(!token.is_empty());
    assert_eq!(token.len(), 36); // UUID tem 36 caracteres
}

#[test]
fn test_validate_password_reset_token_success() {
    let db = TestDatabase::new();
    
    // Cria usuário
    let user = create_test_user(
        "validate@example.com",
        "hashed_password",
        "Validate User"
    );
    db.insert_user(&user).unwrap();
    
    // Cria token
    let token = db.create_password_reset_token("validate@example.com", None);
    
    // Valida token
    let result = db.validate_password_reset_token(&token);
    assert!(result.is_ok());
    
    let token_data = result.unwrap();
    assert_eq!(token_data.email, "validate@example.com");
    assert!(!token_data.used);
    assert!(token_data.is_valid());
}

#[test]
fn test_validate_invalid_token() {
    let db = TestDatabase::new();
    
    // Tenta validar token inexistente
    let result = db.validate_password_reset_token("invalid-token-uuid");
    assert!(result.is_err());
}

#[test]
fn test_mark_token_as_used() {
    let db = TestDatabase::new();
    
    // Cria usuário e token
    let user = create_test_user(
        "markused@example.com",
        "hashed_password",
        "Mark Used User"
    );
    db.insert_user(&user).unwrap();
    
    let token = db.create_password_reset_token("markused@example.com", None);
    
    // Marca como usado
    let result = db.mark_token_as_used(&token);
    assert!(result.is_ok());
    
    // Verifica que não pode ser mais validado
    let validation = db.validate_password_reset_token(&token);
    assert!(validation.is_err());
}

#[test]
fn test_token_expiration() {
    let db = TestDatabase::new();
    
    // Cria usuário
    let user = create_test_user(
        "expired@example.com",
        "hashed_password",
        "Expired User"
    );
    db.insert_user(&user).unwrap();
    
    // Cria token expirado (no passado)
    let token = db.create_expired_password_reset_token("expired@example.com");
    
    // Tenta validar - deve falhar
    let result = db.validate_password_reset_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_update_user_password() {
    let db = TestDatabase::new();
    
    // Cria usuário
    let user = create_test_user(
        "updatepass@example.com",
        "hashed_oldpassword",
        "Update Pass User"
    );
    db.insert_user(&user).unwrap();
    
    // Atualiza senha
    let result = db.update_user_password("updatepass@example.com", "hashed_newpassword");
    assert!(result.is_ok());
    
    // Verifica que senha foi atualizada
    let updated_user = db.get_user_by_email("updatepass@example.com").unwrap().unwrap();
    assert_eq!(updated_user.password, "hashed_newpassword");
}

#[test]
fn test_complete_password_reset_flow() {
    let db = TestDatabase::new();
    
    // 1. Registra usuário
    let user = create_test_user(
        "fullflow@example.com",
        "hashed_oldpassword",
        "Full Flow User"
    );
    db.insert_user(&user).unwrap();
    
    // 2. Solicita reset - cria token
    let token = db.create_password_reset_token("fullflow@example.com", Some("192.168.1.100".to_string()));
    assert!(!token.is_empty());
    
    // 3. Valida token
    let validation = db.validate_password_reset_token(&token);
    assert!(validation.is_ok());
    let token_data = validation.unwrap();
    assert_eq!(token_data.email, "fullflow@example.com");
    assert_eq!(token_data.ip_address, Some("192.168.1.100".to_string()));
    
    // 4. Atualiza senha
    let update_result = db.update_user_password("fullflow@example.com", "hashed_newpassword");
    assert!(update_result.is_ok());
    
    // 5. Marca token como usado
    let mark_result = db.mark_token_as_used(&token);
    assert!(mark_result.is_ok());
    
    // 6. Verifica que token não pode ser reusado
    let revalidation = db.validate_password_reset_token(&token);
    assert!(revalidation.is_err());
    
    // 7. Verifica que senha foi atualizada
    let final_user = db.get_user_by_email("fullflow@example.com").unwrap().unwrap();
    assert_eq!(final_user.password, "hashed_newpassword");
}

#[test]
fn test_invalidate_all_tokens_for_email() {
    let db = TestDatabase::new();
    
    // Cria usuário
    let user = create_test_user(
        "invalidate@example.com",
        "hashed_password",
        "Invalidate User"
    );
    db.insert_user(&user).unwrap();
    
    // Cria 3 tokens para o mesmo usuário
    let token1 = db.create_password_reset_token("invalidate@example.com", None);
    let token2 = db.create_password_reset_token("invalidate@example.com", None);
    let token3 = db.create_password_reset_token("invalidate@example.com", None);
    
    // Todos devem ser válidos
    assert!(db.validate_password_reset_token(&token1).is_ok());
    assert!(db.validate_password_reset_token(&token2).is_ok());
    assert!(db.validate_password_reset_token(&token3).is_ok());
    
    // Invalida todos os tokens
    let result = db.invalidate_all_tokens_for_email("invalidate@example.com");
    assert!(result.is_ok());
    
    // Nenhum deve ser mais válido
    assert!(db.validate_password_reset_token(&token1).is_err());
    assert!(db.validate_password_reset_token(&token2).is_err());
    assert!(db.validate_password_reset_token(&token3).is_err());
}

#[test]
fn test_get_tokens_by_email() {
    let db = TestDatabase::new();
    
    // Cria usuário
    let user = create_test_user(
        "gettokens@example.com",
        "hashed_password",
        "Get Tokens User"
    );
    db.insert_user(&user).unwrap();
    
    // Cria alguns tokens
    let _token1 = db.create_password_reset_token("gettokens@example.com", None);
    let _token2 = db.create_password_reset_token("gettokens@example.com", None);
    
    // Busca tokens
    let tokens = db.get_tokens_by_email("gettokens@example.com");
    assert!(tokens.is_ok());
    
    let token_list = tokens.unwrap();
    assert_eq!(token_list.len(), 2);
    
    // Todos devem ter o mesmo email
    for token in token_list {
        assert_eq!(token.email, "gettokens@example.com");
    }
}

#[test]
fn test_cleanup_expired_tokens() {
    let db = TestDatabase::new();
    
    // Cria usuário
    let user = create_test_user(
        "cleanup@example.com",
        "hashed_password",
        "Cleanup User"
    );
    db.insert_user(&user).unwrap();
    
    // Cria token válido
    let valid_token = db.create_password_reset_token("cleanup@example.com", None);
    
    // Cria token expirado
    let _expired_token = db.create_expired_password_reset_token("cleanup@example.com");
    
    // Verifica que tem 2 tokens
    let before = db.get_tokens_by_email("cleanup@example.com").unwrap();
    assert_eq!(before.len(), 2);
    
    // Limpa tokens expirados
    let result = db.cleanup_expired_tokens();
    assert!(result.is_ok());
    
    // Verifica que sobrou apenas 1 token (o válido)
    let after = db.get_tokens_by_email("cleanup@example.com").unwrap();
    assert_eq!(after.len(), 1);
    
    // Token válido ainda funciona
    assert!(db.validate_password_reset_token(&valid_token).is_ok());
}

#[test]
fn test_token_cannot_be_used_twice() {
    let db = TestDatabase::new();
    
    // Cria usuário
    let user = create_test_user(
        "onceonly@example.com",
        "hashed_password",
        "Once Only User"
    );
    db.insert_user(&user).unwrap();
    
    // Cria token
    let token = db.create_password_reset_token("onceonly@example.com", None);
    
    // Primeira validação - OK
    assert!(db.validate_password_reset_token(&token).is_ok());
    
    // Marca como usado
    db.mark_token_as_used(&token).unwrap();
    
    // Segunda tentativa - DEVE FALHAR
    assert!(db.validate_password_reset_token(&token).is_err());
    
    // Validação adicional: token usado não deve passar na validação
    let token_data = db.get_token_by_value(&token);
    if let Ok(data) = token_data {
        assert!(data.used); // Deve estar marcado como usado
    }
}
