# Project Structure

## Workspace Organization

This is a multi-project workspace with two distinct applications:

```
├── empty-project/           # React frontend application
└── solana-smart-contract/   # Solana blockchain smart contract
```

## Empty Project Structure

```
empty-project/
├── src/
│   ├── assets/             # Static assets (images, icons, etc.)
│   ├── App.tsx             # Main application component
│   ├── App.css             # Application styles
│   ├── main.tsx            # Application entry point
│   ├── index.css           # Global styles
│   └── vite-env.d.ts       # Vite type definitions
├── public/                 # Public static files
├── index.html              # HTML template
├── standalone-login.html   # Standalone login page
├── package.json            # Dependencies and scripts
├── vite.config.ts          # Vite configuration
├── tailwind.config.js      # Tailwind CSS configuration
├── postcss.config.js       # PostCSS configuration
├── eslint.config.js        # ESLint configuration
├── .prettierrc.json        # Prettier configuration
├── tsconfig.json           # TypeScript configuration
├── tsconfig.app.json       # App-specific TypeScript config
└── tsconfig.node.json      # Node-specific TypeScript config
```

## Solana Smart Contract Structure

```
solana-smart-contract/
├── programs/
│   └── solana-smart-contract/
│       ├── src/            # Rust smart contract source code
│       └── Cargo.toml      # Rust package manifest
├── tests/                  # Integration tests
├── Anchor.toml             # Anchor framework configuration
├── Cargo.toml              # Workspace Cargo manifest
└── README.md               # Project documentation
```

## Code Organization Guidelines

### React Project
- **Components**: Place reusable UI components in `src/components/`
- **Pages**: Application pages/routes in `src/pages/`
- **Hooks**: Custom React hooks in `src/hooks/`
- **Utils**: Utility functions in `src/utils/`
- **Types**: TypeScript type definitions in `src/types/`
- **Assets**: Static files in `src/assets/` or `public/`

### Solana Contract
- **Programs**: Smart contract logic in `programs/[contract-name]/src/`
- **Tests**: Integration tests in `tests/`
- **Instructions**: Contract instructions as separate modules
- **State**: Account structures and program state definitions

## File Naming Conventions

### React Project
- **Components**: PascalCase (e.g., `UserProfile.tsx`)
- **Hooks**: camelCase starting with "use" (e.g., `useAuth.ts`)
- **Utils**: camelCase (e.g., `formatDate.ts`)
- **Types**: PascalCase (e.g., `UserTypes.ts`)

### Solana Contract
- **Modules**: snake_case (e.g., `user_management.rs`)
- **Instructions**: snake_case (e.g., `create_user.rs`)
- **Tests**: snake_case (e.g., `test_user_creation.ts`)