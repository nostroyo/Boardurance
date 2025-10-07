# Blockchain - Solana Smart Contracts

Web3 Motorsport Car NFT Collection - A complete Metaplex-compatible Candy Machine implementation for minting 100 unique racing car NFTs on Solana.

## 🚗 Quick Start

1. **Install Sugar CLI**: `bash <(curl -sSf https://sugar.metaplex.com/install.sh)`
2. **Add car images** to `assets/` folder (0.png through 99.png)
3. **Update wallet addresses** in config files
4. **Deploy**: `sugar upload && sugar deploy && sugar verify`

## 📊 Collection Overview

- **Supply**: 100 unique car NFTs
- **Price**: 0.01 SOL per NFT
- **Royalties**: 2.5% creator fee
- **Car Types**: 8 categories (Formula One, Rally, Electric GT, etc.)
- **Rarity System**: Common to Legendary with performance multipliers

## 🏗️ Architecture

### Core Technologies
- **Anchor Framework** - Solana development framework
- **Rust** - Smart contract programming language
- **Metaplex** - NFT standard and tooling
- **Sugar CLI** - Candy Machine deployment tool

### Smart Contract Structure
```
programs/
└── solana-smart-contract/
    ├── src/
    │   └── lib.rs          # Main smart contract logic
    └── Cargo.toml          # Rust dependencies
```

## 📁 Project Structure

```
solana-smart-contract/
├── assets/                 # NFT metadata and images
│   ├── collection.json     # Collection metadata
│   └── 0.json - 99.json   # Individual NFT metadata
├── docs/                   # Detailed documentation
├── programs/               # Solana smart contracts
├── tests/                  # Test files and simulations
├── config.json            # Sugar CLI configuration
├── generate-metadata.js   # Metadata generation script
└── Anchor.toml            # Anchor configuration
```

## 🎮 NFT Features

### Car Attributes
- **Performance Stats**: Speed, Acceleration, Handling, Durability (0-100)
- **Technical Specs**: Engine type, weight, top speed
- **Rarity Levels**: Common (32%), Uncommon (24%), Rare (14%), Epic (21%), Legendary (9%)
- **Game Integration**: Ready for Web3 motorsport gameplay

### Metaplex Compatibility
- ✅ Standard NFT format compatible with all Solana wallets
- ✅ Marketplace ready (Magic Eden, OpenSea, Solanart)
- ✅ Collection verification and royalty enforcement
- ✅ Rich metadata with game attributes

## 🚀 Development Commands

### Setup
```bash
# Install dependencies
npm install

# Install Anchor CLI
npm install -g @coral-xyz/anchor-cli

# Install Sugar CLI
bash <(curl -sSf https://sugar.metaplex.com/install.sh)
```

### Development
```bash
# Generate metadata
node generate-metadata.js

# Build smart contract
anchor build

# Run tests
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

### Sugar CLI Commands
```bash
# Validate configuration
sugar validate

# Upload assets to Arweave
sugar upload

# Deploy Candy Machine
sugar deploy

# Verify deployment
sugar verify

# Show Candy Machine details
sugar show
```

## 🧪 Testing

### Test Files
- `tests/candy-machine.ts` - Candy Machine functionality tests
- `tests/car-token-mint.ts` - NFT minting tests
- `tests/car-token-simulation.js` - Minting simulation

### Running Tests
```bash
# Run all tests
anchor test

# Run specific test
anchor test --skip-local-validator tests/candy-machine.ts
```

## ⚙️ Configuration

### Solana Configuration
```bash
# Set cluster
solana config set --url devnet

# Check configuration
solana config get

# Check wallet balance
solana balance
```

### Anchor Configuration (`Anchor.toml`)
```toml
[features]
seeds = false
skip-lint = false

[programs.localnet]
solana_smart_contract = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "Localnet"
wallet = "~/.config/solana/id.json"
```

## 🚀 Deployment Status

- [x] Smart contract implementation
- [x] Metadata generation (100 cars)
- [x] Sugar CLI configuration
- [x] Test suite and simulation
- [ ] Car images (add 0.png - 99.png to assets/)
- [ ] Wallet address configuration
- [ ] Devnet deployment
- [ ] Mainnet launch

## 🔗 Integration

This smart contract integrates with:
- **Frontend**: React app for minting interface
- **Backend**: Rust API for game data and user management
- **Wallets**: Phantom, Solflare, and other Solana wallets
- **Marketplaces**: Magic Eden, OpenSea, Solanart

## 📚 Learning Resources

- **Metaplex Documentation**: https://docs.metaplex.com/
- **Sugar CLI Guide**: https://docs.metaplex.com/tools/sugar
- **Anchor Framework**: https://www.anchor-lang.com/
- **Solana Documentation**: https://docs.solana.com/

## 🎯 Game Integration

The NFTs are designed for Web3 motorsport gameplay:
- **Performance-based racing** using NFT attributes
- **Upgradeable cars** through additional NFT mechanics
- **Tournament systems** with entry fees and prizes
- **Breeding mechanics** for creating new car variants

---

**Ready to mint 100 unique racing car NFTs on Solana! 🏁🚗💨**