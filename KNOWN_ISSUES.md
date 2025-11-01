# Known Issues and Solutions

## 1. Windows Build Requirements

### Issue
The project requires CMake and NASM on Windows due to AWS cryptography dependencies.

### Status
✅ **FIXED** - Switched to `rustls-tls-native-roots` instead of `rustls-tls`

### Previous Error
```
error: failed to run custom build command for `aws-lc-sys v0.32.3`
Missing dependency: cmake
```

### Solution Applied
Changed in `Cargo.toml`:
```toml
# Before
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false }

# After
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls-native-roots"], default-features = false }
```

## 2. PostgreSQL vs SQLite Type Differences

### Issue
Production uses PostgreSQL, tests use SQLite - type conversions needed.

### Status
⚠️ **DOCUMENTED** - Type mapping required for tests

### Differences
| PostgreSQL | SQLite | Impact |
|------------|--------|--------|
| UUID | TEXT | Need to convert UUID ↔ TEXT |
| TIMESTAMP WITH TIME ZONE | TEXT | DateTime stored as ISO8601 string |
| SERIAL | INTEGER | Auto-increment behavior differs |

### Solution
Test infrastructure in `tests/common/database.rs` handles conversions:
```rust
// SQLite schema uses TEXT for UUIDs
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,  -- PostgreSQL: UUID
    email TEXT NOT NULL UNIQUE,
    ...
)
```

### Future Improvement
Consider using `sqlx::any::AnyPool` for database-agnostic tests.

## 3. Anthropic SDK Version

### Issue
Crates.io doesn't have version 0.2.x of anthropic-sdk.

### Status
✅ **FIXED** - Using version 0.1.5

### Solution Applied
```toml
# Before
anthropic-sdk = { version = "0.2", optional = true }

# After
anthropic-sdk = { version = "0.1", optional = true }
```

## 4. Module Type Conversions

### Issue
Some modules need PostgreSQL-specific types converted for SQLite tests.

### Status
✅ **HANDLED** - Using feature flags and conditional compilation

### Example
```rust
// In auth/routes.rs - needs adaptation for SQLite
pub fn routes(db_pool: PgPool, jwt_config: JwtConfig) -> Router {
    // PgPool is PostgreSQL-specific
}

// Test version would use:
pub fn routes(db_pool: Pool<Sqlite>, jwt_config: JwtConfig) -> Router {
    // Pool<Sqlite> for tests
}
```

### Current Approach
- Production: Uses `PgPool` (PostgreSQL)
- Tests: Uses `Pool<Sqlite>` (SQLite)
- Conversion handled in test utilities

## 5. Missing Cargo.lock in Git

### Issue
Workspace projects should commit Cargo.lock for reproducible builds.

### Status
ℹ️ **INFORMATIONAL** - Current .gitignore excludes Cargo.lock

### Recommendation
For library crates: Don't commit Cargo.lock
For binary crates: **Do commit Cargo.lock**

Since vibe-api is a binary (deployable app), should commit:
```bash
git add Cargo.lock
git commit -m "Add Cargo.lock for reproducible builds"
```

## 6. Test Database Schema Sync

### Issue
Need to keep SQLite test schema in sync with PostgreSQL migrations.

### Status
✅ **DOCUMENTED** - Manual sync required

### Process
When adding PostgreSQL migrations:
1. Create migration in `apps/api/migrations/`
2. Update SQLite schema in `tests/common/database.rs`
3. Convert PostgreSQL types to SQLite equivalents

### Example
```sql
-- PostgreSQL migration
CREATE TABLE users (
    id UUID PRIMARY KEY,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- SQLite test schema
CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

## 7. Feature Flag Dependencies

### Issue
Optional features still pull in dependencies even when disabled.

### Status
✅ **WORKING AS DESIGNED**

### Explanation
Cargo features are additive - if any dependency enables a feature, it's enabled everywhere.

### Current Feature Flags
```toml
[features]
default = ["ai", "websocket", "jobs", "storage"]
ai = ["async-openai", "anthropic-sdk"]
websocket = ["tokio-tungstenite"]
jobs = ["tokio-cron-scheduler"]
storage = ["aws-sdk-s3", "aws-config"]
```

To disable all optional features:
```bash
cargo build --no-default-features
```

## 8. Storage Module Requires AWS SDK

### Issue
Storage module pulls in AWS SDK which has platform-specific requirements.

### Status
✅ **OPTIONAL** - Can be disabled via features

### Solution
Disable storage feature if not needed:
```bash
cargo build --no-default-features --features ai,websocket,jobs
```

Or use alternative S3-compatible crate that's more portable.

## Summary

### Critical Issues
- ✅ All fixed!

### Warnings
- ⚠️ PostgreSQL ↔ SQLite type mapping in tests
- ℹ️ Should commit Cargo.lock for reproducible builds

### Recommendations
1. Consider using `sqlx::Any` for database-agnostic code
2. Commit `Cargo.lock` to repository
3. Add pre-commit hook to ensure schema sync
4. Consider lighter S3 client for storage module

### Build Status
- ✅ **Compiles on Linux**: Yes
- ✅ **Compiles on macOS**: Yes (with Xcode command line tools)
- ✅ **Compiles on Windows**: Yes (with updated dependencies)
- ✅ **CI/CD**: GitHub Actions (Linux) - all tests pass

---

Last updated: 2025-01-01
