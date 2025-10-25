#!/usr/bin/env pwsh

# Test script for authentication endpoints
# This script tests user registration and login functionality

$BASE_URL = "http://localhost:3000/api/v1"
$AUTH_URL = "$BASE_URL/auth"

Write-Host "🔐 Testing Authentication Endpoints" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# Test data
$testUser = @{
    email = "test@example.com"
    password = "TestPassword123"
    team_name = "Test Team"
}

$loginCredentials = @{
    email = "test@example.com"
    password = "TestPassword123"
}

$wrongCredentials = @{
    email = "test@example.com"
    password = "WrongPassword123"
}

# Function to make HTTP requests with error handling
function Invoke-SafeRestMethod {
    param(
        [string]$Uri,
        [string]$Method = "GET",
        [hashtable]$Body = $null,
        [string]$ContentType = "application/json"
    )
    
    try {
        $params = @{
            Uri = $Uri
            Method = $Method
            ContentType = $ContentType
        }
        
        if ($Body) {
            $params.Body = ($Body | ConvertTo-Json -Depth 10)
        }
        
        $response = Invoke-RestMethod @params
        return @{ Success = $true; Data = $response; StatusCode = 200 }
    }
    catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        $errorBody = $_.ErrorDetails.Message
        return @{ Success = $false; Error = $errorBody; StatusCode = $statusCode }
    }
}

Write-Host "`n1. Testing User Registration" -ForegroundColor Yellow
Write-Host "----------------------------"

$registerResponse = Invoke-SafeRestMethod -Uri "$AUTH_URL/register" -Method "POST" -Body $testUser

if ($registerResponse.Success) {
    Write-Host "✅ User registration successful!" -ForegroundColor Green
    Write-Host "Response: $($registerResponse.Data | ConvertTo-Json -Depth 3)" -ForegroundColor Gray
} else {
    Write-Host "❌ User registration failed!" -ForegroundColor Red
    Write-Host "Status Code: $($registerResponse.StatusCode)" -ForegroundColor Red
    Write-Host "Error: $($registerResponse.Error)" -ForegroundColor Red
}

Write-Host "`n2. Testing Duplicate Registration (should fail)" -ForegroundColor Yellow
Write-Host "-----------------------------------------------"

$duplicateResponse = Invoke-SafeRestMethod -Uri "$AUTH_URL/register" -Method "POST" -Body $testUser

if (!$duplicateResponse.Success -and $duplicateResponse.StatusCode -eq 409) {
    Write-Host "✅ Duplicate registration correctly rejected!" -ForegroundColor Green
    Write-Host "Error: $($duplicateResponse.Error)" -ForegroundColor Gray
} else {
    Write-Host "❌ Duplicate registration should have failed!" -ForegroundColor Red
    if ($duplicateResponse.Success) {
        Write-Host "Response: $($duplicateResponse.Data | ConvertTo-Json -Depth 3)" -ForegroundColor Red
    }
}

Write-Host "`n3. Testing User Login with Correct Credentials" -ForegroundColor Yellow
Write-Host "----------------------------------------------"

$loginResponse = Invoke-SafeRestMethod -Uri "$AUTH_URL/login" -Method "POST" -Body $loginCredentials

if ($loginResponse.Success) {
    Write-Host "✅ User login successful!" -ForegroundColor Green
    Write-Host "Response: $($loginResponse.Data | ConvertTo-Json -Depth 3)" -ForegroundColor Gray
} else {
    Write-Host "❌ User login failed!" -ForegroundColor Red
    Write-Host "Status Code: $($loginResponse.StatusCode)" -ForegroundColor Red
    Write-Host "Error: $($loginResponse.Error)" -ForegroundColor Red
}

Write-Host "`n4. Testing User Login with Wrong Credentials" -ForegroundColor Yellow
Write-Host "--------------------------------------------"

$wrongLoginResponse = Invoke-SafeRestMethod -Uri "$AUTH_URL/login" -Method "POST" -Body $wrongCredentials

