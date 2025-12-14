# Backend Documentation

This folder contains documentation specific to the Rust backend API server.

## Contents

- `BACKEND_README.md` - Rust backend API documentation
- `BACKEND_DOCKER_SETUP.md` - Docker MongoDB setup guide

## Technology Stack

- **Rust** with **Axum 0.7** web framework
- **MongoDB 2.8** with async driver
- **Tokio** async runtime
- **JWT authentication**
- **Structured logging** with tracing

## Key Features

- RESTful API endpoints
- Domain-driven design
- Comprehensive error handling
- OpenAPI/Swagger documentation
- MongoDB integration

## Development

```powershell
cd rust-backend
.\Makefile.ps1 dev      # Start with MongoDB
.\Makefile.ps1 test     # Run all tests
```

## Related Features

- [Architecture](../08-architecture/) - Design patterns
- [Authentication](../04-authentication/) - Auth implementation
- [Racing System](../02-racing-system/) - Race API
