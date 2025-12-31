#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Verifies that branch protection rules are properly configured
.DESCRIPTION
    This script checks the current branch protection settings and validates
    that required CI checks are configured correctly.
.EXAMPLE
    .\.github\verify-branch-protection.ps1
#>

param(
    [string]$Branch = "main"
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

Write-Host "Verifying branch protection for branch: $Branch" -ForegroundColor Green
Write-Host ""

try {
    # Get branch protection settings
    $protection = gh api repos/:owner/:repo/branches/$Branch/protection | ConvertFrom-Json
    
    Write-Host "✅ Branch protection is enabled" -ForegroundColor Green
    
    # Check required status checks
    if ($protection.required_status_checks) {
        Write-Host "✅ Required status checks are enabled" -ForegroundColor Green
        Write-Host "   Strict mode: $($protection.required_status_checks.strict)" -ForegroundColor Gray
        
        $requiredChecks = $protection.required_status_checks.contexts
        Write-Host "   Required checks:" -ForegroundColor Gray
        
        $expectedChecks = @("frontend-ci", "backend-ci")
        $missingChecks = @()
        
        foreach ($check in $expectedChecks) {
            if ($requiredChecks -contains $check) {
                Write-Host "     ✅ $check" -ForegroundColor Green
            } else {
                Write-Host "     ❌ $check (missing)" -ForegroundColor Red
                $missingChecks += $check
            }
        }
        
        if ($missingChecks.Count -gt 0) {
            Write-Warning "Missing required status checks: $($missingChecks -join ', ')"
        }
    } else {
        Write-Host "❌ Required status checks are not enabled" -ForegroundColor Red
    }
    
    # Check pull request reviews
    if ($protection.required_pull_request_reviews) {
        Write-Host "✅ Pull request reviews are required" -ForegroundColor Green
        Write-Host "   Required approvals: $($protection.required_pull_request_reviews.required_approving_review_count)" -ForegroundColor Gray
        Write-Host "   Dismiss stale reviews: $($protection.required_pull_request_reviews.dismiss_stale_reviews)" -ForegroundColor Gray
    } else {
        Write-Host "❌ Pull request reviews are not required" -ForegroundColor Red
    }
    
    # Check other settings
    Write-Host "Force pushes allowed: $($protection.allow_force_pushes.enabled)" -ForegroundColor $(if ($protection.allow_force_pushes.enabled) { "Red" } else { "Green" })
    Write-Host "Deletions allowed: $($protection.allow_deletions.enabled)" -ForegroundColor $(if ($protection.allow_deletions.enabled) { "Red" } else { "Green" })
    
    if ($protection.required_conversation_resolution) {
        Write-Host "✅ Conversation resolution required" -ForegroundColor Green
    } else {
        Write-Host "❌ Conversation resolution not required" -ForegroundColor Red
    }
    
} catch {
    if ($_.Exception.Message -like "*Not Found*") {
        Write-Host "❌ Branch protection is not configured for branch '$Branch'" -ForegroundColor Red
        Write-Host ""
        Write-Host "To set up branch protection, run:" -ForegroundColor Yellow
        Write-Host "  .\.github\setup-branch-protection.ps1" -ForegroundColor Cyan
    } else {
        Write-Error "Error checking branch protection: $($_.Exception.Message)"
    }
    exit 1
}

Write-Host ""
Write-Host "Branch protection verification complete!" -ForegroundColor Green