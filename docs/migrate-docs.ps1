# Documentation Migration Script
# Reorganizes docs folder into feature-based structure

Write-Host "Starting documentation reorganization..." -ForegroundColor Cyan

# Create implementation subfolders
$implementationFolders = @(
    "features/02-racing-system/implementation",
    "features/03-boost-card-system/implementation",
    "features/06-player-car-management/implementation"
)

foreach ($folder in $implementationFolders) {
    $path = Join-Path "docs" $folder
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Path $path -Force | Out-Null
        Write-Host "Created: $path" -ForegroundColor Green
    }
}

# Define file movements
$migrations = @(
    # Core Project
    @{Source="PROJECT_OVERVIEW.md"; Dest="features/01-core-project/PROJECT_OVERVIEW.md"},
    @{Source="QUICK_START.md"; Dest="features/01-core-project/QUICK_START.md"},
    @{Source="TECHNOLOGY_STACK.md"; Dest="features/01-core-project/TECHNOLOGY_STACK.md"},
    @{Source="DEVELOPMENT_WORKFLOW.md"; Dest="features/01-core-project/DEVELOPMENT_WORKFLOW.md"},
    @{Source="Main team.png"; Dest="features/01-core-project/Main team.png"},
    @{Source="Main team.pdn"; Dest="features/01-core-project/Main team.pdn"},
    
    # Racing System
    @{Source="GAME_MECHANICS.md"; Dest="features/02-racing-system/GAME_MECHANICS.md"},
    @{Source="API_ROUTES.md"; Dest="features/02-racing-system/API_ROUTES.md"},
    @{Source="MOVEABLE_ITEMS_IMPLEMENTATION.md"; Dest="features/02-racing-system/MOVEABLE_ITEMS_IMPLEMENTATION.md"},
    @{Source="implementation/TURN_CONTROLLER_IMPLEMENTATION.md"; Dest="features/02-racing-system/implementation/TURN_CONTROLLER_IMPLEMENTATION.md"},
    @{Source="implementation/TURN_PHASE_ENDPOINT_IMPLEMENTATION.md"; Dest="features/02-racing-system/implementation/TURN_PHASE_ENDPOINT_IMPLEMENTATION.md"},
    
    # Boost Card System
    @{Source="BOOST_CARD_API.md"; Dest="features/03-boost-card-system/BOOST_CARD_API.md"},
    @{Source="BOOST_CARD_EXAMPLES.md"; Dest="features/03-boost-card-system/BOOST_CARD_EXAMPLES.md"},
    @{Source="openapi-boost-cards.yaml"; Dest="features/03-boost-card-system/openapi-boost-cards.yaml"},
    @{Source="implementation/BOOST_AVAILABILITY_ENDPOINT_IMPLEMENTATION.md"; Dest="features/03-boost-card-system/implementation/BOOST_AVAILABILITY_ENDPOINT_IMPLEMENTATION.md"},
    @{Source="implementation/BOOST_CALCULATION_SIMPLIFICATION.md"; Dest="features/03-boost-card-system/implementation/BOOST_CALCULATION_SIMPLIFICATION.md"},
    
    # NFT & Blockchain
    @{Source="SOLANA_README.md"; Dest="features/05-nft-blockchain/SOLANA_README.md"},
    @{Source="SOLANA_DEPLOYMENT.md"; Dest="features/05-nft-blockchain/SOLANA_DEPLOYMENT.md"},
    @{Source="SOLANA_SIMPLE_DEPLOYMENT.md"; Dest="features/05-nft-blockchain/SOLANA_SIMPLE_DEPLOYMENT.md"},
    
    # Player & Car Management
    @{Source="implementation/CAR_PILOTS_UPDATE_SUMMARY.md"; Dest="features/06-player-car-management/implementation/CAR_PILOTS_UPDATE_SUMMARY.md"},
    @{Source="implementation/PILOT_CREATION_IMPLEMENTATION.md"; Dest="features/06-player-car-management/implementation/PILOT_CREATION_IMPLEMENTATION.md"},
    @{Source="implementation/PLAYER_CAR_PERFORMANCE_IMPLEMENTATION.md"; Dest="features/06-player-car-management/implementation/PLAYER_CAR_PERFORMANCE_IMPLEMENTATION.md"},
    @{Source="implementation/PLAYER_GAME_CONTEXT_IMPLEMENTATION.md"; Dest="features/06-player-car-management/implementation/PLAYER_GAME_CONTEXT_IMPLEMENTATION.md"},
    
    # Testing
    @{Source="TESTING_GUIDE.md"; Dest="features/07-testing/TESTING_GUIDE.md"},
    @{Source="testing/BACKEND_TEST_SUITE.md"; Dest="features/07-testing/BACKEND_TEST_SUITE.md"},
    @{Source="testing/BOOST_CARD_INTEGRATION_TESTS.md"; Dest="features/07-testing/BOOST_CARD_INTEGRATION_TESTS.md"},
    @{Source="testing/TEST_PLAYER_CREATION.md"; Dest="features/07-testing/TEST_PLAYER_CREATION.md"},
    @{Source="implementation/TEST_REORGANIZATION_SUMMARY.md"; Dest="features/07-testing/TEST_REORGANIZATION_SUMMARY.md"},
    
    # Architecture
    @{Source="architecture/FRONTEND_BACKEND_SEPARATION.md"; Dest="features/08-architecture/FRONTEND_BACKEND_SEPARATION.md"},
    
    # Backend
    @{Source="BACKEND_README.md"; Dest="features/09-backend/BACKEND_README.md"},
    @{Source="BACKEND_DOCKER_SETUP.md"; Dest="features/09-backend/BACKEND_DOCKER_SETUP.md"},
    
    # Frontend
    @{Source="FRONTEND_README.md"; Dest="features/10-frontend/FRONTEND_README.md"},
    @{Source="UI_IMPROVEMENTS.md"; Dest="features/10-frontend/UI_IMPROVEMENTS.md"}
)

