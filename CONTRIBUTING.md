# Contributing to vibe-api

Thank you for your interest in contributing to vibe-api! This document provides guidelines and instructions for contributing.

## Development Setup

1. **Install Rust** (1.82+):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone and setup**:
   ```bash
   git clone <your-fork>
   cd vibe-api
   ./scripts/setup.sh
   ```

3. **Install development tools**:
   ```bash
   rustup component add rustfmt clippy
   cargo install cargo-audit cargo-watch
   ```

## Development Workflow

### Running Locally

```bash
# Watch mode (auto-reload on changes)
cargo watch -x run

# Standard run
cargo run

# Run with specific features
cargo run --no-default-features --features ai,websocket
```

### Testing

```bash
# Run all tests
./scripts/test.sh

# Run specific module tests
cargo test --package vibe-api auth::

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Code Quality

Before committing, ensure:

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Check security
cargo audit
```

## Code Guidelines

### File Organization

- **Max 500 lines per file** - split into submodules if larger
- Each module should have:
  - `mod.rs` - Module declaration
  - `model.rs` - Data structures
  - `service.rs` - Business logic
  - `routes.rs` - HTTP handlers

### Naming Conventions

- **Files**: `snake_case.rs`
- **Modules**: `snake_case`
- **Types**: `PascalCase`
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`

### Error Handling

Always use `AppResult<T>` and `AppError`:

```rust
use crate::utils::error::{AppError, AppResult};

pub async fn my_function() -> AppResult<MyType> {
    let result = some_operation()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(result)
}
```

### Async/Await

All I/O operations must be async:

```rust
// Good
pub async fn fetch_user(id: Uuid) -> AppResult<User> {
    sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(Into::into)
}

// Bad
pub fn fetch_user(id: Uuid) -> AppResult<User> {
    // Blocking I/O
}
```

### Documentation

Document public APIs with doc comments:

```rust
/// Authenticates a user and returns JWT tokens
///
/// # Arguments
/// * `request` - Login credentials (email and password)
///
/// # Returns
/// * `Ok(AuthResponse)` - Authentication tokens and user info
/// * `Err(AppError)` - Authentication failure
///
/// # Example
/// ```
/// let response = service.login(request).await?;
/// println!("Token: {}", response.access_token);
/// ```
pub async fn login(&self, request: LoginRequest) -> AppResult<AuthResponse> {
    // ...
}
```

## Module Development

### Creating a New Module

1. Create module directory:
   ```bash
   mkdir -p apps/api/src/modules/my_module
   ```

2. Create required files:
   ```rust
   // mod.rs
   pub mod model;
   pub mod service;
   pub mod routes;

   pub use routes::routes;
   ```

3. Add to `modules/mod.rs`:
   ```rust
   pub mod my_module;
   ```

4. Register routes in `main.rs`:
   ```rust
   app = app.merge(my_module::routes());
   ```

### Testing Modules

Create integration tests in `tests/`:

```rust
// tests/my_module_test.rs
use vibe_api::modules::my_module::*;

#[tokio::test]
async fn test_my_feature() {
    // Setup
    let service = MyService::new(/* ... */);

    // Execute
    let result = service.my_function().await;

    // Assert
    assert!(result.is_ok());
}
```

## Database Migrations

### Creating Migrations

```bash
cd apps/api
sqlx migrate add create_my_table
```

Edit the generated file in `migrations/`:

```sql
-- migrations/YYYYMMDD_create_my_table.sql
CREATE TABLE my_table (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_my_table_name ON my_table(name);
```

Run migrations:
```bash
./scripts/migrate.sh
```

## Pull Request Process

1. **Create a branch**:
   ```bash
   git checkout -b feature/my-awesome-feature
   ```

2. **Make changes** following the guidelines above

3. **Test thoroughly**:
   ```bash
   ./scripts/test.sh
   ```

4. **Commit with descriptive messages**:
   ```bash
   git commit -m "feat: add user profile photo upload"
   ```

   Use conventional commits:
   - `feat:` - New feature
   - `fix:` - Bug fix
   - `docs:` - Documentation only
   - `style:` - Code style changes
   - `refactor:` - Code refactoring
   - `test:` - Adding tests
   - `chore:` - Maintenance tasks

5. **Push and create PR**:
   ```bash
   git push origin feature/my-awesome-feature
   ```

6. **PR Checklist**:
   - [ ] Tests pass locally
   - [ ] Code is formatted (`cargo fmt`)
   - [ ] No clippy warnings
   - [ ] Documentation updated
   - [ ] Changelog updated (if applicable)
   - [ ] CI passes

## Security

- Never commit secrets or API keys
- Use environment variables for sensitive data
- Follow OWASP security guidelines
- Report security issues privately to maintainers

## Questions?

- Open an issue for questions
- Join our Discord (if applicable)
- Check existing issues and PRs

Thank you for contributing! 🦀
