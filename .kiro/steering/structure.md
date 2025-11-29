# Project Organization & Structure

## Repository Layout

```
web3-game-project/
├── empty-project/           # React Frontend Application
├── rust-backend/           # Rust API Server
├── solana-smart-contract/  # Solana Smart Contracts
├── docs/                   # Centralized Documentation
├── start-full-stack.ps1    # Full stack startup script
└── stop-full-stack.ps1     # Full stack shutdown script
```

## Frontend Structure (empty-project/)

### Core Organization
```
empty-project/
├── src/
│   ├── components/         # Reusable UI components
│   │   ├── common/        # Shared components across app
│   │   ├── game/          # Game-specific components
│   │   ├── wallet/        # Web3 wallet components
│   │   └── player-game-interface/  # Player interface components
│   ├── pages/             # Application pages/routes
│   ├── hooks/             # Custom React hooks
│   ├── utils/             # Utility functions
│   ├── types/             # TypeScript type definitions
│   ├── services/          # API and external services
│   └── assets/            # Static files
├── public/                # Public static files
├── index.html             # Main HTML entry point
├── standalone-login.html  # Standalone auth testing
├── package.json           # Dependencies and scripts
├── vite.config.ts         # Vite configuration
├── tailwind.config.js     # Tailwind CSS configuration
├── tsconfig.json          # TypeScript configuration
└── test-frontend-auth.ps1 # Frontend testing script
```

