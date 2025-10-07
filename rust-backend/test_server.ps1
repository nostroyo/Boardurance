# Test script for Rust backend server
Write-Host "Testing Rust Backend Server..." -ForegroundColor Green

# Test health endpoint
Write-Host "`nTesting health endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/health" -Method GET
    Write-Host "✅ Health check successful:" -ForegroundColor Green
    Write-Host ($response | ConvertTo-Json -Depth 2)
} catch {
    Write-Host "❌ Health check failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test Swagger UI
Write-Host "`nTesting Swagger UI..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:3000/swagger-ui" -Method GET
    if ($response.StatusCode -eq 200) {
        Write-Host "✅ Swagger UI accessible" -ForegroundColor Green
    }
} catch {
    Write-Host "❌ Swagger UI failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test OpenAPI JSON
Write-Host "`nTesting OpenAPI JSON..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api-docs/openapi.json" -Method GET
    Write-Host "✅ OpenAPI JSON accessible:" -ForegroundColor Green
    Write-Host "API Title: $($response.info.title)"
    Write-Host "API Version: $($response.info.version)"
    Write-Host "Available paths: $($response.paths.PSObject.Properties.Name -join ', ')"
} catch {
    Write-Host "❌ OpenAPI JSON failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`nTest completed!" -ForegroundColor Green