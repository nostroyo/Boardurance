# Development Workflow & Standards

Comprehensive development workflow following industry best practices and the operational doctrine from the steering rules.

## 🎯 Core Principles

### Autonomous Principal Engineer Approach
- **Complete Ownership**: Take full responsibility for code quality and system health
- **Understand Before Touch**: Never execute without complete understanding of current state
- **Zero-Assumption Discipline**: Verify everything against live system
- **Extreme Ownership**: Fix all related issues, update all consumers, leave system better

### Mandatory Workflow Phases

#### 1. Reconnaissance (Read-Only Phase)
- Repository inventory and dependency analysis
- Configuration corpus review
- Idiomatic pattern inference
- Quality gates identification
- Produce reconnaissance digest (≤200 lines)

#### 2. Planning & Context
- Read before write; reread after write
- Account for full system impact
- Plan updates for all consumers/dependencies

#### 3. Execution
- Wrap all shell commands with timeout
- Use non-interactive flags
- Fail-fast semantics
- Capture full output (stdout & stderr)

#### 4. Verification
- Run all quality gates
- Autonomously fix failures
- Reread altered artifacts
- End-to-end workflow verification

#### 5. Reporting
- Keep narratives in chat (no unsolicited files)
- Use clear status legend (✅ ⚠️ 🚧)

## 🏗️ Project Structure Standards

### Frontend (React) Organization
```
empty-project/
├── src/
│   ├── components/         # Reusable UI components
│   │   ├── common/        # Shared components
│   │   ├── game/          # Game-specific components
│   │   └── wallet/        # Web3 wallet components
│   ├── pages/             # Application pages/routes
│   ├── hooks/             # Custom React hooks
│   ├── utils/             # Utility functions
│   ├── types/             # TypeScript type definitions
│   ├── services/          # API and external services
│   └── assets/            # Static files
└── public/                # Public static files
```

### Backend (Rust) Organization
```
rust-backend/
├── src/
│   ├── domain/            # Business logic and entities
│   ├── routes/            # HTTP handlers and routing
│   ├── configuration.rs   # Config management
│   ├── startup.rs         # Application startup
│   ├── telemetry.rs       # Logging and observability
│   └── lib.rs             # Library root
├── configuration/         # Environment configs
├── docker/               # Docker setup files
├── scripts/              # Development scripts
└── tests/                # Integration tests
```

### Blockchain (Solana) Organization
```
solana-smart-contract/
├── programs/             # Smart contract source code
├── tests/                # Contract tests and simulations
├── assets/               # NFT metadata and images
├── docs/                 # Blockchain-specific documentation
└── scripts/              # Deployment and utility scripts
```

## 📝 File Naming Conventions

### React Project
- **Components**: PascalCase (e.g., `UserProfile.tsx`)
- **Hooks**: camelCase starting with "use" (e.g., `useAuth.ts`)
- **Utils**: camelCase (e.g., `formatDate.ts`)
- **Types**: PascalCase (e.g., `UserTypes.ts`)
- **Pages**: PascalCase (e.g., `GameDashboard.tsx`)

