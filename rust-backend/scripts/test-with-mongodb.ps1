# Complete test script with MongoDB
Write-Host "🧪 Running complete test with MongoDB..." -ForegroundColor Green

# Start test MongoDB
Write-Host "`n1. Starting test MongoDB..." -ForegroundColor Yellow
& .\scripts\start-test-mongodb.ps1

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to start test MongoDB" -ForegroundColor Red
    exit 1
}

# Set test environment
$env:APP_ENVIRONMENT = "test"

Write-Host "`n2. Building application..." -ForegroundColor Yellow
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed" -ForegroundColor Red
    & .\scripts\stop-mongodb.ps1
    exit 1
}

Write-Host "`n3. Starting application in test mode..." -ForegroundColor Yellow
$appProcess = Start-Process -FilePath "cargo" -ArgumentList "run" -NoNewWindow -PassThru

# Wait for application to start
Start-Sleep -Seconds 5

Write-Host "`n4. Running API tests..." -ForegroundColor Yellow

# Test health check
Write-Host "Testing health check..." -ForegroundColor Cyan
try {
    $health = Invoke-RestMethod -Uri "http://localhost:3001/health_check" -Method GET -TimeoutSec 10
    Write-Host "✅ Health check: $($health.status)" -ForegroundColor Green
} catch {
    Write-Host "❌ Health check failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test create item
Write-Host "Testing create test item..." -ForegroundColor Cyan
try {
    $testData = @{
        name = "Docker Test Item"
        description = "Created during Docker MongoDB test"
    } | ConvertTo-Json

    $created = Invoke-RestMethod -Uri "http://localhost:3001/api/v1/test" -Method POST -Body $testData -ContentType "application/json" -TimeoutSec 10
    Write-Host "✅ Created item with UUID: $($created.uuid)" -ForegroundColor Green
} catch {
    Write-Host "❌ Create item failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test get items
Write-Host "Testing get test items..." -ForegroundColor Cyan
try {
    $items = Invoke-RestMethod -Uri "http://localhost:3001/api/v1/test" -Method GET -TimeoutSec 10
    Write-Host "✅ Retrieved $($items.Count) items" -ForegroundColor Green
} catch {
    Write-Host "❌ Get items failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Cleanup
Write-Host "`n5. Cleaning up..." -ForegroundColor Yellow
if ($appProcess -and !$appProcess.HasExited) {
    Stop-Process -Id $appProcess.Id -Force
    Write-Host "Stopped application" -ForegroundColor Gray
}

& .\scripts\stop-mongodb.ps1

Write-Host "`n🎉 Test completed!" -ForegroundColor Green