# Racing Game Project Documentation

This folder contains all documentation for the racing management game project.

## 📁 Documentation Structure

### Getting Started
- `QUICK_START.md` - Quick start guide to get up and running
- `PROJECT_OVERVIEW.md` - Complete project architecture and overview
- `TECHNOLOGY_STACK.md` - Technologies used across all components
- `DEVELOPMENT_WORKFLOW.md` - Development standards and workflow

### Frontend (React)
- `FRONTEND_README.md` - React frontend documentation
- `UI_IMPROVEMENTS.md` - Interface enhancements and design

### Backend (Rust)
- `BACKEND_README.md` - Rust backend API documentation
- `BACKEND_DOCKER_SETUP.md` - Docker MongoDB setup guide
- `API_ROUTES.md` - Complete API endpoint reference

### Game Mechanics
- `GAME_MECHANICS.md` - Core gameplay systems and rules
- `BOOST_CARD_API.md` - Boost card mechanics and API
- `BOOST_CARD_EXAMPLES.md` - Usage examples and patterns
- `openapi-boost-cards.yaml` - OpenAPI specification for boost cards
- `MOVEABLE_ITEMS_IMPLEMENTATION.md` - Dynamic item system

### Testing Documentation (`testing/`)
- `TESTING_GUIDE.md` - Comprehensive testing strategies
- `testing/BACKEND_TEST_SUITE.md` - Backend test organization and usage
- `testing/BOOST_CARD_INTEGRATION_TESTS.md` - Boost card test suite
- `testing/TEST_PLAYER_CREATION.md` - Player creation test scenarios

### Implementation Documentation (`implementation/`)
- `implementation/CAR_PILOTS_UPDATE_SUMMARY.md` - Multi-pilot car system
- `implementation/PILOT_CREATION_IMPLEMENTATION.md` - Automatic pilot creation
- `implementation/TEST_REORGANIZATION_SUMMARY.md` - Test suite restructuring

## 🚀 Quick Start

1. **Frontend**: React + TypeScript + Vite web application (port 5173)
2. **Backend**: Rust + Axum + MongoDB API server (port 3000)

See [QUICK_START.md](QUICK_START.md) for detailed setup instructions.

## 🏗️ Architecture

```
Racing Game Project
├── Frontend (React)     # Game UI
└── Backend (Rust)       # API server with MongoDB
```

## 📚 Learning Resources

This project demonstrates patterns from:
- "Zero to Production in Rust" by Luca Palmieri (Backend)
- Modern React development practices (Frontend)

## 🤝 Contributing Documentation

When adding new documentation:
1. Place files in the appropriate subdirectory:
   - `testing/` - Test documentation and guides
   - `implementation/` - Feature implementation summaries
   - Root level - Core project documentation
2. Update this README with links to new documents
3. Follow the existing documentation style
4. Include code examples where relevant
5. Add cross-references to related documentation

## 🔗 Related Files

- Configuration files in each project's root
- Development scripts in `scripts/` folders
- Docker configurations for local development
- CI/CD configurations in `.github/` folders
