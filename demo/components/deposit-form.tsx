"use client";

import { useState, useEffect } from "react";
import { useAccount, useWriteContract, useWaitForTransactionReceipt, useChainId, useBalance } from "wagmi";
import { parseEther, keccak256, encodePacked, toHex } from "viem";
import { veilocityVaultAbi, VAULT_ADDRESSES } from "@/lib/abi";
import { formatEther } from "@/lib/utils";
import { toast } from "sonner";
import { ArrowRight, Loader2, Shield, AlertCircle, CheckCircle } from "lucide-react";

export function DepositForm() {
  const [amount, setAmount] = useState("");
  const [secret, setSecret] = useState("");
  const chainId = useChainId();
  const { address, isConnected } = useAccount();
  const { data: balance } = useBalance({ address });
  const vaultAddress = VAULT_ADDRESSES[chainId];

  // Generate a random secret on mount
  useEffect(() => {
    const randomBytes = new Uint8Array(32);
    crypto.getRandomValues(randomBytes);
    const hexSecret = Array.from(randomBytes)
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    setSecret(`0x${hexSecret}`);
  }, []);

  const { writeContract, data: txHash, isPending, error: writeError } = useWriteContract();

  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({
    hash: txHash,
  });

  useEffect(() => {
    if (isSuccess && txHash) {
      toast.success("Deposit successful!", {
        description: `Transaction: ${txHash.slice(0, 10)}...`,
      });
      setAmount("");
    }
  }, [isSuccess, txHash]);

  useEffect(() => {
    if (writeError) {
      toast.error("Deposit failed", {
        description: writeError.message.slice(0, 100),
      });
    }
  }, [writeError]);

  const handleDeposit = async () => {
    if (!amount || !secret || !vaultAddress) return;

    try {
      const amountWei = parseEther(amount);

      // Generate commitment: keccak256(secret, amount)
      // In production, this would use Poseidon hash
      const commitment = keccak256(
        encodePacked(["bytes32", "uint256"], [secret as `0x${string}`, amountWei])
      );

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

  const noContract = !vaultAddress || vaultAddress === "0x0000000000000000000000000000000000000000";
  const minDeposit = 0.001;
  const insufficientBalance = balance && amount ? parseEther(amount) > balance.value : false;
  const belowMinimum = amount ? parseFloat(amount) < minDeposit : false;

  return (
    <div className="bg-card border border-border">
      <div className="p-6 border-b border-border">
        <div className="flex items-center gap-2">
          <Shield className="w-5 h-5 text-accent" />
          <h2 className="text-lg font-medium">Private Deposit</h2>
        </div>
        <p className="text-sm text-muted-foreground mt-1 font-mono">
          Deposit MNT into the private execution layer
        </p>
      </div>

      <div className="p-6 space-y-6">
        {noContract ? (
          <div className="flex items-start gap-3 p-4 bg-secondary/50 border border-border">
            <AlertCircle className="w-5 h-5 text-accent shrink-0 mt-0.5" />
            <div>
              <p className="text-sm font-medium">Contract Not Configured</p>
              <p className="text-xs text-muted-foreground font-mono mt-1">
                Set the vault address in lib/abi.ts to enable deposits.
              </p>
            </div>
          </div>
        ) : (
          <>
            {/* Amount Input */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground font-mono uppercase tracking-wider">
                Amount (MNT)
              </label>
              <div className="relative">
                <input
                  type="number"
                  step="0.001"
                  min="0.001"
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  placeholder="0.001"
                  className="w-full bg-input border border-border px-4 py-3 font-mono text-lg focus:outline-none focus:border-accent transition-colors"
                  disabled={!isConnected || isPending || isConfirming}
                />
                {balance && (
                  <button
                    onClick={() => setAmount(formatEther(balance.value))}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-accent font-mono hover:underline"
                    disabled={!isConnected}
                  >
                    MAX
                  </button>
                )}
              </div>
              {balance && (
                <p className="text-xs text-muted-foreground font-mono">
                  Balance: {formatEther(balance.value)} MNT
                </p>
              )}
              {belowMinimum && (
                <p className="text-xs text-red-500 font-mono">
                  Minimum deposit: 0.001 MNT
                </p>
              )}
              {insufficientBalance && (
                <p className="text-xs text-red-500 font-mono">
                  Insufficient balance
                </p>
              )}
            </div>

            {/* Secret Display */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground font-mono uppercase tracking-wider">
                Secret (Save This!)
              </label>
              <div className="bg-secondary/50 border border-border p-3 font-mono text-xs break-all select-all">
                {secret}
              </div>
              <p className="text-xs text-muted-foreground font-mono flex items-center gap-1">
                <AlertCircle className="w-3 h-3" />
                Save this secret to withdraw your funds later
              </p>
            </div>

            {/* Transaction Status */}
            {txHash && (
              <div className="flex items-center gap-2 p-3 bg-secondary/50 border border-border">
                {isConfirming ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin text-accent" />
                    <span className="text-sm font-mono">Confirming transaction...</span>
                  </>
                ) : isSuccess ? (
                  <>
                    <CheckCircle className="w-4 h-4 text-green-500" />
                    <span className="text-sm font-mono">Deposit confirmed!</span>
                  </>
                ) : null}
              </div>
            )}

            {/* Deposit Button */}
            <button
              onClick={handleDeposit}
              disabled={
                !isConnected ||
                !amount ||
                isPending ||
                isConfirming ||
                belowMinimum ||
                insufficientBalance
              }
              className="w-full flex items-center justify-center gap-2 px-6 py-4 bg-accent text-accent-foreground font-mono text-sm disabled:opacity-50 disabled:cursor-not-allowed hover:opacity-90 transition-opacity"
            >
              {isPending || isConfirming ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  {isPending ? "Awaiting approval..." : "Confirming..."}
                </>
              ) : (
                <>
                  Deposit
                  <ArrowRight className="w-4 h-4" />
                </>
              )}
            </button>

            {!isConnected && (
              <p className="text-center text-sm text-muted-foreground font-mono">
                Connect wallet to deposit
              </p>
            )}
          </>
        )}
      </div>
    </div>
  );
}