# Perform migrations
$successCount = 0
$errorCount = 0

foreach ($migration in $migrations) {
    $sourcePath = Join-Path "docs" $migration.Source
    $destPath = Join-Path "docs" $migration.Dest
    
    if (Test-Path $sourcePath) {
        try {
            Move-Item -Path $sourcePath -Destination $destPath -Force
            Write-Host "✓ Moved: $($migration.Source) → $($migration.Dest)" -ForegroundColor Green
            $successCount++
        }
        catch {
            Write-Host "✗ Error moving $($migration.Source): $_" -ForegroundColor Red
            $errorCount++
        }
    }
    else {
        Write-Host "⚠ Source not found: $($migration.Source)" -ForegroundColor Yellow
    }
}

Write-Host "`nMigration Summary:" -ForegroundColor Cyan
Write-Host "  Successfully moved: $successCount files" -ForegroundColor Green
Write-Host "  Errors: $errorCount files" -ForegroundColor $(if ($errorCount -gt 0) { "Red" } else { "Green" })

# Check for empty old folders
Write-Host "`nChecking for empty folders to remove..." -ForegroundColor Cyan

$oldFolders = @("docs/implementation", "docs/testing", "docs/architecture")
foreach ($folder in $oldFolders) {
    if (Test-Path $folder) {
        $items = Get-ChildItem -Path $folder -Recurse
        if ($items.Count -eq 0) {
            Remove-Item -Path $folder -Recurse -Force
            Write-Host "✓ Removed empty folder: $folder" -ForegroundColor Green
        }
        else {
            Write-Host "⚠ Folder not empty: $folder (contains $($items.Count) items)" -ForegroundColor Yellow
        }
    }
}

Write-Host "`nReorganization complete!" -ForegroundColor Cyan
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Review the new structure in docs/features/" -ForegroundColor White
Write-Host "  2. Update docs/README.md with new structure" -ForegroundColor White
Write-Host "  3. Verify all documentation links still work" -ForegroundColor White
