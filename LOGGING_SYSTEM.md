# Kong Security API - Login Logging System

## 📁 Estrutura Modularizada

A funcionalidade de logging foi desmembrada em módulos organizados para melhor manutenibilidade:

```
src/
├── main.rs              # Ponto de entrada da aplicação
├── handlers.rs          # Handlers principais (register, login, profile)
├── log_handlers.rs      # Handlers específicos para logs
├── routes.rs            # Configuração de rotas
├── models.rs            # Modelos básicos (User)
├── logs.rs              # Modelos e serviços de logging
├── auth.rs              # Autenticação JWT
├── db.rs                # Conexão com MongoDB
└── utils/
    ├── mod.rs           # Módulo utils
    └── user_agent_parser.rs  # Parser de User-Agent
```

## 🔧 Módulos Criados

### 1. `logs.rs` - Sistema de Logging Completo

**Modelos:**
- `LoginLog` - Estrutura completa de log de login
- `LoginStats` - Estatísticas de login

**Serviços:**
- `LogService::save_login_log()` - Salva log de tentativa de login
- `LogService::get_login_logs_by_email()` - Busca logs por email
- `LogService::get_login_logs_by_date_range()` - Busca logs por período
- `LogService::get_failed_login_attempts()` - Busca tentativas falhadas
- `LogService::get_login_stats()` - Estatísticas de login

### 2. `log_handlers.rs` - Endpoints de Log

**Endpoints:**
- `GET /logs/my-logins` - Logs do usuário atual
- `GET /admin/logs` - Logs administrativos (com filtros)
- `GET /admin/logs/stats` - Estatísticas de login

### 3. `utils/user_agent_parser.rs` - Parser de User-Agent

**Funcionalidades:**
- Detecção de dispositivo (Mobile, Tablet, Desktop)
- Identificação de navegador (Chrome, Firefox, Safari, etc.)
- Identificação de sistema operacional (Windows, macOS, Linux, etc.)
- Testes unitários incluídos

## 📊 Dados Coletados no Log

Cada tentativa de login registra:

```rust
pub struct LoginLog {
    pub _id: Option<ObjectId>,
    
    // Informações do usuário
    pub user_id: Option<String>,
    pub email: String,
    
    // Detalhes da tentativa
    pub success: bool,
    pub failure_reason: Option<String>,
    
    // Informações da requisição
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_method: String,
    pub request_path: String,
    
    // Timestamps
    pub timestamp: DateTime,
    pub login_date: String,      // YYYY-MM-DD
    pub login_time: String,      // HH:MM:SS
    
    // Informações de segurança
    pub token_generated: bool,
    pub refresh_token_generated: bool,
    
    // Metadados adicionais
    pub session_id: Option<String>,
    pub device_type: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}
```

## 🛡️ Casos de Uso de Segurança

### 1. Monitoramento de Tentativas Suspeitas
```bash
GET /admin/logs?failed_only=true&hours=24
```

### 2. Auditoria por Usuário
```bash
GET /admin/logs?email=usuario@exemplo.com&limit=100
```

### 3. Análise por Período
```bash
GET /admin/logs?start_date=2024-01-01&end_date=2024-01-31
```

### 4. Estatísticas de Login
```bash
GET /admin/logs/stats?days=30
```

### 5. Logs Pessoais do Usuário
```bash
GET /logs/my-logins
Authorization: Bearer <token>
```

## 🔐 Controle de Acesso

- **Usuários comuns**: Podem acessar apenas seus próprios logs (`/logs/my-logins`)
- **Administradores**: Acesso completo a todos os logs e estatísticas (`/admin/logs/*`)

## 📈 Estatísticas Disponíveis

```json
{
  "total_attempts": 1250,
  "successful_logins": 1100,
  "failed_logins": 150,
  "success_rate": 88.0,
  "period_days": 30
}
```

## 🧪 Testes

O módulo `user_agent_parser` inclui testes unitários:

```bash
cargo test
```

Testes cobrem:
- ✅ Detecção de Chrome Desktop
- ✅ Detecção de iPhone Safari
- ✅ Detecção de Android Chrome
- ✅ Tratamento de User-Agent nulo

## 🚀 Próximos Passos

1. **Geolocalização**: Integrar com API de geolocalização para preencher `country` e `city`
2. **Alertas**: Sistema de alertas para tentativas suspeitas
3. **Dashboard**: Interface web para visualização dos logs
4. **Exportação**: Funcionalidade para exportar logs em diferentes formatos
5. **Retenção**: Política de retenção automática de logs antigos

## 📝 Exemplos de Uso

### Registrar Usuário
```bash
curl -X POST http://localhost:8080/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
```

### Fazer Login (com logging automático)
```bash
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -H "User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X)" \
  -d '{"email":"test@example.com","password":"password123"}'
```

### Verificar Seus Logs
```bash
curl -X GET http://localhost:8080/logs/my-logins \
  -H "Authorization: Bearer <seu_token>"
```

---

**✅ Sistema de Logging Implementado com Sucesso!**

O sistema agora coleta automaticamente todos os dados de acesso e os salva na collection `Logs` do MongoDB, proporcionando visibilidade completa sobre todas as tentativas de login na aplicação.