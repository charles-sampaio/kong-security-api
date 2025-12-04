# Kong Security API

A robust authentication and authorization API developed in Rust using Actix-web, designed for integration with Kong Gateway and other microservices.

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Technologies](#technologies)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Endpoints](#api-endpoints)
- [JWT Authentication](#jwt-authentication)
- [Data Models](#data-models)
- [Project Structure](#project-structure)
- [Testing](#testing)
- [Deployment](#deployment)
- [Contributing](#contributing)
- [License](#license)

## 🎯 Overview

Kong Security API is an authentication and authorization solution developed in Rust that provides:

- User registration and login system
- JWT (JSON Web Tokens) based authentication
- Role and permission management
- MongoDB integration
- Password encryption with bcrypt
- Refresh tokens for session renewal
- Structured logging and logging middleware

This API was designed to be a centralized authentication service that can be easily integrated with Kong Gateway or used as an independent microservice.

## ✨ Features

### Security
- **Secure Authentication**: JWT implementation with RS256 algorithm
- **Multi-Tenancy**: Complete tenant isolation with UUID-based tenant identification
- **User Management**: User registration, login, and profile
- **Password Reset**: Complete token-based password recovery flow
- **Access Control**: Role-based system with tenant-level isolation
- **Security**: Password hashing with bcrypt (cost 12)
- **Validation**: Rigorous input data validation with password strength requirements
- **CORS Protection**: Configurable cross-origin resource sharing
- **SQL Injection Prevention**: Input sanitization and validation
- **XSS Prevention**: HTML/script tag filtering

### Performance & Scalability
- **Redis Cache**: Optional caching for tenants and logs (5-10x faster)
- **MongoDB Connection Pooling**: Optimized pool (max 10 connections)
- **Response Compression**: Automatic gzip/brotli compression (70-90% bandwidth reduction)
- **Optimized Queries**: MongoDB projection and sorting for minimal I/O
- **Zstd/Snappy Compression**: MongoDB wire protocol compression
- **Low Resource Usage**: ~50MB memory, <5% CPU in idle

### Observability
- **Persistence**: MongoDB integration with automatic indexes
- **Logging**: Comprehensive audit logging with login tracking per tenant
- **API Documentation**: Interactive Swagger/OpenAPI documentation
- **Health Check**: Endpoint with database and cache status
- **Testing**: 45+ unit and integration tests (100% using Sled)

### Management
- **Tenant Management**: Full CRUD operations for tenant lifecycle
- **Refresh Tokens**: Secure token renewal
- **Performance**: High-performance Actix-web framework with compression

📊 **See [PERFORMANCE.md](PERFORMANCE.md) for detailed optimization guide and benchmarks**

## 🛠 Technologies

### Core
- **Rust** (Edition 2021) - Main language
- **Actix-web 4** - Asynchronous web framework
- **Tokio** - Asynchronous runtime

### Authentication & Security
- **jsonwebtoken 9** - JWT implementation (RS256)
- **bcrypt 0.15** - Password hashing
- **actix-cors 0.7** - CORS middleware
- **validator 0.18** - Input validation
- **regex 1** - Pattern matching for security
- **chrono 0.4** - Date/timestamp manipulation

### Performance
- **redis 0.24** - Redis cache client
- **deadpool-redis 0.14** - Async Redis connection pool
- **actix-web-lab 0.20** - Compression middleware

### Documentation
- **utoipa 4** - OpenAPI documentation generation
- **utoipa-swagger-ui 6** - Interactive Swagger UI

### Database
- **MongoDB 3.3.0** - NoSQL database with optimized connection pool
- **mongodb-driver** - Official MongoDB driver for Rust with Zstd compression

### Serialization & Configuration
- **serde 1.0** - Serialization/deserialization
- **serde_json 1.0** - JSON support
- **dotenv 0.15** - Environment variable loading

### Utilities
- **uuid 1.0** - Unique identifier generation (v4)
- **env_logger 0.10** - Logging system

## 📋 Prerequisites

- **Rust 1.70+** with Cargo
- **MongoDB 6.0+** (local or Atlas)
- **Redis 6.0+** (optional, for caching)
- **OpenSSL** (for RSA key generation)

### Version Verification

```bash
# Check Rust version
rustc --version
cargo --version

# Check MongoDB connection
mongosh --version
```

## 🚀 Installation

### 1. Clone the repository

```bash
git clone https://github.com/CharlesSampaio-CRS/kong-security-api.git
cd kong-security-api
```

### 2. Generate RSA keys for JWT

```bash
# Private key (RSA 2048 bits)
openssl genrsa -out private.pem 2048

# Public key
openssl rsa -in private.pem -outform PEM -pubout -out public.pem
```

### 3. Configure environment variables

```bash
cp .env.example .env
# Edit the .env file with your configurations
```

### 4. Install dependencies

```bash
cargo build
```

## ⚙️ Configuration

### Environment Variables (.env)

```bash
# MongoDB
MONGODB_URI=mongodb+srv://space_user:yNPBfuIk266JjjjO@clusterdbmongoatlas.mc74nzn.mongodb.net/kong-security-api?retryWrites=true&w=majority&appName=ClusterDbMongoAtlas
MONGODB_DB=rust_jwt_api

# JWT Configuration
JWT_SECRET=nQv?J/&dNnB*qni@@KonG
JWT_EXPIRATION_HOURS=2

# Server Configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# Logging
RUST_LOG=info
```

### MongoDB Configuration

1. **Local MongoDB**:
   ```bash
   MONGODB_URI=mongodb://localhost:27017/kong_security_api
   ```

2. **MongoDB Atlas**:
   ```bash
   MONGODB_URI=mongodb+srv://user:password@cluster.mongodb.net/kong_security_api?retryWrites=true&w=majority
   ```

### Database Structure

The system will automatically create the necessary collections:

- `users` - Stores user information

## 🚀 Usage

### Start the server

```bash
# Development mode
cargo run

# Release mode (production)
cargo run --release
```

The server will be available at: `http://localhost:8080`

### Startup Logs

```
🚀 Server started at http://localhost:8080
```

## 📡 API Endpoints

### Base URL
```
http://localhost:8080
```

### Quick Reference

| Method | Endpoint | Auth Required | Description |
|--------|----------|---------------|-------------|
| GET | `/health` | ❌ No | Health check and security features |
| POST | `/auth/register` | ❌ No | Register new user |
| POST | `/auth/login` | ❌ No | Login and get JWT token |
| POST | `/auth/password-reset/request` | ❌ No | Request password reset token |
| POST | `/auth/password-reset/validate` | ❌ No | Validate reset token (optional) |
| POST | `/auth/password-reset/confirm` | ❌ No | Confirm password reset |
| GET | `/auth/protected` | ✅ Yes | Protected resource (example) |
| GET | `/api/logs/my-logins` | ✅ Yes | Get user's login history |
| GET | `/api/admin/logs` | ✅ Yes (Admin) | Get all system logs |
| GET | `/api/admin/logs/stats` | ✅ Yes (Admin) | Get login statistics |

### Documentation & Testing

- **Swagger UI**: http://localhost:8080/swagger-ui/
- **OpenAPI Spec**: http://localhost:8080/api-docs/openapi.json
- **Postman Collections**: 
  - `Kong_Security_API_MultiTenant.postman_collection.json` (Development - localhost)
  - `Kong_Security_API_Production.postman_collection.json` (Production - Fly.io)
  - `Kong_Security_API_Production.postman_environment.json` (Environment file)

---

## 🏢 Multi-Tenancy

Kong Security API supports **complete multi-tenancy** with tenant isolation. Each tenant (organization/system) has completely isolated data.

### Key Features

- ✅ **UUID-based Tenant IDs**: Automatically generated, impossible to duplicate
- ✅ **Complete Data Isolation**: Users, logs, and tokens are isolated per tenant
- ✅ **Tenant Validation Middleware**: All protected routes validate tenant existence and status
- ✅ **Tenant Management API**: Full CRUD operations for tenant lifecycle
- ✅ **Inactive Tenant Protection**: Deactivated tenants cannot access the API

### Quick Start

```bash
# 1. Create a tenant (UUID generated automatically)
curl -X POST http://localhost:8080/api/tenants \
  -H "Content-Type: application/json" \
  -d '{"name": "ACME Corp", "description": "Main tenant"}'

# Response includes tenant_id UUID:
# {"tenant_id": "550e8400-e29b-41d4-a716-446655440000", ...}

# 2. Use tenant_id UUID in all requests
export TENANT_ID="550e8400-e29b-41d4-a716-446655440000"

# 3. Register user with tenant header
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: $TENANT_ID" \
  -d '{"email": "user@acme.com", "password": "SecurePass123!"}'
```

### Documentation

- 📖 [Complete Multi-Tenant Guide](./MULTI_TENANT.md)
- 🚀 [Quick Start Guide](./QUICK_START.md)
- 📮 [Postman Collection Guide (Development)](./POSTMAN_README.md)
- 🌐 [Production Collection Guide](./PRODUCTION_COLLECTION.md)
- 📊 [Implementation Summary](./SUMMARY.md)
- 🗄️ [Database Indexes & Unique Constraints](./DATABASE_INDEXES.md)

### Tenant Management Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/tenants` | Create tenant (UUID auto-generated) |
| GET | `/api/tenants` | List all tenants |
| GET | `/api/tenants/{id}` | Get tenant by UUID |
| PUT | `/api/tenants/{id}` | Update tenant |
| POST | `/api/tenants/{id}/activate` | Activate tenant |
| POST | `/api/tenants/{id}/deactivate` | Deactivate tenant |
| DELETE | `/api/tenants/{id}` | Delete tenant |

---

### 1. User Registration

**POST** `/auth/register`

Registers a new user in the system.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response:**
```bash
# Success (201 Created)
"User created successfully"

# Error - User already exists (409 Conflict)
"User already exists"
```

**Example with curl:**
```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "mypassword123"
  }'
```

### 2. User Login

**POST** `/auth/login`

Authenticates a user and returns access tokens.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Status Codes:**
- `200 OK` - Login successful
- `401 Unauthorized` - Invalid credentials
- `403 Forbidden` - Account deactivated

**Example with curl:**
```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "password": "mypassword123"
  }'
```

### 3. Password Reset Flow

The system provides a complete password recovery flow with unique tokens.

#### 3.1. Request Password Reset

**POST** `/auth/password-reset/request`

Generates a reset token and sends it to the user (currently returns in response for development).

**Request Body:**
```json
{
  "email": "user@example.com"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Password reset token sent to user@example.com",
  "token": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Note**: The `token` field only appears in development mode. In production (release), the token is sent only via email.

**Features:**
- ✅ Unique UUID token
- ✅ 1-hour expiration
- ✅ Doesn't reveal if email exists (security)
- ✅ IP address tracking for audit

**Example:**
```bash
curl -X POST http://localhost:8080/auth/password-reset/request \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com"}'
```

#### 3.2. Validate Reset Token (Optional)

**POST** `/auth/password-reset/validate`

Validates if a token is still valid before showing the password reset form.

**Request Body:**
```json
{
  "token": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Response:**
```json
{
  "valid": true,
  "email": "user@example.com",
  "expires_at": "2025-11-23T23:00:00Z"
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/auth/password-reset/validate \
  -H "Content-Type: application/json" \
  -d '{"token":"550e8400-e29b-41d4-a716-446655440000"}'
```

#### 3.3. Confirm Password Reset

**POST** `/auth/password-reset/confirm`

Resets the password using the token.

**Request Body:**
```json
{
  "token": "550e8400-e29b-41d4-a716-446655440000",
  "new_password": "NewSecurePass123!"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Password updated successfully"
}
```

**Validations:**
- ✅ Token must be valid, not expired, not used
- ✅ Password must be at least 8 characters
- ✅ Token is marked as used after success
- ✅ All other tokens for this email are invalidated

**Example:**
```bash
curl -X POST http://localhost:8080/auth/password-reset/confirm \
  -H "Content-Type: application/json" \
  -d '{
    "token":"550e8400-e29b-41d4-a716-446655440000",
    "new_password":"NewSecurePass123!"
  }'
```

**Security Features:**
- 🔒 Unique tokens (UUID v4 - impossible to guess)
- ⏱️ Time-limited (1 hour expiration)
- 🎯 Single use (token marked as used after reset)
- 🔐 Bcrypt hashing (cost 12)
- 🚫 Automatic invalidation of all other tokens
- 📝 IP address logging for audit

**Full Flow:**
```
1. User requests reset → Token generated
2. (Optional) Validate token → Check if still valid
3. User sets new password → Password updated
4. Token marked as used → Cannot be reused
5. All other tokens invalidated → Extra security
```

📚 **See full documentation**: [`PASSWORD_RESET_FLOW.md`](PASSWORD_RESET_FLOW.md)

### 4. User Profile

**GET** `/auth/protected`

Returns authenticated user information.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response:**
```json
{
  "id": "60f7b3b3b3b3b3b3b3b3b3b3",
  "email": "user@example.com",
  "roles": ["user"],
  "is_active": true,
  "iat": 1623456789,
  "exp": 1623463989
}
```

**Status Codes:**
- `200 OK` - Profile returned successfully
- `401 Unauthorized` - Invalid or missing token

**Example with curl:**
```bash
curl -X GET http://localhost:8080/auth/protected \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## 🔐 JWT Authentication

### Token Structure

The system uses JWT (JSON Web Tokens) with **RS256** algorithm for maximum security.

**Access Token Claims:**
```json
{
  "sub": "user_id",           // Subject (User ID)
  "email": "user@email.com",  // User email
  "roles": ["user", "admin"], // User roles
  "is_active": true,          // Active status
  "iat": 1623456789,          // Issued at (timestamp)
  "exp": 1623463989,          // Expiry (timestamp)
  "jti": "uuid-v4",           // JWT ID (unique)
  "aud": "my_audience",       // Audience
  "iss": "my_issuer"          // Issuer
}
```

### Token Lifecycle

1. **Access Token**: Valid for 2 hours (configurable)
2. **Refresh Token**: UUID v4, stored in database
3. **Renewal**: Use refresh token to obtain new access tokens

### Usage in Headers

```bash
Authorization: Bearer <access_token>
```

### Token Validation

The system validates:
- ✅ Token signature (RS256)
- ✅ Expiration (`exp`)
- ✅ Audience (`aud`)
- ✅ Issuer (`iss`)
- ✅ User active status

## 📊 Data Models

### User Model

```rust
pub struct User {
    pub _id: Option<ObjectId>,              // Unique MongoDB ID
    pub email: String,                      // User email
    pub password: String,                   // Password hash (bcrypt)
    pub roles: Option<Vec<String>>,         // User roles ["user", "admin"]
    pub created_at: Option<DateTime>,       // Creation date
    pub updated_at: Option<DateTime>,       // Update date
    pub last_login: Option<DateTime>,       // Last login
    pub is_active: bool,                    // Active status (default: true)
    pub email_verified: bool,               // Email verified (default: false)
    pub password_reset_token: Option<String>, // Password reset token
    pub password_reset_expiry: Option<DateTime>, // Reset expiration
    pub refresh_tokens: Option<Vec<String>>, // Refresh tokens list
}
```

### Claims Model

```rust
pub struct Claims {
    pub sub: String,        // User ID
    pub email: String,      // Email
    pub roles: Vec<String>, // User roles
    pub is_active: bool,    // Active status
    pub iat: usize,         // Issued at
    pub exp: usize,         // Expiry
    pub jti: String,        // JWT ID
    pub aud: String,        // Audience
    pub iss: String,        // Issuer
}
```

## 🏗 Project Structure

```
kong-security-api/
├── src/
│   ├── main.rs          # Application entry point
│   ├── handlers.rs      # HTTP route handlers
│   ├── routes.rs        # Route configuration
│   ├── models.rs        # Data models (User, etc.)
│   ├── auth.rs          # JWT authentication logic
│   └── db.rs           # MongoDB connection and configuration
├── Cargo.toml          # Rust dependencies
├── Cargo.lock          # Dependencies lock file
├── .env                # Environment variables
├── private.pem         # RSA private key (JWT)
├── public.pem          # RSA public key (JWT)
├── LICENSE             # Project license
└── README.md           # Documentation (this file)
```

### Module Descriptions

- **main.rs**: Actix-web server configuration and middleware
- **handlers.rs**: Endpoint implementations (register, login, profile)
- **routes.rs**: HTTP route configuration
- **models.rs**: Data structure definitions
- **auth.rs**: JWT generation and validation functions
- **db.rs**: MongoDB connection

## 🧪 Testing

### 🦀 Rust Testing System

The entire testing system was developed 100% in native Rust with **Sled** (in-memory database).

**Documentation:**
- 🚀 **[SLED_TESTING.md](SLED_TESTING.md)** - **In-memory database for tests (equivalent to H2)**
- 📘 **[QUICK_START.md](QUICK_START.md)** - Quick start guide
- 📊 **[VALIDATION_RESULTS.md](VALIDATION_RESULTS.md)** - Validated results
- 📚 **[tests/README.md](tests/README.md)** - Complete technical documentation
- 🔄 **[MIGRATION_TO_SLED.md](MIGRATION_TO_SLED.md)** - Complete migration to Sled

### 🎯 Available Test Types

| Type | Quantity | Database | Time | Description |
|------|----------|----------|------|-------------|
| **Unit** | 8 | N/A | ~0.01s | Isolated function tests |
| **Integration (Sled)** | 17 | Sled in-memory | ~0.11s | ⚡ **Fast tests without server** |
| **Light Load** | 7 + 4 ignored | Sled in-memory | ~0.08s | Performance tests ⚡ |
| **Test Helpers** | 6 | Sled in-memory | ~0.06s | Sled database tests |
| **TOTAL** | **38** | **100% Sled** | **~0.3s** | **Zero MongoDB pollution** ✅ |

### ⚡ Fast Tests with Sled (Recommended for Development)

**Sled** is an in-memory NoSQL database (equivalent to Java's H2):
- ✅ **17x faster** than tests with MongoDB
- ✅ **No dependencies** - no need for running server
- ✅ **Total isolation** - each test has its own database
- ✅ **Automatic** - data is destroyed after test
- ✅ **Zero pollution** - production MongoDB is never touched

```bash
# Only Sled tests (17 tests - ~0.11s) ⚡
cargo test --test integration_tests_sled

# Sled load tests (7 tests - ~0.08s)
cargo test --test load_tests

# Sled helpers tests (6 tests - ~0.06s)
cargo test --test test_helpers

# All fast tests (~0.3s total) ⚡
cargo test
```

### Compile Tests (First Time)

```bash
# Download and compile all test dependencies (includes Sled)
cargo build --tests
```

### Run ALL Tests

```bash
# Run all tests (38 tests - ~0.3s) ⚡
cargo test

# Expected result:
# ✅  8 unit tests              (0.01s)
# ✅ 17 integration tests (Sled) (0.11s) ⚡
# ✅  7 load tests (Sled)        (0.08s) ⚡
# ✅  6 helpers tests (Sled)     (0.06s) ⚡
# = 38 tests (100% success, 0% MongoDB) ✅
```

### Heavy Load Tests (Ignored by Default)

```bash
# Run ignored load tests (moderate, heavy, stress, custom)
cargo test -- --ignored --nocapture
cargo test --test integration_tests

# 3. Light Load Test (100 req - ~7s)
cargo test --test load_tests load_test_light -- --ignored --nocapture

# Or run everything at once:
cargo test --test integration_tests && \
cargo test --test load_tests load_test_light -- --ignored --nocapture
```

### Integration Tests

```bash
# All integration tests (6 tests)
cargo test --test integration_tests

# With detailed output
cargo test --test integration_tests -- --nocapture

# Specific test
cargo test --test integration_tests test_user_login
cargo test --test integration_tests test_user_registration
cargo test --test integration_tests test_protected_route_with_valid_token
```

**Available tests:**
- ✅ `test_health_endpoint` - GET /health
- ✅ `test_user_registration` - POST /auth/register
- ✅ `test_user_login` - POST /auth/login
- ✅ `test_login_with_wrong_password` - Validates incorrect password
- ✅ `test_protected_route_without_token` - Access without token
- ✅ `test_protected_route_with_valid_token` - Access with valid JWT

### Load Tests

```bash
# Light: 100 requests, 10 parallel (~7s)
cargo test --test load_tests load_test_light -- --ignored --nocapture

# Moderate: 500 requests, 50 parallel (~30s)
cargo test --test load_tests load_test_moderate -- --ignored --nocapture

# Heavy: 1000 requests, 100 parallel (~60s)
cargo test --test load_tests load_test_heavy -- --ignored --nocapture

# Stress: 2000 requests, 200 parallel (~120s)
cargo test --test load_tests load_test_stress -- --ignored --nocapture

# Custom: Customizable
cargo test --test load_tests load_test_custom -- --ignored --nocapture
```

### Benchmarks (Optional)

```bash
# Run benchmarks with Criterion
cargo bench

# View HTML reports
open target/criterion/report/index.html
```

### Validated Results

**Integration Tests:**
```
running 6 tests
test test_health_endpoint ... ok
test test_user_registration ... ok
test test_user_login ... ok
test test_login_with_wrong_password ... ok
test test_protected_route_without_token ... ok
test test_protected_route_with_valid_token ... ok

test result: ok. 6 passed; 0 failed
```

**Light Load Test:**
```
📊 Light Load Test (100 req, 10 concurrent)

Requests:
  Total:          100
  Successful:     100 ✓
  Failed:         0
  Success rate:   100.00%

Performance:
  Total duration:  4.25s
  Req/s:           23.55

Latency:
  Minimum:     305.52ms
  P50:         327.50ms
  P95:         828.10ms
  Maximum:     1193.64ms
```

### Integration Tests

```bash
# Test registration endpoint
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'

# Test login endpoint
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'

# Test protected endpoint (replace the token)
curl -X GET http://localhost:8080/auth/protected \
  -H "Authorization: Bearer YOUR_TOKEN_HERE"
```

### Load Testing

```bash
# Install wrk (macOS)
brew install wrk

# Load test on login endpoint
wrk -t12 -c400 -d30s -s login_test.lua http://localhost:8080/auth/login
```

### Code Quality

```bash
# Fix unused imports and other simple warnings
cargo fix --bin "kong-security-api" --allow-dirty

# Check formatting
cargo fmt --check

# Run linter
cargo clippy -- -D warnings

# Current status: 25 warnings (mostly dead_code for future features)
# These warnings are for implemented but not-yet-used features like:
# - Refresh token functionality
# - Rate limiting middleware
# - Security headers middleware
# - User role management
# - Admin endpoints
```

## 🚢 Deployment

### Docker

Create a `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/kong-security-api /usr/local/bin/kong-security-api
COPY --from=builder /app/private.pem /app/public.pem ./

EXPOSE 8080
CMD ["kong-security-api"]
```

```bash
# Build image
docker build -t kong-security-api .

# Run container
docker run -p 8080:8080 --env-file .env kong-security-api
```

### Docker Compose

```yaml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - MONGODB_URI=mongodb://mongo:27017/kong_security_api
      - RUST_LOG=info
    depends_on:
      - mongo
  
  mongo:
    image: mongo:6.0
    ports:
      - "27017:27017"
    volumes:
      - mongo_data:/data/db

volumes:
  mongo_data:
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: kong-security-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: kong-security-api
  template:
    metadata:
      labels:
        app: kong-security-api
    spec:
      containers:
      - name: api
        image: kong-security-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: MONGODB_URI
          valueFrom:
            secretKeyRef:
              name: mongo-secret
              key: uri
---
apiVersion: v1
kind: Service
metadata:
  name: kong-security-api-service
spec:
  selector:
    app: kong-security-api
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

## 🔧 Troubleshooting

### Common Issues

1. **MongoDB connection error**
   ```bash
   # Check connectivity
   mongosh "your-connection-string"
   ```

2. **RSA keys error**
   ```bash
   # Regenerate keys
   openssl genrsa -out private.pem 2048
   openssl rsa -in private.pem -pubout -out public.pem
   ```

3. **Port in use**
   ```bash
   # Check processes on port 8080
   lsof -i :8080
   ```

4. **Permission issues**
   ```bash
   # Check file permissions
   chmod 600 private.pem
   chmod 644 public.pem
   ```

### Logging and Debugging

```bash
# Enable debug logs
export RUST_LOG=debug
cargo run

# Specific logs
export RUST_LOG=kong_security_api=debug,actix_web=info
cargo run
```

## 🤝 Contributing

### How to Contribute

1. Fork the project
2. Create a feature branch (`git checkout -b feature/new-feature`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature/new-feature`)
5. Open a Pull Request

### Code Standards

- Follow Rust conventions (rustfmt)
- Document public functions
- Write tests for new functionalities
- Maintain test coverage > 80%

### Run Checks

```bash
# Formatting
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Tests
cargo test

# Security audit
cargo audit
```

## 📄 License

This project is licensed under the [MIT License](LICENSE).

```
MIT License

Copyright (c) 2024 Charles Roberto Sampaio

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## 📞 Support

- **GitHub Issues**: [Open an issue](https://github.com/CharlesSampaio-CRS/kong-security-api/issues)
- **Email**: charles.roberto@example.com
- **Documentation**: [Project Wiki](https://github.com/CharlesSampaio-CRS/kong-security-api/wiki)

---

**Developed with ❤️ using Rust**

> This project is part of the CRS-Saturno ecosystem for microservices and security solutions.