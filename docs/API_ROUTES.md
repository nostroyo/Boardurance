# Rust Backend API Routes Documentation

This document provides a comprehensive overview of all API routes exposed by the Rust backend service.

## Base Configuration

- **Base URL**: `http://localhost:8000` (configurable)
- **API Version**: `v1`
- **Content-Type**: `application/json`
- **Documentation**: Available at `/swagger-ui`
- **OpenAPI Spec**: Available at `/api-docs/openapi.json`

## Health Check

### GET /health_check
Check service health and database connectivity.

**Response:**
```json
{
  "status": "ok|degraded",
  "message": "Service is healthy and database is connected"
}
```

## Test Endpoints

### POST /api/v1/test
Create a test item for development/testing purposes.

**Request Body:**
```json
{
  "name": "string",
  "description": "string (optional)"
}
```

### GET /api/v1/test
Get all test items.

**Response:** Array of test items

## Player Management

### POST /api/v1/players
Create a new player with starter assets.

**Request Body:**
```json
{
  "email": "string",
  "team_name": "string"
}
```

**Creates:**
- Player with email and team name
- 2 empty cars ("Car 1", "Car 2")
- 2 basic engines with different stats
- 2 basic bodies with different stats

### GET /api/v1/players
Get all players.

**Response:** Array of player objects

### GET /api/v1/players/{player_uuid}
Get player by UUID.

**Parameters:**
- `player_uuid`: Player's unique identifier

### GET /api/v1/players/by-wallet/{wallet_address}
Get player by wallet address.

**Parameters:**
- `wallet_address`: Player's Solana wallet address

### GET /api/v1/players/by-email/{email}
Get player by email address.

**Parameters:**
- `email`: Player's email address

### PUT /api/v1/players/{player_uuid}
Update player's team name.

**Request Body:**
```json
{
  "team_name": "string"
}
```

### DELETE /api/v1/players/{player_uuid}
Delete a player.

**Parameters:**
- `player_uuid`: Player's unique identifier

### POST /api/v1/players/{player_uuid}/wallet
Connect wallet to player.

**Request Body:**
```json
{
  "wallet_address": "string"
}
```

### DELETE /api/v1/players/{player_uuid}/wallet
Disconnect wallet from player.

### POST /api/v1/players/{player_uuid}/cars
Add a car to player.

**Request Body:**
```json
{
  "name": "string",
  "nft_mint_address": "string (optional)"
}
```

### DELETE /api/v1/players/{player_uuid}/cars/{car_uuid}
Remove a car from player.

**Parameters:**
- `player_uuid`: Player's unique identifier
- `car_uuid`: Car's unique identifier

### POST /api/v1/players/{player_uuid}/pilots
Add a pilot to player.

**Request Body:**
```json
{
  "name": "string",
  "pilot_class": "Rookie|Veteran|Elite|Champion",
  "rarity": "Common|Uncommon|Rare|Epic|Legendary",
  "skills": {
    "reaction_time": "number (0-100)",
    "precision": "number (0-100)",
    "focus": "number (0-100)",
    "stamina": "number (0-100)"
  },
  "nft_mint_address": "string (optional)"
}
```

### DELETE /api/v1/players/{player_uuid}/pilots/{pilot_uuid}
Remove a pilot from player.

**Parameters:**
- `player_uuid`: Player's unique identifier
- `pilot_uuid`: Pilot's unique identifier

## Race Management

### POST /api/v1/races
Create a new race.

**Request Body:**
```json
{
  "name": "string",
  "track_name": "string",
  "total_laps": "number",
  "sectors": [
    {
      "id": "number",
      "name": "string",
      "min_value": "number",
      "max_value": "number",
      "slot_capacity": "number (optional)",
      "sector_type": "Straight|Curve"
    }
  ]
}
```

### GET /api/v1/races
Get all races.

**Response:** Array of race objects

