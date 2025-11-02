# Design Document

## Overview

The Admin Race Management interface will be implemented as a React-based administrative dashboard that provides comprehensive race creation and management capabilities. The system will integrate with the existing authentication system to ensure only admin users can access these features, and will leverage the existing race API endpoints for backend communication.

## Architecture

### Component Hierarchy
```
AdminDashboard
├── AdminNavigation
├── RaceManagement
│   ├── RaceCreator
│   │   ├── RaceForm
│   │   ├── TrackBuilder
│   │   └── JSONUploader
│   ├── RaceDashboard
│   │   ├── RaceList
│   │   ├── RaceCard
│   │   └── RaceFilters
│   └── RaceDetails (LOW PRIORITY)
│       ├── ParticipantList
│       ├── RaceMonitor
│       └── RaceControls
└── AdminRoute (HOC for protection)
```

### Authentication Flow
```
User Login → JWT Token → Role Validation → Admin Dashboard Access
```

### Data Flow
```
Frontend (React) → API Calls → Backend (Rust) → MongoDB → Response → UI Update
```

## Components and Interfaces

### 1. AdminRoute (Higher-Order Component)
**Purpose**: Protect admin routes and verify admin privileges

```typescript
interface AdminRouteProps {
  children: React.ReactNode;
}

const AdminRoute: React.FC<AdminRouteProps> = ({ children }) => {
  // Check if user has admin role
  // Redirect non-admin users
  // Render children for admin users
};
```

### 2. AdminDashboard
**Purpose**: Main admin interface container

```typescript
interface AdminDashboardProps {}

const AdminDashboard: React.FC<AdminDashboardProps> = () => {
  // Main admin dashboard layout
  // Navigation and routing for admin features
};
```

### 3. RaceCreator
**Purpose**: Interface for creating new races

```typescript
interface RaceCreatorProps {}

interface RaceFormData {
  raceName: string;
  trackName: string;
  totalLaps: number;
  sectors: Sector[];
}

interface Sector {
  id: number;
  name: string;
  min_value: number;
  max_value: number;
  slot_capacity: number | null;
  sector_type: 'Start' | 'Straight' | 'Curve' | 'Finish';
}

const RaceCreator: React.FC<RaceCreatorProps> = () => {
  // Form for race creation
  // JSON upload for track configuration
  // Validation and submission
};
```

### 4. JSONUploader
**Purpose**: Handle JSON file upload for track configuration

```typescript
interface JSONUploaderProps {
  onTrackLoad: (sectors: Sector[]) => void;
  onError: (error: string) => void;
}

const JSONUploader: React.FC<JSONUploaderProps> = ({ onTrackLoad, onError }) => {
  // File input handling
  // JSON validation
  // Error handling
};
```

### 5. RaceDashboard
**Purpose**: Display and manage all races

```typescript
interface RaceDashboardProps {}

interface Race {
  uuid: string;
  name: string;
  track: {
    name: string;
    sectors: Sector[];
  };
  participants: RaceParticipant[];
  status: 'Waiting' | 'InProgress' | 'Finished' | 'Cancelled';
  current_lap: number;
  total_laps: number;
  created_at: string;
  updated_at: string;
}

const RaceDashboard: React.FC<RaceDashboardProps> = () => {
  // List all races
  // Filter and search functionality
  // Race status indicators
};
```

### 6. RaceCard
**Purpose**: Display individual race information

```typescript
interface RaceCardProps {
  race: Race;
  onViewDetails: (raceId: string) => void;
}

const RaceCard: React.FC<RaceCardProps> = ({ race, onViewDetails }) => {
  // Race summary display
  // Status indicators
  // Action buttons
};
```

## Data Models

### Frontend Types
```typescript
// Race-related types
interface Race {
  uuid: string;
  name: string;
  track: Track;
  participants: RaceParticipant[];
  lap_characteristic: 'Straight' | 'Curve';
  current_lap: number;
  total_laps: number;
  status: RaceStatus;
  created_at: string;
  updated_at: string;
}

interface Track {
  uuid: string;
  name: string;
  sectors: Sector[];
}

interface Sector {
  id: number;
  name: string;
  min_value: number;
  max_value: number;
  slot_capacity: number | null;
  sector_type: SectorType;
}

type SectorType = 'Start' | 'Straight' | 'Curve' | 'Finish';
type RaceStatus = 'Waiting' | 'InProgress' | 'Finished' | 'Cancelled';

interface RaceParticipant {
  player_uuid: string;
  car_uuid: string;
  pilot_uuid: string;
  current_sector: number;
  current_position_in_sector: number;
  current_lap: number;
  total_value: number;
  is_finished: boolean;
  finish_position: number | null;
}

// API Request types
interface CreateRaceRequest {
  name: string;
  track_name: string;
  total_laps: number;
  sectors: CreateSectorRequest[];
}

interface CreateSectorRequest {
  id: number;
  name: string;
  min_value: number;
  max_value: number;
  slot_capacity: number | null;
  sector_type: SectorType;
}
```

