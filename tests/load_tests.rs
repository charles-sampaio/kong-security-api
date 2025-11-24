/// Testes de Carga usando Sled (Banco em Memória)
/// 
/// Agora os testes de carga usam Sled, tornando-os:
/// - ⚡ Muito mais rápidos (sem I/O de rede)
/// - 🧹 Isolados (sem poluir MongoDB)
/// - 🚀 Sem dependências externas

mod test_helpers;

use colored::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use test_helpers::{TestDatabase, create_test_user};

#[derive(Debug, Clone)]
struct LoadTestResult {
    total_requests: usize,
    successful: usize,
    failed: usize,
    duration: Duration,
    min_latency: Duration,
    max_latency: Duration,
    avg_latency: Duration,
    p50_latency: Duration,
    p95_latency: Duration,
    p99_latency: Duration,
}

impl LoadTestResult {
    fn requests_per_second(&self) -> f64 {
        self.total_requests as f64 / self.duration.as_secs_f64()
    }

    fn success_rate(&self) -> f64 {
        (self.successful as f64 / self.total_requests as f64) * 100.0
    }

    fn print_results(&self, test_name: &str) {
        println!("\n{}", "═".repeat(80).bright_cyan());
        println!("{} {}", "📊".bright_yellow(), test_name.bright_white().bold());
        println!("{} {}", "💾".bright_blue(), "Usando Sled (In-Memory)".bright_white());
        println!("{}", "═".repeat(80).bright_cyan());
        
        println!("\n{}", "Requisições:".bright_white().bold());
        println!("  Total:          {}", self.total_requests.to_string().bright_cyan());
        println!("  Bem-sucedidas:  {} {}", 
            self.successful.to_string().bright_green(),
            "✓".bright_green()
        );
        println!("  Falhas:         {} {}", 
            self.failed.to_string().bright_red(),
            if self.failed > 0 { "✗" } else { "" }.bright_red()
        );
        println!("  Taxa de sucesso: {:.2}%", self.success_rate());
        
        println!("\n{}", "Performance:".bright_white().bold());
        println!("  Duração total:   {:.2}s", self.duration.as_secs_f64());
        println!("  Req/s:           {:.2}", self.requests_per_second());
        
        println!("\n{}", "Latência:".bright_white().bold());
        println!("  Mínima:    {:>8.2}µs", self.min_latency.as_micros());
        println!("  Média:     {:>8.2}µs", self.avg_latency.as_micros());
        println!("  P50:       {:>8.2}µs", self.p50_latency.as_micros());
        println!("  P95:       {:>8.2}µs", self.p95_latency.as_micros());
        println!("  P99:       {:>8.2}µs", self.p99_latency.as_micros());
        println!("  Máxima:    {:>8.2}µs", self.max_latency.as_micros());
        
        println!("\n{}", "═".repeat(80).bright_cyan());
    }
}

/// Cria usuários de teste no banco Sled
fn create_test_users_in_db(db: &TestDatabase, count: usize) {
    println!("\n{} Criando {} usuários no Sled...", "🔧".bright_yellow(), count);
    
    let names = [
        "Alice Smith", "Bob Johnson", "Carol Williams", "David Brown",
        "Eve Davis", "Frank Miller", "Grace Wilson", "Henry Moore",
        "Iris Taylor", "Jack Anderson", "Kate Thomas", "Leo Jackson",
        "Mary White", "Nathan Harris", "Olivia Martin", "Paul Thompson",
        "Quinn Garcia", "Rose Martinez", "Sam Robinson", "Tina Clark"
    ];
    
    for i in 1..=count {
        let email = format!("loadtest_user{}@example.com", i);
        let password = "hashed_SecurePass123!";
        let name = names[(i - 1) % names.len()];
        
        let user = create_test_user(&email, password, name);
        db.insert_user(&user).expect("Failed to create user");
        
        if i % 10 == 0 {
            print!("{}", "✓ ".bright_green());
        } else {
            print!(".");
        }
    }
    
    println!("\n{} {} usuários prontos", "✅".bright_green(), count);
}

/// Simula uma operação de login (busca + verificação de senha)
async fn simulate_login(
    db: Arc<Mutex<TestDatabase>>,
    email: String,
) -> Result<Duration, String> {
    let start = Instant::now();
    
    let db = db.lock().await;
    
    // Busca usuário
    match db.get_user_by_email(&email) {
        Ok(Some(user)) => {
            // Simula verificação de senha (bcrypt seria aqui)
            if user.password == "hashed_SecurePass123!" {
                Ok(start.elapsed())
            } else {
                Err("Invalid password".to_string())
            }
        }
        Ok(None) => Err("User not found".to_string()),
        Err(e) => Err(e),
    }
}

