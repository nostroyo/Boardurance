# Frontend - React + TypeScript + Vite

Web3 Game frontend application built with modern React development practices.

## 🚀 Technology Stack

- **React 19.1.1** - UI library with latest features
- **TypeScript 5.8.3** - Type-safe JavaScript
- **Vite 7.1.2** - Fast build tool and dev server
- **Tailwind CSS 3.4.17** - Utility-first CSS framework

## 🛠️ Development Tools

- **ESLint 9.33.0** - Code linting with TypeScript support
- **Prettier 3.6.2** - Code formatting
- **PostCSS 8.5.6** - CSS processing

## 📁 Project Structure

```
empty-project/
├── src/
│   ├── assets/             # Static assets (images, icons, etc.)
│   ├── components/         # Reusable UI components
│   ├── App.tsx             # Main application component
│   ├── App.css             # Application styles
│   ├── main.tsx            # Application entry point
│   ├── index.css           # Global styles
│   └── vite-env.d.ts       # Vite type definitions
├── public/                 # Public static files
├── index.html              # HTML template
├── standalone-login.html   # Standalone login page
└── configuration files
```

## 🚀 Quick Start

### Prerequisites
- Node.js (latest LTS)
- npm or yarn

### Setup
```bash
cd empty-project
npm install
npm run dev
```

## 📜 Available Scripts

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

## 🎮 Web3 Game Features

This frontend is designed for Web3 game integration:

- **Wallet Integration** - Connect to Solana wallets
- **NFT Display** - Show game assets and collectibles
- **Game Interface** - Interactive game components
- **Responsive Design** - Works on desktop and mobile

## 🔧 Configuration

### ESLint Configuration

For production applications, update ESLint configuration:

```js
export default tseslint.config([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      ...tseslint.configs.recommendedTypeChecked,
      // For stricter rules:
      ...tseslint.configs.strictTypeChecked,
      // For stylistic rules:
      ...tseslint.configs.stylisticTypeChecked,
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
]);
```

### React-Specific Linting

Install additional React plugins:

```bash
npm install eslint-plugin-react-x eslint-plugin-react-dom
```

```js
import reactX from 'eslint-plugin-react-x';
import reactDom from 'eslint-plugin-react-dom';

export default tseslint.config([
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      reactX.configs['recommended-typescript'],
      reactDom.configs.recommended,
    ],
  },
]);
```

## 🎨 Styling

- **Tailwind CSS** for utility-first styling
- **CSS Modules** support for component-scoped styles
- **PostCSS** for advanced CSS processing

## 🔗 Integration

This frontend integrates with:
- **Rust Backend** - API calls for game data
- **Solana Blockchain** - Wallet and NFT interactions
- **Web3 Libraries** - Solana wallet adapters

## 📱 Responsive Design

Built with mobile-first approach:
- Responsive grid layouts
- Touch-friendly interactions
- Optimized for various screen sizes

## 🚀 Deployment

### Build for Production
```bash
npm run build
```

### Preview Production Build
```bash
npm run preview
```

The built files will be in the `dist/` directory, ready for deployment to any static hosting service.

## 🧪 Testing

Testing setup recommendations:
- **Vitest** for unit testing
- **Testing Library** for component testing
- **Playwright** for e2e testing

## 📚 Learning Resources

- [React Documentation](https://react.dev/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [Vite Guide](https://vitejs.dev/guide/)
- [Tailwind CSS](https://tailwindcss.com/docs)