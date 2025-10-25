# Simple test to verify the Rust backend structure
Write-Host "Testing Rust Backend Structure..." -ForegroundColor Green

# Test build
Write-Host "`nTesting build..." -ForegroundColor Yellow
$buildResult = cargo check 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful!" -ForegroundColor Green
} else {
    Write-Host "Build failed:" -ForegroundColor Red
    Write-Host $buildResult
    exit 1
}

# Check project structure
Write-Host "`nChecking project structure..." -ForegroundColor Yellow
$expectedFiles = @(
    "src/lib.rs",
    "src/main.rs", 
    "src/configuration.rs",
    "src/startup.rs",
    "src/telemetry.rs",
    "src/domain/mod.rs",
    "src/domain/test_item.rs",
    "src/routes/mod.rs",
    "src/routes/health_check.rs",
    "src/routes/test_items.rs",
    "configuration/base.yaml",
    "configuration/local.yaml",
    "configuration/production.yaml"
)

$allFilesExist = $true
foreach ($file in $expectedFiles) {
    if (Test-Path $file) {
        Write-Host "  Found: $file" -ForegroundColor Green
    } else {
        Write-Host "  Missing: $file" -ForegroundColor Red
        $allFilesExist = $false
    }
}

if ($allFilesExist) {
    Write-Host "`nAll expected files found!" -ForegroundColor Green
} else {
    Write-Host "`nSome files are missing!" -ForegroundColor Red
    exit 1
}

Write-Host "`nRust backend with Axum, Swagger, and MongoDB is ready!" -ForegroundColor Green
Write-Host "Following Luca Palmieri's patterns from 'Zero to Production in Rust'" -ForegroundColor Cyan
Write-Host "`nTo run the server:" -ForegroundColor Yellow
Write-Host "  cargo run" -ForegroundColor White
Write-Host "`nEndpoints will be available at:" -ForegroundColor Yellow
Write-Host "  http://localhost:3000/health_check" -ForegroundColor White
Write-Host "  http://localhost:3000/swagger-ui" -ForegroundColor White
Write-Host "  http://localhost:3000/api/v1/test" -ForegroundColor White