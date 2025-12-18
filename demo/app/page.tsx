"use client";

import { ConnectButton } from "@/components/connect-button";
import { VaultStats } from "@/components/vault-stats";
import { DepositForm } from "@/components/deposit-form";
import { RecentDeposits } from "@/components/recent-deposits";
import { NetworkStatus } from "@/components/network-status";
import { useChainId } from "wagmi";
import { Shield, ExternalLink, Github } from "lucide-react";
import { VAULT_ADDRESSES } from "@/lib/abi";

export default function Home() {
  const chainId = useChainId();
  const vaultAddress = VAULT_ADDRESSES[chainId];
  const explorerUrl =
    chainId === 5003
      ? "https://sepolia.mantlescan.xyz"
      : "https://mantlescan.xyz";

  return (
    <div className="min-h-screen grid-bg">
      {/* Header */}
      <header className="border-b border-border sticky top-0 z-40 bg-background/80 backdrop-blur-sm">
        <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Shield className="w-8 h-8 text-accent" />
            <div>
              <h1 className="text-xl font-bold tracking-tight">Veilocity</h1>
              <p className="text-xs text-muted-foreground font-mono">
                Private Execution Layer
              </p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <a
              href="https://github.com/veilocity"
              target="_blank"
              rel="noopener noreferrer"
              className="text-muted-foreground hover:text-foreground transition-colors"
            >
              <Github className="w-5 h-5" />
            </a>
            <ConnectButton />
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-6 py-8 space-y-8">
        {/* Network Warning */}
        <NetworkStatus />

        {/* Hero */}
        <section className="py-8">
          <h2 className="text-4xl md:text-5xl font-bold tracking-tight mb-4">
            Private transactions
            <br />
            <span className="text-accent">on Mantle</span>
          </h2>
          <p className="text-lg text-muted-foreground max-w-2xl font-mono">
            Execute confidential transactions with zero-knowledge proofs.
            Deposit MNT, transfer privately, and withdraw with cryptographic
            guarantees.
          </p>
        </section>

        {/* Stats */}
        <section>
          <VaultStats />
        </section>

        {/* Main Content */}
        <section className="grid lg:grid-cols-2 gap-8">
          <DepositForm />
          <RecentDeposits />
        </section>

        {/* Contract Info */}
        {vaultAddress && vaultAddress !== "0x0000000000000000000000000000000000000000" && (
          <section className="bg-card border border-border p-6">
            <h3 className="text-sm font-mono uppercase tracking-wider text-muted-foreground mb-4">
              Contract Information
            </h3>
            <div className="grid md:grid-cols-2 gap-4">
              <div>
                <p className="text-xs text-muted-foreground font-mono mb-1">
                  VeilocityVault Address
                </p>
                <div className="flex items-center gap-2">
                  <code className="text-sm font-mono break-all">{vaultAddress}</code>
                  <a
                    href={`${explorerUrl}/address/${vaultAddress}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-accent hover:opacity-80 transition-opacity shrink-0"
                  >
                    <ExternalLink className="w-4 h-4" />
                  </a>
                </div>
              </div>
              <div>
                <p className="text-xs text-muted-foreground font-mono mb-1">
                  Network
                </p>
                <p className="text-sm font-mono">
                  {chainId === 5003 ? "Mantle Sepolia Testnet" : "Mantle Mainnet"}
                </p>
              </div>
            </div>
          </section>
        )}
      </main>

      {/* Footer */}
      <footer className="border-t border-border mt-16">
        <div className="max-w-7xl mx-auto px-6 py-8">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-2 text-sm text-muted-foreground font-mono">
              <Shield className="w-4 h-4" />
              Veilocity Demo
            </div>
            <p className="text-xs text-muted-foreground font-mono">
              Private execution layer on Mantle L2
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
}
