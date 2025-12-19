"use client";

import { useState, useEffect } from "react";
import { useChainId } from "wagmi";
import { getPrivateState, PrivateDeposit } from "@/lib/crypto";
import { shortenHash } from "@/lib/utils";
import { formatDistanceToNow } from "date-fns";
import { ExternalLink, ArrowDownLeft, ArrowUpRight } from "lucide-react";

export function TransactionHistory() {
  const chainId = useChainId();
  const [deposits, setDeposits] = useState<PrivateDeposit[]>([]);

  useEffect(() => {
    const loadHistory = () => {
      const state = getPrivateState();
      const sorted = [...state.deposits].sort((a, b) => b.timestamp - a.timestamp);
      setDeposits(sorted);
    };

    loadHistory();
    const interval = setInterval(loadHistory, 5000);
    return () => clearInterval(interval);
  }, []);

  const explorerUrl =
    chainId === 5003
      ? "https://sepolia.mantlescan.xyz"
      : "https://mantlescan.xyz";

  const getStatusColor = (status: PrivateDeposit["status"]) => {
    switch (status) {
      case "confirmed":
        return "text-green-400";
      case "pending":
        return "text-yellow-400";
      case "spent":
        return "text-muted-foreground/60";
      default:
        return "text-muted-foreground/60";
    }
  };

  const getStatusLabel = (status: PrivateDeposit["status"]) => {
    switch (status) {
      case "confirmed":
        return "Available";
      case "pending":
        return "Pending";
      case "spent":
        return "Withdrawn";
      default:
        return status;
    }
  };

  return (
    <div className="card-minimal">
      <div className="p-6 border-b border-border/50">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          History
        </span>
        <p className="mt-2 font-mono text-xs text-muted-foreground">
          Your private transactions (local)
        </p>
      </div>

      <div className="divide-y divide-border/30">
        {deposits.length === 0 ? (
          <div className="p-6 text-center">
            <p className="font-mono text-xs text-muted-foreground">
              No transactions yet
            </p>
            <p className="font-mono text-[10px] text-muted-foreground/60 mt-1">
              Make a deposit to get started
            </p>
          </div>
        ) : (
          deposits.slice(0, 10).map((deposit) => (
            <div
              key={deposit.id}
              className="p-4 hover:bg-secondary/20 transition-colors"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex items-start gap-3">
                  <div className="mt-0.5">
                    {deposit.status === "spent" ? (
                      <ArrowUpRight className="w-3 h-3 text-muted-foreground/60" />
                    ) : (
                      <ArrowDownLeft className="w-3 h-3 text-green-400" />
                    )}
                  </div>
                  <div className="space-y-1 min-w-0">
                    <div className="flex items-center gap-3">
                      <span className="font-mono text-sm">
                        {deposit.status === "spent" ? "-" : "+"}{deposit.amount} MNT
                      </span>
                      <span className={`font-mono text-[10px] uppercase tracking-widest ${getStatusColor(deposit.status)}`}>
                        {getStatusLabel(deposit.status)}
                      </span>
                    </div>
                    <p className="font-mono text-[10px] text-muted-foreground/60 truncate">
                      {shortenHash(deposit.commitment)}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-3 shrink-0">
                  <span className="font-mono text-[10px] text-muted-foreground/60">
                    {formatDistanceToNow(new Date(deposit.timestamp), {
                      addSuffix: true,
                    })}
                  </span>
                  {deposit.transactionHash && (
                    <a
                      href={`${explorerUrl}/tx/${deposit.transactionHash}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-muted-foreground hover:text-accent transition-colors"
                    >
                      <ExternalLink className="w-3 h-3" />
                    </a>
                  )}
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {deposits.length > 10 && (
        <div className="p-4 border-t border-border/30 text-center">
          <p className="font-mono text-[10px] text-muted-foreground/60 uppercase tracking-widest">
            Showing 10 of {deposits.length}
          </p>
        </div>
      )}
    </div>
  );
}
