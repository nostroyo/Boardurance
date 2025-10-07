# Test script for UUID-based Player endpoints
Write-Host "🎮 Testing UUID-based Player Management Endpoints..." -ForegroundColor Green

$baseUrl = "http://localhost:3000"
$testWallet = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"
$playerUuid = $null

# Test 1: Create a new player without wallet (guest player)
Write-Host "`n1. Testing create guest player (no wallet)..." -ForegroundColor Yellow
try {
    $playerData = @{
        team_name = "Guest Racers Team"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players" -Method POST -Body $playerData -ContentType "application/json" -TimeoutSec 10
    $playerUuid = $response.player.uuid
    Write-Host "✅ Guest player created successfully:" -ForegroundColor Green
    Write-Host "   UUID: $($response.player.uuid)" -ForegroundColor Cyan
    Write-Host "   Team: $($response.player.team_name)" -ForegroundColor Cyan
    Write-Host "   Wallet: $($response.player.wallet_address)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Create guest player failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 2: Connect wallet to the player
if ($playerUuid) {
    Write-Host "`n2. Testing connect wallet to player..." -ForegroundColor Yellow
    try {
        $walletData = @{
            wallet_address = $testWallet
        } | ConvertTo-Json

        $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$playerUuid/wallet" -Method POST -Body $walletData -ContentType "application/json" -TimeoutSec 10
        Write-Host "✅ Wallet connected successfully:" -ForegroundColor Green
        Write-Host "   Connected Wallet: $($response.player.wallet_address)" -ForegroundColor Cyan
    } catch {
        if ($_.Exception.Response.StatusCode -eq 409) {
            Write-Host "⚠️  Wallet already connected to another player" -ForegroundColor Yellow
        } else {
            Write-Host "❌ Connect wallet failed: $($_.Exception.Message)" -ForegroundColor Red
        }
    }
}

# Test 3: Get player by UUID
if ($playerUuid) {
    Write-Host "`n3. Testing get player by UUID..." -ForegroundColor Yellow
    try {
        $player = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$playerUuid" -Method GET -TimeoutSec 10
        Write-Host "✅ Player retrieved by UUID successfully:" -ForegroundColor Green
        Write-Host "   UUID: $($player.uuid)" -ForegroundColor Cyan
        Write-Host "   Team: $($player.team_name)" -ForegroundColor Cyan
        Write-Host "   Wallet: $($player.wallet_address)" -ForegroundColor Cyan
        Write-Host "   Cars: $($player.cars.Count)" -ForegroundColor Cyan
        Write-Host "   Pilots: $($player.pilots.Count)" -ForegroundColor Cyan
    } catch {
        Write-Host "❌ Get player by UUID failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 4: Get player by wallet address
Write-Host "`n4. Testing get player by wallet address..." -ForegroundColor Yellow
try {
    $player = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/by-wallet/$testWallet" -Method GET -TimeoutSec 10
    Write-Host "✅ Player retrieved by wallet successfully:" -ForegroundColor Green
    Write-Host "   UUID: $($player.uuid)" -ForegroundColor Cyan
    Write-Host "   Team: $($player.team_name)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Get player by wallet failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Add a car to the player
if ($playerUuid) {
    Write-Host "`n5. Testing add car to player..." -ForegroundColor Yellow
    try {
        $carData = @{
            name = "UUID Lightning Bolt"
            car_type = "Sports"
            rarity = "Rare"
            stats = @{
                speed = 85
                acceleration = 80
                handling = 75
                durability = 70
            }
            nft_mint_address = "CarNFT123UUIDTest"
        } | ConvertTo-Json -Depth 3

        $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$playerUuid/cars" -Method POST -Body $carData -ContentType "application/json" -TimeoutSec 10
        Write-Host "✅ Car added successfully:" -ForegroundColor Green
        Write-Host "   Car Name: $($response.player.cars[-1].name)" -ForegroundColor Cyan
        Write-Host "   Car Type: $($response.player.cars[-1].car_type)" -ForegroundColor Cyan
        Write-Host "   Total Cars: $($response.player.cars.Count)" -ForegroundColor Cyan
    } catch {
        Write-Host "❌ Add car failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 6: Add a pilot to the player
if ($playerUuid) {
    Write-Host "`n6. Testing add pilot to player..." -ForegroundColor Yellow
    try {
        $pilotData = @{
            name = "UUID Speed Racer"
            pilot_class = "Speedster"
            rarity = "Professional"
            skills = @{
                reaction_time = 85
                precision = 70
                focus = 80
                stamina = 75
            }
            nft_mint_address = "PilotNFT456UUIDTest"
        } | ConvertTo-Json -Depth 3

        $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$playerUuid/pilots" -Method POST -Body $pilotData -ContentType "application/json" -TimeoutSec 10
        Write-Host "✅ Pilot added successfully:" -ForegroundColor Green
        Write-Host "   Pilot Name: $($response.player.pilots[-1].name)" -ForegroundColor Cyan
        Write-Host "   Pilot Class: $($response.player.pilots[-1].pilot_class)" -ForegroundColor Cyan
        Write-Host "   Total Pilots: $($response.player.pilots.Count)" -ForegroundColor Cyan
    } catch {
        Write-Host "❌ Add pilot failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 7: Update team name
if ($playerUuid) {
    Write-Host "`n7. Testing update team name..." -ForegroundColor Yellow
    try {
        $updateData = @{
            team_name = "UUID Racing Team Updated"
        } | ConvertTo-Json

        $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$playerUuid" -Method PUT -Body $updateData -ContentType "application/json" -TimeoutSec 10
        Write-Host "✅ Team name updated successfully:" -ForegroundColor Green
        Write-Host "   New Team Name: $($response.player.team_name)" -ForegroundColor Cyan
    } catch {
        Write-Host "❌ Update team name failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 8: Disconnect wallet
if ($playerUuid) {
    Write-Host "`n8. Testing disconnect wallet..." -ForegroundColor Yellow
    try {
        $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$playerUuid/wallet" -Method DELETE -TimeoutSec 10
        Write-Host "✅ Wallet disconnected successfully:" -ForegroundColor Green
        Write-Host "   Wallet Address: $($response.player.wallet_address)" -ForegroundColor Cyan
    } catch {
        Write-Host "❌ Disconnect wallet failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 9: Create player with wallet directly
Write-Host "`n9. Testing create player with wallet..." -ForegroundColor Yellow
try {
    $playerWithWalletData = @{
        wallet_address = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
        team_name = "Direct Wallet Team"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players" -Method POST -Body $playerWithWalletData -ContentType "application/json" -TimeoutSec 10
    Write-Host "✅ Player with wallet created successfully:" -ForegroundColor Green
    Write-Host "   UUID: $($response.player.uuid)" -ForegroundColor Cyan
    Write-Host "   Team: $($response.player.team_name)" -ForegroundColor Cyan
    Write-Host "   Wallet: $($response.player.wallet_address)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Create player with wallet failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n🎉 UUID-based player endpoint testing completed!" -ForegroundColor Green
Write-Host "`n📊 New Features Summary:" -ForegroundColor Cyan
Write-Host "   ✅ Optional wallet addresses - players can be created without wallets" -ForegroundColor White
Write-Host "   ✅ UUID-based identification - primary key is now UUID, not wallet" -ForegroundColor White
Write-Host "   ✅ Wallet connection/disconnection - players can connect/disconnect wallets" -ForegroundColor White
Write-Host "   ✅ Guest player support - players can play without Web3 wallet initially" -ForegroundColor White
Write-Host "   ✅ Flexible onboarding - traditional signup → wallet connection later" -ForegroundColor White
Write-Host "   ✅ Backward compatibility - still supports wallet-based lookups" -ForegroundColor White