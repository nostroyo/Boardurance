// Update the player role to Admin
var result = db.players.updateOne(
    {"email": "tho994@gmail.com"},
    {"$set": {"role": "Admin", "updated_at": "2025-11-02T08:55:00.000Z"}}
);

if (result.modifiedCount > 0) {
    print("✅ Successfully updated tho994@gmail.com to Admin role");
    
    // Verify the update
    var player = db.players.findOne({"email": "tho994@gmail.com"}, {"role": 1, "email": 1});
    print("Verified - Player:", player.email, "| Role:", player.role);
} else {
    print("❌ Failed to update player role");
}