# Rust Backend API

A production-ready REST API built following Luca Palmieri's patterns from "Zero to Production in Rust", featuring Axum, MongoDB, and comprehensive observability.

## Architecture

This project follows the clean architecture principles outlined in "Zero to Production in Rust":

- **Domain Layer**: Core business logic and entities (`src/domain/`)
- **Application Layer**: Use cases and orchestration (`src/routes/`)
- **Infrastructure Layer**: External concerns (`src/startup.rs`, `src/configuration.rs`)
- **Presentation Layer**: HTTP handlers and serialization

## Features

- **Axum** - Fast, ergonomic web framework with excellent ecosystem
- **MongoDB** - Document database with async driver
- **Swagger UI** - Interactive API documentation via utoipa
- **Structured Logging** - JSON logging with tracing and bunyan formatter
- **Configuration Management** - Environment-based config with validation
- **Domain Modeling** - Type-safe domain entities with validation
- **Error Handling** - Comprehensive error types and proper propagation
- **Testability** - Clean separation of concerns for easy testing

## Quick Start

### Prerequisites

- Rust (latest stable)
- Docker and Docker Compose
- MongoDB (via Docker - see setup below)

### Setup

1. Navigate to the project:
```bash
cd rust-backend
```

2. Test the Docker setup:
```powershell
.\test-docker-setup.ps1
```

3. Start development environment:
```powershell
# Start with MongoDB
.\Makefile.ps1 dev

# Or start with MongoDB Express UI
.\Makefile.ps1 dev-ui
.\Makefile.ps1 dev-ui
```

The server will start on `http://localhost:3000`
MongoDB Express UI (if enabled): `http://localhost:8081`

### Configuration

The application uses a layered configuration system:

- `configuration/base.yaml` - Base configuration
- `configuration/local.yaml` - Local development overrides
- `configuration/production.yaml` - Production overrides
- Environment variables with `APP_` prefix

Set the environment with `APP_ENVIRONMENT=local|production`

## API Documentation

Once running, visit:
- **Swagger UI**: http://localhost:3000/swagger-ui
- **OpenAPI JSON**: http://localhost:3000/api-docs/openapi.json

## Available Endpoints

### Health Check
- `GET /health_check` - Check service and database health

### Test Items API
- `POST /api/v1/test` - Create a new test item
- `GET /api/v1/test` - Get all test items

## Docker MongoDB Setup

### Available Commands

```powershell
# Development
.\Makefile.ps1 dev          # Start with MongoDB
.\Makefile.ps1 dev-ui       # Start with MongoDB + UI
.\Makefile.ps1 test         # Run tests with test DB

# Database management
.\Makefile.ps1 db-start     # Start MongoDB only
.\Makefile.ps1 db-stop      # Stop all containers
.\Makefile.ps1 db-logs      # View MongoDB logs

# Build and maintenance
.\Makefile.ps1 build        # Build application
.\Makefile.ps1 check        # Check compilation
.\Makefile.ps1 clean        # Clean everything
```

### Database Environments

- **Development**: `localhost:27017` with user `rust_app`
- **Testing**: `localhost:27018` with user `test_user`
- **MongoDB Express**: `localhost:8081` (when using `dev-ui`)

### Docker Compose Files

- `docker-compose.yml` - Development MongoDB with sample data
- `docker-compose.test.yml` - Test MongoDB for integration tests

## Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root
├── configuration.rs     # Configuration management
├── startup.rs           # Application startup and dependency injection
├── telemetry.rs         # Logging and observability setup
├── domain/              # Domain entities and business logic
│   ├── mod.rs
│   └── test_item.rs     # Test item domain model
└── routes/              # HTTP handlers and routing
    ├── mod.rs
    ├── health_check.rs  # Health check endpoint
    └── test_items.rs    # Test items CRUD operations

configuration/
├── base.yaml            # Base configuration
├── local.yaml           # Local development config
└── production.yaml      # Production config
```

### Running Tests

```powershell
# Run all tests (infrastructure + API + unit)
.\Makefile.ps1 test

# Run specific test suites
.\tests\run-all-tests.ps1 -TestSuite api        # API tests only
.\tests\run-all-tests.ps1 -TestSuite infrastructure  # Infrastructure only
.\tests\run-all-tests.ps1 -TestSuite unit       # Unit tests only

# Individual test categories
.\tests\api\test-auth-endpoints.ps1             # Authentication tests
.\tests\api\test-player-endpoints.ps1           # Player management tests
.\tests\infrastructure\test-with-mongodb.ps1    # MongoDB integration

# Rust unit tests
cargo test
```

#### Test Structure
- `tests/api/` - API endpoint tests (requires running server)
- `tests/infrastructure/` - Docker, MongoDB, project structure tests
- `tests/run-all-tests.ps1` - Comprehensive test runner with reporting

### Building for Production

```bash
cargo build --release
```

## Configuration Examples

### Environment Variables

```bash
# Application settings
APP_APPLICATION__PORT=8000
APP_APPLICATION__HOST=0.0.0.0

# Database settings
APP_DATABASE__HOST=mongodb.example.com
APP_DATABASE__PORT=27017
APP_DATABASE__USERNAME=api_user
APP_DATABASE__PASSWORD=secure_password
APP_DATABASE__DATABASE_NAME=production_db
APP_DATABASE__REQUIRE_SSL=true
```

### Local Development

Create `configuration/local.yaml`:
```yaml
application:
  port: 3000
database:
  host: "localhost"
  port: 27017
  database_name: "rust_backend_dev"
```

## Observability

The application includes comprehensive logging:

- **Structured JSON logs** for production
- **Request tracing** with correlation IDs
- **Database operation tracing**
- **Error context preservation**

Logs include:
- Request/response details
- Database query performance
- Error stack traces with context
- Business operation outcomes

## Error Handling

Following Rust best practices:

- **Domain errors** for business logic violations
- **Infrastructure errors** for external service failures
- **Proper error propagation** with context
- **User-friendly error responses**

## Security Considerations

- Input validation at domain boundaries
- Secure configuration management with `secrecy`
- SQL injection prevention through typed queries
- CORS configuration for cross-origin requests

## Deployment

The application is designed for containerized deployment:

1. **Configuration** via environment variables
2. **Health checks** for orchestrator integration
3. **Graceful shutdown** handling
4. **Structured logging** for log aggregation

## Learning Resources

This implementation demonstrates patterns from:
- "Zero to Production in Rust" by Luca Palmieri
- Domain-Driven Design principles
- Clean Architecture patterns
- Rust async programming best practices