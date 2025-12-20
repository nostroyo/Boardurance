# Race Interface Redesign - Design Document

## Overview

This design document outlines the complete redesign of the player race interface to provide an immersive bird's eye view racing experience. The new interface will feature a top-down track visualization with 8-bit style car sprites, improved sector grids, and prominent boost selection controls. The design maintains all existing functionality while dramatically improving the visual presentation and user experience.

## Architecture

### Component Hierarchy

```
RaceInterfaceRedesign (Main Container)
├── TrackBirdEyeView (Central Track Display)
│   ├── SectorGrid (Individual Sector Visualization)
│   │   ├── PositionSlot (Car Position Markers)
│   │   └── CarSprite (8-bit Car Representations)
│   └── TrackLayout (Sector Arrangement Logic)
├── BoostControlPanel (Boost Selection & Validation)
│   ├── BoostButtonGrid (0-5 Boost Buttons)
│   ├── ValidateTurnButton (Turn Submission)
│   └── BoostStatusDisplay (Availability Indicators)
├── RaceStatusHeader (Race Information)
└── PlayerInfoSidebar (Car Stats & History)
```

### Layout Structure

The interface uses a responsive grid layout:
- **Center**: Bird's eye track view (60% width on desktop)
- **Right Sidebar**: Boost controls and player info (25% width)
- **Top Header**: Race status and lap information (15% height)
- **Bottom Panel**: Turn validation and status (mobile adaptation)

## Components and Interfaces

### TrackBirdEyeView Component

```typescript
interface TrackBirdEyeViewProps {
  localView: LocalView;
  playerUuid: string;
  animationState?: AnimationState;
  onSectorClick?: (sectorId: number) => void;
}

interface SectorGridData {
  sector: LocalView['visible_sectors'][0];
  participants: LocalView['visible_participants'];
  isPlayerSector: boolean;
  gridLayout: PositionSlot[][];
}

interface PositionSlot {
  id: string;
  occupied: boolean;
  participant?: LocalView['visible_participants'][0];
  coordinates: { x: number; y: number };
}
```

### CarSprite Component

```typescript
interface CarSpriteProps {
  participant: LocalView['visible_participants'][0];
  isPlayer: boolean;
  size: 'small' | 'medium' | 'large';
  animationState?: 'idle' | 'moving' | 'highlighted';
}

interface SpriteStyle {
  colors: {
    primary: string;
    secondary: string;
    highlight: string;
  };
  pixelPattern: number[][];
  animations: {
    idle: string;
    moving: string;
    highlighted: string;
  };
}
```

### BoostControlPanel Component

```typescript
interface BoostControlPanelProps {
  selectedBoost: number | null;
  availableBoosts: number[];
  onBoostSelect: (boost: number) => void;
  onValidateTurn: () => void;
  isSubmitting: boolean;
  hasSubmitted: boolean;
  turnPhase: TurnPhaseStatus;
}

interface BoostButtonState {
  value: number;
  available: boolean;
  selected: boolean;
  used: boolean;
}
```

## Data Models

### Enhanced Sector Grid Model

```typescript
interface SectorGridLayout {
  sectorId: number;
  gridDimensions: {
    rows: number;
    cols: number;
  };
  positionSlots: PositionSlot[][];
  capacity: number | null;
  occupancy: number;
  visualStyle: SectorVisualStyle;
}

interface SectorVisualStyle {
  backgroundColor: string;
  borderColor: string;
  borderWidth: number;
  borderStyle: 'solid' | 'dashed' | 'dotted';
  gridLineColor: string;
  highlightColor?: string;
}
```

### Car Sprite Data Model

```typescript
interface CarSpriteData {
  participantId: string;
  spriteType: 'player' | 'opponent';
  carColor: string;
  pixelArt: {
    pattern: number[][];
    colorMap: Record<number, string>;
  };
  position: {
    sectorId: number;
    slotCoordinates: { x: number; y: number };
  };
  animations: {
    current: 'idle' | 'moving' | 'highlighted';
    duration: number;
  };
}
```

### Track Layout Model

```typescript
interface TrackLayoutData {
  centerSector: number;
  visibleSectors: SectorGridLayout[];
  sectorArrangement: {
    center: SectorGridLayout;
    surrounding: SectorGridLayout[];
  };
  trackDimensions: {
    width: number;
    height: number;
  };
  zoomLevel: number;
}
```
## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Sector Grid Consistency
*For any* sector data, when rendered as a grid, the system should display the correct number of position slots based on the sector's capacity and show clear grid markings
**Validates: Requirements 1.2**

### Property 2: Car Sprite Positioning
*For any* set of cars in sectors, each car sprite should appear in a unique position slot within its designated sector without overlapping
**Validates: Requirements 1.3, 4.5**

### Property 3: Player Car Distinction
*For any* race state, the player's car sprite should always have visually distinct styling (highlighting, effects, or colors) compared to opponent cars
**Validates: Requirements 1.4, 4.4**

### Property 4: Player Sector Centering
*For any* player sector position, the interface should position that sector in the center of the display with other sectors arranged around it
**Validates: Requirements 1.5**

