# Testing Guide for vibe-api

## Overview

vibe-api follows a comprehensive testing strategy with:
- **Unit tests** - Test individual functions and modules
- **Integration tests** - Test complete API endpoints
- **Mock external services** - No real API calls in tests
- **In-memory database** - SQLite for isolated tests
- **90%+ code coverage** - Enforced in CI/CD

## Test Structure

```
apps/api/
├── src/
│   └── **/*.rs           # Unit tests inline with #[cfg(test)]
├── tests/
│   ├── common/           # Shared test utilities
│   │   ├── mod.rs
│   │   ├── database.rs   # Test DB setup
│   │   ├── fixtures.rs   # Test data factories
│   │   ├── mocks.rs      # Mock external services
│   │   └── app.rs        # Test app builder
│   ├── auth_integration_test.rs
│   ├── users_integration_test.rs
│   └── *_integration_test.rs
└── Cargo.toml            # Test dependencies
```

## Running Tests

### All Tests
```bash
cargo test --workspace
```

### Specific Module
```bash
cargo test --package vibe-api auth::
```

### Integration Tests Only
```bash
cargo test --test '*'
```

### With Output
```bash
cargo test -- --nocapture
```

### Multi-threaded
```bash
cargo test -- --test-threads=4
```

### Single Test
```bash
cargo test test_user_registration_success
```

## Writing Unit Tests

### Inline Tests
```rust
// src/modules/auth/jwt.rs

pub fn generate_token(user_id: &Uuid, config: &JwtConfig) -> AppResult<String> {
    // Implementation...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_generate_token_success() {
        let config = test_jwt_config();
        let user_id = Uuid::new_v4();

        let token = generate_token(&user_id, &config).unwrap();

        assert!(!token.is_empty());
    }

    #[test]
    fn test_generate_token_invalid_config() {
        // Test error handling
    }
}
```

### Guidelines
- Use `#[tokio::test(flavor = "multi_thread")]` for async tests
- Test both success and error cases
- Keep tests focused (one assertion per test ideally)
- Use descriptive test names: `test_<function>_<scenario>`
- Max 300 lines per test file

## Writing Integration Tests

### Structure
```rust
// tests/auth_integration_test.rs

mod common;

use axum::{body::Body, http::Request};
use tower::ServiceExt;

#[tokio::test(flavor = "multi_thread")]
async fn test_user_registration_success() {
    // Setup
    let db_pool = common::create_test_db().await;
    let app = common::create_test_app(db_pool).await;

    // Execute
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(json_body))
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

### Test Data
Use fixtures from `tests/common/fixtures.rs`:

```rust
use common::fixtures::{TEST_EMAIL, TEST_PASSWORD, create_test_user};

let user = create_test_user();
```

### Database Isolation
Each test gets a fresh in-memory SQLite database:

```rust
let db_pool = common::create_test_db().await;
// Automatically cleaned up after test
```

## Mocking External Services

### AI APIs (OpenAI, Anthropic)
```rust
use common::mocks::setup_mock_openai;

#[tokio::test]
async fn test_ai_chat() {
    let mock_server = setup_mock_openai().await;

    // Configure service to use mock server
    let service = AiService::new_with_url(mock_server.uri());

    let response = service.chat(request).await.unwrap();
    assert!(!response.response.is_empty());
}
```

### Environment Variables
```rust
use temp_env::with_vars;

#[test]
fn test_config_loading() {
    with_vars(
        vec![
            ("DATABASE_URL", Some("test_url")),
            ("JWT_SECRET", Some("test_secret")),
        ],
        || {
            let config = Config::load().unwrap();
            assert_eq!(config.jwt.secret, "test_secret");
        },
    );
}
```

## Code Coverage

### Install Coverage Tool
```bash
cargo install cargo-llvm-cov
```

### Generate Coverage Report
```bash
cargo llvm-cov --html
```

### View Report
```bash
# Open coverage/html/index.html in browser
```

### CI Coverage Check
```bash
cargo llvm-cov --fail-under-lines 90
```

## Test Categories

### Unit Tests (Module Level)
- **Location**: Inline in source files (`#[cfg(test)] mod tests`)
- **Purpose**: Test individual functions, structs, methods
- **Dependencies**: Minimal, use mocks
- **Speed**: Very fast

### Integration Tests (API Level)
- **Location**: `tests/` directory
- **Purpose**: Test complete request/response flows
- **Dependencies**: Full app with test database
- **Speed**: Slower but thorough

### Doc Tests
- **Location**: In documentation comments
- **Purpose**: Ensure examples in docs actually work
- **Speed**: Fast

```rust
/// Generate a JWT token
///
/// # Example
/// ```
/// use vibe_api::modules::auth::jwt::generate_token;
/// let token = generate_token(&user_id, &config)?;
/// ```
pub fn generate_token(...) { }
```

## Best Practices

### ✅ DO
- Write tests for all public APIs
- Test error handling paths
- Use descriptive test names
- Keep tests independent (no shared state)
- Use fixtures for test data
- Mock external dependencies
- Aim for 90%+ coverage

### ❌ DON'T
- Make real API calls to external services
- Use real databases (use SQLite)
- Share mutable state between tests
- Write tests over 300 lines (split them)
- Commit commented-out tests
- Use `sleep()` for timing (use mocks)

## Common Test Patterns

### Testing Async Functions
```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Testing Error Cases
```rust
#[test]
fn test_function_returns_error() {
    let result = function_that_fails();
    assert!(result.is_err());

    match result {
        Err(AppError::Validation(_)) => (),
        _ => panic!("Expected validation error"),
    }
}
```

### Testing with Database
```rust
#[tokio::test]
async fn test_database_operation() {
    let pool = create_test_db().await;

    let result = insert_user(&pool, user_data).await;
    assert!(result.is_ok());

    cleanup_test_db(&pool).await;
}
```

### Testing HTTP Endpoints
```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_endpoint() {
    let app = create_test_app(db_pool).await;

    let response = app
        .oneshot(Request::builder()
            .uri("/endpoint")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
```

## Debugging Tests

### Show Output
```bash
cargo test -- --nocapture
```

### Show Logs
```bash
RUST_LOG=debug cargo test
```

### Run Single Test
```bash
cargo test test_specific_function -- --exact
```

### Run Tests Serially
```bash
cargo test -- --test-threads=1
```

## CI/CD Integration

Tests run automatically on every commit via GitHub Actions:

```yaml
# .github/workflows/ci.yml
- name: Run tests
  run: cargo test --workspace --all-features

- name: Check coverage
  run: cargo llvm-cov --fail-under-lines 90
```

## Benchmarking (Future)

For performance testing, consider adding:
```bash
cargo install cargo-criterion
```

```rust
#[bench]
fn bench_function(b: &mut Bencher) {
    b.iter(|| {
        // Code to benchmark
    });
}
```

## Resources

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing Guide](https://tokio.rs/tokio/topics/testing)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)
- [wiremock-rs](https://github.com/LukeMathWalker/wiremock-rs)

## Test Checklist

Before committing:
- [ ] All tests pass locally
- [ ] New code has tests
- [ ] Coverage is above 90%
- [ ] No commented-out tests
- [ ] Test names are descriptive
- [ ] External services are mocked
- [ ] Tests are independent
- [ ] CI passes

---

Happy Testing! 🧪
