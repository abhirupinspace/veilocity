"use client";

import { PrivyProvider } from "@privy-io/react-auth";
import { WagmiProvider } from "@privy-io/wagmi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { wagmiConfig, mantleSepolia, mantleMainnet } from "@/lib/wagmi";
import { useState } from "react";

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 5 * 1000,
            refetchInterval: 10 * 1000,
          },
        },
      })
  );

  return (
    <PrivyProvider
      appId={process.env.NEXT_PUBLIC_PRIVY_APP_ID || "clxxxxxxxxxxxxxx"}
      config={{
        appearance: {
          theme: "dark",
          accentColor: "#c2410c",
          logo: undefined,
        },
        embeddedWallets: {
          createOnLogin: "users-without-wallets",
        },
        defaultChain: mantleSepolia,
        supportedChains: [mantleSepolia, mantleMainnet],
        loginMethods: ["wallet", "email"],
      }}
    >
      <QueryClientProvider client={queryClient}>
        <WagmiProvider config={wagmiConfig}>{children}</WagmiProvider>
      </QueryClientProvider>
    </PrivyProvider>
  );
}
