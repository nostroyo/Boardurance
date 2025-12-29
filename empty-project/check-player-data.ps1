# Check Player Data Script
# This script checks if you have the required cars and pilots for race registration

Write-Host "=== Player Data Check Script ===" -ForegroundColor Green
Write-Host ""

Write-Host "This script will help identify why race registration is failing." -ForegroundColor Yellow
Write-Host "Please follow these steps:" -ForegroundColor Yellow
Write-Host ""

Write-Host "1. Open browser dev tools (F12)" -ForegroundColor Cyan
Write-Host "2. Go to Console tab" -ForegroundColor Cyan
Write-Host "3. Try to join a race in the frontend" -ForegroundColor Cyan
Write-Host "4. Look for these console messages:" -ForegroundColor Cyan
Write-Host ""

Write-Host "Expected console logs:" -ForegroundColor White
Write-Host "  - 'Fetching player data for: [YOUR_UUID]'" -ForegroundColor Gray
Write-Host "  - 'Player data: [OBJECT WITH CARS/PILOTS]'" -ForegroundColor Gray
Write-Host "  - 'Using car: [CAR_UUID] with pilots: [PILOT_ARRAY]'" -ForegroundColor Gray
Write-Host "  - 'Joining race with data: [JOIN_DATA]'" -ForegroundColor Gray
Write-Host "  - 'Join race response status: [STATUS_CODE]'" -ForegroundColor Gray
Write-Host ""

Write-Host "Common error messages to look for:" -ForegroundColor Red
Write-Host "  - 'Failed to load player data: 404' → Player doesn't exist" -ForegroundColor Gray
Write-Host "  - 'No car with 3 pilots found' → Need to create cars/pilots" -ForegroundColor Gray
Write-Host "  - 'Join race response status: 400/500' → Backend validation error" -ForegroundColor Gray
Write-Host ""

Write-Host "If you see 'No car with 3 pilots found':" -ForegroundColor Yellow
Write-Host "  You need to create player data first. This might be missing from your setup." -ForegroundColor White
Write-Host ""

Write-Host "Next steps:" -ForegroundColor Green
Write-Host "  1. Try to join a race and check console" -ForegroundColor White
Write-Host "  2. Share the console output with me" -ForegroundColor White
Write-Host "  3. I'll help create the missing player data if needed" -ForegroundColor White