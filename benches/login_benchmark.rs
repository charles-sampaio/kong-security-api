/// Benchmarks de Login usando Sled (In-Memory Database)
/// 
/// Este benchmark mede a performance pura da lógica de autenticação,
/// sem overhead de HTTP ou I/O de disco (MongoDB).
/// 
/// Vantagens:
/// - ⚡ Muito mais rápido (sem rede, sem disco)
/// - 🧹 Isolado (não polui MongoDB)
/// - 🎯 Mede apenas a lógica de negócio
/// - 🚀 Não precisa de servidor rodando

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sled::Db;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestUser {
    pub email: String,
    pub password: String,  // hash bcrypt
    pub name: String,
    pub is_active: bool,
}

/// Cria banco Sled em memória para benchmarks
fn create_benchmark_db() -> Db {
    sled::Config::new()
        .temporary(true)  // In-memory
        .open()
        .expect("Failed to create Sled database")
}

/// Insere usuário no banco
fn insert_user(db: &Db, user: &TestUser) {
    let key = format!("user:{}", user.email);
    let value = bincode::serialize(user).expect("Failed to serialize");
    db.insert(key.as_bytes(), value).expect("Failed to insert");
}

/// Busca usuário por email (simula login)
fn get_user_by_email(db: &Db, email: &str) -> Option<TestUser> {
    let key = format!("user:{}", email);
    db.get(key.as_bytes())
        .ok()?
        .map(|data| bincode::deserialize(&data).ok())
        .flatten()
}

/// Simula verificação de login (busca + validação)
fn authenticate_user(db: &Db, email: &str, password: &str) -> bool {
    if let Some(user) = get_user_by_email(db, email) {
        user.is_active && user.password == password
    } else {
        false
    }
}

/// Benchmark de login único
fn benchmark_single_login(c: &mut Criterion) {
    let db = create_benchmark_db();
    
    // Criar usuário de teste
    let user = TestUser {
        email: "user1@example.com".to_string(),
        password: "hashed_password_123".to_string(),
        name: "Test User".to_string(),
        is_active: true,
    };
    insert_user(&db, &user);
    
    c.bench_function("single_login_sled", |b| {
        b.iter(|| {
            authenticate_user(
                &db,
                black_box("user1@example.com"),
                black_box("hashed_password_123"),
            )
        })
    });
}

/// Benchmark de múltiplos logins sequenciais
fn benchmark_sequential_logins(c: &mut Criterion) {
    let db = create_benchmark_db();
    
    // Criar 20 usuários de teste
    for i in 1..=20 {
        let user = TestUser {
            email: format!("loadtest_user{}@example.com", i),
            password: "hashed_password_123".to_string(),
            name: format!("Load Test User {}", i),
            is_active: true,
        };
        insert_user(&db, &user);
    }
    
    let mut group = c.benchmark_group("sequential_logins_sled");
    
    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                for i in 0..count {
                    let user_num = (i % 20) + 1;
                    let email = format!("loadtest_user{}@example.com", user_num);
                    authenticate_user(
                        &db,
                        black_box(&email),
                        black_box("hashed_password_123"),
                    );
                }
            })
        });
    }
    
    group.finish();
}

/// Benchmark de logins paralelos (usando threads)
fn benchmark_parallel_logins(c: &mut Criterion) {
    let db = Arc::new(create_benchmark_db());
    
    // Criar 20 usuários de teste
    for i in 1..=20 {
        let user = TestUser {
            email: format!("loadtest_user{}@example.com", i),
            password: "hashed_password_123".to_string(),
            name: format!("Load Test User {}", i),
            is_active: true,
        };
        insert_user(&db, &user);
    }
    
    let mut group = c.benchmark_group("parallel_logins_sled");
    
    for count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                let mut handles = vec![];
                
                for i in 0..count {
                    let db_clone = Arc::clone(&db);
                    let user_num = (i % 20) + 1;
                    let email = format!("loadtest_user{}@example.com", user_num);
                    
                    let handle = std::thread::spawn(move || {
                        authenticate_user(
                            &db_clone,
                            &email,
                            "hashed_password_123",
                        )
                    });
                    
                    handles.push(handle);
                }
                
                for handle in handles {
                    let _ = handle.join();
                }
            })
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_login,
    benchmark_sequential_logins,
    benchmark_parallel_logins
);
criterion_main!(benches);
