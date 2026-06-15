# GitHub CI/CD Configuration

This directory contains the GitHub Actions workflows and branch protection configuration for the motorsport racing game project.

## 🚀 Quick Start

### Set Up Branch Protection (One-time setup)
```powershell
# Automated setup (requires GitHub CLI)
.\.github\setup-branch-protection.ps1

# Verify configuration
.\.github\verify-branch-protection.ps1
```

### Manual Setup
See [BRANCH_PROTECTION_SETUP.md](./BRANCH_PROTECTION_SETUP.md) for detailed instructions.

## 📁 Files Overview

| File | Purpose |
|------|---------|
| `workflows/frontend-ci.yml` | Frontend CI pipeline (React/TypeScript) |
| `workflows/backend-ci.yml` | Backend CI pipeline (Rust/MongoDB) |
| `branch-protection-config.json` | Branch protection configuration |
| `setup-branch-protection.ps1` | Automated branch protection setup |
| `verify-branch-protection.ps1` | Verify branch protection settings |
| `BRANCH_PROTECTION_SETUP.md` | Detailed setup documentation |

## 🔒 Branch Protection Rules

The main branch is protected with the following rules:

- ✅ **CI Checks Required**: Both frontend and backend CI must pass
- ✅ **Pull Request Reviews**: 1 approval required
- ✅ **Up-to-date Branches**: Must be current with main before merge
- ✅ **Conversation Resolution**: All discussions must be resolved
- ❌ **Force Pushes**: Blocked to prevent history rewriting
- ❌ **Direct Pushes**: All changes must go through pull requests

## 🔄 CI Workflows

### Frontend CI (`frontend-ci`)
Runs on changes to `empty-project/` directory:
- TypeScript compilation
- ESLint linting
- Prettier formatting
- Unit tests (Vitest)
- Production build
- Security audit

### Backend CI (`backend-ci`)
Runs on changes to `rust-backend/` directory:
- Rust formatting (`cargo fmt`)
- Clippy linting
- Compilation check
- Unit & integration tests
- Release build
- Security audit

## 🛠️ Development Workflow

1. **Create Feature Branch**
   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make Changes & Commit**
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

3. **Push & Create PR**
   ```bash
   git push origin feature/my-feature
   # Create PR via GitHub web interface
   ```

4. **CI Checks Run Automatically**
   - Frontend CI (if frontend changes)
   - Backend CI (if backend changes)

5. **Review & Merge**
   - Get required approval
   - Ensure CI passes
   - Merge via GitHub interface

## 🚨 Troubleshooting

### CI Checks Failing?
1. Check the Actions tab for detailed logs
2. Run tests locally before pushing:
   ```powershell
   # Frontend
   cd empty-project
   npm run test -- --run
   npm run lint
   npm run build
   
   # Backend
   cd rust-backend
   cargo test
   cargo clippy
   cargo fmt --check
   ```

### Branch Protection Issues?
1. Ensure you have admin permissions
2. Verify CI workflows have run at least once
3. Check that status check names match exactly

### Need Help?
- Review [BRANCH_PROTECTION_SETUP.md](./BRANCH_PROTECTION_SETUP.md)
- Check GitHub Actions logs in the repository
- Verify GitHub CLI authentication: `gh auth status`

## 📊 Status Badges

Add these to your main README.md:

```markdown
[![Frontend CI](https://github.com/YOUR_USERNAME/YOUR_REPO/workflows/Frontend%20CI/badge.svg)](https://github.com/YOUR_USERNAME/YOUR_REPO/actions/workflows/frontend-ci.yml)
[![Backend CI](https://github.com/YOUR_USERNAME/YOUR_REPO/workflows/Backend%20CI/badge.svg)](https://github.com/YOUR_USERNAME/YOUR_REPO/actions/workflows/backend-ci.yml)
```