# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Instala dependências do sistema necessárias
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copia manifests
COPY Cargo.toml Cargo.lock ./

# Cria um projeto dummy para cachear dependências
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copia código fonte real
COPY src ./src
COPY public.pem ./
COPY private.pem ./

# Build da aplicação real
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Instala dependências de runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copia o binário compilado
COPY --from=builder /app/target/release/kong-security-api .
COPY --from=builder /app/public.pem .
COPY --from=builder /app/private.pem .

# Expõe a porta
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Comando para rodar a aplicação
CMD ["./kong-security-api"]
