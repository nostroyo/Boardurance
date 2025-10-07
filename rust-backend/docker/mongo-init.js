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

// Create test collections and indexes
db.createCollection('test_items');

// Create indexes for better performance
db.test_items.createIndex({ "uuid": 1 }, { unique: true });
db.test_items.createIndex({ "created_at": 1 });
db.test_items.createIndex({ "name": 1 });

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

print('MongoDB initialization completed successfully!');
print('Created database: rust_backend');
print('Created user: rust_app');
print('Created collection: test_items with indexes');
print('Inserted sample test data');