### Component Architecture
- **GameWrapper.tsx** - Main game container component
- **GameLobby.tsx** - Game lobby and matchmaking
- **PlayerGameInterface/** - Player interaction components
- Follow React functional components with hooks pattern
- Use TypeScript for all components with proper typing

## Backend Structure (rust-backend/)

### Domain-Driven Design Layout
```
rust-backend/
├── src/
│   ├── main.rs            # Application entry point
│   ├── lib.rs             # Library root and re-exports
│   ├── startup.rs         # Application startup and DI
│   ├── configuration.rs   # Config management
│   ├── telemetry.rs       # Logging and observability
│   ├── domain/            # Business logic and entities
│   │   ├── mod.rs
│   │   ├── race.rs        # Race domain logic
│   │   ├── boost_hand_manager.rs  # Boost card management
│   │   └── [entity].rs    # Other domain entities
│   └── routes/            # HTTP handlers and routing
│       ├── mod.rs
│       ├── auth.rs        # Authentication endpoints
│       ├── players.rs     # Player management
│       ├── races.rs       # Race endpoints
│       └── health_check.rs # Health monitoring
├── configuration/         # Environment-based configs
│   ├── base.yaml         # Base configuration
│   ├── local.yaml        # Local development
│   └── production.yaml   # Production settings
├── docker/               # Docker setup files
├── scripts/              # Development automation
├── tests/                # Integration and API tests
│   ├── api/              # API endpoint tests
│   ├── infrastructure/   # Infrastructure tests
│   └── run-all-tests.ps1 # Test runner
├── Cargo.toml            # Rust dependencies
├── Makefile.ps1          # Development commands
└── docker-compose.yml    # MongoDB setup
```

### Architecture Patterns
- **Domain Layer**: Pure business logic in `src/domain/`
- **Application Layer**: HTTP handlers in `src/routes/`
- **Infrastructure Layer**: External concerns in startup/config
- Follow "Zero to Production in Rust" patterns by Luca Palmieri

## Blockchain Structure (solana-smart-contract/)

### Anchor Framework Layout
```
solana-smart-contract/
├── programs/             # Smart contract source code
│   └── [program-name]/   # Individual program folders
├── tests/                # Contract tests and simulations
├── assets/               # NFT metadata and images
│   ├── collection.json   # Collection metadata
│   └── 0.json - 99.json  # Individual NFT metadata
├── docs/                 # Blockchain-specific documentation
├── Anchor.toml           # Anchor configuration
├── Cargo.toml            # Rust workspace configuration
├── package.json          # Node.js dependencies
├── generate-metadata.js  # NFT metadata generation
├── config.json           # Sugar CLI configuration
└── deploy.bat            # Deployment script
```

### Smart Contract Organization
- **programs/** - Anchor smart contracts in Rust
- **tests/** - TypeScript/JavaScript test files
- **assets/** - NFT collection metadata and images
- Follow Metaplex standards for NFT compatibility

## Documentation Structure (docs/)

### Centralized Documentation
```
docs/
├── PROJECT_OVERVIEW.md        # Complete project architecture
├── TECHNOLOGY_STACK.md        # Tech stack details
├── DEVELOPMENT_WORKFLOW.md    # Development standards
├── API_ROUTES.md             # API endpoint documentation
├── BOOST_CARD_API.md         # Boost card system API
├── BOOST_CARD_EXAMPLES.md    # Usage examples
├── openapi-boost-cards.yaml  # OpenAPI specification
├── FRONTEND_README.md        # React app documentation
├── BACKEND_README.md         # Rust API documentation
├── SOLANA_README.md          # Smart contract docs
├── TESTING_GUIDE.md          # Testing strategies
└── [feature].md              # Feature-specific docs
```

## File Naming Conventions

### React Frontend
- **Components**: PascalCase (`UserProfile.tsx`)
- **Hooks**: camelCase with "use" prefix (`useAuth.ts`)
- **Utils**: camelCase (`formatDate.ts`)
- **Types**: PascalCase (`UserTypes.ts`)
- **Pages**: PascalCase (`GameDashboard.tsx`)

### Rust Backend
- **Modules**: snake_case (`user_management.rs`)
- **Functions**: snake_case (`create_user`)
- **Types**: PascalCase (`UserProfile`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_RETRY_ATTEMPTS`)

### Solana Contracts
- **Programs**: snake_case (`car_nft_program.rs`)
- **Instructions**: snake_case (`mint_car_nft.rs`)
- **Tests**: snake_case (`test_car_minting.ts`)

## Configuration Management

### Environment-Specific Configs
- **Backend**: YAML files in `configuration/` with `APP_` env vars
- **Frontend**: Vite env vars with `VITE_` prefix
- **Blockchain**: Anchor.toml and config.json for different clusters

### Development Scripts
- **Root level**: `start-full-stack.ps1`, `stop-full-stack.ps1`
- **Backend**: `Makefile.ps1` for common operations
- **Frontend**: npm scripts in package.json
- **Tests**: Dedicated PowerShell test runners

## Integration Points

### API Communication
- **Frontend ↔ Backend**: REST API on port 3000
- **Backend ↔ Database**: MongoDB on port 27017
- **Frontend ↔ Blockchain**: Direct Solana RPC calls
- **Backend ↔ Blockchain**: Solana web3.js integration

### Shared Standards
- **API Contracts**: OpenAPI specifications in docs/
- **Error Handling**: Consistent error formats across tiers
- **Authentication**: JWT tokens with HTTP-only cookies
- **Logging**: Structured JSON logs with correlation IDs

## Development Workflow

### Local Development Setup
1. **Start MongoDB**: Docker containers via backend scripts
2. **Start Backend**: `.\rust-backend\Makefile.ps1 dev`
3. **Start Frontend**: `npm run dev` in empty-project/
4. **Optional**: Start blockchain local validator

### Testing Organization
- **Unit Tests**: Co-located with source code
- **Integration Tests**: Dedicated test folders
- **E2E Tests**: Cross-component testing scripts
- **API Tests**: Comprehensive endpoint testing

### Quality Gates
- **Compilation**: All code must compile without warnings
- **Linting**: ESLint (frontend), Clippy (backend)
- **Formatting**: Prettier (frontend), rustfmt (backend)
- **Testing**: All tests must pass before merge