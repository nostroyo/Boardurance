# Branch Protection Setup Guide

This guide explains how to configure branch protection rules for the main branch to ensure CI checks pass before merging.

## Overview

Branch protection rules enforce the following requirements:
- ✅ **Frontend CI workflow** must pass before merge
- ✅ **Backend CI workflow** must pass before merge  
- ✅ **Pull request reviews** required (1 approval minimum)
- ✅ **Up-to-date branches** required before merge
- ✅ **Conversation resolution** required before merge
- ❌ **Force pushes** and **branch deletions** blocked

## Method 1: Automated Setup (Recommended)

### Prerequisites
- [GitHub CLI](https://cli.github.com/) installed
- Authenticated with GitHub (`gh auth login`)
- Repository admin permissions

### Steps
1. Navigate to the repository root
2. Run the setup script:
   ```powershell
   .\.github\setup-branch-protection.ps1
   ```

The script will automatically apply all protection rules using the configuration in `.github/branch-protection-config.json`.

## Method 2: Manual Setup via GitHub Web Interface

### Steps
1. Go to your repository on GitHub
2. Navigate to **Settings** → **Branches**
3. Click **Add rule** next to "Branch protection rules"
4. Configure the following settings:

#### Branch Name Pattern
- Enter: `main`

#### Protect Matching Branches
- ✅ **Require a pull request before merging**
  - ✅ Require approvals: `1`
  - ✅ Dismiss stale pull request approvals when new commits are pushed
  - ✅ Require review from code owners (optional)

- ✅ **Require status checks to pass before merging**
  - ✅ Require branches to be up to date before merging
  - Add required status checks:
    - `frontend-ci`
    - `backend-ci`

- ✅ **Require conversation resolution before merging**

- ✅ **Restrict pushes that create matching branches**

- ❌ **Allow force pushes** (leave unchecked)
- ❌ **Allow deletions** (leave unchecked)

5. Click **Create** to save the rules

## Verification

After setup, verify the protection rules are working:

1. **Check Protection Status**:
   ```powershell
   gh api repos/:owner/:repo/branches/main/protection
   ```

2. **Test with a Pull Request**:
   - Create a test branch with a small change
   - Open a pull request to main
   - Verify that CI checks are required and must pass
   - Verify that approval is required before merge

## Required Status Checks

The following CI workflows must pass before merging:

### Frontend CI (`frontend-ci`)
- TypeScript compilation check
- ESLint linting
- Prettier formatting check
- Unit tests execution
- Production build verification
- Security audit (advisory only)

### Backend CI (`backend-ci`)
- Rust formatting check (`cargo fmt`)
- Clippy linting (`cargo clippy`)
- Compilation check (`cargo check`)
- Unit tests (`cargo test --lib --bins`)
- Integration tests (`cargo test --test '*'`)
- Release build (`cargo build --release`)
- Security audit (advisory only)

## Troubleshooting

### Status Checks Not Appearing
If the required status checks don't appear in the dropdown:
1. Ensure the CI workflows have run at least once on the main branch
2. Check that the workflow names match exactly: `frontend-ci` and `backend-ci`
3. Verify the workflows are enabled in the Actions tab

### Permission Issues
If you can't modify branch protection rules:
1. Ensure you have admin permissions on the repository
2. For organization repositories, check if branch protection restrictions apply

### CI Workflows Failing
If CI checks are consistently failing:
1. Check the Actions tab for detailed error logs
2. Ensure all dependencies are properly cached
3. Verify environment variables and secrets are configured correctly

## Configuration Files

- **`.github/branch-protection-config.json`**: JSON configuration for GitHub API
- **`.github/setup-branch-protection.ps1`**: PowerShell script for automated setup
- **`.github/workflows/frontend-ci.yml`**: Frontend CI workflow
- **`.github/workflows/backend-ci.yml`**: Backend CI workflow

## Security Considerations

These protection rules help ensure:
- **Code Quality**: All code is linted, formatted, and tested before merge
- **Review Process**: Changes are reviewed by team members
- **Build Integrity**: Code compiles and builds successfully
- **Test Coverage**: Unit and integration tests pass
- **Security**: Dependencies are audited for vulnerabilities

## Next Steps

After setting up branch protection:
1. Inform team members about the new requirements
2. Update development workflow documentation
3. Consider adding additional quality gates (code coverage, performance tests)
4. Monitor CI performance and optimize as needed