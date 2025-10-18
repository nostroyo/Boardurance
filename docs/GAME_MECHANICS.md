# Web3 Racing Game - Game Mechanics Documentation

## 🎯 Overview

This document describes the complete game mechanics for the Web3 Racing Game, a turn-based racing system where players compete using NFT cars and pilots in a dynamic positioning-based race format.

## 🏁 Core Racing Concepts

### Track Structure (Positioning System)

The "track" is **NOT** a physical layout but a **dynamic positioning/ranking system**:

```
Track = Vec<Sector> (e.g., 5 sectors: [Sector0, Sector1, Sector2, Sector3, Sector4])
```

**Sector Hierarchy:**
- **Sector 0** = Worst positions (back of the pack)
- **Sector 1** = Better positions  
- **Sector 2** = Even better positions
- **Sector N** = Best positions (front of the pack)

**Sector Properties:**
- `min_value`: Minimum performance value to stay in this sector
- `max_value`: Maximum performance value before moving up
- `slot_capacity`: Maximum number of cars allowed in this sector (None = infinite)
- `sector_type`: Start, Straight, Curve, or Finish

**Within Each Sector:**
- Cars are ranked by their **total accumulated value**
- Higher total value = better position within the sector

## 🔄 Race Structure

### Lap Mechanics

Each lap has a **characteristic** that affects ALL cars for that entire lap:

- **"Straight"** - Favors speed and acceleration
- **"Curve"** - Favors handling and precision

**Lap Examples:**
- Lap 1: "Straight" → All cars use straight-focused calculations
- Lap 2: "Curve" → All cars use curve-focused calculations  
- Lap 3: "Straight" → Back to straight calculations

#### Lap Sequence
Each lap follows this sequence:

1. **Base Value Calculation**: Engine + Body + Pilot stats + Lap characteristic
2. **Player Boost**: Player adds boost value (0-5)
3. **Final Value**: Base value + Boost
4. **Movement Logic**: Determine sector movement based on final value

### Race Duration
- **Fixed number of laps** (e.g., 3 laps, 5 laps, etc.)
- Race ends when all laps are completed

## 🚗 Car Performance System

### Car Components
Each car consists of three main components:

#### 1. Engine Stats
- **Straight Value**: Performance on straight sections
- **Curve Value**: Performance on curved sections

#### 2. Body Stats  
- **Straight Value**: Aerodynamics and stability on straights
- **Curve Value**: Handling and grip in curves

#### 3. Pilot Skills
- **Straight Value**: Reaction time and focus for straights
- **Curve Value**: Precision and technique for curves

### Performance Calculation

```
Base Value = Engine[lap_characteristic] + Body[lap_characteristic] + Pilot[lap_characteristic]

Where lap_characteristic is either "straight" or "curve"
```

**Example:**
- Lap 1 (Straight): Base = Engine.straight + Body.straight + Pilot.straight
- Lap 2 (Curve): Base = Engine.curve + Body.curve + Pilot.curve

### Sector Performance Ceiling

**CRITICAL MECHANIC**: Each sector has a maximum performance ceiling that limits car potential **BEFORE** boost is applied:

```
Step 1: Capped Base Value = min(Base Value, current_sector.max_value)
Step 2: Final Value = Capped Base Value + Player Boost (0-5)
```

**The sector's max_value acts as a hard ceiling for the car's base performance, ensuring that cars cannot exceed their sector's performance potential without using boost.**

**Example:**
- Car Base Value: 10
- Current Sector Max: 5
- **Sector Cap Applied**: min(10, 5) = 5 ← Car's base value is capped to sector maximum
- Player Boost: +3
- **Final Value**: 5 + 3 = 8

**Another Example:**
- Car Base Value: 20
- Current Sector Max: 15
- **Sector Cap Applied**: min(20, 15) = 15 ← Car's base value is capped to sector maximum
- Player Boost: +3
- **Final Value**: 15 + 3 = 18

**Key Point**: The sector ceiling **only affects the base value calculation**, not the final value after boost. This means:
- A high-performance car in a low sector is limited by that sector's ceiling
- Boost can push the final value beyond the sector ceiling (potentially triggering movement)
- Cars must advance to higher sectors to unlock their full base performance potential

### Strategic Implications

