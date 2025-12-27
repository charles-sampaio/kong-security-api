# 🚀 Como Rodar o Kong Security API

## 📋 Pré-requisitos

### Obrigatórios
- **Rust** 1.70+ ([instalar](https://rustup.rs/))
- **MongoDB** (local ou Atlas)
- **Redis** (local ou Redis Labs)

### Opcional
- **Docker** e **Docker Compose** (para rodar tudo em containers)
- **cargo-watch** (para desenvolvimento com hot reload)

---

## 🏠 Rodando Localmente

### 1️⃣ Instalação do Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verificar instalação
rustc --version
cargo --version
```

### 2️⃣ Clonar o Repositório

```bash
git clone https://github.com/charles-sampaio/kong-security-api.git
cd kong-security-api

# Usar a branch principal (Actix-web puro)
git checkout structure-folder

# OU usar a branch Shuttle (para deploy no Shuttle.rs)
git checkout shuttle-deployment
```

### 3️⃣ Configurar Variáveis de Ambiente

Crie um arquivo `.env` na raiz do projeto:

```bash
# .env
MONGODB_URI=mongodb://localhost:27017/kong-security-api
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=your-super-secret-key-change-this-in-production
PORT=8080
ALLOWED_ORIGINS=http://localhost:3000,http://localhost:8080
RUST_LOG=info
```

### 4️⃣ Subir MongoDB e Redis com Docker

**Opção A: Docker Compose (Recomendado)**

Crie um `docker-compose.yml`:

```yaml
version: '3.8'

services:
  mongodb:
    image: mongo:7
    container_name: kong-mongodb
    ports:
      - "27017:27017"
    environment:
      MONGO_INITDB_DATABASE: kong-security-api
    volumes:
      - mongodb_data:/data/db
    networks:
      - kong-network

  redis:
    image: redis:7-alpine
    container_name: kong-redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    networks:
      - kong-network

volumes:
  mongodb_data:
  redis_data:

networks:
  kong-network:
    driver: bridge
```

Subir os serviços:

```bash
docker-compose up -d
```

**Opção B: Usando serviços na nuvem**

MongoDB Atlas:
```env
MONGODB_URI=mongodb+srv://usuario:senha@cluster.mongodb.net/kong-security-api
```

Redis Labs:
```env
REDIS_URL=redis://default:senha@redis-xxxxx.cloud.redislabs.com:xxxxx
```

### 5️⃣ Instalar Dependências e Compilar

```bash
# Compilar (modo debug - mais rápido)
cargo build

# OU compilar otimizado (mais lento, mas performance de produção)
cargo build --release
```

### 6️⃣ Rodar o Servidor

**Modo Debug:**
```bash
cargo run
```

**Modo Release (otimizado):**
```bash
cargo run --release
```

**Com Hot Reload (desenvolvimento):**
```bash
# Instalar cargo-watch
cargo install cargo-watch

# Rodar com hot reload
cargo watch -x run
```

### 7️⃣ Verificar se Está Funcionando

Abra o navegador em:

- **Health Check:** http://localhost:8080/health
- **Swagger UI:** http://localhost:8080/swagger-ui/
- **API Docs JSON:** http://localhost:8080/api-docs/openapi.json

Ou via curl:

```bash
# Health check
curl http://localhost:8080/health

# Criar tenant
curl -X POST http://localhost:8080/api/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Company",
    "document": "12345678000190",
    "description": "Test tenant"
  }'

# Registrar usuário (use o tenant_id retornado acima)
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: seu-tenant-id-aqui" \
  -d '{
    "email": "user@example.com",
    "password": "Password123!",
    "name": "Test User"
  }'
```

---

## 🐳 Rodando com Docker

### Build da Imagem

```bash
# Criar Dockerfile
cat > Dockerfile << 'EOF'
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/kong-security-api .
COPY --from=builder /app/public.pem .
COPY --from=builder /app/private.pem .

EXPOSE 8080

CMD ["./kong-security-api"]
EOF

# Build
docker build -t kong-security-api .

# Run
docker run -d \
  -p 8080:8080 \
  -e MONGODB_URI=mongodb://host.docker.internal:27017/kong-security-api \
  -e REDIS_URL=redis://host.docker.internal:6379 \
  -e JWT_SECRET=your-secret-key \
  --name kong-api \
  kong-security-api
