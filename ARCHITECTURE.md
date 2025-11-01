# vibe-api Architecture

## Overview

vibe-api is a modular, enterprise-grade REST API built with Rust following 2025 best practices. The architecture prioritizes modularity, testability, and scalability.

## Core Principles

1. **Modularity** - Each feature is an independent module
2. **Dependency Injection** - No hard coupling between modules
3. **Async-First** - 100% async I/O with Tokio
4. **Type Safety** - Compile-time guarantees wherever possible
5. **Feature Flags** - Optional modules via Cargo features
6. **12-Factor App** - Configuration via environment variables

## Technology Stack

### Core
- **Web Framework**: Axum 0.7 (Tokio-based)
- **Async Runtime**: Tokio 1.40
- **Database**: PostgreSQL 15+ with SQLx
- **Serialization**: Serde

### Security
- **Authentication**: JWT (jsonwebtoken)
- **Password Hashing**: Argon2
- **Rate Limiting**: Governor

### Optional Features
- **AI**: OpenAI SDK, Anthropic SDK
- **Storage**: AWS S3 SDK (S3-compatible)
- **WebSocket**: tokio-tungstenite
- **Background Jobs**: tokio-cron-scheduler

## Architecture Layers

```
┌─────────────────────────────────────┐
│         HTTP Layer (Axum)           │
│  ┌──────────┐  ┌──────────┐        │
│  │ Routes   │  │Middleware│         │
│  └──────────┘  └──────────┘         │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│      Service Layer (Business)       │
│  ┌──────────┐  ┌──────────┐        │
│  │ Services │  │ Traits   │         │
│  └──────────┘  └──────────┘         │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│      Data Layer (Persistence)       │
│  ┌──────────┐  ┌──────────┐        │
│  │  SQLx    │  │  Models  │         │
│  └──────────┘  └──────────┘         │
└─────────────────────────────────────┘
              ↓
┌─────────────────────────────────────┐
│         PostgreSQL Database         │
└─────────────────────────────────────┘
```

## Module Structure

Each module follows a consistent structure:

```
modules/
└── module_name/
    ├── mod.rs          # Module exports
    ├── model.rs        # Data structures (DTOs, entities)
    ├── service.rs      # Business logic
    └── routes.rs       # HTTP handlers
```

### Example: Auth Module

```rust
// mod.rs - Public interface
pub mod model;
pub mod service;
pub mod routes;
pub mod jwt;
pub mod hash;
pub mod middleware;

pub use routes::routes;
pub use middleware::AuthMiddleware;
```

```rust
// service.rs - Business logic
pub struct AuthService {
    db_pool: PgPool,
    jwt_config: JwtConfig,
}

impl AuthService {
    pub async fn register(&self, request: RegisterRequest)
        -> AppResult<AuthResponse> {
        // 1. Validate input
        // 2. Hash password
        // 3. Save to database
        // 4. Generate JWT tokens
        // 5. Return response
    }
}
```

```rust
// routes.rs - HTTP handlers
pub fn routes(db_pool: PgPool, jwt_config: JwtConfig) -> Router {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .with_state(state)
}
```

## Request Flow

1. **Request arrives** → Axum receives HTTP request
2. **Middleware** → CORS, rate limiting, logging
3. **Router** → Routes to appropriate handler
4. **Validation** → Input validation (validator crate)
5. **Service** → Business logic execution
6. **Database** → Data persistence (SQLx)
7. **Response** → Serialized JSON response

## Error Handling

Centralized error handling with custom types:

```rust
pub enum AppError {
    Database(String),
    Authentication(String),
    Validation(String),
    NotFound(String),
    // ... etc
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Convert to HTTP response with appropriate status code
    }
}
```

Benefits:
- Type-safe error handling
- Consistent error responses
- Easy error conversion
- Automatic HTTP status mapping

## Configuration Management

Environment-based configuration:

```rust
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    #[cfg(feature = "ai")]
    pub ai: AiConfig,
    // ... etc
}
```

Configuration sources (in order of precedence):
1. Environment variables
2. .env file
3. Default values

