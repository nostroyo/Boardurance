# Start MongoDB for development
Write-Host "🚀 Starting MongoDB for development..." -ForegroundColor Green

# Check if Docker is running, start it if not
try {
    docker version | Out-Null
    Write-Host "✅ Docker is already running" -ForegroundColor Green
} catch {
    Write-Host "🔄 Docker is not running. Starting Docker Desktop..." -ForegroundColor Yellow
    
    # Try to start Docker Desktop
    try {
        Start-Process "C:\Program Files\Docker\Docker\Docker Desktop.exe" -WindowStyle Hidden
        Write-Host "⏳ Waiting for Docker Desktop to start..." -ForegroundColor Yellow
        
        # Wait for Docker to be ready (up to 60 seconds)
        $dockerTimeout = 60
        $dockerAttempt = 0
        
        do {
            $dockerAttempt++
            Start-Sleep -Seconds 2
            
            try {
                docker version | Out-Null
                Write-Host "✅ Docker Desktop is now running!" -ForegroundColor Green
                break
            } catch {
                # Continue waiting
            }
            
            if ($dockerAttempt -eq $dockerTimeout) {
                Write-Host "❌ Docker Desktop failed to start within timeout" -ForegroundColor Red
                Write-Host "Please start Docker Desktop manually and try again." -ForegroundColor Yellow
                exit 1
            }
            
            if ($dockerAttempt % 5 -eq 0) {
                Write-Host "  Still waiting for Docker Desktop... ($dockerAttempt/$dockerTimeout)" -ForegroundColor Gray
            }
        } while ($dockerAttempt -lt $dockerTimeout)
        
    } catch {
        Write-Host "❌ Failed to start Docker Desktop automatically." -ForegroundColor Red
        Write-Host "Please start Docker Desktop manually and try again." -ForegroundColor Yellow
        exit 1
    }
}

# Start MongoDB with Docker Compose
Write-Host "Starting MongoDB container..." -ForegroundColor Yellow
docker-compose up -d mongodb

# Wait for MongoDB to be ready
Write-Host "Waiting for MongoDB to be ready..." -ForegroundColor Yellow
$maxAttempts = 30
$attempt = 0

do {
    $attempt++
    Start-Sleep -Seconds 2
    
    try {
        # Try to connect to MongoDB
        $result = docker exec rust-backend-mongodb mongosh --quiet --eval "db.adminCommand('ping')" 2>$null
        if ($result -match "ok.*1") {
            Write-Host "✅ MongoDB is ready!" -ForegroundColor Green
            break
        }
    } catch {
        # Continue waiting
    }
    
    if ($attempt -eq $maxAttempts) {
        Write-Host "❌ MongoDB failed to start within timeout" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "  Attempt $attempt/$maxAttempts - MongoDB not ready yet..." -ForegroundColor Gray
} while ($attempt -lt $maxAttempts)

Write-Host "📊 MongoDB is running on port 27017" -ForegroundColor Cyan
Write-Host "🔗 Connection string: mongodb://rust_app:rust_password@localhost:27017/rust_backend" -ForegroundColor Cyan
Write-Host ""
Write-Host "To start with MongoDB Express UI, run:" -ForegroundColor Yellow
Write-Host "  docker-compose --profile ui up -d" -ForegroundColor White
Write-Host "  Then visit: http://localhost:8081" -ForegroundColor White