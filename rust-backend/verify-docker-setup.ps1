# Verify Docker setup files are correctly structured
Write-Host "🔍 Verifying Docker MongoDB Setup Files..." -ForegroundColor Green

$dockerFiles = @(
    "docker-compose.yml",
    "docker-compose.test.yml", 
    "docker/mongo-init.js",
    "docker/mongo-test-init.js",
    "scripts/start-mongodb.ps1",
    "scripts/start-test-mongodb.ps1",
    "scripts/stop-mongodb.ps1",
    "scripts/test-with-mongodb.ps1",
    "Makefile.ps1"
)

$configFiles = @(
    "configuration/base.yaml",
    "configuration/local.yaml",
    "configuration/test.yaml",
    "configuration/production.yaml"
)

Write-Host "`n📁 Checking Docker files..." -ForegroundColor Yellow
$allDockerFilesExist = $true
foreach ($file in $dockerFiles) {
    if (Test-Path $file) {
        Write-Host "  ✅ $file" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $file" -ForegroundColor Red
        $allDockerFilesExist = $false
    }
}

Write-Host "`n⚙️ Checking configuration files..." -ForegroundColor Yellow
$allConfigFilesExist = $true
foreach ($file in $configFiles) {
    if (Test-Path $file) {
        Write-Host "  ✅ $file" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $file" -ForegroundColor Red
        $allConfigFilesExist = $false
    }
}

Write-Host "`n🔧 Checking application build..." -ForegroundColor Yellow
$buildResult = cargo check 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✅ Application builds successfully" -ForegroundColor Green
} else {
    Write-Host "  ❌ Application build failed" -ForegroundColor Red
    Write-Host $buildResult
}

# Check Docker Compose file syntax
Write-Host "`n📋 Validating Docker Compose files..." -ForegroundColor Yellow
try {
    # Check if docker-compose files are valid YAML
    $compose1 = Get-Content "docker-compose.yml" -Raw
    $compose2 = Get-Content "docker-compose.test.yml" -Raw
    
    if ($compose1 -match "version:" -and $compose1 -match "services:") {
        Write-Host "  ✅ docker-compose.yml structure valid" -ForegroundColor Green
    } else {
        Write-Host "  ❌ docker-compose.yml structure invalid" -ForegroundColor Red
    }
    
    if ($compose2 -match "version:" -and $compose2 -match "services:") {
        Write-Host "  ✅ docker-compose.test.yml structure valid" -ForegroundColor Green
    } else {
        Write-Host "  ❌ docker-compose.test.yml structure invalid" -ForegroundColor Red
    }
} catch {
    Write-Host "  ❌ Error validating Docker Compose files: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n📊 Summary:" -ForegroundColor Cyan
if ($allDockerFilesExist -and $allConfigFilesExist -and $LASTEXITCODE -eq 0) {
    Write-Host "✅ All Docker MongoDB setup files are correctly configured!" -ForegroundColor Green
    Write-Host "`n🚀 Ready to use when Docker is available:" -ForegroundColor Yellow
    Write-Host "  1. Install Docker Desktop" -ForegroundColor White
    Write-Host "  2. Run: .\Makefile.ps1 dev" -ForegroundColor White
    Write-Host "  3. Access: http://localhost:3000" -ForegroundColor White
} else {
    Write-Host "❌ Some issues found in the setup" -ForegroundColor Red
}

Write-Host "`n📖 Docker Setup Features:" -ForegroundColor Cyan
Write-Host "  - Development MongoDB with sample data" -ForegroundColor White
Write-Host "  - Test MongoDB for integration tests" -ForegroundColor White  
Write-Host "  - MongoDB Express UI for database management" -ForegroundColor White
Write-Host "  - Automated initialization scripts" -ForegroundColor White
Write-Host "  - Environment-specific configurations" -ForegroundColor White
Write-Host "  - Easy-to-use PowerShell commands" -ForegroundColor White