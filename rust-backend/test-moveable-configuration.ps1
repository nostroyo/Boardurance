#!/usr/bin/env pwsh

# Test script for moveable configuration functionality
$baseUrl = "http://localhost:8000/api/v1"

Write-Host "Testing Moveable Configuration Functionality" -ForegroundColor Green
Write-Host "=============================================" -ForegroundColor Green

# Test 1: Create a player with assets
Write-Host "`n1. Creating a player with assets..." -ForegroundColor Yellow
$createPayload = @{
    email = "testmove@example.com"
    team_name = "Moveable Test Team"
} | ConvertTo-Json

try {
    $createResponse = Invoke-RestMethod -Uri "$baseUrl/players" -Method POST -Body $createPayload -ContentType "application/json"
    $playerUuid = $createResponse.player.uuid
    Write-Host "✅ Player created successfully with UUID: $playerUuid" -ForegroundColor Green
    Write-Host "Cars: $($createResponse.player.cars.Count)" -ForegroundColor Cyan
    Write-Host "Pilots: $($createResponse.player.pilots.Count)" -ForegroundColor Cyan
    Write-Host "Engines: $($createResponse.player.engines.Count)" -ForegroundColor Cyan
    Write-Host "Bodies: $($createResponse.player.bodies.Count)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Failed to create player: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 2: Get the player to see initial configuration
Write-Host "`n2. Getting player initial configuration..." -ForegroundColor Yellow
try {
    $playerResponse = Invoke-RestMethod -Uri "$baseUrl/players/$playerUuid" -Method GET
    $player = $playerResponse
    
    Write-Host "✅ Player retrieved successfully" -ForegroundColor Green
    Write-Host "Initial car configurations:" -ForegroundColor Cyan
    for ($i = 0; $i -lt $player.cars.Count; $i++) {
        $car = $player.cars[$i]
        Write-Host "  Car $($i + 1): $($car.name)" -ForegroundColor White
        Write-Host "    Engine: $($car.engine_uuid ?? 'None')" -ForegroundColor Gray
        Write-Host "    Body: $($car.body_uuid ?? 'None')" -ForegroundColor Gray
        Write-Host "    Pilot: $($car.pilot_uuid ?? 'None')" -ForegroundColor Gray
    }
} catch {
    Write-Host "❌ Failed to get player: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Test 3: Modify car configuration (assign components)
Write-Host "`n3. Modifying car configuration..." -ForegroundColor Yellow

# Assign components to first car
$player.cars[0].engine_uuid = $player.engines[0].uuid
$player.cars[0].body_uuid = $player.bodies[0].uuid
$player.cars[0].pilot_uuid = $player.pilots[0].uuid

# Assign components to second car
$player.cars[1].engine_uuid = $player.engines[1].uuid
$player.cars[1].body_uuid = $player.bodies[1].uuid
$player.cars[1].pilot_uuid = $player.pilots[1].uuid

$configPayload = @{
    team_name = $player.team_name
    cars = $player.cars
} | ConvertTo-Json -Depth 10

try {
    $configResponse = Invoke-RestMethod -Uri "$baseUrl/players/$playerUuid/configuration" -Method PUT -Body $configPayload -ContentType "application/json"
    Write-Host "✅ Configuration updated successfully" -ForegroundColor Green
    
    Write-Host "Updated car configurations:" -ForegroundColor Cyan
    for ($i = 0; $i -lt $configResponse.player.cars.Count; $i++) {
        $car = $configResponse.player.cars[$i]
        Write-Host "  Car $($i + 1): $($car.name)" -ForegroundColor White
        Write-Host "    Engine: $($car.engine_uuid ?? 'None')" -ForegroundColor Gray
        Write-Host "    Body: $($car.body_uuid ?? 'None')" -ForegroundColor Gray
        Write-Host "    Pilot: $($car.pilot_uuid ?? 'None')" -ForegroundColor Gray
    }
} catch {
    Write-Host "❌ Failed to update configuration: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Response: $($_.Exception.Response)" -ForegroundColor Red
}

# Test 4: Move components between cars
Write-Host "`n4. Moving components between cars..." -ForegroundColor Yellow

# Swap engines between cars
$tempEngineUuid = $configResponse.player.cars[0].engine_uuid
$configResponse.player.cars[0].engine_uuid = $configResponse.player.cars[1].engine_uuid
$configResponse.player.cars[1].engine_uuid = $tempEngineUuid

$swapPayload = @{
    team_name = $configResponse.player.team_name
    cars = $configResponse.player.cars
} | ConvertTo-Json -Depth 10

try {
    $swapResponse = Invoke-RestMethod -Uri "$baseUrl/players/$playerUuid/configuration" -Method PUT -Body $swapPayload -ContentType "application/json"
    Write-Host "✅ Components swapped successfully" -ForegroundColor Green
    
    Write-Host "Final car configurations:" -ForegroundColor Cyan
    for ($i = 0; $i -lt $swapResponse.player.cars.Count; $i++) {
        $car = $swapResponse.player.cars[$i]
        Write-Host "  Car $($i + 1): $($car.name)" -ForegroundColor White
        Write-Host "    Engine: $($car.engine_uuid ?? 'None')" -ForegroundColor Gray
        Write-Host "    Body: $($car.body_uuid ?? 'None')" -ForegroundColor Gray
        Write-Host "    Pilot: $($car.pilot_uuid ?? 'None')" -ForegroundColor Gray
    }
} catch {
    Write-Host "❌ Failed to swap components: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Remove components (move back to inventory)
Write-Host "`n5. Removing components from cars..." -ForegroundColor Yellow

# Remove all components from first car
$swapResponse.player.cars[0].engine_uuid = $null
$swapResponse.player.cars[0].body_uuid = $null
$swapResponse.player.cars[0].pilot_uuid = $null

$removePayload = @{
    team_name = $swapResponse.player.team_name
    cars = $swapResponse.player.cars
} | ConvertTo-Json -Depth 10

try {
    $removeResponse = Invoke-RestMethod -Uri "$baseUrl/players/$playerUuid/configuration" -Method PUT -Body $removePayload -ContentType "application/json"
    Write-Host "✅ Components removed successfully" -ForegroundColor Green
    
    Write-Host "Car configurations after removal:" -ForegroundColor Cyan
    for ($i = 0; $i -lt $removeResponse.player.cars.Count; $i++) {
        $car = $removeResponse.player.cars[$i]
        Write-Host "  Car $($i + 1): $($car.name)" -ForegroundColor White
        Write-Host "    Engine: $($car.engine_uuid ?? 'None')" -ForegroundColor Gray
        Write-Host "    Body: $($car.body_uuid ?? 'None')" -ForegroundColor Gray
        Write-Host "    Pilot: $($car.pilot_uuid ?? 'None')" -ForegroundColor Gray
    }
} catch {
    Write-Host "❌ Failed to remove components: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n🎯 Moveable configuration testing completed!" -ForegroundColor Green