## Database Access

Type-safe SQL with SQLx:

```rust
let user = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE id = $1"
)
.bind(user_id)
.fetch_one(&pool)
.await?;
```

Benefits:
- Compile-time query verification
- Type-safe result mapping
- Automatic parameter binding
- SQL injection prevention

## Authentication Flow

```
┌──────────┐                ┌──────────┐
│  Client  │                │  Server  │
└──────────┘                └──────────┘
     │                           │
     │ POST /auth/register       │
     │──────────────────────────>│
     │                           │ 1. Validate input
     │                           │ 2. Hash password
     │                           │ 3. Create user
     │                           │ 4. Generate JWT
     │                           │
     │ {access_token, ...}       │
     │<──────────────────────────│
     │                           │
     │ GET /users/me             │
     │ Authorization: Bearer ... │
     │──────────────────────────>│
     │                           │ 1. Validate JWT
     │                           │ 2. Extract claims
     │                           │ 3. Fetch user
     │                           │
     │ {user data}               │
     │<──────────────────────────│
```

## Feature Flags

Modules can be enabled/disabled via Cargo features:

```toml
[features]
default = ["ai", "websocket", "jobs", "storage"]
ai = ["async-openai", "anthropic-sdk"]
websocket = ["tokio-tungstenite"]
jobs = ["tokio-cron-scheduler"]
storage = ["aws-sdk-s3"]
```

Benefits:
- Reduced binary size
- Faster compile times
- Pay only for what you use
- Clear dependency management

## Observability

### Structured Logging
```rust
use tracing::{info, error};

info!(user_id = %user.id, "User logged in");
error!(error = %e, "Database connection failed");
```

### Metrics
```rust
metrics::counter!("http_requests_total",
    "method" => method,
    "status" => status.to_string()
).increment(1);
```

### Health Checks
- `/health` - Liveness check
- `/ready` - Readiness check
- `/metrics` - Prometheus metrics

## Scalability Considerations

### Horizontal Scaling
- Stateless design (JWT auth)
- No server-side sessions
- Database connection pooling
- Async I/O for concurrency

### Performance
- Compile-time optimizations (LTO, codegen-units=1)
- Efficient serialization (Serde)
- Connection pooling (SQLx)
- Rate limiting (Governor)

### Caching
- Redis can be added for caching (not included by default)
- HTTP caching headers
- Database query result caching

## Security

### Input Validation
```rust
#[derive(Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8))]
    pub password: String,
}
```

### SQL Injection Prevention
- Parameterized queries via SQLx
- Compile-time query verification

### XSS Prevention
- JSON responses (no HTML)
- Proper content-type headers

### CSRF Protection
- Stateless JWT authentication
- SameSite cookie attributes (if using cookies)

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let hash = hash_password("password123").unwrap();
        assert!(verify_password("password123", &hash).unwrap());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_user_registration() {
    let app = create_test_app().await;
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

## Deployment Architecture

### Railway Deployment

```
┌─────────────────┐
│   Railway App   │
│                 │
│ ┌─────────────┐ │
│ │ vibe-api    │ │
│ │ (Rust)      │ │
│ └─────────────┘ │
│       ↓         │
│ ┌─────────────┐ │
│ │ PostgreSQL  │ │
│ └─────────────┘ │
└─────────────────┘
```

### External Services (Optional)
- OpenAI API (AI features)
- Anthropic API (AI features)
- AWS S3 / MinIO (file storage)

## Future Enhancements

Potential additions:
- [ ] GraphQL support (async-graphql)
- [ ] gRPC support (tonic)
- [ ] Redis caching
- [ ] Message queue (RabbitMQ, Kafka)
- [ ] Event sourcing
- [ ] CQRS pattern
- [ ] Distributed tracing (OpenTelemetry)

## References

- [Axum Documentation](https://docs.rs/axum)
- [SQLx Documentation](https://docs.rs/sqlx)
- [Tokio Documentation](https://tokio.rs)
- [Railway Documentation](https://docs.railway.app)