**Performance Ceiling Effect:**
- Cars cannot exceed their current sector's max_value **for base performance only**
- The sector ceiling creates a **performance bottleneck** that limits car potential
- Higher sectors unlock higher base performance potential
- Sector advancement becomes crucial for competitive performance
- **High-stat cars are "wasted" in low sectors** until they can advance

**Boost Value Impact:**
- When base value is capped by sector ceiling, **boost becomes proportionally more important**
- Boost can push the final value beyond the sector ceiling (potentially triggering movement)
- Strategic boost timing becomes critical for sector advancement
- **Boost is the only way to exceed sector performance limits**

**Sector Advancement Strategy:**
- Cars with high base stats need to prioritize moving up sectors quickly
- Lower sectors act as "performance traps" for high-stat cars
- Players must balance boost usage between staying competitive and advancing sectors
- **The sector ceiling mechanic creates natural progression gates**

## 🎮 Movement Mechanics

### Sector Movement Rules

For each lap, after calculating the final value:

#### Moving DOWN (Worse Position)
```
IF final_value < current_sector.min_value:
    Move to next lower sector (Sector N → Sector N-1)
    Place at LAST position in destination sector
    Continue moving down until finding sector with available space
```

#### Moving UP (Better Position)  apply to only one car
```
IF first car before rank == first_car after rank
    IF next_higher_sector has available space:
        move_up
    ELSE:
        Stay in current sector (blocked by capacity)
```

#### Staying in Place
```
IF current_sector.min_value ≤ final_value ≤ current_sector.max_value:
    Stay in current sector
    Re-rank within sector based on total accumulated value
```

### Sector Capacity Handling

#### When Moving UP and Sector is Full:
- Car **stays in current sector**
- No movement occurs
- Position within current sector may change based on performance

#### When Moving DOWN and Sectors are Full:
- Car continues moving down sector by sector
- Places at **last position** in each sector until finding available space
- Ensures car always finds a valid position

### Special Sector Rules

#### Start Sector (Sector 0)
- **Infinite capacity** (slot_capacity = None)
- Can always accommodate cars moving down

#### Finish Sector (Last Sector)
- **Infinite capacity** (slot_capacity = None)  
- Represents the leading positions
- No upper limit on cars that can reach it

## 🏁 Race Initialization

### Qualification System

Cars do **NOT** all start in Sector 0. Instead, they start in different sectors based on qualification:

**Qualification Process:**
- **Random qualification** (current implementation)
- Cars are randomly distributed across sectors at race start
- Future: Qualification races, previous performance, or car stats-based placement

**Starting Position Benefits:**
- Cars starting in higher sectors have positional advantage
- Better starting position = closer to victory
- Creates strategic importance for qualification performance

## 🏆 Race Progression

### Game Flow Example

**Race Setup:** 3 laps, 4 sectors

```
LAP 1 (Characteristic: "Straight"):
  All cars calculate base value using straight stats + boost → sector movement

LAP 2 (Characteristic: "Curve"):
  All cars calculate base value using curve stats + boost → sector movement

LAP 3 (Characteristic: "Straight"):  
  All cars calculate base value using straight stats + boost → final positioning
```

### Lap Processing Algorithm

The lap calculation follows a specific order to ensure fair and consistent movement:

#### Processing Order: Best Sector → Worst Sector
```
Process sectors in descending order: [Sector N, Sector N-1, ..., Sector 1, Sector 0]
```

#### Detailed Algorithm Steps:

1. **Determine Lap Characteristic** (straight or curve)
2. **Calculate Values** for ALL cars using lap characteristic + player boost
3. **Process Movement by Sector** (from best to worst):

```
FOR each sector FROM highest_sector DOWN TO sector_0:
    FOR each car IN current_sector:
        Calculate final_value = base_value + boost
        
        IF final_value < sector.min_value:
            Handle MOVING DOWN
       
    
    Re-rank ALL cars remaining in this sector by total_value
    IF FIRST_CAR_BFORE_RANK == FIRST_CAR_AFTER_RANK:
        handle MOVING_UP for this car 
```

#### Movement Handling Details:

**Moving DOWN Process:**
```
1. Remove car from current sector
2. Try to place in next lower sector
3. If lower sector is full, continue to next lower sector
4. Place at LAST position in destination sector
5. Car gets worst position in that sector initially
```

