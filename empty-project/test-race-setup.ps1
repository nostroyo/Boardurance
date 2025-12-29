# Test Race Setup Script
# This script helps debug race registration issues

Write-Host "=== Race Registration Debug Script ===" -ForegroundColor Green
Write-Host ""

# Check if backend is running
Write-Host "1. Checking backend connection..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:3000/api/v1/races" -Method GET -ErrorAction Stop
    Write-Host "✓ Backend is running and accessible" -ForegroundColor Green
    Write-Host "Found $($response.Count) races" -ForegroundColor Cyan
} catch {
    Write-Host "✗ Backend not accessible: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Make sure to run: .\start-full-stack.ps1" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "2. Testing authentication..." -ForegroundColor Yellow
Write-Host "Please check the frontend at http://localhost:5173" -ForegroundColor Cyan
Write-Host "Make sure you're logged in and can see your team name" -ForegroundColor Cyan

Write-Host ""
Write-Host "3. Common issues and solutions:" -ForegroundColor Yellow
Write-Host "   - Not logged in: Go to frontend and authenticate" -ForegroundColor White
Write-Host "   - No cars/pilots: Need to create test data first" -ForegroundColor White
Write-Host "   - Network errors: Check if all services are running" -ForegroundColor White

Write-Host ""
Write-Host "4. Next steps:" -ForegroundColor Yellow
Write-Host "   - Open browser dev tools (F12)" -ForegroundColor White
Write-Host "   - Try to join/create a race" -ForegroundColor White
Write-Host "   - Check Console tab for error messages" -ForegroundColor White
Write-Host "   - Check Network tab for failed API calls" -ForegroundColor White

Write-Host ""
Write-Host "If you see specific error messages, please share them!" -ForegroundColor Green