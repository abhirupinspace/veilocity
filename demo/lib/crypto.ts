import { keccak256, encodePacked, toHex, parseEther, formatEther } from "viem";

// BN254 field prime (for future Poseidon compatibility)
export const BN254_FIELD_PRIME = BigInt(
  "21888242871839275222246405745257275088548364400416034343698204186575808495617"
);

// Generate a random 32-byte secret
export function generateSecret(): `0x${string}` {
  const randomBytes = new Uint8Array(32);
  crypto.getRandomValues(randomBytes);
  const hexSecret = Array.from(randomBytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  return `0x${hexSecret}`;
}

// Compute deposit commitment
// In production, this uses Poseidon hash: commitment = poseidon(secret, amount)
// For demo, we use keccak256 which the contract accepts
export function computeCommitment(
  secret: `0x${string}`,
  amountWei: bigint
): `0x${string}` {
  return keccak256(
    encodePacked(["bytes32", "uint256"], [secret, amountWei])
  );
}

// Derive public key from secret (simulated for demo)
// In production: pubkey = poseidon(secret)
export function derivePubkey(secret: `0x${string}`): `0x${string}` {
  return keccak256(encodePacked(["bytes32", "string"], [secret, "pubkey"]));
}

// Compute nullifier for spending
// In production: nullifier = poseidon(secret, leafIndex, nonce)
export function computeNullifier(
  secret: `0x${string}`,
  leafIndex: bigint,
  nonce: bigint
): `0x${string}` {
  return keccak256(
    encodePacked(
      ["bytes32", "uint256", "uint256"],
      [secret, leafIndex, nonce]
    )
  );
}

// Compute leaf hash for Merkle tree
// In production: leaf = poseidon(pubkey, balance, nonce)
export function computeLeaf(
  pubkey: `0x${string}`,
  balance: bigint,
  nonce: bigint
): `0x${string}` {
  return keccak256(
    encodePacked(["bytes32", "uint256", "uint256"], [pubkey, balance, nonce])
  );
}

// Types for private state management
export interface PrivateDeposit {
  id: string;
  secret: `0x${string}`;
  commitment: `0x${string}`;
  amount: string; // in ETH
  amountWei: string;
  leafIndex: number;
  timestamp: number;
  transactionHash: string;
  status: "pending" | "confirmed" | "spent";
  nonce: number;
}

export interface PrivateState {
  deposits: PrivateDeposit[];
  totalBalance: string;
  lastSyncBlock: number;
}

const STORAGE_KEY = "veilocity_private_state";

// Get private state from localStorage
export function getPrivateState(): PrivateState {
  if (typeof window === "undefined") {
    return { deposits: [], totalBalance: "0", lastSyncBlock: 0 };
  }

  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) {
    return { deposits: [], totalBalance: "0", lastSyncBlock: 0 };
  }

  try {
    return JSON.parse(stored);
  } catch {
    return { deposits: [], totalBalance: "0", lastSyncBlock: 0 };
  }
}

// Save private state to localStorage
export function savePrivateState(state: PrivateState): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

// Add a new deposit to private state
export function addDeposit(deposit: Omit<PrivateDeposit, "id">): PrivateDeposit {
  const state = getPrivateState();
  const newDeposit: PrivateDeposit = {
    ...deposit,
    id: `${deposit.transactionHash}-${Date.now()}`,
  };

  state.deposits.push(newDeposit);
  state.totalBalance = calculateTotalBalance(state.deposits);
  savePrivateState(state);

  return newDeposit;
}

// Update deposit status
export function updateDepositStatus(
  transactionHash: string,
  status: PrivateDeposit["status"],
  leafIndex?: number
): void {
  const state = getPrivateState();
  const deposit = state.deposits.find(
    (d) => d.transactionHash === transactionHash
  );

  if (deposit) {
    deposit.status = status;
    if (leafIndex !== undefined) {
      deposit.leafIndex = leafIndex;
    }
    state.totalBalance = calculateTotalBalance(state.deposits);
    savePrivateState(state);
  }
}

// Mark a deposit as spent (for withdrawals)
export function markDepositSpent(depositId: string): void {
  const state = getPrivateState();
  const deposit = state.deposits.find((d) => d.id === depositId);

  if (deposit) {
    deposit.status = "spent";
    state.totalBalance = calculateTotalBalance(state.deposits);
    savePrivateState(state);
  }
}

// Calculate total unspent balance
function calculateTotalBalance(deposits: PrivateDeposit[]): string {
  const totalWei = deposits
    .filter((d) => d.status === "confirmed")
    .reduce((sum, d) => sum + BigInt(d.amountWei), BigInt(0));

  return formatEther(totalWei);
}

// Get available deposits for withdrawal
export function getAvailableDeposits(): PrivateDeposit[] {
  const state = getPrivateState();
  return state.deposits.filter((d) => d.status === "confirmed");
}

// Get total private balance
export function getPrivateBalance(): string {
  const state = getPrivateState();
  return state.totalBalance;
}

// Clear all private state (for testing/reset)
export function clearPrivateState(): void {
  if (typeof window === "undefined") return;
  localStorage.removeItem(STORAGE_KEY);
}

// Export secret as backup string
export function exportSecretBackup(deposit: PrivateDeposit): string {
  return JSON.stringify({
    secret: deposit.secret,
    amount: deposit.amount,
    leafIndex: deposit.leafIndex,
    commitment: deposit.commitment,
  });
}

// Import secret from backup
export function importSecretBackup(backup: string): Partial<PrivateDeposit> | null {
  try {
    const parsed = JSON.parse(backup);
    if (!parsed.secret || !parsed.amount) return null;
    return parsed;
  } catch {
    return null;
  }
}
