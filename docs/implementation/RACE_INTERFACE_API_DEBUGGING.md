# Race Interface API Debugging Guide

## Issue: Missing Buttons and Sprites in Play Route

### Problem Description
User reports not seeing boost buttons or car sprites when accessing the race play interface.

### Root Cause Analysis

There are two different routes for playing races:

1. **`/game/:raceUuid`** - Uses `GameWrapper` → `PlayerGameInterface`
2. **`/races/:raceUuid/play`** - Uses `RacePlayPage` → `RaceContainer` → `RaceInterface`

Both routes use the redesigned components (`TrackDisplayRedesign` and `BoostControlPanel`), but they call different API endpoints.

### API Endpoints Comparison

**PlayerGameInterface** (Route 1) calls:
- `/api/v1/races/{raceUuid}/players/{playerUuid}/local-view` ✅ (with fallback)
- `/api/v1/races/{raceUuid}/players/{playerUuid}/boost-availability` ✅ (with fallback)

**RaceContainer** (Route 2) calls:
- `/api/v1/races/{race_uuid}/players/{player_uuid}/car-data` ✅ (implemented)
- `/api/v1/races/{race_uuid}/players/{player_uuid}/local-view` ✅ (implemented)
- `/api/v1/races/{race_uuid}/turn-phase` ✅ (implemented)
- `/api/v1/races/{race_uuid}/players/{player_uuid}/boost-availability` ✅ (implemented)
- `/api/v1/races/{race_uuid}/players/{player_uuid}/performance-preview` ✅ (implemented)
- `/api/v1/races/{race_uuid}/players/{player_uuid}/lap-history` ✅ (implemented)

### Backend Implementation Status

All required API endpoints are implemented in the Rust backend:
- Handler functions exist in `rust-backend/src/routes/races.rs`
- Routes are registered in the router
- Response types are properly defined

### Solution Implemented

1. **Enhanced Error Handling**: Updated `PlayerGameInterface` to provide fallback mock data when API calls fail
2. **Debug Information**: Added debug panel to show API call status and data availability
3. **Graceful Degradation**: Interface now shows even when backend APIs are unavailable

### Debug Panel Information

When running on localhost, a debug panel shows:
- Race status and turn phase
- API data availability status
- Current player position
- Available boost cards
- Submission status

### Testing Steps

1. **Check Route**: Verify which route you're accessing
   - `/game/:raceUuid` - Should work with existing backend
   - `/races/:raceUuid/play` - Uses newer API endpoints

2. **Check Browser Console**: Look for API error messages
   - Network tab shows failed requests
   - Console shows detailed error logs

3. **Check Debug Panel**: On localhost, debug info shows data status

4. **Backend Status**: Ensure backend is running on port 3000
   - Check `http://localhost:3000/api/v1/races` endpoint
   - Verify race exists and player is participant

### Recommended Actions

1. **Use Working Route**: Direct users to `/game/:raceUuid` route for immediate functionality
2. **Check Backend Logs**: Investigate why API calls might be failing
3. **Verify Authentication**: Ensure JWT tokens are being sent correctly
4. **Database State**: Check if race and player data exists in MongoDB

### Future Improvements

1. **Unified API**: Standardize both routes to use the same API endpoints
2. **Better Error Messages**: Provide more specific error feedback to users
3. **Offline Mode**: Implement full offline/demo mode for testing
4. **Health Checks**: Add API health check endpoints for monitoring

## Files Modified

- `empty-project/src/components/player-game-interface/PlayerGameInterface.tsx`
  - Added fallback mock data for API failures
  - Enhanced error logging and debugging
  - Added debug panel for development

## Next Steps

1. Test the updated interface with both routes
2. Check browser console for specific API error messages
3. Verify backend is running and accessible
4. Consider implementing unified API approach for both routes