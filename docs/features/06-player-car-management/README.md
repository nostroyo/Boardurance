# Player & Car Management Documentation

This folder contains documentation for player management, car systems, pilot mechanics, and performance calculations.

## Contents

See `implementation/` subfolder for:
- `CAR_PILOTS_UPDATE_SUMMARY.md` - Multi-pilot car system
- `PILOT_CREATION_IMPLEMENTATION.md` - Automatic pilot creation
- `PLAYER_CAR_PERFORMANCE_IMPLEMENTATION.md` - Performance calculations
- `PLAYER_GAME_CONTEXT_IMPLEMENTATION.md` - Player game state

## Key Concepts

### Car Components
- **Engine** - Speed and acceleration stats
- **Body** - Aerodynamics and handling
- **Pilot** - Skills and class bonuses

### Performance System
- Base value calculation from car components
- Lap characteristic effects (straight vs curve)
- Sector performance ceilings
- Boost multipliers

## Related Features

- [Racing System](../02-racing-system/) - How cars perform in races
- [NFT & Blockchain](../05-nft-blockchain/) - NFT car attributes
- [Testing](../07-testing/) - Player creation tests
