use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use reqwest::Client;
use serde::Serialize;
use tokio::runtime::Runtime;

const API_URL: &str = "http://localhost:8080";

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

/// Benchmark de login único
fn benchmark_single_login(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = Client::new();
    
    c.bench_function("single_login", |b| {
        b.iter(|| {
            rt.block_on(async {
                let login_data = LoginRequest {
                    email: black_box("user1@example.com".to_string()),
                    password: black_box("SecurePass123!".to_string()),
                };
                
                let _ = client
                    .post(format!("{}/auth/login", API_URL))
                    .json(&login_data)
                    .send()
                    .await;
            })
        })
    });
}

/// Benchmark de múltiplos logins sequenciais
fn benchmark_sequential_logins(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = Client::new();
    
    let mut group = c.benchmark_group("sequential_logins");
    
    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                rt.block_on(async {
                    for i in 0..count {
                        let user_num = (i % 20) + 1;
                        let login_data = LoginRequest {
                            email: format!("loadtest_user{}@example.com", user_num),
                            password: "SecurePass123!".to_string(),
                        };
                        
                        let _ = client
                            .post(format!("{}/auth/login", API_URL))
                            .json(&login_data)
                            .send()
                            .await;
                    }
                })
            })
        });
    }
    
    group.finish();
}

/// Benchmark de logins paralelos
fn benchmark_parallel_logins(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("parallel_logins");
    
    for count in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter(|| {
                rt.block_on(async {
                    let mut tasks = Vec::new();
                    
                    for i in 0..count {
                        let client = Client::new();
                        let user_num = (i % 20) + 1;
                        
                        let task = tokio::spawn(async move {
                            let login_data = LoginRequest {
                                email: format!("loadtest_user{}@example.com", user_num),
                                password: "SecurePass123!".to_string(),
                            };
                            
                            client
                                .post(format!("{}/auth/login", API_URL))
                                .json(&login_data)
                                .send()
                                .await
                        });
                        
                        tasks.push(task);
                    }
                    
                    futures::future::join_all(tasks).await
                })
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
