"use client";

import { useState, useEffect } from "react";
import {
  useAccount,
  useWriteContract,
  useWaitForTransactionReceipt,
  useChainId,
  useBalance,
} from "wagmi";
import { parseEther } from "viem";
import { veilocityVaultAbi, VAULT_ADDRESSES } from "@/lib/abi";
import { formatEther } from "@/lib/utils";
import {
  generateSecret,
  computeCommitment,
  addDeposit,
} from "@/lib/crypto";
import { toast } from "sonner";
import {
  ArrowRight,
  Loader2,
  AlertCircle,
  Copy,
  Download,
} from "lucide-react";

export function DepositForm() {
  const [amount, setAmount] = useState("");
  const [secret, setSecret] = useState<`0x${string}` | "">("");
  const [commitment, setCommitment] = useState<`0x${string}` | "">("");
  const [showSecret, setShowSecret] = useState(false);

  const chainId = useChainId();
  const { address, isConnected } = useAccount();
  const { data: balance } = useBalance({ address });
  const vaultAddress = VAULT_ADDRESSES[chainId];

  useEffect(() => {
    const newSecret = generateSecret();
    setSecret(newSecret);
  }, []);

  useEffect(() => {
    if (secret && amount && parseFloat(amount) > 0) {
      try {
        const amountWei = parseEther(amount);
        const newCommitment = computeCommitment(secret, amountWei);
        setCommitment(newCommitment);
      } catch {
        setCommitment("");
      }
    } else {
      setCommitment("");
    }
  }, [secret, amount]);

  const {
    writeContract,
    data: txHash,
    isPending,
    error: writeError,
  } = useWriteContract();

  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({
    hash: txHash,
  });

  useEffect(() => {
    if (isSuccess && txHash && secret && amount) {
      const amountWei = parseEther(amount);

      addDeposit({
        secret,
        commitment: commitment as `0x${string}`,
        amount,
        amountWei: amountWei.toString(),
        leafIndex: -1,
        timestamp: Date.now(),
        transactionHash: txHash,
        status: "confirmed",
        nonce: 0,
      });

      toast.success("Deposit confirmed");

      const newSecret = generateSecret();
      setSecret(newSecret);
      setAmount("");
    }
  }, [isSuccess, txHash, secret, amount, commitment]);

  useEffect(() => {
    if (writeError) {
      toast.error("Transaction failed");
    }
  }, [writeError]);

  const handleDeposit = async () => {
    if (!amount || !secret || !vaultAddress || !commitment) return;

    try {
      const amountWei = parseEther(amount);

      writeContract({
        address: vaultAddress,
        abi: veilocityVaultAbi,
        functionName: "deposit",
        args: [commitment],
        value: amountWei,
      });
    } catch (err) {
      console.error("Deposit error:", err);
      toast.error("Failed to create deposit");
    }
  };

  const copySecret = async () => {
    if (!secret) return;
    await navigator.clipboard.writeText(secret);
    toast.success("Copied");
  };

  const downloadBackup = () => {
    if (!secret || !amount || !commitment) return;

    const backup = {
      secret,
      commitment,
      amount,
      amountWei: parseEther(amount).toString(),
      timestamp: new Date().toISOString(),
      network: chainId === 5003 ? "mantle-sepolia" : "mantle",
    };

    const blob = new Blob([JSON.stringify(backup, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `veilocity-backup-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
    toast.success("Backup saved");
  };

  const noContract =
    !vaultAddress ||
    vaultAddress === "0x0000000000000000000000000000000000000000";
  const minDeposit = 0.001;
  const insufficientBalance =
    balance && amount ? parseEther(amount) > balance.value : false;
  const belowMinimum = amount ? parseFloat(amount) < minDeposit : false;

  return (
    <div className="card-minimal">
      <div className="p-6 border-b border-border/50">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          Deposit
        </span>
        <p className="mt-2 font-mono text-xs text-muted-foreground">
          Add MNT to the private layer
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
            {/* Amount Input */}
            <div className="space-y-2">
              <label className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
                Amount
              </label>
              <div className="relative">
                <input
                  type="number"
                  step="0.001"
                  min="0.001"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0.00"
                  className="w-full bg-input border border-border/50 px-4 py-3 font-mono text-lg focus:outline-none focus:border-accent/50 transition-colors"
                  disabled={!isConnected || isPending || isConfirming}
                />
                <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-3">
                  <span className="font-mono text-xs text-muted-foreground">MNT</span>
                  {balance && (
                    <button
                      onClick={() => setAmount(formatEther(balance.value))}
                      className="font-mono text-[10px] text-accent hover:opacity-70 transition-opacity uppercase tracking-widest"
                      disabled={!isConnected}
                    >
                      Max
                    </button>
                  )}
                </div>
              </div>
              {balance && (
                <p className="font-mono text-[10px] text-muted-foreground/60">
                  Balance: {formatEther(balance.value)} MNT
                </p>
              )}
              {belowMinimum && (
                <p className="font-mono text-[10px] text-red-400">
                  Min: 0.001 MNT
                </p>
              )}
              {insufficientBalance && (
                <p className="font-mono text-[10px] text-red-400">
                  Insufficient balance
                </p>
              )}
            </div>

            {/* Secret Display */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <label className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
                  Secret Key
                </label>
                <div className="flex items-center gap-3">
                  <button
                    onClick={() => setShowSecret(!showSecret)}
                    className="font-mono text-[10px] text-muted-foreground hover:text-foreground transition-colors uppercase tracking-widest"
                  >
                    {showSecret ? "Hide" : "Show"}
                  </button>
                  <button
                    onClick={copySecret}
                    className="text-muted-foreground hover:text-foreground transition-colors"
                    title="Copy"
                  >
                    <Copy className="w-3 h-3" />
                  </button>
                  <button
                    onClick={downloadBackup}
                    className="text-muted-foreground hover:text-foreground transition-colors"
                    title="Download"
                    disabled={!amount || parseFloat(amount) <= 0}
                  >
                    <Download className="w-3 h-3" />
                  </button>
                </div>
              </div>
              <div className="bg-secondary/30 border border-border/30 p-3 font-mono text-[10px] break-all select-all text-muted-foreground">
                {showSecret ? secret : "•".repeat(66)}
              </div>
              <p className="font-mono text-[10px] text-muted-foreground/60 flex items-center gap-1">
                <AlertCircle className="w-3 h-3 text-accent" />
                Save this to withdraw later
              </p>
            </div>

            {/* Commitment */}
            {commitment && (
              <div className="space-y-2">
                <label className="font-mono text-[10px] uppercase tracking-[0.3em] text-muted-foreground">
                  Commitment
                </label>
                <div className="bg-secondary/20 border border-border/20 p-3 font-mono text-[10px] break-all text-muted-foreground/60">
                  {commitment}
                </div>
              </div>
            )}

            {/* Transaction Status */}
            {txHash && (
              <div className="flex items-center gap-2 py-3 border-t border-border/30">
                {isConfirming ? (
                  <>
                    <Loader2 className="w-3 h-3 animate-spin text-accent" />
                    <span className="font-mono text-xs text-muted-foreground">
                      Confirming...
                    </span>
                  </>
                ) : isSuccess ? (
                  <span className="font-mono text-xs text-green-400">
                    Confirmed
                  </span>
                ) : null}
              </div>
            )}

            {/* Deposit Button */}
            <button
              onClick={handleDeposit}
              disabled={
                !isConnected ||
                !amount ||
                !commitment ||
                isPending ||
                isConfirming ||
                belowMinimum ||
                insufficientBalance
              }
              className="w-full flex items-center justify-center gap-2 px-6 py-4 bg-foreground text-background font-mono text-xs uppercase tracking-widest disabled:opacity-30 disabled:cursor-not-allowed hover:opacity-90 transition-opacity"
            >
              {isPending || isConfirming ? (
                <>
                  <Loader2 className="w-3 h-3 animate-spin" />
                  {isPending ? "Awaiting approval" : "Confirming"}
                </>
              ) : (
                <>
                  Deposit
                  <ArrowRight className="w-3 h-3" />
                </>
              )}
            </button>

            {!isConnected && (
              <p className="text-center font-mono text-[10px] text-muted-foreground uppercase tracking-widest">
                Connect wallet to deposit
              </p>
            )}
          </>
        )}
      </div>
    </div>
  );
}