/// Executa teste de carga com N operações paralelas
async fn run_load_test(
    user_count: usize,
    total_requests: usize,
    _concurrent_limit: usize,
) -> LoadTestResult {
    // Cria banco e usuários
    let db = Arc::new(Mutex::new(TestDatabase::new()));
    
    // Criar usuários antes do teste
    {
        let db_guard = db.lock().await;
        create_test_users_in_db(&db_guard, user_count);
    }
    
    let mut tasks = Vec::new();
    let start = Instant::now();

    // Executa requisições
    for i in 0..total_requests {
        let db_clone = Arc::clone(&db);
        let user_index = (i % user_count) + 1;
        let email = format!("loadtest_user{}@example.com", user_index);

        let task = tokio::spawn(async move {
            simulate_login(db_clone, email).await
        });

        tasks.push(task);
    }

    // Aguarda todos completarem
    let results: Vec<_> = futures::future::join_all(tasks).await;
    let total_duration = start.elapsed();

    // Processa resultados
    let mut latencies = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    for result in results {
        match result {
            Ok(Ok(duration)) => {
                latencies.push(duration);
                successful += 1;
            }
            _ => failed += 1,
        }
    }

    // Calcula estatísticas
    latencies.sort();
    let min_latency = latencies.first().copied().unwrap_or(Duration::ZERO);
    let max_latency = latencies.last().copied().unwrap_or(Duration::ZERO);
    let avg_latency = if !latencies.is_empty() {
        Duration::from_secs_f64(
            latencies.iter().map(|d| d.as_secs_f64()).sum::<f64>() / latencies.len() as f64,
        )
    } else {
        Duration::ZERO
    };

    let p50_latency = percentile(&latencies, 50);
    let p95_latency = percentile(&latencies, 95);
    let p99_latency = percentile(&latencies, 99);

    LoadTestResult {
        total_requests,
        successful,
        failed,
        duration: total_duration,
        min_latency,
        max_latency,
        avg_latency,
        p50_latency,
        p95_latency,
        p99_latency,
    }
}

/// Calcula percentil de latências
fn percentile(latencies: &[Duration], p: usize) -> Duration {
    if latencies.is_empty() {
        return Duration::ZERO;
    }
    let index = (latencies.len() * p / 100).min(latencies.len() - 1);
    latencies[index]
}

/// Test de carga leve: 100 requisições, 10 concorrentes
#[tokio::test]
async fn load_test_light() {
    let result = run_load_test(10, 100, 10).await;
    result.print_results("Teste de Carga Leve (100 req, 10 users, Sled)");
    
    assert!(result.success_rate() > 95.0, "Taxa de sucesso deve ser > 95%");
}

/// Test de carga moderada: 500 requisições, 50 concorrentes
#[tokio::test]
async fn load_test_moderate() {
    let result = run_load_test(20, 500, 50).await;
    result.print_results("Teste de Carga Moderada (500 req, 20 users, Sled)");
    
    assert!(result.success_rate() > 90.0, "Taxa de sucesso deve ser > 90%");
}

/// Test de carga pesada: 1000 requisições, 100 concorrentes
#[tokio::test]
async fn load_test_heavy() {
    let result = run_load_test(50, 1000, 100).await;
    result.print_results("Teste de Carga Pesada (1000 req, 50 users, Sled)");
    
    assert!(result.success_rate() > 85.0, "Taxa de sucesso deve ser > 85%");
}

/// Test de stress: 2000 requisições, 200 concorrentes
#[tokio::test]
async fn load_test_stress() {
    let result = run_load_test(100, 2000, 200).await;
    result.print_results("Teste de Stress (2000 req, 100 users, Sled)");
    
    assert!(result.success_rate() > 80.0, "Taxa de sucesso deve ser > 80%");
}

/// Test customizado - ajuste os parâmetros conforme necessário
#[tokio::test]
async fn load_test_custom() {
    // Customize aqui:
    let user_count = 50;
    let total_requests = 500;
    let concurrent_limit = 50;
    
    let result = run_load_test(user_count, total_requests, concurrent_limit).await;
    result.print_results(&format!(
        "Teste Customizado ({} req, {} users, Sled)",
        total_requests, user_count
    ));
}
