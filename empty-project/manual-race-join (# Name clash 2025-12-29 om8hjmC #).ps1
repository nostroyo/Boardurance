# Manual Race Join Test Script
# This script tests race registration directly via API

Write-Host "=== Manual Race Join Test ===" -ForegroundColor Green
Write-Host ""

$raceUuid = "484ec0a7-6d2e-48ae-8113-ac3777ba9fcb"
$playerUuid = "068564eb-8109-4862-9875-87089c48b7ac"

Write-Host "Testing race join for:" -ForegroundColor Yellow
Write-Host "  Race UUID: $raceUuid" -ForegroundColor Cyan
Write-Host "  Player UUID: $playerUuid" -ForegroundColor Cyan
Write-Host ""

# First, let's check if the player exists
Write-Host "1. Checking if player exists..." -ForegroundColor Yellow
try {
    $player = Invoke-RestMethod -Uri "http://localhost:3000/api/v1/players/$playerUuid" -Method GET
    Write-Host "✓ Player found: $($player.team_name)" -ForegroundColor Green
    
    if ($player.cars -and $player.cars.Count -gt 0) {
        Write-Host "✓ Player has $($player.cars.Count) cars" -ForegroundColor Green
        $car = $player.cars[0]
        Write-Host "  First car UUID: $($car.uuid)" -ForegroundColor Cyan
    } else {
        Write-Host "✗ Player has no cars" -ForegroundColor Red
    }
    
    if ($player.pilots -and $player.pilots.Count -gt 0) {
        Write-Host "✓ Player has $($player.pilots.Count) pilots" -ForegroundColor Green
        $pilot = $player.pilots[0]
        Write-Host "  First pilot UUID: $($pilot.uuid)" -ForegroundColor Cyan
    } else {
        Write-Host "✗ Player has no pilots" -ForegroundColor Red
    }
} catch {
    Write-Host "✗ Player not found: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "This means the player doesn't exist in the database" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "2. Attempting to join race..." -ForegroundColor Yellow

if ($player.cars -and $player.cars.Count -gt 0 -and $player.pilots -and $player.pilots.Count -gt 0) {
    $joinData = @{
        player_uuid = $playerUuid
        car_uuid = $player.cars[0].uuid
        pilot_uuid = $player.pilots[0].uuid
    } | ConvertTo-Json
    
    Write-Host "Join data: $joinData" -ForegroundColor Cyan
    
    try {
        $response = Invoke-RestMethod -Uri "http://localhost:3000/api/v1/races/$raceUuid/join" -Method POST -Body $joinData -ContentType "application/json"
        Write-Host "✓ Successfully joined race!" -ForegroundColor Green
        Write-Host "Response: $($response | ConvertTo-Json)" -ForegroundColor Cyan
    } catch {
        Write-Host "✗ Failed to join race: $($_.Exception.Message)" -ForegroundColor Red
        if ($_.Exception.Response) {
            $errorResponse = $_.Exception.Response.GetResponseStream()
            $reader = New-Object System.IO.StreamReader($errorResponse)
            $errorBody = $reader.ReadToEnd()
            Write-Host "Error details: $errorBody" -ForegroundColor Red
        }
    }
} else {
    Write-Host "✗ Cannot join race - missing cars or pilots" -ForegroundColor Red
    Write-Host "You need to create cars and pilots first" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "3. Checking race participants after join attempt..." -ForegroundColor Yellow
try {
    $raceAfter = Invoke-RestMethod -Uri "http://localhost:3000/api/v1/races/$raceUuid" -Method GET
    if ($raceAfter.participants -and $raceAfter.participants.Count -gt 0) {
        Write-Host "✓ Race now has $($raceAfter.participants.Count) participants" -ForegroundColor Green
        foreach ($participant in $raceAfter.participants) {
            Write-Host "  - Player: $($participant.player_uuid)" -ForegroundColor Cyan
        }
    } else {
        Write-Host "✗ Race still has no participants" -ForegroundColor Red
    }
} catch {
    Write-Host "✗ Failed to check race: $($_.Exception.Message)" -ForegroundColor Red
}