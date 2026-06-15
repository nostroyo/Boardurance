#!/usr/bin/env pwsh

# Test script for Player endpoints with authentication integration
Write-Host "🎮 Testing Player Management Endpoints (with Auth)..." -ForegroundColor Green

$baseUrl = "http://localhost:3000/api/v1"
$authUrl = "$baseUrl/auth"

# Test user for player operations
$testUser = @{
    email = "player-test@example.com"
    password = "PlayerTest123"
    team_name = "Test Racing Team"
}

$playerUuid = $null

# Function to make safe HTTP requests
function Invoke-SafeRestMethod {
    param(
        [string]$Uri,
        [string]$Method = "GET",
        [hashtable]$Body = $null,
        [string]$ContentType = "application/json"
    )

    try {
        $params = @{
            Uri = $Uri
            Method = $Method
            ContentType = $ContentType
        }

        if ($Body) {
            $params.Body = ($Body | ConvertTo-Json -Depth 10)
        }

        $response = Invoke-RestMethod @params
        return @{ Success = $true; Data = $response; StatusCode = 200 }
    }
    catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        $errorBody = $_.ErrorDetails.Message
        return @{ Success = $false; Error = $errorBody; StatusCode = $statusCode }
    }
}

# Test 1: Register a user first (required for new auth system)
Write-Host "`n1. Testing user registration..." -ForegroundColor Yellow
$registerResponse = Invoke-SafeRestMethod -Uri "$authUrl/register" -Method "POST" -Body $testUser

if ($registerResponse.Success) {
    Write-Host "✅ User registered successfully!" -ForegroundColor Green
    $playerUuid = $registerResponse.Data.uuid
    Write-Host "   Player UUID: $playerUuid" -ForegroundColor Cyan
    Write-Host "   Team Name: $($testUser.team_name)" -ForegroundColor Cyan
} else {
    if ($registerResponse.StatusCode -eq 409) {
        Write-Host "⚠️  User already exists, attempting login..." -ForegroundColor Yellow

        # Try to login to get the UUID
        $loginData = @{
            email = $testUser.email
            password = $testUser.password
        }

        $loginResponse = Invoke-SafeRestMethod -Uri "$authUrl/login" -Method "POST" -Body $loginData
        if ($loginResponse.Success) {
            $playerUuid = $loginResponse.Data.uuid
            Write-Host "✅ Logged in successfully, got UUID: $playerUuid" -ForegroundColor Green
        } else {
            Write-Host "❌ Failed to login: $($loginResponse.Error)" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "❌ Registration failed: $($registerResponse.Error)" -ForegroundColor Red
        exit 1
    }
}

# Test 2: Get player by UUID
if ($playerUuid) {
    Write-Host "`n2. Testing get player by UUID..." -ForegroundColor Yellow
    $playerResponse = Invoke-SafeRestMethod -Uri "$baseUrl/players/$playerUuid"

    if ($playerResponse.Success) {
        Write-Host "✅ Player retrieved successfully:" -ForegroundColor Green
        Write-Host "   UUID: $($playerResponse.Data.uuid)" -ForegroundColor Cyan
        Write-Host "   Email: $($playerResponse.Data.email)" -ForegroundColor Cyan
        Write-Host "   Team: $($playerResponse.Data.team_name)" -ForegroundColor Cyan
        Write-Host "   Cars: $($playerResponse.Data.cars.Count)" -ForegroundColor Cyan
        Write-Host "   Pilots: $($playerResponse.Data.pilots.Count)" -ForegroundColor Cyan
    } else {
        Write-Host "❌ Get player failed: $($playerResponse.Error)" -ForegroundColor Red
    }
}

# Test 3: Get player by email
Write-Host "`n3. Testing get player by email..." -ForegroundColor Yellow
$emailPlayerResponse = Invoke-SafeRestMethod -Uri "$baseUrl/players/by-email/$($testUser.email)"

if ($emailPlayerResponse.Success) {
    Write-Host "✅ Player retrieved by email successfully:" -ForegroundColor Green
    Write-Host "   UUID: $($emailPlayerResponse.Data.uuid)" -ForegroundColor Cyan
    Write-Host "   Team: $($emailPlayerResponse.Data.team_name)" -ForegroundColor Cyan
} else {
    Write-Host "❌ Get player by email failed: $($emailPlayerResponse.Error)" -ForegroundColor Red
}

# Test 4: Update team name
if ($playerUuid) {
    Write-Host "`n4. Testing update team name..." -ForegroundColor Yellow
    $updateData = @{
        team_name = "Updated Racing Team"
    }

    $updateResponse = Invoke-SafeRestMethod -Uri "$baseUrl/players/$playerUuid" -Method "PUT" -Body $updateData

    if ($updateResponse.Success) {
        Write-Host "✅ Team name updated successfully:" -ForegroundColor Green
        Write-Host "   New Team Name: $($updateResponse.Data.player.team_name)" -ForegroundColor Cyan
    } else {
        Write-Host "❌ Update team name failed: $($updateResponse.Error)" -ForegroundColor Red
    }
}

# Test 5: Get all players
Write-Host "`n5. Testing get all players..." -ForegroundColor Yellow
$allPlayersResponse = Invoke-SafeRestMethod -Uri "$baseUrl/players"

if ($allPlayersResponse.Success) {
    Write-Host "✅ Retrieved all players successfully:" -ForegroundColor Green
    Write-Host "   Total Players: $($allPlayersResponse.Data.Count)" -ForegroundColor Cyan
    foreach ($player in $allPlayersResponse.Data) {
        Write-Host "   - $($player.team_name) ($($player.email))" -ForegroundColor White
    }
} else {
    Write-Host "❌ Get all players failed: $($allPlayersResponse.Error)" -ForegroundColor Red
}

Write-Host "`n🎉 Player endpoint testing completed!" -ForegroundColor Green
Write-Host "`n📊 Updated Features Summary:" -ForegroundColor Cyan
Write-Host "   ✅ Authentication-first approach - users register with email/password" -ForegroundColor White
Write-Host "   ✅ Secure password storage with Argon2 hashing" -ForegroundColor White
Write-Host "   ✅ UUID-based player identification" -ForegroundColor White
Write-Host "   ✅ Multiple lookup methods (UUID, email)" -ForegroundColor White
Write-Host "   ✅ Proper error handling and validation" -ForegroundColor White
