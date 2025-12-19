"use client";

import Link from "next/link";
import { TransactionHistory } from "@/components/transaction-history";
import { RecentDeposits } from "@/components/recent-deposits";
import { ArrowLeft, Download, Upload } from "lucide-react";
import { getPrivateState, clearPrivateState } from "@/lib/crypto";
import { toast } from "sonner";

export default function HistoryPage() {
  const handleExport = () => {
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
    toast.success("Backup exported");
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
          toast.success("Backup imported - refreshing...");
          setTimeout(() => window.location.reload(), 1000);
        } else {
          toast.error("Invalid backup file");
        }
      } catch {
        toast.error("Failed to parse backup");
      }
    };
    input.click();
  };

  return (
    <div className="space-y-8">
      {/* Header */}
      <section>
        <Link
          href="/"
          className="inline-flex items-center gap-2 font-mono text-[10px] uppercase tracking-widest text-muted-foreground hover:text-foreground transition-colors mb-4"
        >
          <ArrowLeft className="w-3 h-3" />
          Back to Dashboard
        </Link>
        <div className="flex items-start justify-between">
          <div>
            <span className="block font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
              Transaction History
            </span>
            <h1 className="mt-3 text-3xl md:text-4xl font-medium tracking-tight">
              Activity
            </h1>
            <p className="mt-4 max-w-md font-mono text-sm text-muted-foreground leading-relaxed">
              View your private transaction history. This data is stored locally
              and is not shared with anyone.
            </p>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-2">
            <button
              onClick={handleExport}
              className="flex items-center gap-2 px-3 py-2 border border-border/50 font-mono text-[10px] uppercase tracking-widest text-muted-foreground hover:text-foreground hover:border-border transition-colors"
            >
              <Download className="w-3 h-3" />
              Export
            </button>
            <button
              onClick={handleImport}
              className="flex items-center gap-2 px-3 py-2 border border-border/50 font-mono text-[10px] uppercase tracking-widest text-muted-foreground hover:text-foreground hover:border-border transition-colors"
            >
              <Upload className="w-3 h-3" />
              Import
            </button>
          </div>
        </div>
      </section>

      {/* Your Transactions */}
      <section>
        <span className="block font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground mb-4">
          Your Transactions
        </span>
        <TransactionHistory />
      </section>

      {/* On-chain Activity */}
      <section>
        <span className="block font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground mb-4">
          On-chain Activity
        </span>
        <RecentDeposits />
      </section>

      {/* Info */}
      <section className="pt-8 border-t border-border/30">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          Privacy Notice
        </span>
        <div className="mt-4 space-y-4 font-mono text-xs text-muted-foreground leading-relaxed">
          <p>
            <strong className="text-foreground">Local storage only.</strong>{" "}
            Your transaction history and secret keys are stored in your browser&apos;s
            local storage. They are never sent to any server.
          </p>
          <p>
            <strong className="text-foreground">Export regularly.</strong>{" "}
            We recommend exporting backups regularly and storing them securely.
            If you clear your browser data, your local history will be lost.
          </p>
          <p>
            <strong className="text-foreground">On-chain data.</strong>{" "}
            The on-chain activity shows public deposit events. These do not
            reveal any information about the depositor or the amount relationship.
          </p>
        </div>
      </section>
    </div>
  );
}
