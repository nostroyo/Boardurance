# Environment Configuration Setup

This document explains how to set up environment variables and configuration files for the Web3 Racing Game project.

## Overview

The project uses environment variables and configuration files to manage sensitive information like database credentials, API keys, and service URLs. **Never commit actual secrets to version control.**

## Quick Setup

### 1. Backend Configuration (rust-backend/)

```bash
cd rust-backend
cp .env.example .env
# Edit .env with your actual values
```

Required environment variables:
- `APP_DATABASE__PASSWORD`: Your MongoDB password
- `JWT_SECRET`: Secure random string (minimum 32 characters)
- `CORS_ALLOWED_ORIGINS`: Frontend URLs for CORS

### 2. Frontend Configuration (empty-project/)

```bash
cd empty-project
cp .env.example .env
# Edit .env with your actual values
```

Required environment variables:
- `VITE_API_BASE_URL`: Backend API URL (default: http://localhost:3000)

### 3. Solana Smart Contract Configuration (solana-smart-contract/)

```bash
cd solana-smart-contract
cp .env.example .env
# Edit .env with your actual values
```

Required environment variables:
- `SOLANA_WALLET_PATH`: Path to your Solana wallet keypair
- `CANDY_MACHINE_ID`: Your deployed Candy Machine ID

## Security Best Practices

### ✅ DO
- Copy `.env.example` files to `.env` and customize
- Use strong, unique passwords and secrets
- Keep `.env` files local and never commit them
- Use different secrets for development, staging, and production
- Regularly rotate secrets and API keys

### ❌ DON'T
- Commit `.env` files to version control
- Share secrets in chat, email, or documentation
- Use default or weak passwords
- Hardcode secrets in source code
- Reuse secrets across environments

## Environment Variables Reference

### Backend (rust-backend/)

| Variable | Description | Example |
|----------|-------------|---------|
| `APP_ENVIRONMENT` | Environment name | `local`, `staging`, `production` |
| `APP_APPLICATION__PORT` | Server port | `3000` |
| `APP_DATABASE__HOST` | MongoDB host | `localhost` |
| `APP_DATABASE__PASSWORD` | MongoDB password | `secure_password_123` |
| `JWT_SECRET` | JWT signing key | `your_32_char_secret_key_here` |
| `CORS_ALLOWED_ORIGINS` | Allowed CORS origins | `http://localhost:5173` |

### Frontend (empty-project/)

| Variable | Description | Example |
|----------|-------------|---------|
| `VITE_API_BASE_URL` | Backend API URL | `http://localhost:3000` |
| `VITE_SOLANA_NETWORK` | Solana network | `devnet`, `mainnet-beta` |
| `VITE_DEBUG_LOGGING` | Enable debug logs | `true`, `false` |

### Solana Smart Contract (solana-smart-contract/)

| Variable | Description | Example |
|----------|-------------|---------|
| `SOLANA_NETWORK` | Target network | `devnet`, `mainnet-beta` |
| `SOLANA_WALLET_PATH` | Wallet keypair path | `~/.config/solana/id.json` |
| `CANDY_MACHINE_ID` | Candy Machine address | `ABC123...` |

## Configuration Files

### Backend YAML Configuration

The backend uses layered YAML configuration:
- `configuration/base.yaml` - Base settings (committed)
- `configuration/local.yaml` - Local overrides (committed)
- `configuration/production.yaml` - Production settings (NOT committed)

Environment variables override YAML settings using the `APP_` prefix.

### Git Ignore Protection

The following files are automatically ignored by Git:
- `.env*` files
- `configuration/production.yaml`
- `config.json` (Solana)
- `*.key`, `*.pem` files
- `secrets/` directories

## Generating Secure Secrets

### JWT Secret (32+ characters)
```bash
# Using OpenSSL
openssl rand -base64 32

# Using Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('base64'))"
```

### MongoDB Password
```bash
# Generate random password
openssl rand -base64 16
```

## Troubleshooting

### Common Issues

1. **Backend won't start**: Check MongoDB connection and credentials
2. **Frontend API calls fail**: Verify `VITE_API_BASE_URL` matches backend
3. **CORS errors**: Add frontend URL to `CORS_ALLOWED_ORIGINS`
4. **JWT errors**: Ensure `JWT_SECRET` is set and consistent

### Validation Commands

```bash
# Test backend configuration
cd rust-backend
cargo check

# Test frontend environment
cd empty-project
npm run build

# Test Solana configuration
cd solana-smart-contract
anchor build
```

## Production Deployment

For production deployment:

1. Use strong, unique secrets for each environment
2. Set `APP_ENVIRONMENT=production`
3. Enable SSL/TLS (`APP_DATABASE__REQUIRE_SSL=true`)
4. Use secure database hosts and credentials
5. Configure proper CORS origins
6. Enable production logging levels

## Support

If you encounter issues with environment configuration:
1. Check this documentation
2. Verify all required variables are set
3. Test with example values first
4. Check application logs for specific errors