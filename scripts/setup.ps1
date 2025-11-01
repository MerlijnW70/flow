# PowerShell setup script for Windows
Write-Host "🚀 Setting up vibe-api development environment..." -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust is not installed. Please install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Rust is installed: $(rustc --version)" -ForegroundColor Green

# Check if Docker is installed
if (!(Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "⚠️  Docker is not installed. You'll need it for local PostgreSQL." -ForegroundColor Yellow
} else {
    Write-Host "✅ Docker is installed: $(docker --version)" -ForegroundColor Green
}

# Install cargo tools
Write-Host "📦 Installing cargo tools..." -ForegroundColor Cyan
cargo install sqlx-cli --no-default-features --features postgres,rustls

# Copy .env.example to .env if it doesn't exist
if (!(Test-Path "apps\api\.env")) {
    Write-Host "📝 Creating .env file from .env.example..." -ForegroundColor Cyan
    Copy-Item "apps\api\.env.example" "apps\api\.env"
    Write-Host "⚠️  Please update apps\api\.env with your actual configuration!" -ForegroundColor Yellow
}

# Start PostgreSQL with Docker (optional)
$response = Read-Host "Do you want to start a local PostgreSQL instance with Docker? (y/n)"
if ($response -eq 'y' -or $response -eq 'Y') {
    Write-Host "🐘 Starting PostgreSQL container..." -ForegroundColor Cyan
    docker run -d `
        --name vibe-api-postgres `
        -e POSTGRES_USER=postgres `
        -e POSTGRES_PASSWORD=postgres `
        -e POSTGRES_DB=vibe_api `
        -p 5432:5432 `
        postgres:16-alpine

    Write-Host "Waiting for PostgreSQL to be ready..." -ForegroundColor Cyan
    Start-Sleep -Seconds 3
}

# Run migrations
$response = Read-Host "Do you want to run database migrations? (y/n)"
if ($response -eq 'y' -or $response -eq 'Y') {
    Write-Host "🗄️  Running database migrations..." -ForegroundColor Cyan
    Push-Location apps\api
    sqlx migrate run
    Pop-Location
}

Write-Host "✨ Setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. Update apps\api\.env with your configuration"
Write-Host "2. Run 'cargo run' to start the server"
Write-Host "3. Visit http://localhost:3000/health to verify"