```

### Docker Compose Completo

```yaml
version: '3.8'

services:
  app:
    build: .
    container_name: kong-api
    ports:
      - "8080:8080"
    environment:
      MONGODB_URI: mongodb://mongodb:27017/kong-security-api
      REDIS_URL: redis://redis:6379
      JWT_SECRET: your-secret-key-change-in-production
      PORT: 8080
      RUST_LOG: info
    depends_on:
      - mongodb
      - redis
    networks:
      - kong-network
    restart: unless-stopped

  mongodb:
    image: mongo:7
    container_name: kong-mongodb
    ports:
      - "27017:27017"
    volumes:
      - mongodb_data:/data/db
    networks:
      - kong-network
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    container_name: kong-redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    networks:
      - kong-network
    restart: unless-stopped

volumes:
  mongodb_data:
  redis_data:

networks:
  kong-network:
    driver: bridge
```

Rodar tudo:

```bash
docker-compose up -d

# Ver logs
docker-compose logs -f app

# Parar
docker-compose down
```

---

## ☁️ Deploy em Produção

### Opção 1: Shuttle.rs (Mais Fácil)

```bash
# Mudar para branch shuttle
git checkout shuttle-deployment

# Instalar CLI do Shuttle
cargo install cargo-shuttle

# Login
cargo shuttle login

# Configurar secrets
cargo shuttle project start

# Adicionar secrets (um por vez)
cargo shuttle secret set MONGODB_URI="mongodb+srv://..."
cargo shuttle secret set REDIS_URL="redis://..."
cargo shuttle secret set JWT_SECRET="your-secret"

# Deploy!
cargo shuttle deploy

# Ver logs
cargo shuttle logs
```

### Opção 2: Fly.io

```bash
# Instalar flyctl
curl -L https://fly.io/install.sh | sh

# Login
flyctl auth login

# Criar app
flyctl launch

# Configurar secrets
flyctl secrets set MONGODB_URI="mongodb+srv://..."
flyctl secrets set REDIS_URL="redis://..."
flyctl secrets set JWT_SECRET="your-secret"

# Deploy
flyctl deploy

# Ver logs
flyctl logs
```

### Opção 3: Railway

```bash
# Instalar Railway CLI
npm i -g @railway/cli

# Login
railway login

# Criar projeto
railway init

# Adicionar variáveis
railway variables set MONGODB_URI="mongodb+srv://..."
railway variables set REDIS_URL="redis://..."
railway variables set JWT_SECRET="your-secret"

# Deploy
railway up
```

### Opção 4: Render.com

1. Conecte seu repositório GitHub no Render.com
2. Crie um novo Web Service
3. Configure as variáveis de ambiente
4. Deploy automático a cada push

### Opção 5: VPS (Digital Ocean, AWS, etc)

```bash
# SSH no servidor
ssh user@your-server.com

# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clonar repo
git clone https://github.com/charles-sampaio/kong-security-api.git
cd kong-security-api

# Configurar .env
nano .env

# Compilar release
cargo build --release

# Rodar com systemd
sudo nano /etc/systemd/system/kong-api.service
```

Arquivo `kong-api.service`:

```ini
[Unit]
Description=Kong Security API
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/home/youruser/kong-security-api
EnvironmentFile=/home/youruser/kong-security-api/.env
ExecStart=/home/youruser/kong-security-api/target/release/kong-security-api
Restart=always

[Install]
WantedBy=multi-user.target
```

Ativar o serviço:

```bash
sudo systemctl daemon-reload
sudo systemctl enable kong-api
sudo systemctl start kong-api
sudo systemctl status kong-api
```

---

## 🧪 Testes

### Rodar Todos os Testes

```bash
cargo test
```

### Rodar Testes Específicos

```bash
# Testes de integração
cargo test --test integration_tests_sled

# Testes com output
cargo test -- --nocapture