### JSON Track Schema Validation
```typescript
const trackSchemaValidator = {
  validateTrackJSON: (jsonData: any): Sector[] => {
    // Validate JSON structure
    // Ensure required fields
    // Validate sector types and values
    // Check first/last sector capacity rules
    // Return validated sectors array
  }
};
```

## API Integration

### Race Management Endpoints
```typescript
class RaceAPI {
  // Create new race
  static async createRace(raceData: CreateRaceRequest): Promise<Race> {
    return fetch('/api/v1/races', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${getAuthToken()}`
      },
      body: JSON.stringify(raceData)
    });
  }

  // Get all races
  static async getAllRaces(): Promise<Race[]> {
    return fetch('/api/v1/races', {
      headers: {
        'Authorization': `Bearer ${getAuthToken()}`
      }
    });
  }

  // Get specific race
  static async getRace(raceId: string): Promise<Race> {
    return fetch(`/api/v1/races/${raceId}`, {
      headers: {
        'Authorization': `Bearer ${getAuthToken()}`
      }
    });
  }

  // Get race status
  static async getRaceStatus(raceId: string): Promise<RaceStatus> {
    return fetch(`/api/v1/races/${raceId}/status`, {
      headers: {
        'Authorization': `Bearer ${getAuthToken()}`
      }
    });
  }
}
```

## Error Handling

### Validation Errors
- JSON schema validation errors
- Form validation errors
- API request/response errors
- Authentication/authorization errors

### Error Display Strategy
```typescript
interface ErrorState {
  type: 'validation' | 'api' | 'auth' | 'network';
  message: string;
  field?: string; // For form validation errors
}

const ErrorDisplay: React.FC<{ error: ErrorState }> = ({ error }) => {
  // Display appropriate error messages
  // Provide user-friendly error descriptions
  // Suggest resolution actions
};
```

## Testing Strategy

### Unit Tests
- Component rendering tests
- JSON validation logic tests
- API integration tests
- Form validation tests

### Integration Tests
- Admin authentication flow
- Race creation end-to-end
- JSON upload and validation
- API error handling

### Test Data
```typescript
const mockTrackJSON = {
  sectors: [
    {
      id: 0,
      name: "Start Line",
      min_value: 0,
      max_value: 10,
      slot_capacity: null,
      sector_type: "Start"
    },
    // ... more sectors
  ]
};

const mockRaceData = {
  name: "Test Race",
  track_name: "Test Track",
  total_laps: 3,
  sectors: mockTrackJSON.sectors
};
```

## Security Considerations

### Admin Authentication
- JWT token validation on every admin request
- Role-based access control verification
- Automatic logout on token expiration
- Secure token storage and transmission

### Input Validation
- Client-side JSON schema validation
- Server-side validation redundancy
- XSS prevention in user inputs
- File upload security (JSON only)

### API Security
- HTTPS enforcement in production
- CORS configuration for admin endpoints
- Rate limiting for admin operations
- Audit logging for admin actions

## Performance Considerations

### Optimization Strategies
- Lazy loading of admin components
- Efficient race list pagination
- Debounced search and filtering
- Optimistic UI updates for race creation

### Caching Strategy
- Cache race list data with refresh capability
- Local storage for form draft data
- Session storage for navigation state
- Invalidate cache on race updates

## Implementation Phases

### Phase 1: Core Admin Interface (HIGH PRIORITY)
1. AdminRoute protection component
2. AdminDashboard layout
3. RaceCreator with JSON upload
4. Basic RaceDashboard with race list

### Phase 2: Enhanced Management (LOW PRIORITY)
1. Real-time race monitoring
2. Participant management
3. Race control features
4. Advanced filtering and search

### Phase 3: Advanced Features (FUTURE)
1. Race analytics and statistics
2. Bulk race operations
3. Race templates and presets
4. Advanced admin reporting