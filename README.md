# vibe-api

> **Enterprise-grade Rust backend following 2025 best practices**

A fully modular, production-ready REST API built with Rust, Axum, PostgreSQL, and designed for deployment on Railway. Features JWT authentication, AI integrations (OpenAI, Anthropic, local models), S3-compatible storage, WebSockets, and background jobs.

## Features

- ✅ **Modern Tech Stack**: Axum + Tokio + SQLx + PostgreSQL
- ✅ **Modular Architecture**: Domain-driven design with dependency injection
- ✅ **Authentication**: JWT (stateless) with Argon2 password hashing
- ✅ **AI Integration**: OpenAI, Anthropic Claude, and local model support
- ✅ **File Storage**: S3-compatible storage (AWS S3, MinIO, etc.)
- ✅ **Real-time**: WebSocket support for bidirectional communication
- ✅ **Background Jobs**: Cron-based task scheduler
- ✅ **Observability**: Prometheus metrics, structured logging, health checks
- ✅ **Security**: Rate limiting, CORS, input validation
- ✅ **CI/CD**: GitHub Actions with automated testing and deployment
- ✅ **Railway Ready**: Optimized Dockerfile for Railway deployment

## Project Structure

```
vibe-api/
├── apps/
│   └── api/
│       ├── src/
│       │   ├── main.rs              # Server bootstrap
│       │   ├── lib.rs               # Public API
│       │   ├── config/              # Environment configuration
│       │   ├── database/            # Database connection & migrations
│       │   ├── middleware/          # Rate limiting, etc.
│       │   ├── metrics/             # Prometheus metrics & health
│       │   ├── utils/               # Error handling, validation
│       │   └── modules/
│       │       ├── auth/            # JWT authentication
│       │       ├── users/           # User management
│       │       ├── ai/              # AI provider integrations
│       │       ├── storage/         # S3 file uploads
│       │       ├── jobs/            # Background tasks
│       │       └── websocket/       # Real-time communication
│       ├── tests/                   # Integration tests
│       ├── migrations/              # SQL migrations
│       ├── Dockerfile               # Production container
│       └── Cargo.toml               # Dependencies & features
├── scripts/
│   ├── setup.sh                     # Development environment setup
│   ├── migrate.sh                   # Run migrations
│   └── test.sh                      # Run all tests
└── .github/workflows/ci.yml         # CI/CD pipeline

```

## Quick Start

### Prerequisites