### Property 5: Sector Layout Consistency
*For any* number of visible sectors, the spacing between sectors should be consistent and evenly distributed across the display area
**Validates: Requirements 2.1**

### Property 6: Sector Visual Uniformity
*For any* sector, the border styling, sizing, and grid appearance should be consistent with all other sectors of the same type
**Validates: Requirements 2.2**

### Property 7: Visual Hierarchy Maintenance
*For any* multi-sector display, the player's current sector should always have enhanced visual emphasis compared to surrounding sectors
**Validates: Requirements 2.3**

### Property 8: Sector Information Completeness
*For any* sector, the display should include all required information: capacity (if applicable) and value ranges
**Validates: Requirements 2.4**

### Property 10: Boost Button State Accuracy
*For any* boost availability state, the button visual states should correctly reflect which boost values are available versus already used
**Validates: Requirements 3.2, 7.3**

### Property 11: Boost Selection Feedback
*For any* available boost button clicked, the system should provide immediate visual feedback and highlight the selection
**Validates: Requirements 3.3**

### Property 12: Validate Button Enablement
*For any* valid boost selection, the "Validate Turn" button should become enabled and prominently displayed
**Validates: Requirements 3.4**

### Property 9: Sprite Style Consistency
*For any* car sprite displayed, it should use 8-bit pixel art styling with consistent visual characteristics
**Validates: Requirements 4.1**

### Property 10: Player Visual Distinction
*For any* combination of players in the race, each player's car sprite should have a unique visual appearance (color, design, or pattern)
**Validates: Requirements 4.2**

### Property 11: Movement Animation Smoothness
*For any* car movement between sectors, the animation should complete smoothly without visual artifacts or abrupt transitions
**Validates: Requirements 4.3**

### Property 12: Functional Data Preservation
*For any* race data displayed in the original interface, the same data should be accessible and displayed in the redesigned interface
**Validates: Requirements 5.1**

### Property 13: Boost Validation Consistency
*For any* boost selection interaction, the validation logic should behave identically to the existing boost validation system
**Validates: Requirements 5.2**

### Property 14: Race Information Completeness
*For any* race state, all current race information (lap number, turn phase, race status) should be visible in the interface
**Validates: Requirements 5.3**

### Property 15: Error Handling Consistency
*For any* error condition, the error should be handled and displayed using the same error handling mechanisms as the existing interface
**Validates: Requirements 5.4**

### Property 16: Boost Button Labeling
*For any* boost button displayed, it should have clear, correct labeling (0, 1, 2, 3, 4, 5) that matches its boost value
**Validates: Requirements 7.2**

## Error Handling

### Error States and Recovery

1. **Sector Rendering Failures**
   - Fallback to simplified sector display
   - Error boundary to prevent complete interface failure
   - Retry mechanism for sector data loading

2. **Car Sprite Loading Errors**
   - Default sprite fallback system
   - Graceful degradation to text-based car representation
   - Error logging for sprite asset issues

3. **Animation Performance Issues**
   - Automatic animation quality reduction on low-performance devices
   - Frame rate monitoring and adaptive rendering
   - Option to disable animations entirely

4. **Responsive Layout Failures**
   - Breakpoint fallback system
   - Minimum viable layout for extreme screen sizes
   - Scroll-based navigation for overflow content

### Error Boundaries

```typescript
interface RaceInterfaceErrorBoundary {
  componentDidCatch: (error: Error, errorInfo: ErrorInfo) => void;
  fallbackComponent: React.ComponentType<{error: Error}>;
  recoveryActions: {
    resetInterface: () => void;
    reloadRaceData: () => void;
    switchToSimpleView: () => void;
  };
}
```

## Testing Strategy

### Dual Testing Approach

The testing strategy employs both unit testing and property-based testing to ensure comprehensive coverage:

- **Unit tests** verify specific examples, edge cases, and integration points
- **Property tests** verify universal properties across all inputs using **fast-check** library
- Together they provide complete coverage: unit tests catch concrete bugs, property tests verify general correctness

### Property-Based Testing Requirements

- Use **fast-check** library for TypeScript/React property-based testing
- Configure each property test to run a minimum of 100 iterations
- Tag each property test with comments referencing the design document property
- Use format: `**Feature: race-interface-redesign, Property {number}: {property_text}**`
- Each correctness property must be implemented by a single property-based test

### Unit Testing Focus Areas

- Component rendering with specific race data scenarios
- User interaction flows (boost selection, turn validation)
- Error boundary behavior and recovery
- Responsive layout breakpoint transitions
- Animation state management and cleanup

### Integration Testing

- Full race interface workflow testing
- API integration with existing race endpoints
- Cross-browser compatibility verification
- Performance testing under various data loads
- Accessibility compliance validation

### Test Data Generation

Property tests will use smart generators that:
- Generate realistic race data within valid constraints
- Create varied sector configurations and car positions
- Simulate different screen sizes and device capabilities
- Test edge cases like maximum capacity sectors and empty sectors
- Validate animation states and transitions