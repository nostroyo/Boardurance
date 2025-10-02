# Technology Stack

## Empty Project (React Frontend)

### Core Technologies
- **React 19.1.1** - UI library with latest features
- **TypeScript 5.8.3** - Type-safe JavaScript
- **Vite 7.1.2** - Fast build tool and dev server
- **Tailwind CSS 3.4.17** - Utility-first CSS framework

### Development Tools
- **ESLint 9.33.0** - Code linting with TypeScript support
- **Prettier 3.6.2** - Code formatting
- **PostCSS 8.5.6** - CSS processing

### Common Commands
```bash
# Development
npm run dev          # Start development server
npm run build        # Build for production
npm run preview      # Preview production build

# Code Quality
npm run lint         # Run ESLint
npm run format       # Format code with Prettier
npm run format:check # Check formatting without changes
```

## Solana Smart Contract

### Core Technologies
- **Anchor Framework** - Solana development framework
- **Rust** - Smart contract programming language
- **Solana CLI** - Blockchain interaction tools

### Development Tools
- **Cargo** - Rust package manager
- **TypeScript/Mocha** - Testing framework

### Common Commands
```bash
# Setup
anchor build         # Compile smart contracts
anchor deploy        # Deploy to configured network
anchor test          # Run integration tests

# Solana CLI
solana config get    # Check current configuration
solana balance       # Check wallet balance
```

### Configuration
- **Localnet** cluster by default
- Wallet: `~/.config/solana/id.json`
- Program ID: `Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS`