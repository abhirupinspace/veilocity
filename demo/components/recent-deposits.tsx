"use client";

import { useWatchContractEvent, useChainId } from "wagmi";
import { veilocityVaultAbi, VAULT_ADDRESSES } from "@/lib/abi";
import { formatEther, shortenHash } from "@/lib/utils";
import { useState } from "react";
import { ExternalLink } from "lucide-react";
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
    <div className="card-minimal">
      <div className="p-6 border-b border-border/50">
        <div className="flex items-center justify-between">
          <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
            Live Feed
          </span>
          <div className="flex items-center gap-2">
            <div className="relative">
              <div className="w-1.5 h-1.5 bg-green-500" />
              <div className="w-1.5 h-1.5 bg-green-500 absolute top-0 left-0 pulse-ring" />
            </div>
            <span className="font-mono text-[10px] text-muted-foreground/60 uppercase tracking-widest">
              Watching
            </span>
          </div>
        </div>
        <p className="mt-2 font-mono text-xs text-muted-foreground">
          Recent deposits on-chain
        </p>
      </div>

      <div className="divide-y divide-border/30">
        {deposits.length === 0 ? (
          <div className="p-6 text-center">
            <p className="font-mono text-xs text-muted-foreground">
              Waiting for deposits...
            </p>
          </div>
        ) : (
          deposits.map((deposit, index) => (
            <div
              key={`${deposit.transactionHash}-${index}`}
              className="p-4 hover:bg-secondary/20 transition-colors"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="space-y-1 min-w-0">
                  <div className="flex items-center gap-3">
                    <span className="font-mono text-[10px] text-muted-foreground/60">
                      #{Number(deposit.leafIndex)}
                    </span>
                    <span className="font-mono text-sm">
                      {formatEther(deposit.amount)} MNT
                    </span>
                  </div>
                  <p className="font-mono text-[10px] text-muted-foreground/60 truncate">
                    {shortenHash(deposit.commitment)}
                  </p>
                </div>
                <div className="flex items-center gap-3 shrink-0">
                  <span className="font-mono text-[10px] text-muted-foreground/60">
                    {deposit.timestamp > 0n
                      ? formatDistanceToNow(new Date(Number(deposit.timestamp) * 1000), {
                          addSuffix: true,
                        })
                      : "now"}
                  </span>
                  <a
                    href={`${explorerUrl}/tx/${deposit.transactionHash}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-muted-foreground hover:text-accent transition-colors"
                  >
                    <ExternalLink className="w-3 h-3" />
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
