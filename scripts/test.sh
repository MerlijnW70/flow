#!/bin/bash
set -e

echo "🧪 Running tests for vibe-api..."

# Run formatting check
echo "📝 Checking code formatting..."
cargo fmt --all -- --check

# Run clippy
echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
echo "🧪 Running unit tests..."
cargo test --workspace

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --workspace --test '*'

# Run security audit
echo "🔒 Running security audit..."
cargo audit || echo "⚠️  Security audit found issues (non-blocking)"

# Run build
echo "🏗️  Building release version..."
cargo build --release

echo "✅ All tests passed!"
