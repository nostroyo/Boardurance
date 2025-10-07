# Simple Solana Deployment Guide

Simplified deployment options for the Web3 Motorsport Car NFT Collection, including web-based and command-line approaches.

## 🌐 Option 1: Metaplex Studio Web Interface (Recommended for Beginners)

### Step 1: Access Metaplex Studio
1. Visit: https://studio.metaplex.com/
2. Click "Switch to Devnet" (top-right corner)
3. Click "Connect Wallet"

### Step 2: Create/Import Wallet
- **Option A**: Create new wallet in browser
- **Option B**: Import existing wallet with seed phrase

### Step 3: Get Test SOL
1. Copy your wallet address from Metaplex Studio
2. Visit: https://faucet.solana.com/
3. Paste address and request 2 SOL

### Step 4: Create Collection
1. In Metaplex Studio, click "Create Collection"
2. Upload collection metadata:
   - **Name**: "Web3 Motorsport Cars"
   - **Symbol**: "W3MC"
   - **Description**: "100 unique racing car NFTs with game attributes"
   - **Upload**: collection image from `assets/collection.png`

### Step 5: Upload Assets
1. Click "Upload Assets"
2. Select all files from `assets/` folder (JSON + PNG files)
3. Wait for upload to complete (may take several minutes)

### Step 6: Configure Candy Machine
- **Price**: 0.01 SOL
- **Supply**: 100
- **Royalties**: 2.5%
- **Creator**: Your wallet address

## 🛠️ Option 2: Command Line Installation (Advanced Users)

### Windows Installation Methods

#### Method 1: Direct Download
```powershell
$url = "https://github.com/solana-labs/solana/releases/download/v1.18.26/solana-install-init-x86_64-pc-windows-msvc.exe"
Invoke-WebRequest -Uri $url -OutFile "solana-install.exe"
.\solana-install.exe
```

#### Method 2: Using Scoop
```powershell
# Install Scoop first if not installed
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
irm get.scoop.sh | iex

# Install Solana
scoop install solana
```

#### Method 3: Using Chocolatey
```powershell
# Install Chocolatey first if not installed
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install Solana
choco install solana
```

### Install Sugar CLI
```bash
# Using npm (requires Node.js)
npm install -g @metaplex-foundation/sugar-cli

# Or download binary from:
# https://github.com/metaplex-foundation/sugar/releases
```

### Verify Installation
```bash
solana --version
sugar --version
```

### Command Line Deployment
```bash
# Navigate to project
cd solana-smart-contract

# Configure Solana
solana config set --url devnet
solana-keygen new --outfile ~/.config/solana/devnet-wallet.json
solana config set --keypair ~/.config/solana/devnet-wallet.json

# Get test SOL
solana airdrop 2

# Update addresses
node update-addresses.js

# Deploy with Sugar
sugar validate
sugar upload
sugar deploy
sugar verify
```

## 🎯 Method Comparison

### Web Interface (Option 1)
**Pros:**
- No installation required
- User-friendly interface
- Visual feedback
- Works on any device with browser

**Cons:**
- Less automation
- Manual file uploads
- Limited batch operations

### Command Line (Option 2)
**Pros:**
- Full automation capability
- Professional workflow
- Better for developers
- Scriptable and repeatable

**Cons:**
- Requires installation
- Command line knowledge needed
- Platform-specific setup

## 🎉 Expected Results

Both methods will produce:
- ✅ 100 unique car NFTs deployed on Solana devnet
- ✅ Collection visible on all NFT platforms
- ✅ Mintable at 0.01 SOL each
- ✅ Full marketplace compatibility
- ✅ Game-ready with performance attributes

## 📊 Collection Features

### Car Types (8 Categories)
1. Formula One - High-speed racing
2. Rally - Off-road performance
3. Electric GT - Sustainable racing
4. Muscle Car - Classic power
5. Supercar - Luxury performance
6. Touring Car - Professional racing
7. Prototype - Experimental tech
8. Vintage - Classic heritage

### Rarity Distribution
- **Common** (32%): Gray background
- **Uncommon** (24%): Green background
- **Rare** (14%): Blue background
- **Epic** (21%): Purple background
- **Legendary** (9%): Gold background

### Game Attributes
- **Speed** (0-100): Top speed capability
- **Acceleration** (0-100): 0-60 performance
- **Handling** (0-100): Cornering ability
- **Durability** (0-100): Damage resistance

## 🆘 Troubleshooting

### Common Issues

**Installation Problems:**
- Try web interface first (no installation needed)
- Use different installation method
- Run as Administrator on Windows

**Wallet Issues:**
- Ensure sufficient SOL balance (2+ SOL)
- Check network setting (devnet vs mainnet)
- Verify wallet address is correct

**Upload Failures:**
- Check internet connection
- Retry upload (Arweave can be slow)
- Ensure all files are present in assets/

### Getting Help

1. **Start with web interface** - easiest option
2. **Check detailed guides** in other documentation files
3. **Verify prerequisites** - Node.js, proper network settings
4. **Test with small batch** - try uploading 1-2 NFTs first

## 🚀 Next Steps

After successful deployment:

1. **Test minting** - mint 2-3 NFTs to verify functionality
2. **View in wallet** - check NFTs appear in Phantom/Solflare
3. **Marketplace listing** - list on Magic Eden (optional)
4. **Community sharing** - announce your collection
5. **Game integration** - connect to frontend application

## 📚 Additional Resources

- **Metaplex Studio**: https://studio.metaplex.com/
- **Solana Faucet**: https://faucet.solana.com/
- **Sugar CLI Docs**: https://docs.metaplex.com/tools/sugar
- **Solana Explorer**: https://explorer.solana.com/

---

**Choose the method that works best for you - both lead to the same awesome car NFT collection! 🏁🚗💨**