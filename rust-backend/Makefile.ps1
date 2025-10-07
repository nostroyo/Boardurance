# Makefile-style commands for Rust Backend
param(
    [Parameter(Mandatory=$true)]
    [string]$Command
)

switch ($Command.ToLower()) {
    "help" {
        Write-Host "Available commands:" -ForegroundColor Green
        Write-Host "  dev          - Start development environment with MongoDB" -ForegroundColor White
        Write-Host "  dev-ui       - Start development with MongoDB Express UI" -ForegroundColor White
        Write-Host "  test         - Run tests with test MongoDB" -ForegroundColor White
        Write-Host "  build        - Build the application" -ForegroundColor White
        Write-Host "  check        - Check code compilation" -ForegroundColor White
        Write-Host "  clean        - Clean build artifacts and stop containers" -ForegroundColor White
        Write-Host "  db-start     - Start development MongoDB only" -ForegroundColor White
        Write-Host "  db-stop      - Stop all MongoDB containers" -ForegroundColor White
        Write-Host "  db-logs      - Show MongoDB logs" -ForegroundColor White
    }
    
    "dev" {
        Write-Host "🚀 Starting development environment..." -ForegroundColor Green
        & .\scripts\start-mongodb.ps1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "`nStarting application..." -ForegroundColor Yellow
            $env:APP_ENVIRONMENT = "local"
            cargo run
        }
    }
    
    "dev-ui" {
        Write-Host "🚀 Starting development environment with UI..." -ForegroundColor Green
        docker-compose --profile ui up -d
        Write-Host "✅ MongoDB and MongoDB Express started" -ForegroundColor Green
        Write-Host "📊 MongoDB Express UI: http://localhost:8081" -ForegroundColor Cyan
        Write-Host "`nStarting application..." -ForegroundColor Yellow
        $env:APP_ENVIRONMENT = "local"
        cargo run
    }
    
    "test" {
        Write-Host "🧪 Running tests..." -ForegroundColor Green
        & .\scripts\test-with-mongodb.ps1
    }
    
    "build" {
        Write-Host "🔨 Building application..." -ForegroundColor Green
        cargo build --release
    }
    
    "check" {
        Write-Host "🔍 Checking code..." -ForegroundColor Green
        cargo check
    }
    
    "clean" {
        Write-Host "🧹 Cleaning up..." -ForegroundColor Green
        cargo clean
        & .\scripts\stop-mongodb.ps1
        docker system prune -f
    }
    
    "db-start" {
        Write-Host "🗄️ Starting MongoDB..." -ForegroundColor Green
        & .\scripts\start-mongodb.ps1
    }
    
    "db-stop" {
        Write-Host "🛑 Stopping MongoDB..." -ForegroundColor Yellow
        & .\scripts\stop-mongodb.ps1
    }
    
    "db-logs" {
        Write-Host "📋 MongoDB logs:" -ForegroundColor Green
        docker-compose logs -f mongodb
    }
    
    default {
        Write-Host "❌ Unknown command: $Command" -ForegroundColor Red
        Write-Host "Run './Makefile.ps1 help' for available commands" -ForegroundColor Yellow
    }
}