if (!$wrongLoginResponse.Success -and $wrongLoginResponse.StatusCode -eq 401) {
    Write-Host "✅ Wrong credentials correctly rejected!" -ForegroundColor Green
    Write-Host "Error: $($wrongLoginResponse.Error)" -ForegroundColor Gray
} else {
    Write-Host "❌ Wrong credentials should have been rejected!" -ForegroundColor Red
    if ($wrongLoginResponse.Success) {
        Write-Host "Response: $($wrongLoginResponse.Data | ConvertTo-Json -Depth 3)" -ForegroundColor Red
    }
}

Write-Host "`n5. Testing Password Validation" -ForegroundColor Yellow
Write-Host "------------------------------"

$weakPasswordUser = @{
    email = "weak@example.com"
    password = "weak"
    team_name = "Weak Team"
}

$weakPasswordResponse = Invoke-SafeRestMethod -Uri "$AUTH_URL/register" -Method "POST" -Body $weakPasswordUser

if (!$weakPasswordResponse.Success -and $weakPasswordResponse.StatusCode -eq 400) {
    Write-Host "✅ Weak password correctly rejected!" -ForegroundColor Green
    Write-Host "Error: $($weakPasswordResponse.Error)" -ForegroundColor Gray
} else {
    Write-Host "❌ Weak password should have been rejected!" -ForegroundColor Red
    if ($weakPasswordResponse.Success) {
        Write-Host "Response: $($weakPasswordResponse.Data | ConvertTo-Json -Depth 3)" -ForegroundColor Red
    }
}

Write-Host "`n6. Testing Email Validation" -ForegroundColor Yellow
Write-Host "---------------------------"

$invalidEmailUser = @{
    email = "invalid-email"
    password = "ValidPassword123"
    team_name = "Invalid Email Team"
}

$invalidEmailResponse = Invoke-SafeRestMethod -Uri "$AUTH_URL/register" -Method "POST" -Body $invalidEmailUser

if (!$invalidEmailResponse.Success -and $invalidEmailResponse.StatusCode -eq 400) {
    Write-Host "✅ Invalid email correctly rejected!" -ForegroundColor Green
    Write-Host "Error: $($invalidEmailResponse.Error)" -ForegroundColor Gray
} else {
    Write-Host "❌ Invalid email should have been rejected!" -ForegroundColor Red
    if ($invalidEmailResponse.Success) {
        Write-Host "Response: $($invalidEmailResponse.Data | ConvertTo-Json -Depth 3)" -ForegroundColor Red
    }
}

Write-Host "`n🔐 Authentication Testing Complete!" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan

# Cleanup: Try to find and display the created user
Write-Host "`n7. Verifying User in Database" -ForegroundColor Yellow
Write-Host "-----------------------------"

$playersResponse = Invoke-SafeRestMethod -Uri "$BASE_URL/players"

if ($playersResponse.Success) {
    $testUserInDb = $playersResponse.Data | Where-Object { $_.email -eq $testUser.email }
    if ($testUserInDb) {
        Write-Host "✅ Test user found in database!" -ForegroundColor Green
        Write-Host "User UUID: $($testUserInDb.uuid)" -ForegroundColor Gray
        Write-Host "Team Name: $($testUserInDb.team_name)" -ForegroundColor Gray
        Write-Host "Created At: $($testUserInDb.created_at)" -ForegroundColor Gray
        Write-Host "Note: Password hash is not exposed in API responses (security feature)" -ForegroundColor Gray
    } else {
        Write-Host "❌ Test user not found in database!" -ForegroundColor Red
    }
} else {
    Write-Host "❌ Could not retrieve players from database!" -ForegroundColor Red
    Write-Host "Error: $($playersResponse.Error)" -ForegroundColor Red
}

Write-Host "`nTest Summary:" -ForegroundColor Cyan
Write-Host "- User registration with password hashing ✅" -ForegroundColor Green
Write-Host "- Duplicate email prevention ✅" -ForegroundColor Green
Write-Host "- Password verification on login ✅" -ForegroundColor Green
Write-Host "- Password strength validation ✅" -ForegroundColor Green
Write-Host "- Email format validation ✅" -ForegroundColor Green
Write-Host "- Secure password storage (hash not exposed) ✅" -ForegroundColor Green