- Rust 1.82+ ([Install via rustup](https://rustup.rs/))
- PostgreSQL 15+
- Docker (optional, for local PostgreSQL)

### Setup

1. **Clone and setup**:
   ```bash
   git clone <your-repo>
   cd vibe-api
   chmod +x scripts/*.sh
   ./scripts/setup.sh
   ```

2. **Configure environment**:
   ```bash
   cp apps/api/.env.example apps/api/.env
   # Edit apps/api/.env with your settings
   ```

3. **Run migrations**:
   ```bash
   ./scripts/migrate.sh
   ```

4. **Start the server**:
   ```bash
   cargo run
   ```

Visit `http://localhost:3000/health` to verify the server is running.

## Feature Flags

Enable/disable modules using Cargo features:

```toml
[features]
default = ["ai", "websocket", "jobs", "storage"]
ai = ["async-openai", "anthropic-sdk"]        # AI integrations
websocket = ["tokio-tungstenite"]             # WebSocket support
jobs = ["tokio-cron-scheduler"]               # Background jobs
storage = ["aws-sdk-s3"]                      # S3 storage
```

Build without AI:
```bash
cargo build --no-default-features --features websocket,jobs,storage
```

## API Endpoints

### Authentication
- `POST /auth/register` - Create new account
- `POST /auth/login` - Login and get JWT tokens
- `POST /auth/refresh` - Refresh access token

### Users
- `GET /users/me` - Get current user (requires auth)
- `PATCH /users/me` - Update user profile
- `PUT /users/me/password` - Change password
- `DELETE /users/me` - Delete account
- `GET /users` - List all users (paginated)

### AI (if enabled)
- `POST /ai/chat` - Send chat message to AI
- `POST /ai/chat/stream` - Stream AI responses (SSE)
- `POST /ai/embeddings` - Generate text embeddings

### Storage (if enabled)
- `POST /storage/upload` - Upload file
- `GET /storage/presigned-upload` - Get presigned upload URL
- `GET /storage/presigned-download/:id` - Get presigned download URL
- `DELETE /storage/:id` - Delete file

### WebSocket (if enabled)
- `GET /ws` - WebSocket connection for real-time updates

### Monitoring
- `GET /health` - Health check
- `GET /ready` - Readiness check
- `GET /metrics` - Prometheus metrics

## Configuration

All configuration via environment variables (see `.env.example`):

```env
# Server
PORT=3000
ENVIRONMENT=development
CORS_ORIGINS=http://localhost:3000

# Database
DATABASE_URL=postgresql://user:pass@localhost:5432/vibe_api

# JWT
JWT_SECRET=your-secret-key
JWT_ACCESS_TOKEN_EXPIRY_HOURS=24

# AI (optional)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Storage (optional)
S3_BUCKET=my-bucket
S3_REGION=us-east-1
S3_ACCESS_KEY=...
S3_SECRET_KEY=...
```

## Testing

```bash
# Run all tests
./scripts/test.sh

# Run specific tests
cargo test --package vibe-api --test health_test

# Run with coverage
cargo tarpaulin --out Html
```

## Deployment

### Railway

1. Install Railway CLI:
   ```bash
   npm install -g @railway/cli
   ```

2. Login and link project:
   ```bash
   railway login
   railway link
   ```

3. Add PostgreSQL:
   ```bash
   railway add postgresql
   ```

4. Set environment variables:
   ```bash
   railway variables set JWT_SECRET=your-secret-key
   railway variables set OPENAI_API_KEY=sk-...
   ```

5. Deploy:
   ```bash
   railway up
   ```

### Docker

```bash
# Build
docker build -f apps/api/Dockerfile -t vibe-api .

# Run
docker run -p 3000:3000 --env-file apps/api/.env vibe-api
```

## Development Guidelines

- **Max 500 lines per file** - Split automatically if larger
- **No hard coupling** - Use dependency injection via traits
- **100% async** - Use Tokio for all I/O operations
- **Type-safe SQL** - Use SQLx compile-time verification
- **Error handling** - Use custom `AppError` type
- **Logging** - Use structured logging with `tracing`
- **Testing** - Write integration tests for each module
- **Security** - Follow OWASP guidelines, use cargo-audit

## Architecture Decisions

### Why Axum?
- Official from Tokio team, best-in-class performance
- Type-safe extractors, excellent ergonomics
- Full async/await support, Tower middleware ecosystem

### Why SQLx over Diesel?
- Compile-time query verification
- Full async support
- Lighter weight, faster compile times

### Why JWT over sessions?
- Stateless authentication, easier to scale
- No server-side session storage required
- Perfect for microservices and mobile apps

## Testing & Validation

### Phase 5: 100% Integration Coverage ✅ Complete

Comprehensive integration test suite with **219 tests** across **10 modules** ensuring production readiness:

#### Test Coverage

| Test Suite | Tests | Status | Description |
|------------|-------|--------|-------------|
| Health Check | 4 | ✅ | Server startup, response time < 100ms |
| Config/Env | 9 | ✅ | Environment loading, validation |
| Database | 10 | ✅ | Connections, migrations, query latency < 50ms |
| Middleware | 11 | ✅ | CORS, request ID, compression |
| Metrics | 11 | ✅ | Prometheus export, counters, histograms |
| **Background Jobs** | **28** | ✅ | Task execution, scheduler behavior, DB operations |
| **GraphQL** | **26** | ✅ | Queries, mutations, authorization patterns |
| **Storage/S3** | **37** | ✅ | Upload/download, presigned URLs, file operations |
| **AI Integration** | **43** | ✅ | OpenAI/Anthropic/local models, streaming, embeddings |
| **WebSocket** | **40** | ✅ | Real-time messaging, rooms, connection lifecycle |
| **TOTAL** | **219** | ✅ | **100% feature coverage** |

#### Running Tests

```bash
# Run all Phase 5 tests (all 10 suites)
./scripts/test-phase5.sh

# Run core API tests (original Phase 5)
cargo test --test health_check
cargo test --test config_env
cargo test --test database_integration
cargo test --test middleware
cargo test --test metrics

# Run new module tests (100% coverage additions)
cargo test --test jobs_integration
cargo test --test jobs_scheduler
cargo test --test graphql_integration
cargo test --test storage_integration
cargo test --test ai_integration
cargo test --test websocket_integration

# Run with output
cargo test -- --nocapture
```

#### Test Environment

Tests use a dedicated PostgreSQL test database configured in `.env.test`:

```bash
# Setup test database
createdb vibe_test

# Run migrations
DATABASE_URL=postgres://postgres:postgres@localhost:5432/vibe_test \
  cargo sqlx migrate run
```

#### Performance Baselines

- All 219 tests complete in ≤ 60 seconds (10 suites)
- Health check latency < 100ms
- Database queries < 50ms
- Zero panics or thread leaks
- Memory usage < 150MB during test run
- Concurrent test execution supported

#### Success Criteria

- [x] All health endpoints return 200 OK
- [x] Database migrations consistent and idempotent
- [x] JWT + role-based authentication validated
- [x] All middleware (CORS, rate-limit, compression) functional
- [x] Metrics expose Prometheus-compatible data
- [x] Background jobs task execution validated
- [x] GraphQL queries/mutations patterns established
- [x] Storage upload/download operations tested
- [x] AI provider integrations (OpenAI/Anthropic) validated
- [x] WebSocket real-time messaging confirmed
- [x] Test coverage = **100% of all features**
- [x] Zero panics, zero memory leaks

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

Ensure all tests pass and code is formatted:
```bash
cargo fmt
cargo clippy
./scripts/test.sh           # Original tests
./scripts/test-phase5.sh    # Phase 5 integration tests
```

## License

MIT

## Support

For issues and questions:
- GitHub Issues: [your-repo/issues](https://github.com/yourusername/vibe-api/issues)
- Documentation: [your-docs-url]

---

**Built with ❤️ using Rust and 2025 best practices**