### Rust Project
- **Modules**: snake_case (e.g., `user_management.rs`)
- **Functions**: snake_case (e.g., `create_user`)
- **Types**: PascalCase (e.g., `UserProfile`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRY_ATTEMPTS`)

### Solana Project
- **Programs**: snake_case (e.g., `car_nft_program.rs`)
- **Instructions**: snake_case (e.g., `mint_car_nft.rs`)
- **Tests**: snake_case (e.g., `test_car_minting.ts`)

## 🔄 Git Workflow

### Branch Strategy
```
main                    # Production-ready code
├── develop            # Integration branch
├── feature/           # Feature development
│   ├── feature/wallet-integration
│   ├── feature/nft-display
│   └── feature/racing-mechanics
├── hotfix/            # Critical production fixes
└── release/           # Release preparation
```

### Commit Message Format
```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

**Types**: feat, fix, docs, style, refactor, test, chore
**Scopes**: frontend, backend, blockchain, docs, config

**Examples**:
```
feat(frontend): add wallet connection component
fix(backend): resolve database connection timeout
docs(blockchain): update deployment guide
test(frontend): add unit tests for game components
```

### Pull Request Process
1. **Create Feature Branch**: `git checkout -b feature/description`
2. **Implement Changes**: Follow coding standards
3. **Run Quality Gates**: Tests, linting, formatting
4. **Create PR**: Descriptive title and body
5. **Code Review**: Peer review required
6. **Merge**: Squash and merge to develop

## 🧪 Testing Strategy

### Frontend Testing
```bash
# Unit tests with Vitest
npm run test

# Component tests with Testing Library
npm run test:components

# E2E tests with Playwright
npm run test:e2e

# Coverage report
npm run test:coverage
```

### Backend Testing
```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# Test with coverage
cargo tarpaulin --out html

# Performance tests
cargo bench
```

### Blockchain Testing
```bash
# Smart contract tests
anchor test

# Local validator tests
anchor test --skip-local-validator

# Deployment simulation
anchor test --provider.cluster devnet
```

## 🔍 Quality Gates

### Automated Checks
- **Compilation**: All code must compile without warnings
- **Tests**: All tests must pass (unit, integration, e2e)
- **Linting**: ESLint (frontend), Clippy (backend)
- **Formatting**: Prettier (frontend), rustfmt (backend)
- **Type Checking**: TypeScript strict mode
- **Security**: Dependency vulnerability scanning

### Manual Reviews
- **Code Review**: Peer review for all changes
- **Architecture Review**: For significant changes
- **Security Review**: For security-sensitive code
- **Performance Review**: For performance-critical paths

## 🚀 Development Commands

### Frontend Development
```bash
cd empty-project

# Development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Code quality
npm run lint
npm run format
npm run type-check
```

### Backend Development
```bash
cd rust-backend

# Development with MongoDB
.\Makefile.ps1 dev

# Development with UI
.\Makefile.ps1 dev-ui

# Testing
.\Makefile.ps1 test

# Code quality
cargo check
cargo clippy
cargo fmt
```

### Blockchain Development
```bash
cd solana-smart-contract

# Build contracts
anchor build

# Run tests
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Generate metadata
node generate-metadata.js
```

## 📊 Performance Standards

### Frontend Performance
- **First Contentful Paint**: <1.5s
- **Largest Contentful Paint**: <2.5s
- **Cumulative Layout Shift**: <0.1
- **First Input Delay**: <100ms

### Backend Performance
- **API Response Time**: <100ms (95th percentile)
- **Database Query Time**: <50ms (average)
- **Memory Usage**: <512MB per instance
- **CPU Usage**: <70% under normal load

### Blockchain Performance
- **Transaction Confirmation**: <30s on devnet
- **Smart Contract Execution**: <400ms (Solana block time)
- **NFT Minting**: <60s end-to-end
- **Metadata Upload**: <5 minutes for 100 items

## 🔒 Security Standards

### Code Security
- **Input Validation**: All user inputs validated
- **Output Encoding**: Prevent XSS attacks
- **Authentication**: Secure token handling
- **Authorization**: Proper access controls

### Dependency Security
- **Vulnerability Scanning**: Regular dependency audits
- **Version Pinning**: Lock dependency versions
- **License Compliance**: Check license compatibility
- **Supply Chain**: Verify package integrity

### Infrastructure Security
- **Environment Variables**: Secure configuration
- **Network Security**: Proper firewall rules
- **Access Control**: Principle of least privilege
- **Audit Logging**: Comprehensive audit trails

## 📈 Monitoring & Observability

### Application Monitoring
- **Structured Logging**: JSON format with correlation IDs
- **Metrics Collection**: Performance and business metrics
- **Error Tracking**: Comprehensive error reporting
- **Health Checks**: Service availability monitoring

### Development Metrics
- **Build Times**: Track build performance
- **Test Coverage**: Maintain >80% coverage
- **Code Quality**: Track technical debt
- **Deployment Frequency**: Measure delivery velocity

## 🤝 Collaboration Standards

### Communication
- **Daily Standups**: Progress and blockers
- **Code Reviews**: Constructive feedback
- **Documentation**: Keep docs up-to-date
- **Knowledge Sharing**: Regular tech talks

### Tools
- **Version Control**: Git with proper branching
- **Issue Tracking**: GitHub Issues or Jira
- **Documentation**: Markdown in repository
- **Communication**: Slack or Discord

## 🎯 Definition of Done

### Feature Completion Checklist
- [ ] **Functionality**: Feature works as specified
- [ ] **Tests**: Unit and integration tests written
- [ ] **Documentation**: Updated relevant documentation
- [ ] **Code Review**: Peer review completed
- [ ] **Quality Gates**: All automated checks pass
- [ ] **Performance**: Meets performance requirements
- [ ] **Security**: Security review completed
- [ ] **Deployment**: Successfully deployed to staging

### Release Readiness Checklist
- [ ] **All Features**: Complete and tested
- [ ] **Performance**: Load testing completed
- [ ] **Security**: Security audit completed
- [ ] **Documentation**: User and developer docs updated
- [ ] **Monitoring**: Observability in place
- [ ] **Rollback Plan**: Rollback procedure documented
- [ ] **Stakeholder Approval**: Business sign-off received

---

**Following this workflow ensures high-quality, maintainable, and secure code across all project components! 🚀**