**Moving UP Process:**
```
1. Check if next higher sector has available space
2. IF space available:
   - Remove car from current sector
   - Add to higher sector
   - Position determined by total_value ranking
3. IF no space available:
   - Car STAYS in current sector
   - No movement occurs
```

**Re-ranking Within Sector:**
```
After all movements processed for a sector:
1. Sort remaining cars by total_accumulated_value (descending)
2. Assign positions: 0, 1, 2, ... (0 = best position in sector)
3. Higher total_value = better position within sector
```

#### Complete Lap Sequence:
1. **Determine Lap Characteristic** (straight or curve)
2. **Calculate Base Values** for all cars using lap characteristic
3. **Collect Player Boosts** (0-5 for each player)
4. **Calculate Final Values** (base + boost)
5. **Process Sectors** from highest to lowest (detailed algorithm above)
6. **Update Total Values** (add current lap's final value to total)
7. **Check Race Completion** (all laps finished?)

## 📊 Scoring and Ranking

### Total Accumulated Value
- Each car maintains a **total_value** across all laps
- **Updated each lap**: total_value += current_lap_final_value
- Used for ranking within sectors
- Higher total = better position within same sector

### Final Race Positions
- Determined by **sector + position within sector**
- Sector 4, Position 1 > Sector 4, Position 2 > Sector 3, Position 1, etc.
- Cars finishing in higher sectors rank better regardless of total value

### Lap Completion
- Cars advance through sectors during the race
- Completing a lap may reset sector position or continue progression
- Final ranking based on sector positions at race end

## 🎯 Strategic Elements

### Boost Management
- Players have limited boost per lap (0-5)
- Strategic timing of boosts affects race outcome
- Higher boost = better chance to move up sectors

### Car Specialization
- **Straight-focused cars**: Better on straight laps
- **Curve-focused cars**: Better on curve laps  
- **Balanced cars**: Consistent across all lap types

### Pilot Classes
- **Speedster**: Bonus to straight performance
- **Technician**: Bonus to curve performance
- **Endurance**: Consistent performance over time
- **AllRounder**: Balanced bonuses

## 🔧 Technical Implementation

### Race State Management
```rust
pub struct Race {
    pub current_lap: u32,
    pub total_laps: u32,
    pub lap_characteristic: LapCharacteristic, // Straight or Curve
    pub participants: Vec<RaceParticipant>,
    pub track: Track,
}

pub enum LapCharacteristic {
    Straight,
    Curve,
}

pub struct RaceParticipant {
    pub player_uuid: Uuid,
    pub car_uuid: Uuid,
    pub pilot_uuid: Uuid,
    pub current_sector: u32,
    pub current_position_in_sector: u32,
    pub total_value: u32,
    pub current_lap: u32,
}
```

### Movement Processing
```rust
pub struct LapAction {
    pub player_uuid: Uuid,
    pub boost_value: u32, // 0-5
}

pub struct LapResult {
    pub lap: u32,
    pub lap_characteristic: LapCharacteristic,
    pub sector_positions: HashMap<u32, Vec<RaceParticipant>>,
    pub movements: Vec<ParticipantMovement>,
}

pub enum MovementType {
    StayedInSector,
    MovedUp,
    MovedDown,
    FinishedLap,
    FinishedRace,
}
```

## 🎪 Race Types and Variations

### Standard Race
- Fixed number of laps
- Alternating or random lap characteristics
- Standard sector configuration

### Sprint Race
- Fewer laps (1-2)
- Higher impact per turn
- Smaller sector capacities

### Endurance Race
- Many laps (5-10)
- Pilot stamina affects performance over time
- Larger sector capacities

### Championship Series
- Multiple races with points
- Car and pilot experience gains
- Seasonal rankings

## 🏅 Rewards and Progression

### Race Rewards
- **Position-based**: Better finishing position = better rewards
- **Performance-based**: High total values earn bonus rewards
- **Participation**: Base rewards for completing races

### Experience System
- **Pilots gain experience** from races
- **Experience affects** future performance calculations
- **Rarity multipliers** affect experience gain rates

### NFT Integration
- **Car NFTs** determine base stats and rarity
- **Pilot NFTs** provide skills and class bonuses
- **Marketplace integration** for trading assets

---

**This game mechanics system creates a strategic, turn-based racing experience where player decisions, car optimization, and tactical boost usage determine race outcomes in a dynamic positioning-based competition! 🏁🚗💨**