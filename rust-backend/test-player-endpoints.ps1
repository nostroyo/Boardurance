# Test script for Player endpoints
Write-Host "🎮 Testing Player Management Endpoints..." -ForegroundColor Green

$baseUrl = "http://localhost:3000"
$testWallet = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"

# Test 1: Create a new player
Write-Host "`n1. Testing create player..." -ForegroundColor Yellow
try {
    $playerData = @{
        wallet_address = $testWallet
        team_name = "PowerShell Racers"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players" -Method POST -Body $playerData -ContentType "application/json" -TimeoutSec 10
    Write-Host "✅ Player created successfully:" -ForegroundColor Green
    Write-Host "   UUID: $($response.player.uuid)" -ForegroundColor Cyan
    Write-Host "   Team: $($response.player.team_name)" -ForegroundColor Cyan
    Write-Host "   Wallet: $($response.player.wallet_address)" -ForegroundColor Cyan
} catch {
    if ($_.Exception.Response.StatusCode -eq 409) {
        Write-Host "⚠️  Player already exists (expected if running multiple times)" -ForegroundColor Yellow
    } else {
        Write-Host "❌ Create player failed: $($_.Exception.Message)" -ForegroundColor Red
    }
}

# Test 2: Get player by wallet address
Write-Host "`n2. Testing get player by wallet..." -ForegroundColor Yellow
try {
    $player = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$testWallet" -Method GET -TimeoutSec 10
    Write-Host "✅ Player retrieved successfully:" -ForegroundColor Green
    Write-Host "   Team: $($player.team_name)" -ForegroundColor Cyan
    Write-Host "   Cars: $($player.cars.Count)" -ForegroundColor Cyan
    Write-Host "   Pilots: $($player.pilots.Count)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Get player failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Add a car to the player
Write-Host "`n3. Testing add car to player..." -ForegroundColor Yellow
try {
    $carData = @{
        name = "Lightning Bolt"
        car_type = "Sports"
        rarity = "Rare"
        stats = @{
            speed = 85
            acceleration = 80
            handling = 75
            durability = 70
        }
        nft_mint_address = "CarNFT123TestPS"
    } | ConvertTo-Json -Depth 3

    $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$testWallet/cars" -Method POST -Body $carData -ContentType "application/json" -TimeoutSec 10
    Write-Host "✅ Car added successfully:" -ForegroundColor Green
    Write-Host "   Car Name: $($response.player.cars[-1].name)" -ForegroundColor Cyan
    Write-Host "   Car Type: $($response.player.cars[-1].car_type)" -ForegroundColor Cyan
    Write-Host "   Total Cars: $($response.player.cars.Count)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Add car failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Add a pilot to the player
Write-Host "`n4. Testing add pilot to player..." -ForegroundColor Yellow
try {
    $pilotData = @{
        name = "Speed Racer"
        pilot_class = "Speedster"
        rarity = "Professional"
        skills = @{
            reaction_time = 85
            precision = 70
            focus = 80
            stamina = 75
        }
        nft_mint_address = "PilotNFT456TestPS"
    } | ConvertTo-Json -Depth 3

    $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$testWallet/pilots" -Method POST -Body $pilotData -ContentType "application/json" -TimeoutSec 10
    Write-Host "✅ Pilot added successfully:" -ForegroundColor Green
    Write-Host "   Pilot Name: $($response.player.pilots[-1].name)" -ForegroundColor Cyan
    Write-Host "   Pilot Class: $($response.player.pilots[-1].pilot_class)" -ForegroundColor Cyan
    Write-Host "   Total Pilots: $($response.player.pilots.Count)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Add pilot failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Update team name
Write-Host "`n5. Testing update team name..." -ForegroundColor Yellow
try {
    $updateData = @{
        team_name = "PowerShell Racing Team Updated"
    } | ConvertTo-Json

    $response = Invoke-RestMethod -Uri "$baseUrl/api/v1/players/$testWallet" -Method PUT -Body $updateData -ContentType "application/json" -TimeoutSec 10
    Write-Host "✅ Team name updated successfully:" -ForegroundColor Green
    Write-Host "   New Team Name: $($response.player.team_name)" -ForegroundColor Cyan
} catch {
    Write-Host "❌ Update team name failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 6: Get all players
Write-Host "`n6. Testing get all players..." -ForegroundColor Yellow
try {
    $players = Invoke-RestMethod -Uri "$baseUrl/api/v1/players" -Method GET -TimeoutSec 10
    Write-Host "✅ Retrieved all players successfully:" -ForegroundColor Green
    Write-Host "   Total Players: $($players.Count)" -ForegroundColor Cyan
    foreach ($player in $players) {
        Write-Host "   - $($player.team_name) ($($player.wallet_address))" -ForegroundColor White
    }
} catch {
    Write-Host "❌ Get all players failed: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`n🎉 Player endpoint testing completed!" -ForegroundColor Green
Write-Host "`n📊 Game Data Model Summary:" -ForegroundColor Cyan
Write-Host "   - Players have wallet addresses and team names" -ForegroundColor White
Write-Host "   - Players can have up to 2 cars with different stats and rarities" -ForegroundColor White
Write-Host "   - Players can have multiple pilots with different classes and skills" -ForegroundColor White
Write-Host "   - Cars and pilots can be linked to Solana NFTs" -ForegroundColor White
Write-Host "   - All data follows domain-driven design patterns" -ForegroundColor White