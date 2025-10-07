# Start MongoDB for development
Write-Host "🚀 Starting MongoDB for development..." -ForegroundColor Green

# Check if Docker is running
try {
    docker version | Out-Null
} catch {
    Write-Host "❌ Docker is not running. Please start Docker Desktop first." -ForegroundColor Red
    exit 1
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