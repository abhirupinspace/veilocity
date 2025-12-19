"use client";

import { useReadContract } from "wagmi";
import { veilocityVaultAbi, VAULT_ADDRESSES } from "@/lib/abi";
import { formatEther, shortenHash } from "@/lib/utils";
import { useChainId } from "wagmi";
import { RefreshCw } from "lucide-react";

export function VaultStats() {
  const chainId = useChainId();
  const vaultAddress = VAULT_ADDRESSES[chainId];

  const { data: tvl, isLoading: tvlLoading, refetch: refetchTvl } = useReadContract({
    address: vaultAddress,
    abi: veilocityVaultAbi,
    functionName: "totalValueLocked",
    query: {
      enabled: !!vaultAddress && vaultAddress !== "0x0000000000000000000000000000000000000000",
      refetchInterval: 10000,
    },
  });

  const { data: depositCount, isLoading: countLoading, refetch: refetchCount } = useReadContract({
    address: vaultAddress,
    abi: veilocityVaultAbi,
    functionName: "depositCount",
    query: {
      enabled: !!vaultAddress && vaultAddress !== "0x0000000000000000000000000000000000000000",
      refetchInterval: 10000,
    },
  });

  const { data: currentRoot, isLoading: rootLoading, refetch: refetchRoot } = useReadContract({
    address: vaultAddress,
    abi: veilocityVaultAbi,
    functionName: "currentRoot",
    query: {
      enabled: !!vaultAddress && vaultAddress !== "0x0000000000000000000000000000000000000000",
      refetchInterval: 10000,
    },
  });

  const { data: isPaused, isLoading: pausedLoading } = useReadContract({
    address: vaultAddress,
    abi: veilocityVaultAbi,
    functionName: "paused",
    query: {
      enabled: !!vaultAddress && vaultAddress !== "0x0000000000000000000000000000000000000000",
      refetchInterval: 30000,
    },
  });

  const handleRefresh = () => {
    refetchTvl();
    refetchCount();
    refetchRoot();
  };

  const noContract = !vaultAddress || vaultAddress === "0x0000000000000000000000000000000000000000";

  if (noContract) {
    return (
      <div className="py-8 text-center">
        <p className="font-mono text-xs text-muted-foreground">
          Configure contract address to view stats
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="relative">
            <div className="w-1.5 h-1.5 bg-green-500" />
            <div className="w-1.5 h-1.5 bg-green-500 absolute top-0 left-0 pulse-ring" />
          </div>
          <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
            Live
          </span>
        </div>
        <button
          onClick={handleRefresh}
          className="flex items-center gap-2 font-mono text-[10px] text-muted-foreground hover:text-foreground transition-colors uppercase tracking-widest"
        >
          <RefreshCw className="w-3 h-3" />
          Refresh
        </button>
      </div>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-6">
        <div>
          <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground/60">
            TVL
          </span>
          {tvlLoading ? (
            <div className="mt-2 h-8 w-20 bg-secondary/50 animate-pulse" />
          ) : (
            <p className="mt-2 text-2xl font-mono font-medium">
              {tvl !== undefined ? formatEther(tvl as bigint) : "—"}
              <span className="text-sm text-muted-foreground ml-1">MNT</span>
            </p>
          )}
        </div>

        <div>
          <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground/60">
            Deposits
          </span>
          {countLoading ? (
            <div className="mt-2 h-8 w-16 bg-secondary/50 animate-pulse" />
          ) : (
            <p className="mt-2 text-2xl font-mono font-medium">
              {depositCount !== undefined ? Number(depositCount).toLocaleString() : "—"}
            </p>
          )}
        </div>

        <div className="min-w-0">
          <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground/60">
            State Root
          </span>
          {rootLoading ? (
            <div className="mt-2 h-8 w-24 bg-secondary/50 animate-pulse" />
          ) : (
            <p className="mt-2 text-lg font-mono text-muted-foreground truncate">
              {currentRoot ? shortenHash(currentRoot as string) : "—"}
            </p>
          )}
        </div>

        <div>
          <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground/60">
            Status
          </span>
          {pausedLoading ? (
            <div className="mt-2 h-8 w-16 bg-secondary/50 animate-pulse" />
          ) : (
            <div className="mt-2 flex items-center gap-2">
              <div className={`w-1.5 h-1.5 ${isPaused ? "bg-red-500" : "bg-green-500"}`} />
              <span className="text-lg font-mono">
                {isPaused ? "Paused" : "Active"}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
