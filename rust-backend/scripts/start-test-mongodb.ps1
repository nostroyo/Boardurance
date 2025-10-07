# Start MongoDB for testing
Write-Host "🧪 Starting MongoDB for testing..." -ForegroundColor Green

# Check if Docker is running
try {
    docker version | Out-Null
} catch {
    Write-Host "❌ Docker is not running. Please start Docker Desktop first." -ForegroundColor Red
    exit 1
}

# Start test MongoDB with Docker Compose
Write-Host "Starting test MongoDB container..." -ForegroundColor Yellow
docker-compose -f docker-compose.test.yml up -d mongodb-test

# Wait for MongoDB to be ready
Write-Host "Waiting for test MongoDB to be ready..." -ForegroundColor Yellow
$maxAttempts = 30
$attempt = 0

do {
    $attempt++
    Start-Sleep -Seconds 2
    
    try {
        # Try to connect to test MongoDB
        $result = docker exec rust-backend-mongodb-test mongosh --quiet --eval "db.adminCommand('ping')" 2>$null
        if ($result -match "ok.*1") {
            Write-Host "✅ Test MongoDB is ready!" -ForegroundColor Green
            break
        }
    } catch {
        # Continue waiting
    }
    
    if ($attempt -eq $maxAttempts) {
        Write-Host "❌ Test MongoDB failed to start within timeout" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "  Attempt $attempt/$maxAttempts - Test MongoDB not ready yet..." -ForegroundColor Gray
} while ($attempt -lt $maxAttempts)

Write-Host "📊 Test MongoDB is running on port 27018" -ForegroundColor Cyan
Write-Host "🔗 Connection string: mongodb://test_user:test_password@localhost:27018/rust_backend_test" -ForegroundColor Cyan