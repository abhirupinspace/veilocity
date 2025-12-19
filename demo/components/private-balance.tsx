"use client";

import { useState, useEffect } from "react";
import { useAccount } from "wagmi";
import {
  getPrivateState,
  getPrivateBalance,
  getAvailableDeposits,
  clearPrivateState,
  PrivateDeposit,
} from "@/lib/crypto";
import { shortenHash } from "@/lib/utils";
import {
  Eye,
  EyeOff,
  Trash2,
  Download,
  Upload,
  RefreshCw,
} from "lucide-react";
import { toast } from "sonner";

export function PrivateBalance() {
  const { isConnected } = useAccount();
  const [balance, setBalance] = useState("0");
  const [deposits, setDeposits] = useState<PrivateDeposit[]>([]);
  const [showDetails, setShowDetails] = useState(false);

  useEffect(() => {
    const loadBalance = () => {
      const bal = getPrivateBalance();
      setBalance(bal);
      setDeposits(getAvailableDeposits());
    };

    loadBalance();
    const interval = setInterval(loadBalance, 5000);
    return () => clearInterval(interval);
  }, []);

  const handleRefresh = () => {
    const bal = getPrivateBalance();
    setBalance(bal);
    setDeposits(getAvailableDeposits());
    toast.success("Refreshed");
  };

  const handleClearState = () => {
    if (confirm("Clear all private state? This cannot be undone.")) {
      clearPrivateState();
      setBalance("0");
      setDeposits([]);
      toast.success("Cleared");
    }
  };

  const handleExportAll = () => {
    const state = getPrivateState();
    const blob = new Blob([JSON.stringify(state, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `veilocity-backup-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
    toast.success("Exported");
  };

  const handleImport = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      try {
        const text = await file.text();
        const imported = JSON.parse(text);

        if (imported.deposits && Array.isArray(imported.deposits)) {
          localStorage.setItem("veilocity_private_state", text);
          setBalance(imported.totalBalance || "0");
          setDeposits(imported.deposits.filter((d: PrivateDeposit) => d.status === "confirmed"));
          toast.success("Imported");
        } else {
          toast.error("Invalid file");
        }
      } catch {
        toast.error("Failed to import");
      }
    };
    input.click();
  };

  const hasDeposits = parseFloat(balance) > 0 || deposits.length > 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
          Private Balance
        </span>
        <div className="flex items-center gap-3">
          <button
            onClick={handleRefresh}
            className="text-muted-foreground hover:text-foreground transition-colors"
            title="Refresh"
          >
            <RefreshCw className="w-3 h-3" />
          </button>
          <button
            onClick={() => setShowDetails(!showDetails)}
            className="text-muted-foreground hover:text-foreground transition-colors"
            title={showDetails ? "Hide" : "Show details"}
          >
            {showDetails ? <EyeOff className="w-3 h-3" /> : <Eye className="w-3 h-3" />}
          </button>
        </div>
      </div>

      {/* Balance */}
      <div>
        <p className="text-4xl md:text-5xl font-mono font-medium tracking-tight">
          {balance}
          <span className="text-lg text-muted-foreground ml-2">MNT</span>
        </p>
        <p className="mt-2 font-mono text-[10px] text-muted-foreground/60 uppercase tracking-widest">
          {deposits.length} deposit{deposits.length !== 1 ? "s" : ""} available
        </p>
      </div>

      {/* Deposits List */}
      {showDetails && hasDeposits && (
        <div className="space-y-3 pt-4 border-t border-border/30">
          {deposits.map((deposit) => (
            <div
              key={deposit.id}
              className="flex items-center justify-between py-2"
            >
              <div>
                <span className="font-mono text-sm">{deposit.amount} MNT</span>
                <p className="font-mono text-[10px] text-muted-foreground/60">
                  {shortenHash(deposit.commitment)}
                </p>
              </div>
              <span className="font-mono text-[10px] text-green-400 uppercase tracking-widest">
                Available
              </span>
            </div>
          ))}
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-3 pt-4 border-t border-border/30">
        <button
          onClick={handleExportAll}
          className="flex items-center gap-2 font-mono text-[10px] text-muted-foreground hover:text-foreground transition-colors uppercase tracking-widest"
          disabled={!hasDeposits}
        >
          <Download className="w-3 h-3" />
          Export
        </button>
        <button
          onClick={handleImport}
          className="flex items-center gap-2 font-mono text-[10px] text-muted-foreground hover:text-foreground transition-colors uppercase tracking-widest"
        >
          <Upload className="w-3 h-3" />
          Import
        </button>
        {hasDeposits && (
          <button
            onClick={handleClearState}
            className="flex items-center gap-2 font-mono text-[10px] text-red-400 hover:text-red-300 transition-colors uppercase tracking-widest ml-auto"
          >
            <Trash2 className="w-3 h-3" />
            Clear
          </button>
        )}
      </div>

      {!isConnected && (
        <p className="font-mono text-[10px] text-muted-foreground/60 uppercase tracking-widest">
          Connect wallet to view balance
        </p>
      )}
    </div>
  );
}
