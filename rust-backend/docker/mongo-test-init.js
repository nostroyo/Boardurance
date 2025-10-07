// MongoDB initialization script for test environment
print('Starting MongoDB test initialization...');

// Switch to the test database
db = db.getSiblingDB('rust_backend_test');

// Create a user for testing
db.createUser({
  user: 'test_user',
  pwd: 'test_password',
  roles: [
    {
      role: 'readWrite',
      db: 'rust_backend_test'
    }
  ]
});

// Create test collections
db.createCollection('test_items');
db.createCollection('players');

// Create indexes
// Test items indexes
db.test_items.createIndex({ "uuid": 1 }, { unique: true });
db.test_items.createIndex({ "created_at": 1 });

// Players indexes
db.players.createIndex({ "wallet_address": 1 }, { unique: true });
db.players.createIndex({ "uuid": 1 }, { unique: true });
db.players.createIndex({ "team_name": 1 });
db.players.createIndex({ "created_at": 1 });

print('MongoDB test initialization completed!');
print('Created test database: rust_backend_test');
print('Created test user: test_user');
print('Created collections: test_items, players with indexes');