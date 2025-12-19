"use client";

import Link from "next/link";
import { DepositForm } from "@/components/deposit-form";
import { RecentDeposits } from "@/components/recent-deposits";
import { ArrowLeft } from "lucide-react";

export default function DepositPage() {
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
        <span className="block font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          Private Deposit
        </span>
        <h1 className="mt-3 text-3xl md:text-4xl font-medium tracking-tight">
          Add Funds
        </h1>
        <p className="mt-4 max-w-md font-mono text-sm text-muted-foreground leading-relaxed">
          Deposit MNT into the private execution layer. A cryptographic commitment
          will be created, and you&apos;ll receive a secret key to prove ownership.
        </p>
      </section>

      {/* Deposit Form */}
      <section>
        <DepositForm />
      </section>

      {/* Recent Activity */}
      <section>
        <span className="block font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground mb-4">
          Recent Activity
        </span>
        <RecentDeposits />
      </section>

      {/* Info */}
      <section className="pt-8 border-t border-border/30">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          Important
        </span>
        <div className="mt-4 space-y-4 font-mono text-xs text-muted-foreground leading-relaxed">
          <p>
            <strong className="text-foreground">Save your secret key.</strong>{" "}
            This is the only way to prove ownership of your deposit. Without it,
            your funds cannot be recovered.
          </p>
          <p>
            <strong className="text-foreground">Download a backup.</strong>{" "}
            We recommend downloading a backup file immediately after depositing.
            Store it securely offline.
          </p>
          <p>
            <strong className="text-foreground">Minimum deposit:</strong>{" "}
            0.001 MNT
          </p>
        </div>
      </section>
    </div>
  );
}
