# Documentation Reorganization Plan

## Overview

This document outlines the reorganization of the docs folder from a flat structure to a feature-based hierarchy.

## New Structure

```
docs/
├── README.md (updated)
├── features/
│   ├── 01-core-project/
│   │   ├── README.md
│   │   ├── PROJECT_OVERVIEW.md
│   │   ├── QUICK_START.md
│   │   ├── TECHNOLOGY_STACK.md
│   │   ├── DEVELOPMENT_WORKFLOW.md
│   │   ├── Main team.png
│   │   └── Main team.pdn
│   ├── 02-racing-system/
│   │   ├── README.md
│   │   ├── GAME_MECHANICS.md
│   │   ├── API_ROUTES.md
│   │   ├── MOVEABLE_ITEMS_IMPLEMENTATION.md
│   │   └── implementation/
│   │       ├── TURN_CONTROLLER_IMPLEMENTATION.md
│   │       └── TURN_PHASE_ENDPOINT_IMPLEMENTATION.md
│   ├── 03-boost-card-system/
│   │   ├── README.md
│   │   ├── BOOST_CARD_API.md
│   │   ├── BOOST_CARD_EXAMPLES.md
│   │   ├── openapi-boost-cards.yaml
│   │   └── implementation/
│   │       ├── BOOST_AVAILABILITY_ENDPOINT_IMPLEMENTATION.md
│   │       └── BOOST_CALCULATION_SIMPLIFICATION.md
│   ├── 04-authentication/
│   │   └── README.md
│   ├── 05-nft-blockchain/
│   │   ├── README.md
│   │   ├── SOLANA_README.md
│   │   ├── SOLANA_DEPLOYMENT.md
│   │   └── SOLANA_SIMPLE_DEPLOYMENT.md
│   ├── 06-player-car-management/
│   │   ├── README.md
│   │   └── implementation/
│   │       ├── CAR_PILOTS_UPDATE_SUMMARY.md
│   │       ├── PILOT_CREATION_IMPLEMENTATION.md
│   │       ├── PLAYER_CAR_PERFORMANCE_IMPLEMENTATION.md
│   │       └── PLAYER_GAME_CONTEXT_IMPLEMENTATION.md
│   ├── 07-testing/
│   │   ├── README.md
│   │   ├── TESTING_GUIDE.md
│   │   ├── BACKEND_TEST_SUITE.md
│   │   ├── BOOST_CARD_INTEGRATION_TESTS.md
│   │   ├── TEST_PLAYER_CREATION.md
│   │   └── TEST_REORGANIZATION_SUMMARY.md
│   ├── 08-architecture/
│   │   ├── README.md
│   │   └── FRONTEND_BACKEND_SEPARATION.md
│   ├── 09-backend/
│   │   ├── README.md
│   │   ├── BACKEND_README.md
│   │   └── BACKEND_DOCKER_SETUP.md
│   └── 10-frontend/
│       ├── README.md
│       ├── FRONTEND_README.md
│       └── UI_IMPROVEMENTS.md
└── REORGANIZATION_PLAN.md (this file)
```

## Migration Steps

### Phase 1: Create New Structure (DONE)
- ✅ Created feature folders with README files
- ✅ Created implementation subfolders where needed

### Phase 2: Move Files (NEXT)
Execute the PowerShell migration script to move files to new locations.

### Phase 3: Update References
- Update cross-references in documentation files
- Update README.md with new structure
- Update any scripts that reference old paths

### Phase 4: Cleanup
- Remove old empty folders
- Verify all files moved correctly
- Test documentation links

## Benefits

1. **Feature-Based Organization** - Related docs grouped together
2. **Clear Navigation** - Each feature has its own README
3. **Scalability** - Easy to add new features
4. **Discoverability** - Logical structure for finding information
5. **Maintainability** - Clear ownership and organization

## File Mapping

### Core Project (01-core-project/)
- PROJECT_OVERVIEW.md
- QUICK_START.md
- TECHNOLOGY_STACK.md
- DEVELOPMENT_WORKFLOW.md
- Main team.png
- Main team.pdn

### Racing System (02-racing-system/)
- GAME_MECHANICS.md
- API_ROUTES.md
- MOVEABLE_ITEMS_IMPLEMENTATION.md
- implementation/TURN_CONTROLLER_IMPLEMENTATION.md
- implementation/TURN_PHASE_ENDPOINT_IMPLEMENTATION.md

### Boost Card System (03-boost-card-system/)
- BOOST_CARD_API.md
- BOOST_CARD_EXAMPLES.md
- openapi-boost-cards.yaml
- implementation/BOOST_AVAILABILITY_ENDPOINT_IMPLEMENTATION.md
- implementation/BOOST_CALCULATION_SIMPLIFICATION.md

### NFT & Blockchain (05-nft-blockchain/)
- SOLANA_README.md
- SOLANA_DEPLOYMENT.md
- SOLANA_SIMPLE_DEPLOYMENT.md

### Player & Car Management (06-player-car-management/)
- implementation/CAR_PILOTS_UPDATE_SUMMARY.md
- implementation/PILOT_CREATION_IMPLEMENTATION.md
- implementation/PLAYER_CAR_PERFORMANCE_IMPLEMENTATION.md
- implementation/PLAYER_GAME_CONTEXT_IMPLEMENTATION.md

### Testing (07-testing/)
- TESTING_GUIDE.md
- testing/BACKEND_TEST_SUITE.md
- testing/BOOST_CARD_INTEGRATION_TESTS.md
- testing/TEST_PLAYER_CREATION.md
- implementation/TEST_REORGANIZATION_SUMMARY.md

### Architecture (08-architecture/)
- architecture/FRONTEND_BACKEND_SEPARATION.md

### Backend (09-backend/)
- BACKEND_README.md
- BACKEND_DOCKER_SETUP.md

### Frontend (10-frontend/)
- FRONTEND_README.md
- UI_IMPROVEMENTS.md

## Notes

- Numbered prefixes (01-, 02-, etc.) ensure consistent ordering
- Implementation subfolders keep detailed docs organized
- Each feature has a README for navigation
- Old structure preserved until migration verified
