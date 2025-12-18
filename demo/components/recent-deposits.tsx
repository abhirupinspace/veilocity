"use client";

import { useWatchContractEvent, useChainId } from "wagmi";
import { veilocityVaultAbi, VAULT_ADDRESSES } from "@/lib/abi";
import { formatEther, shortenHash } from "@/lib/utils";
import { useState, useEffect } from "react";
import { History, ExternalLink } from "lucide-react";
import { formatDistanceToNow } from "date-fns";

interface DepositEvent {
  commitment: string;
  amount: bigint;
  leafIndex: bigint;
  timestamp: bigint;
  transactionHash: string;
}

export function RecentDeposits() {
  const chainId = useChainId();
  const vaultAddress = VAULT_ADDRESSES[chainId];
  const [deposits, setDeposits] = useState<DepositEvent[]>([]);

  // Watch for new deposit events
  useWatchContractEvent({
    address: vaultAddress,
    abi: veilocityVaultAbi,
    eventName: "Deposit",
    onLogs(logs) {
      const newDeposits = logs.map((log) => ({
        commitment: log.args.commitment as string,
        amount: log.args.amount as bigint,
        leafIndex: log.args.leafIndex as bigint,
        timestamp: log.args.timestamp as bigint,
        transactionHash: log.transactionHash,
      }));

      setDeposits((prev) => {
        const combined = [...newDeposits, ...prev];
        // Keep only the last 10 deposits
        return combined.slice(0, 10);
      });
    },
    enabled: !!vaultAddress && vaultAddress !== "0x0000000000000000000000000000000000000000",
  });

  const explorerUrl =
    chainId === 5003
      ? "https://sepolia.mantlescan.xyz"
      : "https://mantlescan.xyz";

  const noContract = !vaultAddress || vaultAddress === "0x0000000000000000000000000000000000000000";

  if (noContract) {
    return null;
  }

  return (
    <div className="bg-card border border-border">
      <div className="p-6 border-b border-border">
        <div className="flex items-center gap-2">
          <History className="w-5 h-5 text-accent" />
          <h2 className="text-lg font-medium">Recent Deposits</h2>
        </div>
        <p className="text-sm text-muted-foreground mt-1 font-mono">
          Live feed of deposits (updates in real-time)
        </p>
      </div>

      <div className="divide-y divide-border">
        {deposits.length === 0 ? (
          <div className="p-6 text-center">
            <p className="text-sm text-muted-foreground font-mono">
              No deposits yet. Waiting for events...
            </p>
          </div>
        ) : (
          deposits.map((deposit, index) => (
            <div
              key={`${deposit.transactionHash}-${index}`}
              className="p-4 hover:bg-secondary/30 transition-colors"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="space-y-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-xs bg-secondary px-2 py-0.5 font-mono">
                      #{Number(deposit.leafIndex)}
                    </span>
                    <span className="font-mono text-sm font-medium">
                      {formatEther(deposit.amount)} MNT
                    </span>
                  </div>
                  <p className="text-xs text-muted-foreground font-mono truncate">
                    Commitment: {shortenHash(deposit.commitment)}
                  </p>
                </div>
                <div className="flex items-center gap-2 shrink-0">
                  <span className="text-xs text-muted-foreground font-mono">
                    {deposit.timestamp > 0n
                      ? formatDistanceToNow(new Date(Number(deposit.timestamp) * 1000), {
                          addSuffix: true,
                        })
                      : "just now"}
                  </span>
                  <a
                    href={`${explorerUrl}/tx/${deposit.transactionHash}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-muted-foreground hover:text-accent transition-colors"
                  >
                    <ExternalLink className="w-4 h-4" />
                  </a>
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
