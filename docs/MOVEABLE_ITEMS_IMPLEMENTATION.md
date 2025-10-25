# Moveable Items Implementation

## Overview
Implemented drag-and-drop functionality for moving game components (engines, bodies, pilots) between car slots and inventory in the frontend, with a new backend endpoint to save the complete car configuration.

## Backend Changes

### New Route
- `PUT /api/v1/players/{player_uuid}/configuration` - Updates player's car configuration and team name simultaneously

### New Request Structure
```rust
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePlayerConfigurationRequest {
    pub team_name: String,
    pub cars: Vec<Car>,
}
```

### Database Function
- `update_player_configuration_by_uuid()` - Atomically updates both team name and car configurations in MongoDB

## Frontend Changes

### Enhanced Drag-and-Drop
1. **Car Slots as Drop Zones**: Each car component slot (engine, body, pilot) now accepts drops
2. **Inventory as Drop Zone**: Inventory sections accept drops to "unassign" components
3. **Bidirectional Movement**: Components can be dragged from inventory to cars and vice versa
4. **Visual Feedback**: Clear visual indicators for drag sources and drop zones

### Improved UI Features
- **Drop Indicators**: Dashed borders show where items can be dropped
- **Drag Hints**: Hover text explains drag-and-drop functionality
- **Save Button**: Appears when changes are made, saves to new backend endpoint
- **Error Handling**: Proper error display for failed operations

### Component States
- **Assigned Components**: Can be dragged to move or clicked to remove
- **Empty Slots**: Show "Drop here" placeholders
- **Inventory Items**: Draggable with clear visual feedback

## API Usage

### Save Configuration
```javascript
const response = await fetch(`/api/v1/players/${playerUuid}/configuration`, {
  method: 'PUT',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    team_name: player.team_name,
    cars: player.cars
  })
});
```

## Testing

### Test Script
- `rust-backend/test-moveable-configuration.ps1` - Comprehensive test of the moveable functionality
- Tests creation, assignment, swapping, and removal of components
- Verifies backend persistence of configurations

### Test Scenarios
1. Create player with starter assets
2. Assign components to cars
3. Move components between cars
4. Remove components back to inventory
5. Verify persistence across operations

## Benefits

1. **Intuitive UX**: Natural drag-and-drop interface for component management
2. **Atomic Updates**: Single API call saves entire configuration
3. **Real-time Feedback**: Immediate visual updates with save confirmation
4. **Flexible Movement**: Components can move freely between any valid locations
5. **Persistent State**: All changes are saved to the backend database

## Usage Instructions

1. **Drag from Inventory**: Drag any component from inventory to a car slot
2. **Drag Between Cars**: Drag components directly between car slots
3. **Return to Inventory**: Drag components back to inventory or click to remove
4. **Save Changes**: Click "Save Configuration" button when changes are made
5. **Visual Feedback**: Watch for color changes and drag hints during operations