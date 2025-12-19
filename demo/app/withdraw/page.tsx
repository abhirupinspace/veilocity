"use client";

import Link from "next/link";
import { WithdrawForm } from "@/components/withdraw-form";
import { ArrowLeft, Terminal } from "lucide-react";

export default function WithdrawPage() {
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
          Private Withdrawal
        </span>
        <h1 className="mt-3 text-3xl md:text-4xl font-medium tracking-tight">
          Withdraw Funds
        </h1>
        <p className="mt-4 max-w-md font-mono text-sm text-muted-foreground leading-relaxed">
          Exit from the private layer to any address. A zero-knowledge proof
          will be generated to verify your ownership without revealing your identity.
        </p>
      </section>

      {/* Withdraw Form */}
      <section>
        <WithdrawForm />
      </section>

      {/* CLI Instructions */}
      <section className="pt-8 border-t border-border/30">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          CLI Usage
        </span>
        <div className="mt-4 space-y-4">
          <p className="font-mono text-xs text-muted-foreground leading-relaxed">
            Full withdrawals require the CLI to generate zero-knowledge proofs locally.
            This ensures your privacy is maintained throughout the process.
          </p>

          <div className="card-minimal p-4">
            <div className="flex items-center gap-2 mb-3">
              <Terminal className="w-4 h-4 text-accent" />
              <span className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
                Commands
              </span>
            </div>
            <div className="space-y-3 font-mono text-xs">
              <div>
                <p className="text-muted-foreground/60 mb-1"># Install CLI</p>
                <code className="text-foreground">cargo install veilocity-cli</code>
              </div>
              <div>
                <p className="text-muted-foreground/60 mb-1"># Withdraw to address</p>
                <code className="text-foreground">veilocity withdraw 0.1 --recipient 0x...</code>
              </div>
              <div>
                <p className="text-muted-foreground/60 mb-1"># Check balance</p>
                <code className="text-foreground">veilocity balance</code>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Info */}
      <section className="pt-8 border-t border-border/30">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
          How It Works
        </span>
        <div className="mt-4 space-y-4 font-mono text-xs text-muted-foreground leading-relaxed">
          <p>
            <strong className="text-foreground">1. Select deposit.</strong>{" "}
            Choose which deposit you want to withdraw from.
          </p>
          <p>
            <strong className="text-foreground">2. Generate proof.</strong>{" "}
            The CLI generates a ZK proof using your secret key.
          </p>
          <p>
            <strong className="text-foreground">3. Submit transaction.</strong>{" "}
            The proof is verified on-chain and funds are sent to your recipient.
          </p>
        </div>
      </section>
    </div>
  );
}
