var player = db.players.findOne({"email": "tho994@gmail.com"}, {"role": 1, "email": 1});
if (player) {
    print("Player:", player.email);
    print("Current role:", player.role);
} else {
    print("Player not found");
}