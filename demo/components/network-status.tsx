"use client";

import { useAccount, useChainId, useSwitchChain } from "wagmi";
import { mantleSepolia } from "@/lib/wagmi";
import { AlertTriangle } from "lucide-react";

export function NetworkStatus() {
  const { isConnected } = useAccount();
  const chainId = useChainId();
  const { switchChain, isPending } = useSwitchChain();

  const isCorrectNetwork = chainId === mantleSepolia.id;

  if (!isConnected || isCorrectNetwork) {
    return null;
  }

  return (
    <div className="bg-accent/10 border border-accent/30 p-4 flex items-center justify-between gap-4">
      <div className="flex items-center gap-3">
        <AlertTriangle className="w-5 h-5 text-accent shrink-0" />
        <div>
          <p className="text-sm font-medium">Wrong Network</p>
          <p className="text-xs text-muted-foreground font-mono">
            Please switch to Mantle Sepolia to use this demo
          </p>
        </div>
      </div>
      <button
        onClick={() => switchChain({ chainId: mantleSepolia.id })}
        disabled={isPending}
        className="px-4 py-2 bg-accent text-accent-foreground font-mono text-sm hover:opacity-90 transition-opacity disabled:opacity-50"
      >
        {isPending ? "Switching..." : "Switch Network"}
      </button>
    </div>
  );
}
