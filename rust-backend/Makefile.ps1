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
        Write-Host "  start-docker - Start Docker containers only (MongoDB)" -ForegroundColor White
        Write-Host "  test         - Run tests with test MongoDB" -ForegroundColor White
        Write-Host "  test-unit    - Run unit tests only (no database required)" -ForegroundColor White
        Write-Host "  test-all     - Run all test suites" -ForegroundColor White
        Write-Host "  test-api     - Test API endpoints only" -ForegroundColor White
        Write-Host "  test-infra   - Test infrastructure only" -ForegroundColor White
        Write-Host "  build        - Build the application" -ForegroundColor White
        Write-Host "  check        - Check code compilation" -ForegroundColor White
        Write-Host "  clean        - Clean build artifacts and stop containers" -ForegroundColor White
        Write-Host "  db-start     - Start development MongoDB only" -ForegroundColor White
        Write-Host "  db-stop      - Stop all MongoDB containers" -ForegroundColor White
        Write-Host "  db-logs      - Show MongoDB logs" -ForegroundColor White
    }
    
    "dev" {
        Write-Host "Starting development environment..." -ForegroundColor Green
        & .\scripts\start-mongodb.ps1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "`nStarting application..." -ForegroundColor Yellow
            $env:APP_ENVIRONMENT = "local"
            cargo run
        }
    }
    
    "dev-ui" {
        Write-Host "Starting development environment with UI..." -ForegroundColor Green
        docker-compose --profile ui up -d
        Write-Host "MongoDB and MongoDB Express started" -ForegroundColor Green
        Write-Host "MongoDB Express UI: http://localhost:8081" -ForegroundColor Cyan
        Write-Host "`nStarting application..." -ForegroundColor Yellow
        $env:APP_ENVIRONMENT = "local"
        cargo run
    }
    
    "start-docker" {
        Write-Host "Starting Docker and containers..." -ForegroundColor Green
        
        # First ensure Docker Desktop is running
        try {
            docker version | Out-Null
            Write-Host "Docker is already running" -ForegroundColor Green
        } catch {
            Write-Host "Starting Docker Desktop..." -ForegroundColor Yellow
            
            # Try multiple common Docker Desktop paths
            $dockerPaths = @(
                "C:\Program Files\Docker\Docker\Docker Desktop.exe",
                "$env:LOCALAPPDATA\Docker\Docker Desktop.exe",
                "$env:PROGRAMFILES\Docker\Docker\Docker Desktop.exe"
            )
            
            $dockerStarted = $false
            foreach ($dockerPath in $dockerPaths) {
                if (Test-Path $dockerPath) {
                    Start-Process $dockerPath -WindowStyle Hidden
                    $dockerStarted = $true
                    break
                }
            }
            
            if (-not $dockerStarted) {
                Write-Host "Docker Desktop not found. Please install Docker Desktop or start it manually." -ForegroundColor Red
                Write-Host "Download from: https://www.docker.com/products/docker-desktop" -ForegroundColor Yellow
                exit 1
            }
            
            # Wait for Docker to be ready
            $timeout = 60
            $attempt = 0
            do {
                $attempt++
                Start-Sleep -Seconds 2
                try {
                    docker version | Out-Null
                    Write-Host "Docker Desktop is now running!" -ForegroundColor Green
                    break
                } catch { }
                
                if ($attempt -eq $timeout) {
                    Write-Host "Docker Desktop failed to start within $timeout seconds" -ForegroundColor Red
                    Write-Host "Please start Docker Desktop manually and try again" -ForegroundColor Yellow
                    exit 1
                }
                
                if ($attempt % 10 -eq 0) {
                    Write-Host "Still waiting for Docker Desktop... ($attempt/$timeout)" -ForegroundColor Gray
                }
            } while ($attempt -lt $timeout)
        }
        
        # Now start MongoDB containers
        & .\scripts\start-mongodb.ps1
        if ($LASTEXITCODE -eq 0) {
            Write-Host "Docker containers started successfully!" -ForegroundColor Green
            Write-Host "MongoDB: localhost:27017" -ForegroundColor Cyan
            Write-Host "Use '.\Makefile.ps1 dev' to start the Rust application" -ForegroundColor Yellow
        } else {
            Write-Host "Failed to start Docker containers" -ForegroundColor Red
            Write-Host "Try running: docker-compose up -d" -ForegroundColor Yellow
        }
    }
    
    "test" {
        Write-Host "Running comprehensive test suite..." -ForegroundColor Green
        & .\tests\run-all-tests.ps1
    }
    
    "test-unit" {
        Write-Host "Running unit tests (no database required)..." -ForegroundColor Green
        & .\run-unit-tests.ps1
    }
    
    "test-all" {
        Write-Host "Running all test suites..." -ForegroundColor Green
        & .\tests\run-all-tests.ps1 -TestSuite "all" -Verbose
    }
    
    "test-api" {
        Write-Host "Testing API endpoints..." -ForegroundColor Green
        & .\tests\run-all-tests.ps1 -TestSuite "api" -Verbose
    }
    
    "test-infra" {
        Write-Host "Testing infrastructure..." -ForegroundColor Green
        & .\tests\run-all-tests.ps1 -TestSuite "infrastructure" -Verbose
    }
    
    "build" {
        Write-Host "Building application..." -ForegroundColor Green
        cargo build --release
    }
    
    "check" {
        Write-Host "Checking code..." -ForegroundColor Green
        cargo check
    }
    
    "clean" {
        Write-Host "Cleaning up..." -ForegroundColor Green
        cargo clean
        & .\scripts\stop-mongodb.ps1
        docker system prune -f
    }
    
    "db-start" {
        Write-Host "Starting MongoDB..." -ForegroundColor Green
        & .\scripts\start-mongodb.ps1
    }
    
    "db-stop" {
        Write-Host "Stopping MongoDB..." -ForegroundColor Yellow
        & .\scripts\stop-mongodb.ps1
    }
    
    "db-logs" {
        Write-Host "MongoDB logs:" -ForegroundColor Green
        docker-compose logs -f mongodb
    }
    
    default {
        Write-Host "Unknown command: $Command" -ForegroundColor Red
        Write-Host "Run '.\Makefile.ps1 help' for available commands" -ForegroundColor Yellow
    }
}