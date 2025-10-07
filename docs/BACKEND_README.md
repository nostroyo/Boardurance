# Backend - Rust API Server

A production-ready REST API built following Luca Palmieri's patterns from "Zero to Production in Rust", featuring Axum, MongoDB, and comprehensive observability.

## 🏗️ Architecture

This project follows the clean architecture principles outlined in "Zero to Production in Rust":

- **Domain Layer**: Core business logic and entities (`src/domain/`)
- **Application Layer**: Use cases and orchestration (`src/routes/`)
- **Infrastructure Layer**: External concerns (`src/startup.rs`, `src/configuration.rs`)
- **Presentation Layer**: HTTP handlers and serialization

## ✨ Features

- **Axum** - Fast, ergonomic web framework with excellent ecosystem
- **MongoDB** - Document database with async driver
- **Swagger UI** - Interactive API documentation via utoipa
- **Structured Logging** - JSON logging with tracing and bunyan formatter
- **Configuration Management** - Environment-based config with validation
- **Domain Modeling** - Type-safe domain entities with validation
- **Error Handling** - Comprehensive error types and proper propagation
- **Testability** - Clean separation of concerns for easy testing

## 🚀 Quick Start

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
```

The server will start on `http://localhost:3000`
MongoDB Express UI (if enabled): `http://localhost:8081`

## ⚙️ Configuration

The application uses a layered configuration system:

- `configuration/base.yaml` - Base configuration
- `configuration/local.yaml` - Local development overrides
- `configuration/test.yaml` - Test environment config
- `configuration/production.yaml` - Production overrides
- Environment variables with `APP_` prefix

Set the environment with `APP_ENVIRONMENT=local|test|production`

## 📚 API Documentation

Once running, visit:
- **Swagger UI**: http://localhost:3000/swagger-ui
- **OpenAPI JSON**: http://localhost:3000/api-docs/openapi.json

## 🔗 Available Endpoints

### Health Check
- `GET /health_check` - Check service and database health

### Test Items API
- `POST /api/v1/test` - Create a new test item
- `GET /api/v1/test` - Get all test items

## 📁 Project Structure

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
├── test.yaml            # Test environment config
└── production.yaml      # Production config
```

## 🧪 Development

### Running Tests

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Building for Production

```bash
cargo build --release
```

## 📊 Observability

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

## 🛡️ Error Handling

Following Rust best practices:

- **Domain errors** for business logic violations
- **Infrastructure errors** for external service failures
- **Proper error propagation** with context
- **User-friendly error responses**

## 🔒 Security Considerations

- Input validation at domain boundaries
- Secure configuration management with `secrecy`
- SQL injection prevention through typed queries
- CORS configuration for cross-origin requests

## 🚀 Deployment

The application is designed for containerized deployment:

1. **Configuration** via environment variables
2. **Health checks** for orchestrator integration
3. **Graceful shutdown** handling
4. **Structured logging** for log aggregation

## 📚 Learning Resources

This implementation demonstrates patterns from:
- "Zero to Production in Rust" by Luca Palmieri
- Domain-Driven Design principles
- Clean Architecture patterns
- Rust async programming best practices