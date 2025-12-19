"use client";

import { useState, useEffect } from "react";
import { useAccount, useChainId, useReadContract } from "wagmi";
import { veilocityVaultAbi, VAULT_ADDRESSES } from "@/lib/abi";
import { formatEther, shortenHash } from "@/lib/utils";
import {
  getAvailableDeposits,
  PrivateDeposit,
} from "@/lib/crypto";
import { toast } from "sonner";
import {
  AlertCircle,
  Terminal,
} from "lucide-react";

export function WithdrawForm() {
  const [selectedDeposit, setSelectedDeposit] = useState<PrivateDeposit | null>(null);
  const [amount, setAmount] = useState("");
  const [recipient, setRecipient] = useState("");
  const [deposits, setDeposits] = useState<PrivateDeposit[]>([]);

  const chainId = useChainId();
  const { address, isConnected } = useAccount();
  const vaultAddress = VAULT_ADDRESSES[chainId];

  useEffect(() => {
    const loadDeposits = () => {
      const available = getAvailableDeposits();
      setDeposits(available);
    };
    loadDeposits();
    const interval = setInterval(loadDeposits, 5000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    if (address && !recipient) {
      setRecipient(address);
    }
  }, [address, recipient]);

  const { data: tvl } = useReadContract({
    address: vaultAddress,
    abi: veilocityVaultAbi,
    functionName: "totalValueLocked",
    query: {
      enabled:
        !!vaultAddress &&
        vaultAddress !== "0x0000000000000000000000000000000000000000",
    },
  });

  const handleSelectDeposit = (deposit: PrivateDeposit) => {
    setSelectedDeposit(deposit);
    setAmount(deposit.amount);
  };

  const handleWithdraw = () => {
    toast.info("Use CLI for withdrawals", {
      description: `veilocity withdraw ${amount}`,
    });
  };

  const noContract =
    !vaultAddress ||
    vaultAddress === "0x0000000000000000000000000000000000000000";
  const hasDeposits = deposits.length > 0;
  const canWithdraw =
    selectedDeposit &&
    amount &&
    recipient &&
    parseFloat(amount) > 0 &&
    parseFloat(amount) <= parseFloat(selectedDeposit.amount);

  return (
    <div className="card-minimal">
      <div className="p-6 border-b border-border/50">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          Withdraw
        </span>
        <p className="mt-2 font-mono text-xs text-muted-foreground">
          Remove MNT from the private layer
        </p>
      </div>

      <div className="p-6 space-y-6">
        {noContract ? (
          <div className="flex items-start gap-3 p-4 border border-border/50">
            <AlertCircle className="w-4 h-4 text-accent shrink-0 mt-0.5" />
            <div>
              <p className="text-sm font-medium">Contract Not Configured</p>
              <p className="text-xs text-muted-foreground font-mono mt-1">
                Set the vault address in lib/abi.ts
              </p>
            </div>
          </div>
        ) : (
          <>
            {/* CLI Notice */}
            <div className="flex items-start gap-3 p-4 border border-accent/30 bg-accent/5">
              <Terminal className="w-4 h-4 text-accent shrink-0 mt-0.5" />
              <div>
                <p className="font-mono text-xs">ZK Proof Required</p>
                <p className="font-mono text-[10px] text-muted-foreground mt-1">
                  Withdrawals require CLI for proof generation
                </p>
                <code className="block mt-2 font-mono text-[10px] text-muted-foreground/80">
                  veilocity withdraw &lt;amount&gt; --recipient &lt;address&gt;
                </code>
              </div>
            </div>

            {!hasDeposits ? (
              <div className="py-8 text-center">
                <p className="font-mono text-xs text-muted-foreground">
                  No deposits available
                </p>
                <p className="font-mono text-[10px] text-muted-foreground/60 mt-1">
                  Make a deposit first
                </p>
              </div>
            ) : (
              <>
                {/* Select Deposit */}
                <div className="space-y-2">
                  <label className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
                    Select Deposit
                  </label>
                  <div className="space-y-2 max-h-32 overflow-y-auto scrollbar-hide">
                    {deposits.map((deposit) => (
                      <button
                        key={deposit.id}
                        onClick={() => handleSelectDeposit(deposit)}
                        className={`w-full flex items-center justify-between p-3 border transition-colors ${
                          selectedDeposit?.id === deposit.id
                            ? "border-accent/50 bg-accent/5"
                            : "border-border/30 hover:border-border/50"
                        }`}
                      >
                        <div className="text-left">
                          <span className="font-mono text-sm">
                            {deposit.amount} MNT
                          </span>
                          <p className="font-mono text-[10px] text-muted-foreground/60">
                            {shortenHash(deposit.commitment)}
                          </p>
                        </div>
                        {selectedDeposit?.id === deposit.id && (
                          <div className="w-1.5 h-1.5 bg-accent" />
                        )}
                      </button>
                    ))}
                  </div>
                </div>

                {/* Amount */}
                <div className="space-y-2">
                  <label className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
                    Amount
                  </label>
                  <div className="relative">
                    <input
                      type="number"
                      step="0.001"
                      min="0.001"
                      max={selectedDeposit?.amount || "0"}
                      value={amount}
                      onChange={(e) => setAmount(e.target.value)}
                      placeholder="0.00"
                      className="w-full bg-input border border-border/50 px-4 py-3 font-mono text-lg focus:outline-none focus:border-accent/50 transition-colors disabled:opacity-30"
                      disabled={!selectedDeposit}
                    />
                    <span className="absolute right-4 top-1/2 -translate-y-1/2 font-mono text-xs text-muted-foreground">
                      MNT
                    </span>
                  </div>
                  {selectedDeposit && (
                    <p className="font-mono text-[10px] text-muted-foreground/60">
                      Max: {selectedDeposit.amount} MNT
                    </p>
                  )}
                </div>

                {/* Recipient */}
                <div className="space-y-2">
                  <label className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
                    Recipient
                  </label>
                  <input
                    type="text"
                    value={recipient}
                    onChange={(e) => setRecipient(e.target.value)}
                    placeholder="0x..."
                    className="w-full bg-input border border-border/50 px-4 py-3 font-mono text-xs focus:outline-none focus:border-accent/50 transition-colors"
                  />
                  {address && recipient !== address && (
                    <button
                      onClick={() => setRecipient(address)}
                      className="font-mono text-[10px] text-accent hover:opacity-70 transition-opacity uppercase tracking-widest"
                    >
                      Use connected wallet
                    </button>
                  )}
                </div>

                {/* TVL */}
                {tvl !== undefined && (
                  <p className="font-mono text-[10px] text-muted-foreground/60">
                    Vault TVL: {formatEther(tvl as bigint)} MNT
                  </p>
                )}

                {/* Withdraw Button */}
                <button
                  onClick={handleWithdraw}
                  disabled={!canWithdraw}
                  className="w-full flex items-center justify-center gap-2 px-6 py-4 border border-foreground/20 font-mono text-xs uppercase tracking-widest disabled:opacity-30 disabled:cursor-not-allowed hover:border-accent hover:text-accent transition-colors"
                >
                  <Terminal className="w-3 h-3" />
                  Generate via CLI
                </button>

                {!isConnected && (
                  <p className="text-center font-mono text-[10px] text-muted-foreground uppercase tracking-widest">
                    Connect wallet to withdraw
                  </p>
                )}
              </>
            )}
          </>
        )}
      </div>
    </div>
  );
}
