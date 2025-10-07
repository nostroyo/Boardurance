# Stop MongoDB containers
Write-Host "🛑 Stopping MongoDB containers..." -ForegroundColor Yellow

# Stop development MongoDB
Write-Host "Stopping development MongoDB..." -ForegroundColor Gray
docker-compose down

# Stop test MongoDB
Write-Host "Stopping test MongoDB..." -ForegroundColor Gray
docker-compose -f docker-compose.test.yml down

Write-Host "✅ All MongoDB containers stopped" -ForegroundColor Green