# Player Creation Test

## Test the new player creation with 6 pilots

### Test Request
```bash
curl -X POST http://localhost:8000/api/v1/players \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "team_name": "Racing Team Alpha"
  }'
```

### Expected Response
The response should include:
- 1 Player with email and team name
- 2 Cars ("Car 1", "Car 2")
- 6 Pilots with different classes:
  1. "Speedster Ace" (Speedster, Rookie)
  2. "Tech Master" (Technician, Rookie) 
  3. "Endurance Pro" (Endurance, Rookie)
  4. "All-Round Rookie" (AllRounder, Rookie)
  5. "Speed Demon" (Speedster, Professional)
  6. "Precision Driver" (Technician, Professional)
- 6 Engines with different stats
- 6 Bodies with different stats

### Pilot Details
Each pilot should have:
- Unique UUID
- Name and class as specified
- Skills appropriate to their class
- Performance values matching their specialization
- Ready to be assigned to cars for racing

### Racing Readiness
With this setup, players can immediately:
- Assign pilots to cars
- Assign engines and bodies to cars
- Join races with properly equipped vehicles
