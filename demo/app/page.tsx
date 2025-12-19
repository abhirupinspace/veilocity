"use client";

import Link from "next/link";
import { VaultStats } from "@/components/vault-stats";
import { PrivateBalance } from "@/components/private-balance";
import { useChainId } from "wagmi";
import { ArrowDownLeft, ArrowUpRight, ExternalLink } from "lucide-react";
import { VAULT_ADDRESSES } from "@/lib/abi";

export default function Dashboard() {
  const chainId = useChainId();
  const vaultAddress = VAULT_ADDRESSES[chainId];
  const explorerUrl =
    chainId === 5003
      ? "https://sepolia.mantlescan.xyz"
      : "https://mantlescan.xyz";

  return (
    <div className="space-y-12">
      {/* Hero */}
      <section>
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          Private Execution Layer
        </span>
        <h1 className="mt-3 text-3xl md:text-4xl font-medium tracking-tight">
          Dashboard
        </h1>
        <p className="mt-4 max-w-md font-mono text-sm text-muted-foreground leading-relaxed">
          Manage your private balance and execute confidential transactions on Mantle.
        </p>
      </section>

      {/* Private Balance */}
      <section className="card-minimal p-6 md:p-8">
        <PrivateBalance />
      </section>

      {/* Quick Actions */}
      <section className="grid md:grid-cols-2 gap-4">
        <Link
          href="/deposit"
          className="group card-minimal p-6 flex items-center justify-between hover:border-accent/50 transition-colors"
        >
          <div>
            <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
              Quick Action
            </span>
            <h3 className="mt-2 text-lg font-medium group-hover:text-accent transition-colors">
              Deposit MNT
            </h3>
            <p className="mt-1 font-mono text-xs text-muted-foreground">
              Add funds to the private layer
            </p>
          </div>
          <ArrowDownLeft className="w-5 h-5 text-muted-foreground group-hover:text-accent transition-colors" />
        </Link>

        <Link
          href="/withdraw"
          className="group card-minimal p-6 flex items-center justify-between hover:border-accent/50 transition-colors"
        >
          <div>
            <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
              Quick Action
            </span>
            <h3 className="mt-2 text-lg font-medium group-hover:text-accent transition-colors">
              Withdraw MNT
            </h3>
            <p className="mt-1 font-mono text-xs text-muted-foreground">
              Exit to any address privately
            </p>
          </div>
          <ArrowUpRight className="w-5 h-5 text-muted-foreground group-hover:text-accent transition-colors" />
        </Link>
      </section>

      {/* Vault Stats */}
      <section>
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
          Protocol Stats
        </span>
        <div className="mt-4">
          <VaultStats />
        </div>
      </section>

      {/* How It Works */}
      <section className="pt-8 border-t border-border/30">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          How It Works
        </span>
        <div className="mt-6 grid md:grid-cols-3 gap-8">
          <div>
            <span className="font-mono text-[10px] text-muted-foreground/60 uppercase tracking-widest">
              01
            </span>
            <h4 className="mt-2 font-medium">Deposit</h4>
            <div className="mt-2 w-6 h-px bg-accent/60" />
            <p className="mt-3 font-mono text-xs text-muted-foreground leading-relaxed">
              Deposit MNT with a cryptographic commitment. Your secret key proves ownership.
            </p>
          </div>
          <div>
            <span className="font-mono text-[10px] text-muted-foreground/60 uppercase tracking-widest">
              02
            </span>
            <h4 className="mt-2 font-medium">Transfer</h4>
            <div className="mt-2 w-6 h-px bg-accent/60" />
            <p className="mt-3 font-mono text-xs text-muted-foreground leading-relaxed">
              Transfer privately between accounts using the CLI without revealing amounts.
            </p>
          </div>
          <div>
            <span className="font-mono text-[10px] text-muted-foreground/60 uppercase tracking-widest">
              03
            </span>
            <h4 className="mt-2 font-medium">Withdraw</h4>
            <div className="mt-2 w-6 h-px bg-accent/60" />
            <p className="mt-3 font-mono text-xs text-muted-foreground leading-relaxed">
              Generate a ZK proof to withdraw to any address without revealing identity.
            </p>
          </div>
        </div>
      </section>

      {/* Contract Info */}
      {vaultAddress && vaultAddress !== "0x0000000000000000000000000000000000000000" && (
        <section className="pt-8 border-t border-border/30">
          <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
            Contract
          </span>
          <div className="mt-3 flex items-center gap-3">
            <code className="font-mono text-xs text-muted-foreground">{vaultAddress}</code>
            <a
              href={`${explorerUrl}/address/${vaultAddress}`}
              target="_blank"
              rel="noopener noreferrer"
              className="text-accent hover:opacity-70 transition-opacity"
            >
              <ExternalLink className="w-3 h-3" />
            </a>
          </div>
          <p className="mt-1 font-mono text-[10px] text-muted-foreground/60 uppercase tracking-widest">
            {chainId === 5003 ? "Mantle Sepolia" : "Mantle Mainnet"}
          </p>
        </section>
      )}
    </div>
  );
}
