#!/bin/bash
set -e

echo "🚀 Setting up vibe-api development environment..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust is installed: $(rustc --version)"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "⚠️  Docker is not installed. You'll need it for local PostgreSQL."
else
    echo "✅ Docker is installed: $(docker --version)"
fi

# Install cargo tools
echo "📦 Installing cargo tools..."
cargo install sqlx-cli --no-default-features --features postgres,rustls || true

# Copy .env.example to .env if it doesn't exist
if [ ! -f "apps/api/.env" ]; then
    echo "📝 Creating .env file from .env.example..."
    cp apps/api/.env.example apps/api/.env
    echo "⚠️  Please update apps/api/.env with your actual configuration!"
fi

# Start PostgreSQL with Docker (optional)
read -p "Do you want to start a local PostgreSQL instance with Docker? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🐘 Starting PostgreSQL container..."
    docker run -d \
        --name vibe-api-postgres \
        -e POSTGRES_USER=postgres \
        -e POSTGRES_PASSWORD=postgres \
        -e POSTGRES_DB=vibe_api \
        -p 5432:5432 \
        postgres:16-alpine || echo "Container may already exist"

    echo "Waiting for PostgreSQL to be ready..."
    sleep 3
fi

# Run migrations
read -p "Do you want to run database migrations? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🗄️  Running database migrations..."
    cd apps/api && sqlx migrate run || echo "Migration failed - make sure DATABASE_URL is correct"
    cd ../..
fi

echo "✨ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Update apps/api/.env with your configuration"
echo "2. Run 'cargo run' to start the server"
echo "3. Visit http://localhost:3000/health to verify"
