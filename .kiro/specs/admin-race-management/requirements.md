# Requirements Document

## Introduction

This document specifies the requirements for implementing an admin race management interface in the React frontend. The system will provide administrators with comprehensive tools to create, configure, monitor, and manage races in the Web3 Racing Game, including real-time race status updates and participant management.

## Glossary

- **Admin_Interface**: The administrative user interface for race management
- **Race_Creator**: UI component for creating new races with track configuration
- **Race_Dashboard**: Overview interface showing all races and their statuses
- **Track_Builder**: Interface for configuring race sectors via JSON upload or manual creation
- **Track_JSON_Schema**: Standardized JSON format for defining track sector configurations
- **Sector_Vector**: MongoDB storage format using Vec<Sector> for track sector data
- **Race_Monitor**: Real-time interface for monitoring active races
- **Admin_User**: User with administrative privileges for race management
- **Race_Configuration**: Settings including track layout, lap count, and sector parameters

## Track JSON Schema & MongoDB Storage

### JSON Track Configuration Format
```json
{
  "sectors": [
    {
      "id": 0,
      "name": "Start Line",
      "min_value": 0,
      "max_value": 10,
      "slot_capacity": null,
      "sector_type": "Start"
    },
    {
      "id": 1,
      "name": "Casino Corner",
      "min_value": 8,
      "max_value": 15,
      "slot_capacity": 3,
      "sector_type": "Curve"
    },
    {
      "id": 2,
      "name": "Tunnel Straight",
      "min_value": 12,
      "max_value": 20,
      "slot_capacity": 2,
      "sector_type": "Straight"
    },
    {
      "id": 3,
      "name": "Finish Line",
      "min_value": 18,
      "max_value": 25,
      "slot_capacity": null,
      "sector_type": "Finish"
    }
  ]
}
```

### MongoDB Race Document Structure
```rust
// Existing Race struct in domain/race.rs
pub struct Race {
    pub id: Option<ObjectId>,
    pub uuid: Uuid,
    pub name: String,
    pub track: Track,           // Contains Vec<Sector>
    pub participants: Vec<RaceParticipant>,
    pub lap_characteristic: LapCharacteristic,
    pub current_lap: u32,
    pub total_laps: u32,
    pub status: RaceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Track contains the sector vector
pub struct Track {
    pub uuid: Uuid,
    pub name: String,
    pub sectors: Vec<Sector>,   // This is our Vec<Sector> storage
}

// Individual sector definition
pub struct Sector {
    pub id: u32,
    pub name: String,
    pub min_value: u32,
    pub max_value: u32,
    pub slot_capacity: Option<u32>,  // None = infinite capacity
    pub sector_type: SectorType,
}
```

### MongoDB Storage Benefits
- **Native Array Support**: MongoDB's BSON format natively supports arrays, making Vec<Sector> storage efficient
- **Atomic Operations**: Entire track configuration can be updated atomically
- **Indexing**: Can create indexes on sector properties for efficient queries
- **Flexible Schema**: Easy to add new sector properties without migration
- **JSON Compatibility**: Direct mapping between JSON upload and BSON storage

## Requirements

### Requirement 1

**User Story:** As a game administrator, I want to access a dedicated admin interface for race management, so that I can efficiently create and manage races without using regular player interfaces.

#### Acceptance Criteria

1. WHEN an admin user logs in, THE Admin_Interface SHALL display an admin-specific navigation menu
2. WHEN accessing the admin interface, THE Admin_Interface SHALL verify admin role permissions
3. IF a non-admin user attempts access, THEN THE Admin_Interface SHALL redirect to the regular dashboard
4. WHEN admin privileges are confirmed, THE Admin_Interface SHALL display the race management dashboard
5. WHERE admin status changes, THE Admin_Interface SHALL update navigation and access permissions accordingly

### Requirement 2

**User Story:** As a game administrator, I want to create new races with custom track configurations using JSON files, so that I can efficiently set up complex racing experiences without manual sector-by-sector configuration.

#### Acceptance Criteria

