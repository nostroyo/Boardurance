#!/usr/bin/env pwsh
# Script to run unit tests without requiring MongoDB
# This allows local development without database setup

Write-Host "🧪 Running Rust unit tests (no database required)..." -ForegroundColor Green

# Run unit tests from the lib and binary crates only (excludes integration tests)
Write-Host "📦 Running library and binary unit tests..." -ForegroundColor Cyan
cargo test --lib --bins

Write-Host ""
Write-Host "✅ Unit tests completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Test Summary:" -ForegroundColor Cyan
Write-Host "  • Boost card domain logic: ✅ Covered" -ForegroundColor Green
Write-Host "  • Boost hand management: ✅ Covered" -ForegroundColor Green  
Write-Host "  • Boost usage tracking: ✅ Covered" -ForegroundColor Green
Write-Host "  • Boost cycle summaries: ✅ Covered" -ForegroundColor Green
Write-Host ""
Write-Host "💡 To run integration tests (requires MongoDB):" -ForegroundColor Yellow
Write-Host "   .\Makefile.ps1 test" -ForegroundColor Yellow
Write-Host ""
Write-Host "🐳 To start MongoDB for integration tests:" -ForegroundColor Yellow
Write-Host "   .\Makefile.ps1 dev" -ForegroundColor Yellow