# Testes em paralelo
cargo test -- --test-threads=4
```

### Load Testing

```bash
# Rodar benchmark
cargo bench
```

---

## 🔧 Desenvolvimento

### Estrutura do Projeto

```
kong-security-api/
├── src/
│   ├── main.rs              # Entry point
│   ├── api/                 # HTTP handlers
│   ├── auth/                # JWT e autenticação
│   ├── cache/               # Redis cache
│   ├── database/            # MongoDB
│   ├── middleware/          # Rate limit, validação
│   ├── models/              # Data models
│   ├── services/            # Business logic
│   └── utils/               # Utilities
├── tests/                   # Integration tests
├── benches/                 # Benchmarks
├── Cargo.toml              # Dependencies
└── .env                    # Environment variables
```

### Comandos Úteis

```bash
# Verificar código (mais rápido que build)
cargo check

# Formatar código
cargo fmt

# Lint
cargo clippy

# Ver dependências desatualizadas
cargo outdated

# Atualizar dependências
cargo update

# Limpar build artifacts
cargo clean

# Ver tamanho do binário
ls -lh target/release/kong-security-api

# Profile de compilação
cargo build --release --timings
```

### Hot Reload com cargo-watch

```bash
# Instalar
cargo install cargo-watch

# Rodar com reload automático
cargo watch -x run

# Rodar testes automaticamente
cargo watch -x test

# Limpar + compilar + rodar
cargo watch -x "run --release"
```

### Debugar

```bash
# Com logs detalhados
RUST_LOG=debug cargo run

# Com backtrace
RUST_BACKTRACE=1 cargo run

# Com backtrace completo
RUST_BACKTRACE=full cargo run
```

---

## 📊 Monitoramento

### Logs

```bash
# Em produção, use o nível info
RUST_LOG=info cargo run

# Ver logs específicos
RUST_LOG=kong_security_api=debug cargo run

# Ver apenas erros
RUST_LOG=error cargo run
```

### Health Check

```bash
# Check básico
curl http://localhost:8080/health

# Check com detalhes (se implementado)
curl http://localhost:8080/health/detailed
```

### Métricas

O servidor expõe informações sobre:
- Conexões Redis (pool status)
- Conexões MongoDB
- Rate limit status
- Cache hit/miss ratio

---

## 🔐 Segurança

### Gerar Chaves JWT RS256

```bash
# Gerar chave privada
openssl genrsa -out private.pem 2048

# Extrair chave pública
openssl rsa -in private.pem -pubout -out public.pem

# Verificar chaves
openssl rsa -in private.pem -check
openssl rsa -in public.pem -pubin -text
```

### Variáveis Sensíveis

**NUNCA** commite:
- `.env` com secrets
- `private.pem`
- Credenciais de banco

Use `.gitignore`:
```
.env
private.pem
*.pem
target/
```

---

## 🐛 Troubleshooting

### Erro: "connection refused" no MongoDB

```bash
# Verificar se MongoDB está rodando
docker ps | grep mongo

# Ou localmente
sudo systemctl status mongodb
```

### Erro: "connection refused" no Redis

```bash
# Verificar Redis
docker ps | grep redis

# Ou localmente
redis-cli ping
# Deve retornar: PONG
```

### Erro de compilação

```bash
# Limpar cache e recompilar
cargo clean
cargo build
```

### Porta 8080 em uso

```bash
# Ver o que está usando a porta
lsof -i :8080

# Ou usar outra porta
PORT=3000 cargo run
```

### Erro: "JWT secret not found"

Certifique-se de ter o `JWT_SECRET` no `.env`:
```bash
echo "JWT_SECRET=your-secret-key" >> .env
```

---

## 📚 Recursos Adicionais

- **Documentação da API:** `/swagger-ui/`
- **Postman Collection:** `Kong_Security_API_Production.postman_collection.json`
- **Guia de Endpoints:** `API_ENDPOINTS.md`
- **Exemplos Frontend:** `FRONTEND_INTEGRATION.md`
- **Guia de Deploy:** `DEPLOY_*.md`

---

## 🤝 Suporte

Se tiver problemas:

1. Verifique os logs: `RUST_LOG=debug cargo run`
2. Teste o health check: `curl http://localhost:8080/health`
3. Verifique se MongoDB e Redis estão acessíveis
4. Consulte a documentação específica do deploy

---

## 📝 Notas

- **Performance:** Modo release é ~10x mais rápido que debug
- **Memória:** O servidor usa ~50-100MB em idle
- **CPU:** Otimizado para multi-core
- **Cache:** Redis opcional mas recomendado para produção
- **Banco:** MongoDB 4.4+ requerido

**Pronto para produção!** 🚀
