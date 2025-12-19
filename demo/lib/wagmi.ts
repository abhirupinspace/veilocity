import { http, createConfig } from "wagmi";
import { defineChain } from "viem";

// Define Mantle Sepolia Testnet
export const mantleSepolia = defineChain({
  id: 5003,
  name: "Mantle Sepolia Testnet",
  nativeCurrency: {
    decimals: 18,
    name: "MNT",
    symbol: "MNT",
  },
  rpcUrls: {
    default: {
      http: ["https://rpc.sepolia.mantle.xyz"],
    },
  },
  blockExplorers: {
    default: {
      name: "Mantle Sepolia Explorer",
      url: "https://sepolia.mantlescan.xyz",
      apiUrl: "https://api-sepolia.mantlescan.xyz/api",
    },
  },
  testnet: true,
});

// Define Mantle Mainnet
export const mantleMainnet = defineChain({
  id: 5000,
  name: "Mantle",
  nativeCurrency: {
    decimals: 18,
    name: "MNT",
    symbol: "MNT",
  },
  rpcUrls: {
    default: {
      http: ["https://rpc.mantle.xyz"],
    },
  },
  blockExplorers: {
    default: {
      name: "Mantle Explorer",
      url: "https://mantlescan.xyz",
      apiUrl: "https://api.mantlescan.xyz/api",
    },
  },
  testnet: false,
});

export const wagmiConfig = createConfig({
  chains: [mantleSepolia, mantleMainnet],
  transports: {
    [mantleSepolia.id]: http("https://rpc.sepolia.mantle.xyz"),
    [mantleMainnet.id]: http("https://rpc.mantle.xyz"),
  },
  ssr: true,
});

export const supportedChains = [mantleSepolia, mantleMainnet];