### GET /api/v1/races/{race_uuid}
Get race by UUID.

**Parameters:**
- `race_uuid`: Race's unique identifier

### POST /api/v1/races/{race_uuid}/join
Join a race with a player, car, and pilot.

**Request Body:**
```json
{
  "player_uuid": "string",
  "car_uuid": "string",
  "pilot_uuid": "string"
}
```

### POST /api/v1/races/{race_uuid}/start
Start a race (must have participants).

### POST /api/v1/races/{race_uuid}/turn
Process a turn/lap in the race.

**Request Body:**
```json
{
  "actions": [
    {
      "player_uuid": "string",
      "boost_value": "number"
    }
  ]
}
```

**Response:**
```json
{
  "result": {
    "lap_number": "number",
    "movements": [
      {
        "player_uuid": "string",
        "movement_type": "Forward|Backward|Stay",
        "sectors_moved": "number",
        "new_position": "number"
      }
    ]
  },
  "race_status": "Waiting|InProgress|Finished"
}
```

### GET /api/v1/races/{race_uuid}/status
Get current race status.

**Response:**
```json
"Waiting|InProgress|Finished"
```

## Data Models

### Player Object
```json
{
  "uuid": "string",
  "email": "string",
  "team_name": "string",
  "wallet_address": "string (optional)",
  "cars": ["Car objects"],
  "pilots": ["Pilot objects"],
  "engines": ["Engine objects"],
  "bodies": ["Body objects"],
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

### Car Object
```json
{
  "uuid": "string",
  "name": "string",
  "nft_mint_address": "string (optional)",
  "pilot_uuid": "string (optional)",
  "engine_uuid": "string (optional)",
  "body_uuid": "string (optional)"
}
```

### Pilot Object
```json
{
  "uuid": "string",
  "name": "string",
  "pilot_class": "Rookie|Veteran|Elite|Champion",
  "rarity": "Common|Uncommon|Rare|Epic|Legendary",
  "skills": {
    "reaction_time": "number",
    "precision": "number",
    "focus": "number",
    "stamina": "number"
  },
  "performance": {
    "straight_value": "number",
    "curve_value": "number"
  },
  "nft_mint_address": "string (optional)"
}
```

### Engine Object
```json
{
  "uuid": "string",
  "name": "string",
  "rarity": "Common|Uncommon|Rare|Epic|Legendary",
  "straight_value": "number",
  "curve_value": "number",
  "nft_mint_address": "string (optional)"
}
```

### Body Object
```json
{
  "uuid": "string",
  "name": "string",
  "rarity": "Common|Uncommon|Rare|Epic|Legendary",
  "straight_value": "number",
  "curve_value": "number",
  "nft_mint_address": "string (optional)"
}
```

### Race Object
```json
{
  "uuid": "string",
  "name": "string",
  "track": {
    "name": "string",
    "sectors": ["Sector objects"]
  },
  "total_laps": "number",
  "current_lap": "number",
  "participants": ["RaceParticipant objects"],
  "status": "Waiting|InProgress|Finished",
  "lap_characteristic": "Straight|Curve",
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

## HTTP Status Codes

- **200 OK**: Successful GET, PUT, DELETE operations
- **201 Created**: Successful POST operations
- **400 Bad Request**: Invalid request data or parameters
- **404 Not Found**: Resource not found
- **409 Conflict**: Resource already exists or invalid state transition
- **500 Internal Server Error**: Server-side error

## Error Handling

All endpoints return appropriate HTTP status codes. Error responses typically include:
- Status code indicating the type of error
- Error message in the response body (when applicable)
- Detailed logging for debugging purposes

## Authentication & Authorization

Currently, the API does not implement authentication or authorization. All endpoints are publicly accessible.

## CORS

The API is configured with permissive CORS settings for development purposes.

## Logging & Tracing

All endpoints include comprehensive logging and distributed tracing for monitoring and debugging.