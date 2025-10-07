# Solana Smart Contract Deployment Guide

Interactive deployment guide for the Web3 Motorsport Car NFT Collection on Solana.

## 🎯 Deployment Overview

Deploy 100 unique racing car NFTs to Solana testnet/mainnet using Metaplex Candy Machine.

**Estimated Time**: ~15 minutes
**Requirements**: Solana CLI, Sugar CLI, ~2 SOL for deployment

## 🚀 Step-by-Step Deployment

### Step 1: Install Required Tools

#### Install Solana CLI

**Windows (PowerShell as Administrator):**
```powershell
Invoke-WebRequest -Uri "https://release.solana.com/v1.18.26/solana-install-init-x86_64-pc-windows-msvc.exe" -OutFile "$env:TEMP\solana-install.exe"
Start-Process -FilePath "$env:TEMP\solana-install.exe" -Wait
```

**macOS/Linux:**
```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"
```

#### Install Sugar CLI
```bash
npm install -g @metaplex-foundation/sugar-cli
```

#### Verify Installation
```bash
solana --version
sugar --version
```

### Step 2: Create and Configure Wallet

```bash
# Create new wallet (SAVE THE SEED PHRASE!)
solana-keygen new --outfile ~/.config/solana/devnet-wallet.json

# Configure for devnet
solana config set --url devnet
solana config set --keypair ~/.config/solana/devnet-wallet.json

# Get wallet address
solana address
```

**⚠️ CRITICAL**: Save your seed phrase securely!

### Step 3: Fund Wallet

```bash
# Request test SOL from faucet
solana airdrop 2

# Check balance
solana balance
```

**Alternative**: Use web faucet at https://faucet.solana.com/

### Step 4: Update Configuration

```bash
cd solana-smart-contract

# Update wallet addresses in all config files
node update-addresses.js
```

Enter your wallet address when prompted.

### Step 5: Deploy Collection

```bash
# Validate configuration
sugar validate

# Upload assets to Arweave
sugar upload

# Deploy Candy Machine
sugar deploy

# Verify deployment
sugar verify

# Show collection details
sugar show
```

### Step 6: Test Minting

```bash
# Mint test NFTs
sugar mint --number 3
```

## 📊 Collection Details

### NFT Specifications
- **Supply**: 100 unique cars
- **Price**: 0.01 SOL per NFT
- **Royalties**: 2.5% creator fee
- **Symbol**: W3MC

### Car Categories (8 Types)
1. **Formula One** - High-speed racing cars
2. **Rally** - Off-road performance vehicles
3. **Electric GT** - Sustainable racing technology
4. **Muscle Car** - Classic American power
5. **Supercar** - Luxury performance vehicles
6. **Touring Car** - Professional racing sedans
7. **Prototype** - Experimental racing technology
8. **Vintage** - Classic racing heritage

### Rarity Distribution
- **Common** (32%): 32 cars - Gray background
- **Uncommon** (24%): 24 cars - Green background
- **Rare** (14%): 14 cars - Blue background
- **Epic** (21%): 21 cars - Purple background
- **Legendary** (9%): 9 cars - Gold background

### Performance Attributes
Each car includes game-ready attributes:
- **Speed** (0-100): Top speed capability
- **Acceleration** (0-100): 0-60 mph performance
- **Handling** (0-100): Cornering and control
- **Durability** (0-100): Resistance to damage

## 🌐 Viewing Your Collection

### Metaplex Studio
1. Visit: https://studio.metaplex.com/
2. Switch to Devnet (top-right)
3. Connect your wallet
4. View "My Collections"

### Wallets
- **Phantom**: https://phantom.app/ (devnet mode)
- **Solflare**: https://solflare.com/ (devnet mode)

### Explorers
- **Solscan**: https://solscan.io/ (devnet)
- **SolanaFM**: https://solana.fm/ (devnet)

### Marketplaces
- **Magic Eden**: https://magiceden.io/ (devnet)

## 🛠️ Troubleshooting

### Common Issues

**"Command not found"**
- Restart terminal after installation
- Check PATH environment variable
- Run as Administrator (Windows)

**"Insufficient funds"**
```bash
solana airdrop 2
# Or use web faucet
```

**"Upload failed"**
```bash
sugar upload --retry
```

**"Invalid metadata"**
```bash
node update-addresses.js
sugar validate
```

### Useful Commands

```bash
# Check Solana configuration
solana config get

# Check wallet balance
solana balance

# View transaction history
solana transaction-history

# Reset configuration
solana config set --url devnet
```

## 🚀 Mainnet Deployment

### Prerequisites
- Tested successfully on devnet
- Real SOL for deployment costs (~0.5-1 SOL)
- Production wallet with proper security

### Mainnet Steps
```bash
# Switch to mainnet
solana config set --url mainnet-beta

# Use production wallet
solana config set --keypair ~/.config/solana/mainnet-wallet.json

# Update addresses for mainnet
node update-addresses.js

# Deploy to mainnet
sugar validate && sugar upload && sugar deploy
```

## 📋 Deployment Checklist

### Pre-Deployment
- [ ] Solana CLI installed and configured
- [ ] Sugar CLI installed
- [ ] Wallet created with sufficient SOL
- [ ] All car images added (0.png - 99.png)
- [ ] Wallet addresses updated in config

### Deployment
- [ ] `sugar validate` passes
- [ ] `sugar upload` completes
- [ ] `sugar deploy` successful
- [ ] `sugar verify` confirms deployment
- [ ] Test minting works

### Post-Deployment
- [ ] Collection visible on Metaplex Studio
- [ ] NFTs appear in wallet
- [ ] Marketplace listing (if desired)
- [ ] Community announcement

## 🎊 Success Metrics

After successful deployment:
- ✅ 100 unique car NFTs available for minting
- ✅ Collection verified on Solana blockchain
- ✅ Compatible with all major Solana wallets
- ✅ Ready for marketplace integration
- ✅ Game-ready with performance attributes

## 📚 Additional Resources

- **Metaplex Documentation**: https://docs.metaplex.com/
- **Sugar CLI Guide**: https://docs.metaplex.com/tools/sugar
- **Solana Documentation**: https://docs.solana.com/
- **Candy Machine Guide**: https://docs.metaplex.com/programs/candy-machine/

---

**Ready to launch your Web3 Motorsport car collection! 🏁🚗💨**