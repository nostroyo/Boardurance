#!/usr/bin/env pwsh

# Full Stack Startup Script - Racing Game
# Starts Docker, MongoDB, Rust Backend, and React Frontend

param(
    [switch]$SkipDocker,
    [switch]$SkipBackend,
    [switch]$SkipFrontend,
    [switch]$Verbose
)

Write-Host "🏁 Starting Racing Game Full Stack" -ForegroundColor Cyan
Write-Host "=======================================" -ForegroundColor Cyan

$ErrorActionPreference = "Stop"

# Refresh environment PATH to include recently installed tools
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

# Function to check if a port is in use
function Test-Port {
    param([int]$Port)
    try {
        $connection = New-Object System.Net.Sockets.TcpClient
        $connection.Connect("localhost", $Port)
        $connection.Close()
        return $true
    } catch {
        return $false
    }
}

# Function to wait for service to be ready
function Wait-ForService {
    param(
        [string]$ServiceName,
        [string]$Url,
        [int]$TimeoutSeconds = 60
    )
    
    Write-Host "⏳ Waiting for $ServiceName to be ready..." -ForegroundColor Yellow
    $attempt = 0
    do {
        $attempt++
        Start-Sleep -Seconds 2
        try {
            $response = Invoke-RestMethod -Uri $Url -Method GET -TimeoutSec 5
            Write-Host "✅ $ServiceName is ready!" -ForegroundColor Green
            return $true
        } catch {
            if ($Verbose) {
                Write-Host "   Attempt $attempt failed: $($_.Exception.Message)" -ForegroundColor Gray
            }
        }
        
        if ($attempt -ge ($TimeoutSeconds / 2)) {
            Write-Host "❌ $ServiceName failed to start within $TimeoutSeconds seconds" -ForegroundColor Red
            return $false
        }
    } while ($attempt -lt ($TimeoutSeconds / 2))
}

