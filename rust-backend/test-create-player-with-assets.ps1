#!/usr/bin/env pwsh

# Test script for the new create player with assets endpoint

$baseUrl = "http://localhost:8000/api/v1"

Write-Host "Testing Create Player with Assets Endpoint" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green

# Test data
$playerData = @{
    email = "test@example.com"
    team_name = "Test Racing Team"
} | ConvertTo-Json

Write-Host "Creating player with assets..." -ForegroundColor Yellow
Write-Host "Request body: $playerData" -ForegroundColor Cyan

try {
    $response = Invoke-RestMethod -Uri "$baseUrl/players/create-with-assets" -Method POST -Body $playerData -ContentType "application/json"
    
    Write-Host "✅ Player created successfully!" -ForegroundColor Green
    Write-Host "Player UUID: $($response.player.uuid)" -ForegroundColor Cyan
    Write-Host "Email: $($response.player.email)" -ForegroundColor Cyan
    Write-Host "Team Name: $($response.player.team_name)" -ForegroundColor Cyan
    Write-Host "Cars: $($response.player.cars.Count)" -ForegroundColor Cyan
    Write-Host "Engines: $($response.player.engines.Count)" -ForegroundColor Cyan
    Write-Host "Bodies: $($response.player.bodies.Count)" -ForegroundColor Cyan
    Write-Host "Pilots: $($response.player.pilots.Count)" -ForegroundColor Cyan
    
    # Test getting player by email
    Write-Host "`nTesting get player by email..." -ForegroundColor Yellow
    $emailResponse = Invoke-RestMethod -Uri "$baseUrl/players/by-email/test@example.com" -Method GET
    Write-Host "✅ Player retrieved by email successfully!" -ForegroundColor Green
    Write-Host "Retrieved UUID: $($emailResponse.uuid)" -ForegroundColor Cyan
    
    # Test duplicate email
    Write-Host "`nTesting duplicate email (should fail)..." -ForegroundColor Yellow
    try {
        $duplicateResponse = Invoke-RestMethod -Uri "$baseUrl/players/create-with-assets" -Method POST -Body $playerData -ContentType "application/json"
        Write-Host "❌ Duplicate email test failed - should have returned error" -ForegroundColor Red
    } catch {
        Write-Host "✅ Duplicate email correctly rejected!" -ForegroundColor Green
        Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "❌ Test failed!" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.Exception.Response) {
        $statusCode = $_.Exception.Response.StatusCode
        Write-Host "Status Code: $statusCode" -ForegroundColor Red
    }
}

Write-Host "`nTest completed!" -ForegroundColor Green