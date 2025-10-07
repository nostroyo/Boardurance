# Player Data System Documentation

## Overview

The Player Data System implements a comprehensive game backend for managing players, their teams, cars, and pilots in a Web3 racing game. The system follows Domain-Driven Design principles and Luca Palmieri's patterns from "Zero to Production in Rust".

## Domain Models

### Player
The core entity representing a game player.

```rust
pub struct Player {
    pub id: Option<ObjectId>,           // MongoDB document ID
    pub uuid: Uuid,                     // Unique identifier
    pub wallet_address: WalletAddress,  // Solana wallet address
    pub team_name: TeamName,            // Player's team name
    pub cars: Vec<Car>,                 // Player's cars (max 2)
    pub pilots: Vec<Pilot>,             // Player's pilots
    pub created_at: DateTime<Utc>,      // Creation timestamp
    pub updated_at: DateTime<Utc>,      // Last update timestamp
}
```

**Business Rules:**
- Each player must have a unique wallet address
- Players can have up to 2 cars maximum
- Players must have at least 1 pilot to participate in games
- Team names must be 2-50 characters, no forbidden characters

### Car
Represents a racing car with stats and rarity.

```rust
pub struct Car {
    pub uuid: Uuid,                     // Unique identifier
    pub nft_mint_address: Option<String>, // Solana NFT mint address
    pub name: CarName,                  // Car display name
    pub car_type: CarType,              // Sports, Racing, Luxury, Electric, Vintage
    pub rarity: CarRarity,              // Common, Uncommon, Rare, Epic, Legendary
    pub stats: CarStats,                // Performance statistics
    pub is_equipped: bool,              // Whether car is currently equipped
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Car Types:**
- `Sports` - Balanced performance cars
- `Racing` - High-speed track cars
- `Luxury` - Premium comfort cars
- `Electric` - Eco-friendly high-tech cars
- `Vintage` - Classic collectible cars

**Car Rarities & Stat Limits:**
- `Common` - Max stats: 70, Multiplier: 1.0x
- `Uncommon` - Max stats: 75, Multiplier: 1.1x
- `Rare` - Max stats: 85, Multiplier: 1.25x
- `Epic` - Max stats: 95, Multiplier: 1.5x
- `Legendary` - Max stats: 100, Multiplier: 2.0x

**Car Stats (1-100 each):**
- `Speed` - Maximum velocity
- `Acceleration` - How quickly the car reaches top speed
- `Handling` - Cornering and maneuverability
- `Durability` - Resistance to damage and wear

### Pilot
Represents a racing pilot with skills and experience.

```rust
pub struct Pilot {
    pub uuid: Uuid,                     // Unique identifier
    pub nft_mint_address: Option<String>, // Solana NFT mint address
    pub name: PilotName,                // Pilot display name
    pub pilot_class: PilotClass,        // Specialization class
    pub rarity: PilotRarity,            // Skill tier
    pub skills: PilotSkills,            // Pilot abilities
    pub experience_level: u32,          // Experience points
    pub is_active: bool,                // Whether pilot is currently active
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Pilot Classes & Bonuses:**
- `Speedster` - Speed +15, Acceleration +20
- `Technician` - Acceleration +5, Handling +25, Durability +10
- `Endurance` - Handling +10, Durability +30
- `AllRounder` - All stats +8

**Pilot Rarities & Skill Limits:**
- `Rookie` - Max skills: 60, Multiplier: 1.0x, XP: 1.0x
- `Professional` - Max skills: 70, Multiplier: 1.15x, XP: 1.2x
- `Expert` - Max skills: 80, Multiplier: 1.3x, XP: 1.4x
- `Champion` - Max skills: 90, Multiplier: 1.5x, XP: 1.6x
- `Legend` - Max skills: 100, Multiplier: 1.8x, XP: 2.0x

**Pilot Skills (1-100 each):**
- `Reaction Time` - Affects acceleration performance
- `Precision` - Affects handling accuracy
- `Focus` - Affects consistency and error reduction
- `Stamina` - Affects performance over long races

## API Endpoints

### Player Management

#### Create Player
```http
POST /api/v1/players
Content-Type: application/json

{
  "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "team_name": "Lightning Racers"
}
```

#### Get All Players
```http
GET /api/v1/players
```

#### Get Player by Wallet
```http
GET /api/v1/players/{wallet_address}
```

#### Update Team Name
```http
PUT /api/v1/players/{wallet_address}
Content-Type: application/json

{
  "team_name": "New Team Name"
}
```

#### Delete Player
```http
DELETE /api/v1/players/{wallet_address}
```

### Car Management

#### Add Car to Player
```http
POST /api/v1/players/{wallet_address}/cars
Content-Type: application/json

{
  "name": "Lightning Bolt",
  "car_type": "Sports",
  "rarity": "Rare",
  "stats": {
    "speed": 85,
    "acceleration": 80,
    "handling": 75,
    "durability": 70
  },
  "nft_mint_address": "CarNFT123456789"
}
```

#### Remove Car from Player
```http
DELETE /api/v1/players/{wallet_address}/cars/{car_uuid}
```

### Pilot Management

#### Add Pilot to Player
```http
POST /api/v1/players/{wallet_address}/pilots
Content-Type: application/json

{
  "name": "Speed Racer",
  "pilot_class": "Speedster",
  "rarity": "Professional",
  "skills": {
    "reaction_time": 85,
    "precision": 70,
    "focus": 80,
    "stamina": 75
  },
  "nft_mint_address": "PilotNFT123456789"
}
```

#### Remove Pilot from Player
```http
DELETE /api/v1/players/{wallet_address}/pilots/{pilot_uuid}
```

## Database Schema

### MongoDB Collections

#### Players Collection
```javascript
{
  _id: ObjectId,
  uuid: "550e8400-e29b-41d4-a716-446655440010",
  wallet_address: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  team_name: "Lightning Racers",
  cars: [
    {
      uuid: "550e8400-e29b-41d4-a716-446655440011",
      nft_mint_address: "CarNFT123456789",
      name: "Thunder Bolt",
      car_type: "Sports",
      rarity: "Rare",
      stats: {
        speed: 85,
        acceleration: 80,
        handling: 75,
        durability: 70
      },
      is_equipped: true,
      created_at: ISODate,
      updated_at: ISODate
    }
  ],
  pilots: [
    {
      uuid: "550e8400-e29b-41d4-a716-446655440013",
      nft_mint_address: "PilotNFT111222333",
      name: "Alex Thunder",
      pilot_class: "Speedster",
      rarity: "Professional",
      skills: {
        reaction_time: 85,
        precision: 70,
        focus: 80,
        stamina: 75
      },
      experience_level: 15,
      is_active: true,
      created_at: ISODate,
      updated_at: ISODate
    }
  ],
  created_at: ISODate,
  updated_at: ISODate
}
```

### Database Indexes
```javascript
// Players collection indexes
db.players.createIndex({ "wallet_address": 1 }, { unique: true });
db.players.createIndex({ "uuid": 1 }, { unique: true });
db.players.createIndex({ "team_name": 1 });
db.players.createIndex({ "created_at": 1 });
db.players.createIndex({ "cars.uuid": 1 });
db.players.createIndex({ "pilots.uuid": 1 });
```

## Game Mechanics Integration

### Performance Calculation
```rust
// Car overall rating
fn calculate_car_rating(car: &Car) -> u8 {
    let base_rating = (car.stats.speed + car.stats.acceleration + 
                      car.stats.handling + car.stats.durability) / 4;
    let rarity_multiplier = car.rarity.get_stat_multiplier();
    (base_rating as f32 * rarity_multiplier) as u8
}

// Pilot overall skill
fn calculate_pilot_skill(pilot: &Pilot) -> u8 {
    let base_skill = (pilot.skills.reaction_time + pilot.skills.precision + 
                     pilot.skills.focus + pilot.skills.stamina) / 4;
    let rarity_multiplier = pilot.rarity.get_skill_multiplier();
    (base_skill as f32 * rarity_multiplier) as u8
}

// Combined performance with pilot bonuses
fn calculate_performance(car: &Car, pilot: &Pilot) -> CarStats {
    let pilot_bonus = pilot.get_class_bonus();
    CarStats {
        speed: car.stats.speed + pilot_bonus.speed_bonus,
        acceleration: car.stats.acceleration + pilot_bonus.acceleration_bonus,
        handling: car.stats.handling + pilot_bonus.handling_bonus,
        durability: car.stats.durability + pilot_bonus.durability_bonus,
    }
}
```

### NFT Integration
- Cars and pilots can be linked to Solana NFT mint addresses
- NFT metadata can be used to generate or validate stats
- Ownership verification through wallet signatures
- Trading and marketplace integration ready

## Validation Rules

### Input Validation
- **Wallet Address**: 32-44 characters, base58 encoded
- **Team Name**: 2-50 characters, no HTML/script tags
- **Car Name**: 1-30 characters, no forbidden characters
- **Pilot Name**: 2-25 characters, no forbidden characters
- **Stats/Skills**: 1-100 range for all numeric values

### Business Logic Validation
- Maximum 2 cars per player
- Minimum 1 pilot per player for game participation
- Unique UUIDs for all entities
- Unique wallet addresses across players
- Stat limits based on rarity levels

## Error Handling

### HTTP Status Codes
- `200 OK` - Successful operations
- `201 Created` - Resource created successfully
- `400 Bad Request` - Invalid input data
- `404 Not Found` - Resource not found
- `409 Conflict` - Resource already exists (duplicate wallet)
- `500 Internal Server Error` - Database or server errors

### Error Response Format
```json
{
  "error": "Invalid input",
  "message": "Team name must be between 2 and 50 characters",
  "code": "VALIDATION_ERROR"
}
```

## Testing

### Test Commands
```powershell
# Test all player endpoints
.\Makefile.ps1 test-players

# Start development environment
.\Makefile.ps1 dev

# Run with test database
.\Makefile.ps1 test
```

### Sample Test Data
The system includes sample players, cars, and pilots for development and testing purposes, automatically created during MongoDB initialization.

## Future Enhancements

### Planned Features
1. **Race Results** - Track race performance and statistics
2. **Leaderboards** - Global and seasonal rankings
3. **Tournaments** - Organized competitive events
4. **Car Upgrades** - Stat improvements and modifications
5. **Pilot Training** - Experience-based skill improvements
6. **Team Management** - Multi-player team functionality
7. **Marketplace Integration** - NFT trading and auctions
8. **Achievement System** - Unlockable rewards and badges

### Technical Improvements
1. **Caching Layer** - Redis for frequently accessed data
2. **Event Sourcing** - Track all state changes
3. **Real-time Updates** - WebSocket notifications
4. **Analytics** - Player behavior and game metrics
5. **Backup & Recovery** - Automated data protection
6. **Load Balancing** - Horizontal scaling support

This player system provides a solid foundation for a Web3 racing game with comprehensive data management, validation, and integration capabilities.