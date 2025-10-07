# Test the complete Docker MongoDB setup
Write-Host "🐳 Testing Docker MongoDB Setup..." -ForegroundColor Green

# Check Docker availability
Write-Host "`n1. Checking Docker..." -ForegroundColor Yellow
try {
    $dockerVersion = docker --version
    Write-Host "✅ Docker found: $dockerVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker not found. Please install Docker Desktop." -ForegroundColor Red
    exit 1
}

# Check Docker Compose availability
try {
    $composeVersion = docker-compose --version
    Write-Host "✅ Docker Compose found: $composeVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Docker Compose not found." -ForegroundColor Red
    exit 1
}

# Test development MongoDB
Write-Host "`n2. Testing development MongoDB..." -ForegroundColor Yellow
Write-Host "Starting development MongoDB..." -ForegroundColor Gray
docker-compose up -d mongodb

# Wait and test connection
Start-Sleep -Seconds 10
try {
    $result = docker exec rust-backend-mongodb mongosh --quiet --eval "db.adminCommand('ping')"
    if ($result -match "ok.*1") {
        Write-Host "✅ Development MongoDB is working" -ForegroundColor Green
    } else {
        Write-Host "❌ Development MongoDB connection failed" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Failed to test development MongoDB: $($_.Exception.Message)" -ForegroundColor Red
}

# Test database initialization
Write-Host "Testing database initialization..." -ForegroundColor Gray
try {
    $collections = docker exec rust-backend-mongodb mongosh rust_backend --quiet --eval "db.getCollectionNames()"
    if ($collections -match "test_items") {
        Write-Host "✅ Database initialization successful" -ForegroundColor Green
    } else {
        Write-Host "❌ Database initialization failed" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Failed to check database initialization: $($_.Exception.Message)" -ForegroundColor Red
}

# Test test MongoDB
Write-Host "`n3. Testing test MongoDB..." -ForegroundColor Yellow
Write-Host "Starting test MongoDB..." -ForegroundColor Gray
docker-compose -f docker-compose.test.yml up -d mongodb-test

Start-Sleep -Seconds 10
try {
    $result = docker exec rust-backend-mongodb-test mongosh --quiet --eval "db.adminCommand('ping')"
    if ($result -match "ok.*1") {
        Write-Host "✅ Test MongoDB is working" -ForegroundColor Green
    } else {
        Write-Host "❌ Test MongoDB connection failed" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Failed to test MongoDB: $($_.Exception.Message)" -ForegroundColor Red
}

# Test application build
Write-Host "`n4. Testing application build..." -ForegroundColor Yellow
$buildResult = cargo check 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Application builds successfully" -ForegroundColor Green
} else {
    Write-Host "❌ Application build failed:" -ForegroundColor Red
    Write-Host $buildResult
}

# Cleanup
Write-Host "`n5. Cleaning up..." -ForegroundColor Yellow
docker-compose down
docker-compose -f docker-compose.test.yml down

Write-Host "`n🎉 Docker MongoDB setup test completed!" -ForegroundColor Green
Write-Host "`nTo use the setup:" -ForegroundColor Cyan
Write-Host "  Development: .\Makefile.ps1 dev" -ForegroundColor White
Write-Host "  With UI:     .\Makefile.ps1 dev-ui" -ForegroundColor White
Write-Host "  Testing:     .\Makefile.ps1 test" -ForegroundColor White