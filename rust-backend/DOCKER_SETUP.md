# Docker MongoDB Setup for Rust Backend

This document describes the complete Docker MongoDB setup for the Rust backend following Luca Palmieri's patterns.

## 🐳 Docker Configuration

### Development Environment
- **Container**: `rust-backend-mongodb`
- **Port**: `27017`
- **Database**: `rust_backend`
- **User**: `rust_app` / `rust_password`
- **Features**: Sample data, indexes, MongoDB Express UI

### Test Environment  
- **Container**: `rust-backend-mongodb-test`
- **Port**: `27018`
- **Database**: `rust_backend_test`
- **User**: `test_user` / `test_password`
- **Features**: Clean test database, isolated from development

## 📁 File Structure

```
rust-backend/
├── docker-compose.yml              # Development MongoDB
├── docker-compose.test.yml         # Test MongoDB
├── docker/
│   ├── mongo-init.js              # Development DB initialization
│   └── mongo-test-init.js         # Test DB initialization
├── scripts/
│   ├── start-mongodb.ps1          # Start development MongoDB
│   ├── start-test-mongodb.ps1     # Start test MongoDB
│   ├── stop-mongodb.ps1           # Stop all containers
│   └── test-with-mongodb.ps1      # Full integration test
├── configuration/
│   ├── base.yaml                  # Base configuration
│   ├── local.yaml                 # Development overrides
│   ├── test.yaml                  # Test environment config
│   └── production.yaml            # Production config
└── Makefile.ps1                   # Command shortcuts
```

## 🚀 Quick Commands

```powershell
# Development
.\Makefile.ps1 dev          # Start MongoDB + application
.\Makefile.ps1 dev-ui       # Start with MongoDB Express UI

# Testing
.\Makefile.ps1 test         # Run integration tests

# Database Management
.\Makefile.ps1 db-start     # Start MongoDB only
.\Makefile.ps1 db-stop      # Stop all containers
.\Makefile.ps1 db-logs      # View MongoDB logs

# Maintenance
.\Makefile.ps1 build        # Build application
.\Makefile.ps1 clean        # Clean everything
```

## 🔧 Configuration

### Environment Variables

```bash
# Set environment
APP_ENVIRONMENT=local|test|production

# Override database settings
APP_DATABASE__HOST=localhost
APP_DATABASE__PORT=27017
APP_DATABASE__USERNAME=rust_app
APP_DATABASE__PASSWORD=rust_password
```

### Connection Strings

```
Development: mongodb://rust_app:rust_password@localhost:27017/rust_backend
Test:        mongodb://test_user:test_password@localhost:27018/rust_backend_test
```

## 📊 Database Initialization

### Development Database
- Creates `test_items` collection with indexes
- Inserts sample test data
- Sets up proper user permissions
- Configures performance indexes

### Test Database
- Clean database for each test run
- Minimal setup for fast test execution
- Isolated from development data

## 🌐 MongoDB Express UI

When using `.\Makefile.ps1 dev-ui`:
- **URL**: http://localhost:8081
- **Features**: Browse collections, run queries, manage data
- **Authentication**: Disabled for development ease

## 🧪 Testing Strategy

### Integration Tests
1. Start clean test MongoDB on port 27018
2. Run application in test mode
3. Execute API tests against test database
4. Clean up containers after tests

### Test Isolation
- Separate database instance for tests
- Different port to avoid conflicts
- Clean state for each test run

## 🔒 Security Considerations

### Development
- Simple credentials for ease of use
- No SSL required for local development
- MongoDB Express UI without authentication

### Production
- Strong passwords via environment variables
- SSL/TLS encryption enabled
- Restricted network access
- No MongoDB Express UI

## 📋 Troubleshooting

### Common Issues

1. **Port conflicts**: Change ports in docker-compose files
2. **Permission errors**: Ensure Docker has proper permissions
3. **Connection timeouts**: Wait for MongoDB to fully start
4. **Data persistence**: Development data persists in Docker volumes

### Useful Commands

```powershell
# Check container status
docker ps

# View MongoDB logs
docker logs rust-backend-mongodb

# Connect to MongoDB directly
docker exec -it rust-backend-mongodb mongosh

# Reset everything
docker-compose down -v
docker system prune -f
```

## 🎯 Benefits

1. **Consistent Environment**: Same MongoDB version across team
2. **Easy Setup**: One command to start everything
3. **Test Isolation**: Separate test database prevents conflicts
4. **Development Tools**: MongoDB Express for database management
5. **Production Ready**: Configuration system supports all environments
6. **Luca Palmieri Patterns**: Follows "Zero to Production" best practices

## 📚 Next Steps

1. Install Docker Desktop
2. Run `.\verify-docker-setup.ps1` to check setup
3. Use `.\Makefile.ps1 dev` to start development
4. Access Swagger UI at http://localhost:3000/swagger-ui
5. Use MongoDB Express at http://localhost:8081 (with dev-ui)

This setup provides a complete, production-ready development environment following modern Rust backend practices.