try {
    # Step 1: Start Docker and MongoDB
    if (-not $SkipDocker) {
        Write-Host "`n🐳 Step 1: Starting Docker and MongoDB..." -ForegroundColor Yellow
        
        # Check if MongoDB is already running
        if (Test-Port -Port 27017) {
            Write-Host "✅ MongoDB is already running on port 27017" -ForegroundColor Green
        } else {
            Write-Host "🔄 Starting Docker containers..." -ForegroundColor Cyan
            Set-Location "rust-backend"
            & .\Makefile.ps1 start-docker
            Set-Location ".."
            
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to start Docker containers"
            }
            
            # Wait for MongoDB to be ready
            Write-Host "⏳ Waiting for MongoDB to be ready..." -ForegroundColor Yellow
            $mongoReady = $false
            for ($i = 1; $i -le 30; $i++) {
                try {
                    # Try to connect to MongoDB
                    $testConnection = docker exec rust-backend-mongodb-1 mongosh --eval "db.runCommand('ping')" --quiet 2>$null
                    if ($testConnection -match "ok.*1") {
                        Write-Host "✅ MongoDB is ready!" -ForegroundColor Green
                        $mongoReady = $true
                        break
                    }
                } catch { }
                Start-Sleep -Seconds 2
                Write-Host "   Waiting for MongoDB... ($i/30)" -ForegroundColor Gray
            }
            
            if (-not $mongoReady) {
                throw "MongoDB failed to start properly"
            }
        }
    } else {
        Write-Host "⏭️  Skipping Docker startup" -ForegroundColor Gray
    }

    # Step 2: Start Rust Backend
    if (-not $SkipBackend) {
        Write-Host "`n🦀 Step 2: Starting Rust Backend..." -ForegroundColor Yellow
        
        # Check if backend is already running
        if (Test-Port -Port 3000) {
            Write-Host "⚠️  Port 3000 is already in use. Backend might already be running." -ForegroundColor Yellow
            Write-Host "   You can check with: curl http://localhost:3000/health_check" -ForegroundColor Gray
        } else {
            Write-Host "🔄 Building and starting Rust backend..." -ForegroundColor Cyan
            
            # Start backend in background
            Set-Location "rust-backend"
            $env:APP_ENVIRONMENT = "local"
            
            Write-Host "   Building Rust application..." -ForegroundColor Gray
            cargo build
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to build Rust backend"
            }
            
            Write-Host "   Starting Rust backend server..." -ForegroundColor Gray
            $backendProcess = Start-Process -FilePath "cargo" -ArgumentList "run --bin rust-backend" -NoNewWindow -PassThru
            Set-Location ".."
            
            # Wait for backend to be ready
            if (-not (Wait-ForService -ServiceName "Rust Backend" -Url "http://localhost:3000/health_check" -TimeoutSeconds 60)) {
                if ($backendProcess -and !$backendProcess.HasExited) {
                    Stop-Process -Id $backendProcess.Id -Force
                }
                throw "Rust backend failed to start"
            }
            
            Write-Host "📊 Backend API: http://localhost:3000" -ForegroundColor Cyan
            Write-Host "📚 API Docs: http://localhost:3000/swagger-ui" -ForegroundColor Cyan
        }
    } else {
        Write-Host "⏭️  Skipping Rust backend startup" -ForegroundColor Gray
    }

    # Step 3: Start React Frontend
    if (-not $SkipFrontend) {
        Write-Host "`n⚛️  Step 3: Starting React Frontend..." -ForegroundColor Yellow
        
        # Check if frontend is already running
        if (Test-Port -Port 5173) {
            Write-Host "⚠️  Port 5173 is already in use. Frontend might already be running." -ForegroundColor Yellow
            Write-Host "   You can check with: curl http://localhost:5173" -ForegroundColor Gray
        } else {
            Write-Host "🔄 Starting React development server..." -ForegroundColor Cyan
            
            Set-Location "empty-project"
            
            # Check if node_modules exists
            if (-not (Test-Path "node_modules")) {
                Write-Host "   Installing npm dependencies..." -ForegroundColor Gray
                npm install
                if ($LASTEXITCODE -ne 0) {
                    Set-Location ".."
                    throw "Failed to install npm dependencies"
                }
            }
            
            Write-Host "   Starting development server..." -ForegroundColor Gray
            $currentPath = Get-Location
            $frontendProcess = Start-Process -FilePath "cmd" -ArgumentList "/c", "cd /d `"$currentPath`" && npm run dev" -PassThru
            Set-Location ".."
            
            # Wait for frontend to be ready
            if (-not (Wait-ForService -ServiceName "React Frontend" -Url "http://localhost:5173" -TimeoutSeconds 60)) {
                if ($frontendProcess -and !$frontendProcess.HasExited) {
                    Stop-Process -Id $frontendProcess.Id -Force
                }
                throw "React frontend failed to start"
            }
            
            Write-Host "🌐 Frontend: http://localhost:5173" -ForegroundColor Cyan
        }
    } else {
        Write-Host "⏭️  Skipping React frontend startup" -ForegroundColor Gray
    }

    # Step 4: Run Integration Tests
    Write-Host "`n🧪 Step 4: Running Integration Tests..." -ForegroundColor Yellow
    
    Write-Host "🔄 Testing backend health..." -ForegroundColor Cyan
    try {
        $health = Invoke-RestMethod -Uri "http://localhost:3000/health_check" -Method GET -TimeoutSec 10
        Write-Host "✅ Backend health check passed" -ForegroundColor Green
    } catch {
        Write-Host "❌ Backend health check failed: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    Write-Host "🔄 Testing frontend..." -ForegroundColor Cyan
    Set-Location "empty-project"
    & .\test-frontend-auth.ps1
    Set-Location ".."

    # Success Summary
    Write-Host "`n🎉 Full Stack Startup Complete!" -ForegroundColor Green
    Write-Host "================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "🔗 Service URLs:" -ForegroundColor Cyan
    Write-Host "   🌐 Frontend:     http://localhost:5173" -ForegroundColor White
    Write-Host "   🦀 Backend API:  http://localhost:3000" -ForegroundColor White
    Write-Host "   📚 API Docs:     http://localhost:3000/swagger-ui" -ForegroundColor White
    Write-Host "   🍃 MongoDB:      mongodb://localhost:27017" -ForegroundColor White
    Write-Host ""
    Write-Host "🧪 Test the Authentication Flow:" -ForegroundColor Yellow
    Write-Host "   1. Open http://localhost:5173" -ForegroundColor White
    Write-Host "   2. Click 'Create Account' to register" -ForegroundColor White
    Write-Host "   3. Fill out the registration form" -ForegroundColor White
    Write-Host "   4. You'll be automatically logged in and redirected to the team page" -ForegroundColor White
    Write-Host "   5. Try logging out and logging back in" -ForegroundColor White
    Write-Host "   6. Test protected routes and authentication persistence" -ForegroundColor White
    Write-Host ""
    Write-Host "🛑 To stop all services:" -ForegroundColor Red
    Write-Host "   Ctrl+C to stop this script, then run: .\stop-full-stack.ps1" -ForegroundColor White
    Write-Host ""
    Write-Host "🚀 Happy testing!" -ForegroundColor Green

} catch {
    Write-Host "`n❌ Startup failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "🛑 Cleaning up..." -ForegroundColor Yellow
    
    # Cleanup on failure
    try {
        Get-Process | Where-Object {$_.ProcessName -eq "cargo" -or $_.ProcessName -eq "node"} | Stop-Process -Force -ErrorAction SilentlyContinue
    } catch { }
    
    exit 1
}