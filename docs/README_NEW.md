# Racing Game Project Documentation

This folder contains all documentation for the racing management game project.

## 📁 Feature-Based Documentation Structure

Documentation is organized by feature for easy navigation and maintenance:

### 🎯 [01 - Core Project](features/01-core-project/)
General project documentation, setup guides, and overview materials.
- Project overview and architecture
- Quick start guide
- Technology stack
- Development workflow
- Team structure

### 🏁 [02 - Racing System](features/02-racing-system/)
Core racing mechanics, gameplay systems, and race management.
- Game mechanics and rules
- Race API endpoints
- Movement and positioning system
- Turn-based gameplay
- Implementation details

### 🚀 [03 - Boost Card System](features/03-boost-card-system/)
Strategic boost card mechanics and resource management.
- Boost card API documentation
- 5-card hand system
- Cycle-based replenishment
- Performance multipliers
- Usage examples and patterns

### 🔐 [04 - Authentication](features/04-authentication/)
User authentication, authorization, and session management.
- JWT token management
- Protected routes
- Session handling
- Auth middleware

### 🚗 [06 - Player & Car Management](features/06-player-car-management/)
Player management, car systems, and pilot mechanics.
- Car component system (Engine, Body, Pilot)
- Performance calculations
- Multi-pilot support
- Player game context
- Automatic pilot creation

### 🧪 [07 - Testing](features/07-testing/)
Testing documentation, strategies, and test suites.
- Comprehensive testing guide
- Backend test suite
- Boost card integration tests
- Player creation tests
- Test reorganization

### 🏗️ [08 - Architecture](features/08-architecture/)
System architecture and design patterns.
- Frontend/backend separation
- Domain-driven design
- Hexagonal architecture
- Component architecture

### ⚙️ [09 - Backend](features/09-backend/)
Rust backend API server documentation.
- Backend API documentation
- Docker MongoDB setup
- Rust + Axum framework
- Domain-driven design
- API endpoints

### 🎨 [10 - Frontend](features/10-frontend/)
React frontend application documentation.
- Frontend documentation
- UI improvements
- React + TypeScript
- Component architecture

## 🚀 Quick Start

1. **Frontend**: React + TypeScript + Vite web application (port 5173)
2. **Backend**: Rust + Axum + MongoDB API server (port 3000)

See [Quick Start Guide](features/01-core-project/QUICK_START.md) for detailed setup instructions.

## 🏗️ Architecture Overview

```
Racing Game Project
├── Frontend (React)     # Game UI
└── Backend (Rust)       # API server with MongoDB
```

See [Architecture Documentation](features/08-architecture/) for detailed architecture information.

## 📚 Documentation Navigation

### By Role

**Game Developers:**
- Start with [Racing System](features/02-racing-system/) for game mechanics
- Review [Boost Card System](features/03-boost-card-system/) for strategic elements
- Check [Player & Car Management](features/06-player-car-management/) for car systems

**Backend Developers:**
- Start with [Backend](features/09-backend/) for API documentation
- Review [Architecture](features/08-architecture/) for design patterns
- Check [Authentication](features/04-authentication/) for auth implementation

**Frontend Developers:**
- Start with [Frontend](features/10-frontend/) for UI documentation
- Review [Architecture](features/08-architecture/) for component structure
- Check [Racing System](features/02-racing-system/) for game UI requirements

**QA/Testing:**
- Start with [Testing](features/07-testing/) for test strategies
- Review feature-specific implementation docs for test scenarios

### By Task

**Setting Up Development Environment:**
1. [Quick Start Guide](features/01-core-project/QUICK_START.md)
2. [Backend Docker Setup](features/09-backend/BACKEND_DOCKER_SETUP.md)
3. [Technology Stack](features/01-core-project/TECHNOLOGY_STACK.md)

**Understanding Game Mechanics:**
1. [Game Mechanics](features/02-racing-system/GAME_MECHANICS.md)
2. [Boost Card System](features/03-boost-card-system/BOOST_CARD_API.md)
3. [Player & Car Management](features/06-player-car-management/)

**API Integration:**
1. [API Routes](features/02-racing-system/API_ROUTES.md)
2. [Boost Card API](features/03-boost-card-system/BOOST_CARD_API.md)
3. [Backend Documentation](features/09-backend/)

**Testing:**
1. [Testing Guide](features/07-testing/TESTING_GUIDE.md)
2. [Backend Test Suite](features/07-testing/BACKEND_TEST_SUITE.md)
3. Feature-specific test documentation

## 🤝 Contributing Documentation

When adding new documentation:

1. **Identify the Feature** - Determine which feature folder it belongs to
2. **Create or Update** - Add to existing feature or create new feature folder
3. **Add README** - Ensure feature folder has a README.md
4. **Cross-Reference** - Link to related features
5. **Update This File** - Add links to new documentation in this README

### Documentation Standards

- Use Markdown format
- Include code examples where relevant
- Add cross-references to related documentation
- Keep implementation details in `implementation/` subfolders
- Use clear, descriptive headings
- Include diagrams for complex concepts

### Feature Folder Structure

```
features/XX-feature-name/
├── README.md                    # Feature overview and navigation
├── FEATURE_MAIN_DOC.md         # Main feature documentation
├── FEATURE_API.md              # API documentation (if applicable)
└── implementation/             # Implementation details
    └── IMPLEMENTATION_DOC.md
```

## 🔗 External Resources

- **Rust Backend**: Based on "Zero to Production in Rust" by Luca Palmieri
- **React Frontend**: Modern React development practices

## 📋 Migration Notes

This documentation structure was reorganized from a flat structure to a feature-based hierarchy on [DATE]. See [REORGANIZATION_PLAN.md](REORGANIZATION_PLAN.md) for details.

---

**Navigate by feature, find what you need quickly, and build amazing racing experiences! 🏁🚗💨**
