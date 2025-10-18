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
db.players.createIndex({ "wallet_address": 1 }, { unique: true, sparse: true });
db.players.createIndex({ "uuid": 1 }, { unique: true });
db.players.createIndex({ "email": 1 }, { unique: true });
db.players.createIndex({ "team_name": 1 });
db.players.createIndex({ "created_at": 1 });
db.players.createIndex({ "cars.uuid": 1 });
db.players.createIndex({ "pilots.uuid": 1 });
db.players.createIndex({ "engines.uuid": 1 });
db.players.createIndex({ "bodies.uuid": 1 });

print('MongoDB initialization completed successfully!');
print('Created database: rust_backend');
print('Created user: rust_app');
print('Created collections: test_items, players with indexes');
print('Database is ready for use - no test data inserted');