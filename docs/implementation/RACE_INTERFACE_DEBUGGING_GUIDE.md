# Race Interface Debugging Guide

## Issue: Car Sprites and Boost Buttons Not Visible in `/races/:raceUuid/play`

### Current Status
User reports not seeing car sprites or boost buttons in the race play route, despite components being properly implemented.

### Debugging Steps

#### 1. Test Component Isolation
First, verify components work in isolation:
- Visit: `http://localhost:5173/test-interface`
- **Expected**: Should see car sprites and boost buttons clearly
- **If not visible**: Component implementation issue
- **If visible**: Data flow or route-specific issue

#### 2. Check Debug Panel
In the actual race route (`/races/:raceUuid/play`):
- Scroll to bottom of page
- Look for "Debug Info - RaceInterface" panel
- **Check these values**:
  - Car Data: Should show "✓ Loaded" or "✗ Missing"
  - Local View: Should show "✓ Loaded" or "✗ Missing"
  - Boost Availability: Should show "✓ Loaded" or "✗ Missing"
  - Visible Participants: Should show count > 0

#### 3. Browser Console Inspection
Open browser developer tools (F12) and check:
- **Console tab**: Look for error messages
- **Network tab**: Check if API calls are failing
- **Elements tab**: Inspect if components are in DOM but invisible

#### 4. Expected Console Messages
When using mock data (API failures), you should see:
```
[RaceContainer] Car data API failed, using mock data
[RaceContainer] Local view API failed, using mock data
[RaceContainer] Turn phase API failed, using mock data
```

### Component Structure Analysis

#### RaceInterface Layout
```
RaceInterface
├── Header (Race ID display)
├── RaceStatusPanel (if turnPhase exists)
├── Grid Layout (lg:grid-cols-4)
│   ├── Track Display (lg:col-span-3)
│   │   └── TrackDisplayRedesign
│   │       └── SectorGrid
│   │           └── PositionSlot
│   │               └── CarSprite (HERE)
│   └── Controls (lg:col-span-1)
│       ├── Player Car Info
│       ├── BoostControlPanel (HERE)
│       └── Turn Status
└── Debug Panel (localhost only)
```

#### Data Flow
```
RaceContainer → RaceInterface → TrackDisplayRedesign → SectorGrid → PositionSlot → CarSprite
RaceContainer → RaceInterface → BoostControlPanel
```

### Common Issues & Solutions

#### Issue 1: API Calls Failing Silently
**Symptoms**: Debug panel shows "✗ Missing" for all data
**Solution**: Check if backend is running on port 3000
**Test**: Visit `http://localhost:3000/health` - should return OK

#### Issue 2: Mock Data Not Loading
**Symptoms**: Console shows API failures but no mock data messages
**Solution**: Check RaceContainer fallback logic
**Location**: `empty-project/src/components/player-game-interface/RaceContainer.tsx`

#### Issue 3: Components Rendered But Invisible
**Symptoms**: Elements exist in DOM but not visible
**Possible Causes**:
- CSS z-index issues
- Transparent backgrounds
- Zero dimensions
- Overflow hidden

#### Issue 4: Authentication Issues
**Symptoms**: Redirected to login or user data missing
**Solution**: Ensure user is logged in and has valid UUID

### Manual Testing Checklist

#### ✅ Component Isolation Test
- [ ] Visit `/test-interface`
- [ ] Car sprites visible and colorful
- [ ] Boost buttons (0-5) visible and clickable
- [ ] Different sprite sizes work

#### ✅ Route Data Flow Test
- [ ] Visit `/races/ef1a6cf6-6c6f-46fa-96d7-260c1315a55c/play`
- [ ] Page loads without errors
- [ ] Debug panel shows data status
- [ ] Console shows mock data messages (if API fails)

#### ✅ Visual Inspection Test
- [ ] Track display section visible (left 3/4)
- [ ] Control panel visible (right 1/4)
- [ ] Sector grids show position slots
- [ ] Position slots contain car sprites
- [ ] Boost control panel shows 0-5 buttons

### Expected Mock Data Structure

#### Mock Participants (should show in debug panel)
```
Participants:
You - Sector 1, Pos 1
Player 2 - Sector 1, Pos 2  
Player 3 - Sector 2, Pos 1
```

#### Mock Boost Availability
```
Available boosts: 0, 1, 2, 3, 4
```

### Troubleshooting Commands

#### Check if frontend is running
```bash
# Should show process on port 5173
netstat -an | findstr :5173
```

#### Check if backend is running
```bash
# Should show process on port 3000
netstat -an | findstr :3000
```

#### Force refresh browser cache
- Ctrl+F5 (Windows)
- Cmd+Shift+R (Mac)

### Next Steps Based on Findings

#### If test-interface works but race route doesn't:
1. Check authentication state
2. Verify route parameters
3. Inspect API call failures
4. Check data transformation

#### If nothing is visible anywhere:
1. Check CSS compilation
2. Verify Tailwind CSS is working
3. Check for JavaScript errors
4. Verify component imports

#### If components exist but are invisible:
1. Inspect element styles in DevTools
2. Check for CSS conflicts
3. Verify responsive breakpoints
4. Check z-index stacking

### Contact Information
If issue persists after following this guide, provide:
1. Screenshot of `/test-interface` page
2. Screenshot of debug panel from race route
3. Browser console errors
4. Network tab showing API calls