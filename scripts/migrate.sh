#!/bin/bash
set -e

echo "🗄️  Running database migrations..."

# Change to API directory
cd apps/api

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "❌ sqlx-cli is not installed."
    echo "Install with: cargo install sqlx-cli --no-default-features --features postgres,rustls"
    exit 1
fi

# Load .env file
if [ -f ".env" ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL is not set. Please configure it in .env"
    exit 1
fi

echo "📊 Database URL: ${DATABASE_URL%%@*}@***"

# Run migrations
sqlx migrate run

echo "✅ Migrations completed successfully!"