1. WHEN creating a race, THE Race_Creator SHALL provide fields for race name, track name, and total laps
2. WHEN configuring the track, THE Track_Builder SHALL support JSON file upload for track sector definitions
3. WHEN uploading track JSON, THE Track_Builder SHALL validate the JSON structure matches the required sector schema
4. WHEN processing track configuration, THE Track_Builder SHALL ensure first and last sectors have infinite capacity (slot_capacity: null)
5. WHEN submitting race creation, THE Race_Creator SHALL send the track configuration as a Vec<Sector> to the backend API
6. WHERE JSON upload fails, THE Track_Builder SHALL provide clear error messages and fallback to manual sector creation

### Requirement 3

**User Story:** As a game administrator, I want to view all races in a centralized dashboard, so that I can monitor race statuses and manage multiple races efficiently.

#### Acceptance Criteria

1. WHEN accessing the race dashboard, THE Race_Dashboard SHALL display all races with their current status
2. WHEN viewing race information, THE Race_Dashboard SHALL show race name, participant count, current lap, and status
3. WHEN races are updated, THE Race_Dashboard SHALL refresh data automatically or provide manual refresh
4. WHEN selecting a race, THE Race_Dashboard SHALL provide detailed race information and management options
5. WHERE races have different statuses, THE Race_Dashboard SHALL use visual indicators (colors, icons) for quick identification

### Requirement 4 (LOW PRIORITY)

**User Story:** As a game administrator, I want to monitor active races in real-time, so that I can track race progress and intervene if necessary.

#### Acceptance Criteria

1. WHEN viewing an active race, THE Race_Monitor SHALL display current sector positions for all participants
2. WHEN races progress, THE Race_Monitor SHALL show lap-by-lap movement history and results
3. WHEN participants perform actions, THE Race_Monitor SHALL display boost usage and performance calculations
4. WHEN races complete, THE Race_Monitor SHALL show final standings and race statistics
5. WHERE race issues occur, THE Race_Monitor SHALL provide administrative controls for race management

### Requirement 5 (LOW PRIORITY)

**User Story:** As a game administrator, I want to manage race participants and control race flow, so that I can ensure fair and smooth race operations.

#### Acceptance Criteria

1. WHEN viewing race participants, THE Admin_Interface SHALL display player, car, and pilot information
2. WHEN races are in waiting status, THE Admin_Interface SHALL provide controls to start the race
3. WHEN managing participants, THE Admin_Interface SHALL allow viewing participant details and statistics
4. IF race intervention is needed, THEN THE Admin_Interface SHALL provide appropriate administrative controls
5. WHERE races encounter issues, THE Admin_Interface SHALL display error information and resolution options

### Requirement 6

**User Story:** As a database administrator, I want track configurations stored efficiently in MongoDB using a standardized schema, so that race data is properly structured and easily queryable.

#### Acceptance Criteria

1. WHEN storing track data, THE System SHALL use a Vec<Sector> structure within the Race document
2. WHEN defining sector schema, THE System SHALL include id, name, min_value, max_value, slot_capacity, and sector_type fields
3. WHEN storing in MongoDB, THE System SHALL leverage BSON's native array support for the sectors vector
4. WHEN querying races, THE System SHALL enable efficient filtering and indexing on track properties
5. WHERE track data is updated, THE System SHALL maintain referential integrity and proper versioning

### Requirement 7

**User Story:** As a system administrator, I want secure access controls for the admin interface, so that only authorized personnel can manage races.

#### Acceptance Criteria

1. WHEN accessing admin features, THE Admin_Interface SHALL validate JWT tokens with admin role claims
2. WHEN admin sessions expire, THE Admin_Interface SHALL redirect to login with appropriate error messages
3. WHILE using admin features, THE Admin_Interface SHALL maintain secure communication with backend APIs
4. WHEN unauthorized access is attempted, THE Admin_Interface SHALL log security events and deny access
5. WHERE admin permissions are revoked, THE Admin_Interface SHALL immediately restrict access to admin features