"use client";

import { useReadContract } from "wagmi";
import { veilocityVaultAbi, VAULT_ADDRESSES } from "@/lib/abi";
import { formatEther, shortenHash } from "@/lib/utils";
import { useChainId } from "wagmi";
import { Activity, Lock, Database, Hash, RefreshCw } from "lucide-react";

interface StatCardProps {
  label: string;
  value: string | number;
  icon: React.ReactNode;
  subtext?: string;
  isLoading?: boolean;
}

function StatCard({ label, value, icon, subtext, isLoading }: StatCardProps) {
  return (
    <div className="bg-card border border-border p-6 card-hover">
      <div className="flex items-start justify-between">
        <div className="space-y-1">
          <p className="text-xs text-muted-foreground font-mono uppercase tracking-wider">{label}</p>
          {isLoading ? (
            <div className="h-8 w-24 bg-secondary animate-pulse" />
          ) : (
            <p className="text-2xl font-mono font-medium">{value}</p>
          )}
          {subtext && <p className="text-xs text-muted-foreground font-mono">{subtext}</p>}
        </div>
        <div className="text-accent">{icon}</div>
      </div>
    </div>
  );
}

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
      <div className="bg-card border border-border p-8 text-center">
        <Database className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
        <h3 className="text-lg font-medium mb-2">No Contract Deployed</h3>
        <p className="text-sm text-muted-foreground font-mono max-w-md mx-auto">
          Configure the VeilocityVault contract address in <code className="bg-secondary px-1">lib/abi.ts</code> to see real-time stats.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="relative">
            <div className="w-2 h-2 bg-green-500 rounded-full" />
            <div className="w-2 h-2 bg-green-500 rounded-full absolute top-0 left-0 pulse-ring" />
          </div>
          <span className="text-xs text-muted-foreground font-mono uppercase tracking-wider">Live Data</span>
        </div>
        <button
          onClick={handleRefresh}
          className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors font-mono"
        >
          <RefreshCw className="w-3 h-3" />
          Refresh
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Total Value Locked"
          value={tvl !== undefined ? `${formatEther(tvl as bigint)} MNT` : "..."}
          icon={<Lock className="w-5 h-5" />}
          isLoading={tvlLoading}
        />
        <StatCard
          label="Total Deposits"
          value={depositCount !== undefined ? Number(depositCount).toLocaleString() : "..."}
          icon={<Database className="w-5 h-5" />}
          subtext="Commitments in tree"
          isLoading={countLoading}
        />
        <StatCard
          label="Current Root"
          value={currentRoot ? shortenHash(currentRoot as string) : "..."}
          icon={<Hash className="w-5 h-5" />}
          subtext="Merkle state root"
          isLoading={rootLoading}
        />
        <StatCard
          label="Status"
          value={isPaused ? "Paused" : "Active"}
          icon={<Activity className="w-5 h-5" />}
          subtext={isPaused ? "Contract paused" : "Accepting deposits"}
          isLoading={pausedLoading}
        />
      </div>
    </div>
  );
}
