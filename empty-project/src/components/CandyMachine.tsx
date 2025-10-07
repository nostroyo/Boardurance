import React, { useState, useEffect } from 'react';
import { Connection, PublicKey, clusterApiUrl } from '@solana/web3.js';

interface CandyMachineState {
  authority: string;
  maxSupply: number;
  itemsRedeemed: number;
  price: number;
  symbol: string;
  sellerFeeBasisPoints: number;
  isActive: boolean;
}

interface MintedNFT {
  mint: string;
  name: string;
  image: string;
  attributes: {
    carType: string;
    rarity: string;
    speed: number;
    acceleration: number;
    handling: number;
    durability: number;
  };
}

const CandyMachine: React.FC = () => {
  const [candyMachine, setCandyMachine] = useState<CandyMachineState | null>(null);
  const [mintedNFTs, setMintedNFTs] = useState<MintedNFT[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedNFT, setSelectedNFT] = useState<MintedNFT | null>(null);

  // Mock data for demonstration (replace with actual blockchain data)
  useEffect(() => {
    // Simulate loading candy machine state
    const mockCandyMachine: CandyMachineState = {
      authority: "11111111111111111111111111111111",
      maxSupply: 100,
      itemsRedeemed: 100,
      price: 0.01,
      symbol: "CAR",
      sellerFeeBasisPoints: 250,
      isActive: false // Sold out
    };

    // Generate mock NFTs based on our simulation results
    const mockNFTs: MintedNFT[] = Array.from({ length: 100 }, (_, i) => {
      const carTypes = ["Formula One", "Rally Car", "Sports Car", "Muscle Car", "Electric GT", "Hypercar", "Drift Car", "Touring Car"];
      const rarities = ["Common", "Uncommon", "Rare", "Epic", "Legendary"];
      
      const carType = carTypes[Math.floor(Math.random() * carTypes.length)];
      const rarity = rarities[Math.floor(Math.random() * rarities.length)];
      const rarityMultiplier = rarities.indexOf(rarity) * 0.2 + 0.6;
      
      return {
        mint: `${Math.random().toString(36).substring(2, 15)}${Math.random().toString(36).substring(2, 15)}`,
        name: `${carType} #${(i + 1).toString().padStart(3, '0')}`,
        image: `https://api.dicebear.com/7.x/shapes/svg?seed=${carType}${i}&backgroundColor=1e40af,7c3aed,db2777`,
        attributes: {
          carType,
          rarity,
          speed: Math.min(Math.floor((Math.random() * 40 + 60) * rarityMultiplier), 100),
          acceleration: Math.min(Math.floor((Math.random() * 40 + 60) * rarityMultiplier), 100),
          handling: Math.min(Math.floor((Math.random() * 40 + 60) * rarityMultiplier), 100),
          durability: Math.min(Math.floor((Math.random() * 40 + 60) * rarityMultiplier), 100),
        }
      };
    });

    setCandyMachine(mockCandyMachine);
    setMintedNFTs(mockNFTs);
  }, []);

  const getRarityColor = (rarity: string) => {
    switch (rarity) {
      case 'Common': return 'text-gray-600 bg-gray-100';
      case 'Uncommon': return 'text-green-600 bg-green-100';
      case 'Rare': return 'text-blue-600 bg-blue-100';
      case 'Epic': return 'text-purple-600 bg-purple-100';
      case 'Legendary': return 'text-yellow-600 bg-yellow-100';
      default: return 'text-gray-600 bg-gray-100';
    }
  };

  const getPerformanceColor = (value: number) => {
    if (value >= 90) return 'text-green-600';
    if (value >= 75) return 'text-blue-600';
    if (value >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  if (!candyMachine) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-4xl font-bold text-gray-900 mb-4">
            🏁 Car NFT Candy Machine
          </h1>
          <p className="text-xl text-gray-600">
            Web3 Motorsport Game Collection
          </p>
        </div>

        {/* Candy Machine Stats */}
        <div className="bg-white rounded-lg shadow-lg p-6 mb-8">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="text-center">
              <div className="text-3xl font-bold text-blue-600">
                {candyMachine.itemsRedeemed}/{candyMachine.maxSupply}
              </div>
              <div className="text-gray-600">Minted</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-green-600">
                {candyMachine.price} SOL
              </div>
              <div className="text-gray-600">Price</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-purple-600">
                {candyMachine.symbol}
              </div>
              <div className="text-gray-600">Symbol</div>
            </div>
            <div className="text-center">
              <div className="text-3xl font-bold text-red-600">
                {candyMachine.sellerFeeBasisPoints / 100}%
              </div>
              <div className="text-gray-600">Royalty</div>
            </div>
          </div>
          
          <div className="mt-6">
            <div className="bg-gray-200 rounded-full h-4">
              <div 
                className="bg-blue-600 h-4 rounded-full transition-all duration-300"
                style={{ width: `${(candyMachine.itemsRedeemed / candyMachine.maxSupply) * 100}%` }}
              ></div>
            </div>
            <div className="text-center mt-2 text-sm text-gray-600">
              {candyMachine.itemsRedeemed === candyMachine.maxSupply ? '🔥 SOLD OUT!' : 'Minting Progress'}
            </div>
          </div>
        </div>

        {/* Collection Stats */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
          <div className="bg-white rounded-lg shadow-lg p-6">
            <h3 className="text-xl font-bold mb-4">🚗 Car Types</h3>
            <div className="space-y-2">
              {Object.entries(
                mintedNFTs.reduce((acc, nft) => {
                  acc[nft.attributes.carType] = (acc[nft.attributes.carType] || 0) + 1;
                  return acc;
                }, {} as Record<string, number>)
              ).map(([type, count]) => (
                <div key={type} className="flex justify-between">
                  <span>{type}</span>
                  <span className="font-semibold">{count}</span>
                </div>
              ))}
            </div>
          </div>

          <div className="bg-white rounded-lg shadow-lg p-6">
            <h3 className="text-xl font-bold mb-4">💎 Rarity Distribution</h3>
            <div className="space-y-2">
              {Object.entries(
                mintedNFTs.reduce((acc, nft) => {
                  acc[nft.attributes.rarity] = (acc[nft.attributes.rarity] || 0) + 1;
                  return acc;
                }, {} as Record<string, number>)
              ).map(([rarity, count]) => (
                <div key={rarity} className="flex justify-between">
                  <span className={`px-2 py-1 rounded text-sm ${getRarityColor(rarity)}`}>
                    {rarity}
                  </span>
                  <span className="font-semibold">{count}</span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* NFT Grid */}
        <div className="bg-white rounded-lg shadow-lg p-6">
          <h3 className="text-2xl font-bold mb-6">🏆 Car Collection ({mintedNFTs.length} NFTs)</h3>
          
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
            {mintedNFTs.slice(0, 20).map((nft) => (
              <div 
                key={nft.mint}
                className="border rounded-lg p-4 hover:shadow-lg transition-shadow cursor-pointer"
                onClick={() => setSelectedNFT(nft)}
              >
                <img 
                  src={nft.image} 
                  alt={nft.name}
                  className="w-full h-32 object-cover rounded mb-3"
                />
                <h4 className="font-semibold text-sm mb-2">{nft.name}</h4>
                <div className={`inline-block px-2 py-1 rounded text-xs ${getRarityColor(nft.attributes.rarity)}`}>
                  {nft.attributes.rarity}
                </div>
                <div className="mt-2 text-xs text-gray-600">
                  Total: {nft.attributes.speed + nft.attributes.acceleration + nft.attributes.handling + nft.attributes.durability}/400
                </div>
              </div>
            ))}
          </div>

          {mintedNFTs.length > 20 && (
            <div className="text-center mt-6">
              <button className="bg-blue-600 text-white px-6 py-2 rounded-lg hover:bg-blue-700">
                Load More ({mintedNFTs.length - 20} remaining)
              </button>
            </div>
          )}
        </div>

        {/* NFT Detail Modal */}
        {selectedNFT && (
          <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
            <div className="bg-white rounded-lg max-w-md w-full p-6">
              <div className="flex justify-between items-center mb-4">
                <h3 className="text-xl font-bold">{selectedNFT.name}</h3>
                <button 
                  onClick={() => setSelectedNFT(null)}
                  className="text-gray-500 hover:text-gray-700"
                >
                  ✕
                </button>
              </div>
              
              <img 
                src={selectedNFT.image} 
                alt={selectedNFT.name}
                className="w-full h-48 object-cover rounded mb-4"
              />
              
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span>Car Type:</span>
                  <span className="font-semibold">{selectedNFT.attributes.carType}</span>
                </div>
                
                <div className="flex justify-between">
                  <span>Rarity:</span>
                  <span className={`px-2 py-1 rounded text-sm ${getRarityColor(selectedNFT.attributes.rarity)}`}>
                    {selectedNFT.attributes.rarity}
                  </span>
                </div>
                
                <div className="border-t pt-3">
                  <h4 className="font-semibold mb-2">Performance Stats:</h4>
                  <div className="space-y-2">
                    <div className="flex justify-between">
                      <span>Speed:</span>
                      <span className={`font-semibold ${getPerformanceColor(selectedNFT.attributes.speed)}`}>
                        {selectedNFT.attributes.speed}/100
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>Acceleration:</span>
                      <span className={`font-semibold ${getPerformanceColor(selectedNFT.attributes.acceleration)}`}>
                        {selectedNFT.attributes.acceleration}/100
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>Handling:</span>
                      <span className={`font-semibold ${getPerformanceColor(selectedNFT.attributes.handling)}`}>
                        {selectedNFT.attributes.handling}/100
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>Durability:</span>
                      <span className={`font-semibold ${getPerformanceColor(selectedNFT.attributes.durability)}`}>
                        {selectedNFT.attributes.durability}/100
                      </span>
                    </div>
                  </div>
                </div>
                
                <div className="border-t pt-3">
                  <div className="text-sm text-gray-600">
                    <div>Mint: {selectedNFT.mint.substring(0, 8)}...</div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default CandyMachine;