# PowerShell script to fix tracing format issues

$filePath = "src/routes/players.rs"
$content = Get-Content $filePath -Raw

# Fix wallet_address = %wallet_address patterns
$content = $content -replace 'wallet_address = %wallet_address,', 'wallet_address,'
$content = $content -replace 'wallet_address = %wallet_address', 'wallet_address'

# Fix email = %email patterns  
$content = $content -replace 'email = %email,', 'email,'
$content = $content -replace 'email = %email', 'email'

# Fix the tracing format strings to include placeholders
$content = $content -replace '\[FETCHING PLAYER BY WALLET - END\]",\s*wallet_address,', '[FETCHING PLAYER BY WALLET - END] wallet_address={}",\n                wallet_address,'
$content = $content -replace '\[FETCHING PLAYER BY WALLET - EVENT\] Player not found",\s*wallet_address', '[FETCHING PLAYER BY WALLET - EVENT] Player not found wallet_address={}",\n                wallet_address'
$content = $content -replace '\[FETCHING PLAYER BY WALLET - EVENT\] Failed to fetch player: \{\}",\s*e,\s*wallet_address', '[FETCHING PLAYER BY WALLET - EVENT] Failed to fetch player: {} wallet_address={}",\n                e,\n                wallet_address'

$content = $content -replace '\[FETCHING PLAYER BY EMAIL - END\]",\s*email,', '[FETCHING PLAYER BY EMAIL - END] email={}",\n                email,'
$content = $content -replace '\[FETCHING PLAYER BY EMAIL - EVENT\] Player not found",\s*email', '[FETCHING PLAYER BY EMAIL - EVENT] Player not found email={}",\n                email'
$content = $content -replace '\[FETCHING PLAYER BY EMAIL - EVENT\] Failed to fetch player: \{\}",\s*e,\s*email', '[FETCHING PLAYER BY EMAIL - EVENT] Failed to fetch player: {} email={}",\n                e,\n                email'

# Write the fixed content back
$content | Set-Content $filePath -NoNewline

Write-Host "Fixed tracing format issues in $filePath" -ForegroundColor Green