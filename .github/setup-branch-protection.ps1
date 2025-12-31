#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Sets up branch protection rules for the main branch using GitHub CLI
.DESCRIPTION
    This script configures branch protection rules to require CI checks to pass
    before allowing merges to the main branch. It requires GitHub CLI to be installed
    and authenticated.
.EXAMPLE
    .\.github\setup-branch-protection.ps1
#>

param(
    [string]$Branch = "main",
    [string]$ConfigFile = ".github/branch-protection-config.json"
)

# Check if GitHub CLI is installed
if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Error "GitHub CLI (gh) is not installed. Please install it from https://cli.github.com/"
    exit 1
}

# Check if user is authenticated
$authStatus = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "Not authenticated with GitHub CLI. Please run 'gh auth login' first."
    exit 1
}

# Check if config file exists
if (-not (Test-Path $ConfigFile)) {
    Write-Error "Configuration file not found: $ConfigFile"
    exit 1
}

Write-Host "Setting up branch protection for branch: $Branch" -ForegroundColor Green

try {
    # Apply branch protection rules using GitHub CLI
    Write-Host "Applying branch protection rules..." -ForegroundColor Yellow
    
    gh api repos/:owner/:repo/branches/$Branch/protection `
        --method PUT `
        --input $ConfigFile
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Branch protection rules applied successfully!" -ForegroundColor Green
        Write-Host ""
        Write-Host "The following protections are now active on the '$Branch' branch:" -ForegroundColor Cyan
        Write-Host "  • Require status checks to pass before merging" -ForegroundColor White
        Write-Host "    - frontend-ci workflow must pass" -ForegroundColor Gray
        Write-Host "    - backend-ci workflow must pass" -ForegroundColor Gray
        Write-Host "  • Require branches to be up to date before merging" -ForegroundColor White
        Write-Host "  • Require pull request reviews (1 approval required)" -ForegroundColor White
        Write-Host "  • Dismiss stale reviews when new commits are pushed" -ForegroundColor White
        Write-Host "  • Require conversation resolution before merging" -ForegroundColor White
        Write-Host "  • Restrict pushes that create new files" -ForegroundColor White
        Write-Host "  • Prevent force pushes and deletions" -ForegroundColor White
    } else {
        Write-Error "Failed to apply branch protection rules"
        exit 1
    }
} catch {
    Write-Error "Error applying branch protection: $($_.Exception.Message)"
    exit 1
}

Write-Host ""
Write-Host "You can verify the settings at:" -ForegroundColor Cyan
Write-Host "https://github.com/$(gh repo view --json owner,name -q '.owner.login + "/" + .name")/settings/branches" -ForegroundColor Blue