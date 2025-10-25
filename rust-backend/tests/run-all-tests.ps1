#!/usr/bin/env pwsh

# Comprehensive test runner for all backend tests
param(
    [string]$TestSuite = "all",
    [switch]$SkipInfrastructure,
    [switch]$SkipApi,
    [switch]$Verbose
)

Write-Host "🧪 Rust Backend Test Suite Runner" -ForegroundColor Green
Write-Host "=================================" -ForegroundColor Green

$testResults = @{
    Infrastructure = @()
    Api = @()
    Failed = @()
    Passed = @()
}

function Run-Test {
    param(
        [string]$TestName,
        [string]$TestPath,
        [string]$Category
    )
    
    Write-Host "`n🔍 Running $TestName..." -ForegroundColor Yellow
    Write-Host "   Path: $TestPath" -ForegroundColor Gray
    
    try {
        $startTime = Get-Date
        & $TestPath
        $endTime = Get-Date
        $duration = ($endTime - $startTime).TotalSeconds
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ $TestName PASSED (${duration}s)" -ForegroundColor Green
            $testResults.Passed += $TestName
            $testResults.$Category += @{ Name = $TestName; Status = "PASSED"; Duration = $duration }
        } else {
            Write-Host "❌ $TestName FAILED (${duration}s)" -ForegroundColor Red
            $testResults.Failed += $TestName
            $testResults.$Category += @{ Name = $TestName; Status = "FAILED"; Duration = $duration }
        }
    } catch {
        Write-Host "❌ $TestName ERROR: $($_.Exception.Message)" -ForegroundColor Red
        $testResults.Failed += $TestName
        $testResults.$Category += @{ Name = $TestName; Status = "ERROR"; Duration = 0 }
    }
}

# Infrastructure Tests
if (!$SkipInfrastructure -and ($TestSuite -eq "all" -or $TestSuite -eq "infrastructure")) {
    Write-Host "`n🏗️  INFRASTRUCTURE TESTS" -ForegroundColor Cyan
    Write-Host "========================" -ForegroundColor Cyan
    
    Run-Test "Project Structure Validation" ".\tests\infrastructure\test-project-structure.ps1" "Infrastructure"
    Run-Test "Docker Setup Verification" ".\tests\infrastructure\verify-docker-setup.ps1" "Infrastructure"
    Run-Test "Docker MongoDB Integration" ".\tests\infrastructure\test-docker-setup.ps1" "Infrastructure"
    Run-Test "MongoDB Integration Test" ".\tests\infrastructure\test-with-mongodb.ps1" "Infrastructure"
}

# API Tests
if (!$SkipApi -and ($TestSuite -eq "all" -or $TestSuite -eq "api")) {
    Write-Host "`n🌐 API TESTS" -ForegroundColor Cyan
    Write-Host "============" -ForegroundColor Cyan
    
    # Check if server is running
    try {
        $health = Invoke-RestMethod -Uri "http://localhost:3000/health_check" -Method GET -TimeoutSec 5
        Write-Host "✅ Server is running, proceeding with API tests..." -ForegroundColor Green
        
        Run-Test "General API Endpoints" ".\tests\api\test-general-endpoints.ps1" "Api"
        Run-Test "Authentication Endpoints" ".\tests\api\test-auth-endpoints.ps1" "Api"
        Run-Test "Player Management" ".\tests\api\test-player-endpoints.ps1" "Api"
        
    } catch {
        Write-Host "⚠️  Server not running, skipping API tests" -ForegroundColor Yellow
        Write-Host "   Start server with: .\Makefile.ps1 dev" -ForegroundColor Gray
    }
}

# Unit Tests
if ($TestSuite -eq "all" -or $TestSuite -eq "unit") {
    Write-Host "`n🔬 UNIT TESTS" -ForegroundColor Cyan
    Write-Host "=============" -ForegroundColor Cyan
    
    Write-Host "Running Rust unit tests..." -ForegroundColor Yellow
    $startTime = Get-Date
    cargo test
    $endTime = Get-Date
    $duration = ($endTime - $startTime).TotalSeconds
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Unit Tests PASSED (${duration}s)" -ForegroundColor Green
        $testResults.Passed += "Unit Tests"
    } else {
        Write-Host "❌ Unit Tests FAILED (${duration}s)" -ForegroundColor Red
        $testResults.Failed += "Unit Tests"
    }
}

# Test Summary
Write-Host "`n📊 TEST SUMMARY" -ForegroundColor Cyan
Write-Host "===============" -ForegroundColor Cyan

$totalTests = $testResults.Passed.Count + $testResults.Failed.Count
$passRate = if ($totalTests -gt 0) { [math]::Round(($testResults.Passed.Count / $totalTests) * 100, 1) } else { 0 }

Write-Host "Total Tests: $totalTests" -ForegroundColor White
Write-Host "Passed: $($testResults.Passed.Count)" -ForegroundColor Green
Write-Host "Failed: $($testResults.Failed.Count)" -ForegroundColor Red
Write-Host "Pass Rate: $passRate%" -ForegroundColor $(if ($passRate -ge 80) { "Green" } elseif ($passRate -ge 60) { "Yellow" } else { "Red" })

if ($testResults.Failed.Count -gt 0) {
    Write-Host "`n❌ Failed Tests:" -ForegroundColor Red
    foreach ($failed in $testResults.Failed) {
        Write-Host "   - $failed" -ForegroundColor Red
    }
}

if ($Verbose) {
    Write-Host "`n📋 Detailed Results:" -ForegroundColor Gray
    
    if ($testResults.Infrastructure.Count -gt 0) {
        Write-Host "`nInfrastructure Tests:" -ForegroundColor Cyan
        foreach ($test in $testResults.Infrastructure) {
            $color = if ($test.Status -eq "PASSED") { "Green" } else { "Red" }
            Write-Host "   $($test.Status): $($test.Name) ($($test.Duration)s)" -ForegroundColor $color
        }
    }
    
    if ($testResults.Api.Count -gt 0) {
        Write-Host "`nAPI Tests:" -ForegroundColor Cyan
        foreach ($test in $testResults.Api) {
            $color = if ($test.Status -eq "PASSED") { "Green" } else { "Red" }
            Write-Host "   $($test.Status): $($test.Name) ($($test.Duration)s)" -ForegroundColor $color
        }
    }
}

Write-Host "`n🎯 Test Suite Complete!" -ForegroundColor Green

# Exit with appropriate code
if ($testResults.Failed.Count -gt 0) {
    exit 1
} else {
    exit 0
}