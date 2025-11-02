# PowerShell script to create an admin user in MongoDB
# This script creates an admin user for testing the admin interface

param(
    [string]$Email = "admin@racing.game",
    [string]$Password = "AdminPass123",
    [string]$TeamName = "Admin Team"
)

Write-Host "Creating admin user..." -ForegroundColor Green
Write-Host "Email: $Email" -ForegroundColor Yellow
Write-Host "Password: $Password" -ForegroundColor Yellow
Write-Host "Team Name: $TeamName" -ForegroundColor Yellow

# Generate UUID for the admin user
$AdminUuid = [System.Guid]::NewGuid().ToString()
Write-Host "Generated UUID: $AdminUuid" -ForegroundColor Cyan

# Get current timestamp in ISO format
$CurrentTime = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")

# Hash the password using a simple approach (in production, this should use Argon2)
# For now, we'll use a placeholder hash that the Rust backend can verify
$PasswordHash = '$argon2id$v=19$m=65536,t=3,p=4$placeholder$placeholder'

# Create the MongoDB document
$AdminDocument = @"
{
  "uuid": "$AdminUuid",
  "email": "$Email",
  "password_hash": "$PasswordHash",
  "team_name": "$TeamName",
  "role": "Admin",
  "cars": [],
  "pilots": [],
  "engines": [],
  "bodies": [],
  "created_at": { "`$date": "$CurrentTime" },
  "updated_at": { "`$date": "$CurrentTime" }
}
"@

# Write the document to a temporary file
$TempFile = "temp_admin_user.json"
$AdminDocument | Out-File -FilePath $TempFile -Encoding UTF8

Write-Host "MongoDB document created:" -ForegroundColor Green
Write-Host $AdminDocument -ForegroundColor Gray

# MongoDB connection details
$MongoHost = "localhost"
$MongoPort = "27017"
$DatabaseName = "racing_game"
$CollectionName = "players"

Write-Host "`nInserting admin user into MongoDB..." -ForegroundColor Green

try {
    # Use mongoimport to insert the document
    $ImportCommand = "mongoimport --host $MongoHost --port $MongoPort --db $DatabaseName --collection $CollectionName --file $TempFile --jsonArray"
    
    # Wrap the JSON in an array for mongoimport
    "[$AdminDocument]" | Out-File -FilePath $TempFile -Encoding UTF8
    
    # Execute the import
    Invoke-Expression $ImportCommand
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Admin user created successfully!" -ForegroundColor Green
        Write-Host "`nAdmin Login Credentials:" -ForegroundColor Cyan
        Write-Host "Email: $Email" -ForegroundColor White
        Write-Host "Password: $Password" -ForegroundColor White
        Write-Host "Role: Admin" -ForegroundColor White
        Write-Host "`nYou can now login to the admin interface at: http://localhost:5173/admin" -ForegroundColor Yellow
    } else {
        Write-Host "❌ Failed to create admin user" -ForegroundColor Red
    }
} catch {
    Write-Host "❌ Error creating admin user: $_" -ForegroundColor Red
} finally {
    # Clean up temporary file
    if (Test-Path $TempFile) {
        Remove-Item $TempFile
    }
}

Write-Host "`nNote: This script uses a placeholder password hash." -ForegroundColor Yellow
Write-Host "For production use, implement proper Argon2 hashing." -ForegroundColor Yellow