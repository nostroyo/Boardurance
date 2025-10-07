# Test script for Rust backend server endpoints
Write-Host "🚀 Testing Rust Backend Server (Luca Palmieri Style)..." -ForegroundColor Green

# Wait for server to start
Start-Sleep -Seconds 2

# Test health check endpoint
Write-Host "`n🔍 Testing health check endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/health_check" -Method GET -TimeoutSec 5
    Write-Host "✅ Health check successful:" -ForegroundColor Green
    Write-Host "   Status: $($response.status)" -ForegroundColor Cyan
    Write-Host "   Message: $($response.message)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Health check failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test Swagger UI
Write-Host "`n📚 Testing Swagger UI..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:3000/swagger-ui" -Method GET -TimeoutSec 5
    if ($response.StatusCode -eq 200) {
        Write-Host "✅ Swagger UI accessible at http://localhost:3000/swagger-ui" -ForegroundColor Green
    }
} catch {
    Write-Host "❌ Swagger UI failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test OpenAPI JSON
Write-Host "`n📋 Testing OpenAPI specification..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api-docs/openapi.json" -Method GET -TimeoutSec 5
    Write-Host "✅ OpenAPI JSON accessible:" -ForegroundColor Green
    Write-Host "   API Title: $($response.info.title)" -ForegroundColor Cyan
    Write-Host "   API Version: $($response.info.version)" -ForegroundColor Cyan
    Write-Host "   Available paths: $($response.paths.PSObject.Properties.Name -join ', ')" -ForegroundColor Cyan
} catch {
    Write-Host "❌ OpenAPI JSON failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test creating a test item (this will fail gracefully if MongoDB is not available)
Write-Host "`n📝 Testing create test item endpoint..." -ForegroundColor Yellow
try {
    $testData = @{
        name = "Test Item from PowerShell"
        description = "This is a test item created via API call"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/v1/test" -Method POST -Body $testData -ContentType "application/json" -TimeoutSec 5
    Write-Host "✅ Test item created successfully:" -ForegroundColor Green
    Write-Host "   UUID: $($response.uuid)" -ForegroundColor Cyan
    Write-Host "   Name: $($response.name)" -ForegroundColor Cyan
} catch {
    Write-Host "⚠️  Create test item failed (expected if MongoDB not running): $($_.Exception.Message)" -ForegroundColor Yellow
}

# Test getting test items
Write-Host "`n📋 Testing get test items endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/v1/test" -Method GET -TimeoutSec 5
    Write-Host "✅ Get test items successful:" -ForegroundColor Green
    Write-Host "   Found $($response.Count) items" -ForegroundColor Cyan
} catch {
    Write-Host "⚠️  Get test items failed (expected if MongoDB not running): $($_.Exception.Message)" -ForegroundColor Yellow
}

Write-Host "`n🎉 Test completed!" -ForegroundColor Green
Write-Host "Architecture follows Luca Palmieri's 'Zero to Production in Rust' patterns:" -ForegroundColor Cyan
Write-Host "   - Structured configuration with environment support" -ForegroundColor White
Write-Host "   - Proper error handling and domain modeling" -ForegroundColor White
Write-Host "   - Structured logging with tracing" -ForegroundColor White
Write-Host "   - Clean separation of concerns (domain, routes, startup)" -ForegroundColor White
Write-Host "   - Testable application structure" -ForegroundColor White