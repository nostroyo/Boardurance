# Technology Stack & Build System

## Architecture Overview

Two-tier gaming architecture:
- **Frontend**: React + TypeScript + Vite (port 5173)
- **Backend**: Rust + Axum + MongoDB (port 3000)

## Frontend Stack

### Core Technologies
- **React 19.1.1** with TypeScript 5.8.3
- **Vite 7.1.2** for fast builds and HMR
- **Tailwind CSS 3.4.17** for utility-first styling
- **React Router DOM** for client-side routing

### Development Tools
- **ESLint 9.33.0** with TypeScript rules
- **Prettier 3.6.2** for code formatting
- **Vitest** for unit testing

## Backend Stack

### Core Framework
- **Rust** with **Axum 0.7** web framework
- **MongoDB 2.8** with async driver
- **Tokio** async runtime

### Architecture Patterns
- **Domain-Driven Design** following "Zero to Production in Rust" by Luca Palmieri
- **Hexagonal Architecture** with ports and adapters
- **Clean separation**: Domain → Routes → Infrastructure

### Key Libraries
- **utoipa 4.0** for OpenAPI/Swagger documentation
- **tracing** with bunyan formatter for structured JSON logging
- **config 0.14** for layered configuration management
- **secrecy** for secure configuration handling
- **jsonwebtoken** for JWT authentication

## Common Build Commands

### Full Stack Startup
```powershell
# Start entire stack (recommended)
.\start-full-stack.ps1

# Stop everything
.\stop-full-stack.ps1
```

### Frontend Development
```bash
cd empty-project
npm run dev          # Development server (port 5173)
npm run build        # Production build
npm run lint         # ESLint checking
npm run format       # Prettier formatting
```

### Backend Development
```powershell
cd rust-backend
.\Makefile.ps1 dev      # Start with MongoDB
.\Makefile.ps1 dev-ui   # Start with MongoDB Express UI
.\Makefile.ps1 test     # Run all tests
.\Makefile.ps1 build    # Build application
cargo check             # Quick compilation check
cargo clippy            # Rust linting
cargo fmt               # Rust formatting
```

### Testing Commands
```powershell
# Backend comprehensive testing
cd rust-backend
.\tests\run-all-tests.ps1

# Specific test suites
.\tests\run-all-tests.ps1 -TestSuite api
.\tests\run-all-tests.ps1 -TestSuite infrastructure

# Frontend testing
cd empty-project
.\test-frontend-auth.ps1
```

## Development Environment

### Required Tools
- **Docker Desktop** for MongoDB containers
- **Rust** (latest stable) with Cargo
- **Node.js** (v18+) with npm

### Port Configuration
- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:3000
- **API Documentation**: http://localhost:3000/swagger-ui
- **MongoDB**: mongodb://localhost:27017
- **MongoDB Express**: http://localhost:8081 (when enabled)

### Configuration Management
- **Backend**: Layered YAML configs in `configuration/` folder
- **Environment variables**: `APP_` prefix for backend settings
- **Frontend**: Vite environment variables with `VITE_` prefix

## Code Quality Standards

### Rust Backend
- **Clippy pedantic** linting enabled
- **rustfmt** for consistent formatting
- **Comprehensive error handling** with thiserror
- **Structured logging** with tracing and JSON output

### React Frontend
- **TypeScript strict mode** enabled
- **ESLint** with React and TypeScript rules
- **Prettier** for consistent formatting
- **Component-based architecture** with hooks

### Performance Targets
- **Frontend**: <1.5s First Contentful Paint
- **Backend**: <100ms API response times (95th percentile)