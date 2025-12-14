# Boost Card System Documentation

This folder contains documentation for the strategic boost card mechanics that add resource management to races.

## Contents

- `BOOST_CARD_API.md` - Complete API documentation for boost cards
- `BOOST_CARD_EXAMPLES.md` - Usage examples and patterns
- `openapi-boost-cards.yaml` - OpenAPI specification

## Implementation Details

See `implementation/` subfolder for:
- Boost availability endpoint implementation
- Boost calculation simplification

## Key Concepts

- **5-card hand system** (values 0-4)
- **Cycle-based replenishment** when all cards used
- **Performance multipliers** (8% per boost level)
- **Strategic resource management**

## Related Features

- [Racing System](../02-racing-system/) - Core race mechanics
- [Testing](../07-testing/) - Boost card integration tests
