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

// Create indexes
db.test_items.createIndex({ "uuid": 1 }, { unique: true });
db.test_items.createIndex({ "created_at": 1 });

print('MongoDB test initialization completed!');
print('Created test database: rust_backend_test');
print('Created test user: test_user');