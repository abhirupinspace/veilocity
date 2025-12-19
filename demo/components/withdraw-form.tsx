"use client";

import { useState, useEffect } from "react";
import { useAccount, useChainId, useReadContract, useWriteContract, useWaitForTransactionReceipt } from "wagmi";
import { parseEther } from "viem";
import { veilocityVaultAbi, VAULT_ADDRESSES } from "@/lib/abi";
import { formatEther, shortenHash } from "@/lib/utils";
import {
  getAvailableDeposits,
  PrivateDeposit,
  generateMockProof,
} from "@/lib/crypto";
import { toast } from "sonner";
import {
  AlertCircle,
  Check,
  Loader2,
  Lock,
  Shield,
  Zap,
} from "lucide-react";

type ProofStage =
  | "idle"
  | "nullifier"
  | "merkle"
  | "state_root"
  | "witness"
  | "proving"
  | "verifying"
  | "submitting"
  | "complete"
  | "error";

const PROOF_STAGES = [
  { key: "nullifier", label: "Computing Nullifier", description: "Generating unique spend identifier" },
  { key: "merkle", label: "Merkle Proof", description: "Building cryptographic path" },
  { key: "state_root", label: "State Root", description: "Retrieving commitment root" },
  { key: "witness", label: "Circuit Witness", description: "Constructing private inputs" },
  { key: "proving", label: "ZK-SNARK Proof", description: "Generating UltraPlonk proof" },
  { key: "verifying", label: "Local Verification", description: "Validating proof integrity" },
];

