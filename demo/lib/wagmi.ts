import { http, createConfig } from "wagmi";
import { mantleSepoliaTestnet, mantle } from "wagmi/chains";

// Mantle Sepolia configuration
export const mantleSepolia = {
  ...mantleSepoliaTestnet,
  rpcUrls: {
    default: {
      http: ["https://rpc.sepolia.mantle.xyz"],
    },
    public: {
      http: ["https://rpc.sepolia.mantle.xyz"],
    },
  },
  blockExplorers: {
    default: {
      name: "Mantle Sepolia Explorer",
      url: "https://sepolia.mantlescan.xyz",
    },
  },
};

export const wagmiConfig = createConfig({
  chains: [mantleSepolia, mantle],
  transports: {
    [mantleSepolia.id]: http(),
    [mantle.id]: http(),
  },
  ssr: true,
});

export const supportedChains = [mantleSepolia, mantle];
