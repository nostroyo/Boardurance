// MongoDB initialization script for test environment
print('Starting MongoDB initialization for rust-backend...');

// Switch to the rust_backend database
db = db.getSiblingDB('rust_backend');

// Create a user for the application
db.createUser({
  user: 'rust_app',
  pwd: 'rust_password',
  roles: [
    {
      role: 'readWrite',
      db: 'rust_backend'
    }
  ]
});

// Create collections and indexes
db.createCollection('test_items');
db.createCollection('players');

// Create indexes for better performance
// Test items indexes
db.test_items.createIndex({ "uuid": 1 }, { unique: true });
db.test_items.createIndex({ "created_at": 1 });
db.test_items.createIndex({ "name": 1 });

// Players indexes
db.players.createIndex({ "wallet_address": 1 }, { unique: true });
db.players.createIndex({ "uuid": 1 }, { unique: true });
db.players.createIndex({ "team_name": 1 });
db.players.createIndex({ "created_at": 1 });
db.players.createIndex({ "cars.uuid": 1 });
db.players.createIndex({ "pilots.uuid": 1 });

// Insert some test data
db.test_items.insertMany([
  {
    uuid: "550e8400-e29b-41d4-a716-446655440001",
    name: "Sample Test Item 1",
    description: "This is a sample test item created during database initialization",
    created_at: new Date(),
    updated_at: new Date()
  },
  {
    uuid: "550e8400-e29b-41d4-a716-446655440002", 
    name: "Sample Test Item 2",
    description: "Another sample test item for testing purposes",
    created_at: new Date(),
    updated_at: new Date()
  }
]);

// Insert sample player data
db.players.insertMany([
  {
    uuid: "550e8400-e29b-41d4-a716-446655440010",
    wallet_address: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
    team_name: "Lightning Racers",
    cars: [
      {
        uuid: "550e8400-e29b-41d4-a716-446655440011",
        nft_mint_address: "CarNFT123456789",
        name: "Thunder Bolt",
        car_type: "Sports",
        rarity: "Rare",
        stats: {
          speed: 85,
          acceleration: 80,
          handling: 75,
          durability: 70
        },
        is_equipped: true,
        created_at: new Date(),
        updated_at: new Date()
      },
      {
        uuid: "550e8400-e29b-41d4-a716-446655440012",
        nft_mint_address: "CarNFT987654321",
        name: "Speed Demon",
        car_type: "Racing",
        rarity: "Epic",
        stats: {
          speed: 95,
          acceleration: 90,
          handling: 85,
          durability: 75
        },
        is_equipped: false,
        created_at: new Date(),
        updated_at: new Date()
      }
    ],
    pilots: [
      {
        uuid: "550e8400-e29b-41d4-a716-446655440013",
        nft_mint_address: "PilotNFT111222333",
        name: "Alex Thunder",
        pilot_class: "Speedster",
        rarity: "Professional",
        skills: {
          reaction_time: 85,
          precision: 70,
          focus: 80,
          stamina: 75
        },
        experience_level: 15,
        is_active: true,
        created_at: new Date(),
        updated_at: new Date()
      },
      {
        uuid: "550e8400-e29b-41d4-a716-446655440014",
        nft_mint_address: "PilotNFT444555666",
        name: "Sarah Velocity",
        pilot_class: "Technician",
        rarity: "Expert",
        skills: {
          reaction_time: 75,
          precision: 90,
          focus: 85,
          stamina: 80
        },
        experience_level: 25,
        is_active: false,
        created_at: new Date(),
        updated_at: new Date()
      }
    ],
    created_at: new Date(),
    updated_at: new Date()
  }
]);

print('MongoDB initialization completed successfully!');
print('Created database: rust_backend');
print('Created user: rust_app');
print('Created collections: test_items, players with indexes');
print('Inserted sample test data and player data');