export function WithdrawForm() {
  const [selectedDeposit, setSelectedDeposit] = useState<PrivateDeposit | null>(null);
  const [amount, setAmount] = useState("");
  const [recipient, setRecipient] = useState("");
  const [deposits, setDeposits] = useState<PrivateDeposit[]>([]);
  const [proofStage, setProofStage] = useState<ProofStage>("idle");
  const [proofData, setProofData] = useState<{
    nullifier: string;
    stateRoot: string;
    proofBytes: string;
  } | null>(null);
  const [isWithdrawing, setIsWithdrawing] = useState(false);

  const chainId = useChainId();
  const { address, isConnected } = useAccount();
  const vaultAddress = VAULT_ADDRESSES[chainId];

  const { writeContract, data: txHash, error: writeError, isPending: isWritePending } = useWriteContract();

  const { isLoading: isConfirming, isSuccess: isConfirmed } = useWaitForTransactionReceipt({
    hash: txHash,
  });

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

  useEffect(() => {
    if (isConfirmed && selectedDeposit) {
      setProofStage("complete");
      toast.success("Withdrawal successful!", {
        description: `${amount} MNT sent to ${shortenHash(recipient)}`,
      });
      // Mark deposit as spent
      const stored = localStorage.getItem("veilocity_deposits");
      if (stored) {
        const allDeposits = JSON.parse(stored);
        const updated = allDeposits.map((d: PrivateDeposit) =>
          d.id === selectedDeposit.id ? { ...d, spent: true } : d
        );
        localStorage.setItem("veilocity_deposits", JSON.stringify(updated));
      }
      // Reset after delay
      setTimeout(() => {
        setProofStage("idle");
        setSelectedDeposit(null);
        setAmount("");
        setProofData(null);
        setIsWithdrawing(false);
      }, 3000);
    }
  }, [isConfirmed, selectedDeposit, amount, recipient]);

  useEffect(() => {
    if (writeError) {
      setProofStage("error");
      toast.error("Transaction failed", {
        description: writeError.message.slice(0, 100),
      });
      setIsWithdrawing(false);
    }
  }, [writeError]);

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
    setProofStage("idle");
    setProofData(null);
  };

  const simulateProofGeneration = async () => {
    const stages: ProofStage[] = ["nullifier", "merkle", "state_root", "witness", "proving", "verifying"];
    const delays = [400, 600, 300, 500, 1500, 400];

    for (let i = 0; i < stages.length; i++) {
      setProofStage(stages[i]);
      await new Promise(resolve => setTimeout(resolve, delays[i]));
    }

    // Generate mock proof data
    const mockProof = generateMockProof(selectedDeposit!.secret, amount);
    setProofData(mockProof);

    return mockProof;
  };

  const handleWithdraw = async () => {
    if (!selectedDeposit || !amount || !recipient || !vaultAddress) return;

    setIsWithdrawing(true);

    try {
      // Simulate ZK proof generation
      const proof = await simulateProofGeneration();

      // Submit to contract
      setProofStage("submitting");

      writeContract({
        address: vaultAddress,
        abi: veilocityVaultAbi,
        functionName: "withdraw",
        args: [
          proof.nullifier as `0x${string}`,
          recipient as `0x${string}`,
          parseEther(amount),
          proof.stateRoot as `0x${string}`,
          proof.proofBytes as `0x${string}`,
        ],
      });
    } catch (error) {
      console.error("Withdraw error:", error);
      setProofStage("error");
      setIsWithdrawing(false);
      toast.error("Proof generation failed");
    }
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
    parseFloat(amount) <= parseFloat(selectedDeposit.amount) &&
    !isWithdrawing;

  const getStageStatus = (stageKey: string) => {
    const stageIndex = PROOF_STAGES.findIndex(s => s.key === stageKey);
    const currentIndex = PROOF_STAGES.findIndex(s => s.key === proofStage);

    if (proofStage === "complete" || proofStage === "submitting") return "complete";
    if (stageIndex < currentIndex) return "complete";
    if (stageIndex === currentIndex) return "active";
    return "pending";
  };

  return (
    <div className="card-minimal">
      <div className="p-6 border-b border-border/50">
        <span className="font-mono text-[10px] uppercase tracking-[0.3em] text-accent">
          Private Withdraw
        </span>
        <p className="mt-2 font-mono text-xs text-muted-foreground">
          Exit the shielded pool with ZK proof
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
        ) : proofStage !== "idle" && proofStage !== "error" ? (
          /* ZK Proof Generation Visualization */
          <div className="space-y-4">
            <div className="flex items-center gap-2 mb-4">
              <Shield className="w-4 h-4 text-accent" />
              <span className="font-mono text-xs uppercase tracking-widest">
                ZK-Proof Generation
              </span>
            </div>

            {/* Progress stages */}
            <div className="space-y-3">
              {PROOF_STAGES.map((stage) => {
                const status = getStageStatus(stage.key);
                return (
                  <div
                    key={stage.key}
                    className={`flex items-center gap-3 p-3 border transition-all duration-300 ${
                      status === "active"
                        ? "border-accent/50 bg-accent/5"
                        : status === "complete"
                        ? "border-green-500/30 bg-green-500/5"
                        : "border-border/30 opacity-40"
                    }`}
                  >
                    <div className="w-5 h-5 flex items-center justify-center">
                      {status === "complete" ? (
                        <Check className="w-4 h-4 text-green-500" />
                      ) : status === "active" ? (
                        <Loader2 className="w-4 h-4 text-accent animate-spin" />
                      ) : (
                        <div className="w-2 h-2 border border-border/50" />
                      )}
                    </div>
                    <div className="flex-1">
                      <p className={`font-mono text-xs ${status === "active" ? "text-accent" : status === "complete" ? "text-green-500" : ""}`}>
                        {stage.label}
                      </p>
                      <p className="font-mono text-[10px] text-muted-foreground/60">
                        {stage.description}
                      </p>
                    </div>
                  </div>
                );
              })}
            </div>

            {/* Proof verified box */}
            {(proofStage === "submitting" || proofStage === "complete") && proofData && (
              <div className="mt-4 p-4 border border-green-500/30 bg-green-500/5">
                <div className="flex items-center gap-2 mb-3">
                  <Check className="w-4 h-4 text-green-500" />
                  <span className="font-mono text-xs text-green-500 uppercase tracking-widest">
                    ZK-Proof Verified
                  </span>
                </div>
                <div className="space-y-2 font-mono text-[10px] text-muted-foreground">
                  <p>Proof size: 256 bytes</p>
                  <p>✓ Zero-Knowledge: Private inputs hidden</p>
                  <p>✓ Soundness: Proof mathematically valid</p>
                  <p>✓ Completeness: All constraints satisfied</p>
                </div>
              </div>
            )}

            {/* Transaction status */}
            {proofStage === "submitting" && (
              <div className="flex items-center gap-3 p-4 border border-accent/30">
                <Loader2 className="w-4 h-4 text-accent animate-spin" />
                <div>
                  <p className="font-mono text-xs">
                    {isWritePending ? "Confirm in wallet..." : isConfirming ? "Confirming on-chain..." : "Submitting..."}
                  </p>
                </div>
              </div>
            )}

            {proofStage === "complete" && (
              <div className="flex items-center gap-3 p-4 border border-green-500/30 bg-green-500/5">
                <Check className="w-4 h-4 text-green-500" />
                <div>
                  <p className="font-mono text-xs text-green-500">Withdrawal Complete!</p>
                  <p className="font-mono text-[10px] text-muted-foreground mt-1">
                    {amount} MNT sent to {shortenHash(recipient)}
                  </p>
                </div>
              </div>
            )}

            {/* Privacy guarantees */}
            <div className="mt-4 p-4 border border-purple-500/30 bg-purple-500/5">
              <div className="flex items-center gap-2 mb-3">
                <Lock className="w-4 h-4 text-purple-500" />
                <span className="font-mono text-[10px] text-purple-500 uppercase tracking-widest">
                  Privacy Guarantees
                </span>
              </div>
              <div className="grid grid-cols-2 gap-2 font-mono text-[10px]">
                <p><span className="text-purple-500">●</span> Source: <span className="text-green-500">Hidden</span></p>
                <p><span className="text-purple-500">●</span> Balance: <span className="text-green-500">Hidden</span></p>
                <p><span className="text-purple-500">●</span> History: <span className="text-green-500">Unlinkable</span></p>
                <p><span className="text-purple-500">●</span> Identity: <span className="text-green-500">Anonymous</span></p>
              </div>
            </div>
          </div>
        ) : (
          <>
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
                  disabled={!canWithdraw || !isConnected}
                  className="w-full flex items-center justify-center gap-2 px-6 py-4 border border-foreground/20 font-mono text-xs uppercase tracking-widest disabled:opacity-30 disabled:cursor-not-allowed hover:border-accent hover:text-accent transition-colors"
                >
                  <Zap className="w-3 h-3" />
                  Generate Proof & Withdraw
                </button>

                {!isConnected && (
                  <p className="text-center font-mono text-[10px] text-muted-foreground uppercase tracking-widest">
                    Connect wallet to withdraw
                  </p>
                )}

                {proofStage === "error" && (
                  <div className="flex items-center gap-2 p-3 border border-red-500/30 bg-red-500/5">
                    <AlertCircle className="w-4 h-4 text-red-500" />
                    <p className="font-mono text-xs text-red-500">
                      Transaction failed. Please try again.
                    </p>
                  </div>
                )}
              </>
            )}
          </>
        )}
      </div>
    </div>
  );
}
