# Debug UUID Mismatch Script
# This script helps identify UUID inconsistencies between frontend and backend

Write-Host "=== UUID Mismatch Debug Script ===" -ForegroundColor Green
Write-Host ""

Write-Host "1. Checking available races..." -ForegroundColor Yellow
try {
    $races = Invoke-RestMethod -Uri "http://localhost:3000/api/v1/races" -Method GET
    
    foreach ($race in $races) {
        Write-Host "Race: $($race.name) ($($race.uuid))" -ForegroundColor Cyan
        Write-Host "Status: $($race.status)" -ForegroundColor White
        Write-Host "Participants:" -ForegroundColor White
        
        if ($race.participants -and $race.participants.Count -gt 0) {
            foreach ($participant in $race.participants) {
                Write-Host "  - Player UUID: $($participant.player_uuid)" -ForegroundColor Green
                Write-Host "    Car UUID: $($participant.car_uuid)" -ForegroundColor Gray
                Write-Host "    Pilot UUID: $($participant.pilot_uuid)" -ForegroundColor Gray
            }
        } else {
            Write-Host "  No participants" -ForegroundColor Red
        }
        Write-Host ""
    }
} catch {
    Write-Host "✗ Failed to fetch races: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "2. Instructions to find your frontend UUID:" -ForegroundColor Yellow
Write-Host "   - Open browser dev tools (F12)" -ForegroundColor White
Write-Host "   - Go to Console tab" -ForegroundColor White
Write-Host "   - Look for logs like 'PlayerGameInterface' or 'initializeRace'" -ForegroundColor White
Write-Host "   - Find the playerUuid being used in the frontend" -ForegroundColor White

Write-Host ""
Write-Host "3. Common UUID issues:" -ForegroundColor Yellow
Write-Host "   - Frontend uses truncated UUID (e.g., '9c48b7ac')" -ForegroundColor White
Write-Host "   - Backend expects full UUID (e.g., '068564eb-8109-4862-9875-87089c48b7ac')" -ForegroundColor White
Write-Host "   - Authentication context has different UUID than race registration" -ForegroundColor White

Write-Host ""
Write-Host "4. Next steps:" -ForegroundColor Yellow
Write-Host "   - Compare the Player UUIDs above with what you see in browser console" -ForegroundColor White
Write-Host "   - If they don't match, we need to fix the UUID handling" -ForegroundColor White

Write-Host ""
Write-Host "Please share:" -ForegroundColor Green
Write-Host "   1. The Player UUID from the race participants above" -ForegroundColor Cyan
Write-Host "   2. The playerUuid from browser console logs" -ForegroundColor Cyan