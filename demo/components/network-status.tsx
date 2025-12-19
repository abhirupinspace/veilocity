"use client";

import { useAccount, useChainId, useSwitchChain } from "wagmi";
import { mantleSepolia, mantleMainnet } from "@/lib/wagmi";
import { AlertTriangle } from "lucide-react";

export function NetworkStatus() {
  const { isConnected } = useAccount();
  const chainId = useChainId();
  const { switchChain, isPending } = useSwitchChain();

  const isCorrectNetwork = chainId === mantleSepolia.id || chainId === mantleMainnet.id;

  if (!isConnected || isCorrectNetwork) {
    return null;
  }

  return (
    <div className="mb-8 flex items-center justify-between gap-4 p-4 border border-accent/30 bg-accent/5">
      <div className="flex items-center gap-3">
        <AlertTriangle className="w-4 h-4 text-accent shrink-0" />
        <div>
          <p className="font-mono text-xs">Wrong Network</p>
          <p className="font-mono text-[10px] text-muted-foreground">
            Switch to Mantle Sepolia or Mainnet
          </p>
        </div>
      </div>
      <button
        onClick={() => switchChain({ chainId: mantleSepolia.id })}
        disabled={isPending}
        className="px-4 py-2 border border-foreground/20 font-mono text-[10px] uppercase tracking-widest hover:border-accent hover:text-accent transition-colors disabled:opacity-50"
      >
        {isPending ? "Switching..." : "Switch Network"}
      </button>
    </div>
  );
}
