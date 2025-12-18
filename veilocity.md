#  **Veilocity — A Private Execution Layer on Mantle**

### *Confidential execution with provable settlement.*

---

## 1. The One-Sentence Definition (Use This Everywhere)

> **Veilocity is a private execution layer that allows transactions to run confidentially off-chain and settle with zero-knowledge proofs on Mantle.**

That’s it.
No “new chain”.
No “rollup hype”.
No confusion.

---

## 2. What a “Private Execution Layer” Means (Plain English)

Public blockchains execute transactions **in the open**.

Veilocity introduces a **separate execution environment** where:

* Transactions execute privately
* Balances and amounts are hidden
* Correctness is proven with ZK
* Final state is committed to Mantle

Mantle remains:

* the settlement layer
* the security anchor
* the source of truth

Veilocity only handles **execution**, not consensus or security.

This is **modular Ethereum done right**.

---

## 3. Why This Framing Is Better (Brutally Honest)

### “ZK-L3” framing problems:

* Sounds research-heavy
* Judges think “unfinished rollup”
* Raises questions about decentralization
* Competes mentally with zkSync / Aztec

###  “Private Execution Layer” framing benefits:

* Clear scope
* Clear value
* Easier to understand
* Matches Mantle’s modular narrative
* Positions you as **infrastructure**, not a chain

Judges instantly get:

> “Ah — this adds privacy without changing Mantle.”

---

## 4. Veilocity’s Role in the Mantle Stack

```
Applications
   ↓
Veilocity (Private Execution Layer)
   ↓
Mantle L2 (Settlement & Security)
   ↓
Ethereum
```

Veilocity:

* Executes sensitive logic privately
* Generates ZK proofs

Mantle:

* Verifies proofs
* Anchors state
* Handles assets

---

## 5. What Veilocity Does (Clear Responsibilities)

### Veilocity DOES:

* Execute private transactions
* Maintain confidential balances
* Generate ZK state transition proofs
* Batch or sequence execution (single-node for MVP)

### Veilocity DOES NOT:

* Replace Mantle
* Compete with L2s
* Provide consensus
* Expose transaction data

This clarity makes the project **credible**.

---

## 6. Core Use Case (Tell This Story)

### **Private OTC Settlement on Mantle**

Problem:

* Large trades cannot be public
* Amounts and pricing must stay private
* Settlement must be trustless

With Veilocity:

1. Parties deposit funds on Mantle
2. Trade executes privately in Veilocity
3. ZK proof validates correctness
4. Mantle settles final balances

Nothing sensitive is revealed.

This is a **real institutional problem**.

---

## 7. Technical Flow (Simplified for Judges)

1. User deposits funds on Mantle
2. Funds enter Veilocity private state
3. User submits private transaction
4. Veilocity executes privately
5. ZK proof generated
6. Proof + new state root sent to Mantle
7. Mantle verifies and accepts state

One flow. One diagram. No confusion.

---

## 8. Hackathon MVP (Aligned With This Framing)

You are **not** building:

* a chain
* a rollup
* a sequencer network

You are building:

* a private execution engine
* a ZK state transition circuit
* a Mantle settlement contract
* a real deposit → private transfer → withdraw flow

That’s perfect.

---

## 9. How to Talk About It in the Pitch

### ❌ Don’t say:

* “We built a new Layer 3”
* “We built a rollup”
* “We built a privacy chain”

### ✅ Say:

* “We built a private execution layer on Mantle”
* “Execution is private, settlement is public”
* “Mantle verifies correctness with ZK proofs”

This language keeps judges on your side.

---

## 10. Updated One-Sentence Pitch (Final)

> **Veilocity is a private execution layer that enables confidential transactions while settling securely and cheaply on Mantle using zero-knowledge proofs.**

If you say nothing else, say this.

---
---

# Part II: Technical System Design

---

## 11. Tech Stack

### Core Technologies

| Layer | Technology | Purpose |
|-------|------------|---------|
| **ZK Circuits** | Noir | Privacy proofs, state transitions |
| **Execution Engine** | Rust | CLI, state management, proof orchestration |
| **Smart Contracts** | Solidity + Foundry | Mantle settlement |
| **State Storage** | SQLite + redb | Local encrypted state |
| **Blockchain RPC** | alloy-rs | Mantle interaction |
| **Merkle Trees** | Poseidon hash | ZK-friendly state commitments |

### Why Noir?

Noir is Aztec's domain-specific language for zero-knowledge circuits.

**Advantages:**
* **Rust-like syntax** — Familiar to systems programmers
* **Privacy-native** — Designed specifically for confidential applications
* **UltraPlonk backend** — Efficient proof generation, no trusted setup per-circuit
* **Solidity verifier generation** — Auto-generates on-chain verifier contracts
* **Active ecosystem** — Strong tooling, documentation, and community

**Comparison:**

| Feature | Noir | Circom | Halo2 |
|---------|------|--------|-------|
| Syntax | Rust-like | Custom DSL | Rust |
| Learning curve | Medium | Medium | Steep |
| Proof size | ~2KB | ~200B (Groth16) | ~5KB |
| Trusted setup | Universal (once) | Per-circuit | None |
| Verifier gas | ~300k | ~200k | ~500k |
| Privacy focus | Native | Retrofitted | General |

For a **privacy-first execution layer**, Noir's design philosophy aligns perfectly.

### Why Rust?

**Advantages:**
* **Performance** — Native speed for Merkle tree operations and proof orchestration
* **Type safety** — Catch errors at compile time
* **Noir integration** — Native `noir_rs` bindings available
* **Ecosystem** — `alloy-rs` for Ethereum/Mantle, robust crypto libraries
* **CLI excellence** — `clap` provides best-in-class argument parsing

### Why Foundry?

**Advantages:**
* **Speed** — Solidity compilation in milliseconds
* **Testing** — Native Solidity tests, fuzzing, invariant testing
* **Scripting** — Deploy scripts in Solidity
* **Verification** — Built-in Etherscan/Blockscout verification
* **Modern** — Active development, excellent DX

### Key Dependencies

**Rust Crates:**
```
clap          — CLI framework (4.5)
tokio         — Async runtime (1.41)
alloy         — Ethereum/Mantle RPC (0.15)
rusqlite      — Local state storage (0.32)
redb          — Merkle node storage (2.2)
ark-ff        — Finite field arithmetic (0.5)
ark-bn254     — BN254 curve (0.5)
light-poseidon— Poseidon hash implementation (0.3)
serde         — Serialization (=1.0.219, pinned)
thiserror     — Error handling (2.0)
anyhow        — Error context (1.0)
tracing       — Logging (0.1)
rand          — Random number generation (0.8)
rpassword     — Password input (7.4)
```

**Noir Libraries:**
```
std           — Standard library (Noir built-in)
```

**Solidity:**
```
OpenZeppelin  — Access control, reentrancy guards
```

---

## 12. System Architecture

### High-Level Component Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                         USER LAYER                                │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Veilocity CLI                            │  │
│  │   veilocity deposit | transfer | withdraw | balance         │  │
│  └────────────────────────┬───────────────────────────────────┘  │
└───────────────────────────│──────────────────────────────────────┘
                            │
┌───────────────────────────▼──────────────────────────────────────┐
│                    EXECUTION LAYER (Off-chain)                    │
│                                                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐   │
│  │  veilocity-core │  │veilocity-prover │  │veilocity-contracts│ │
│  │                 │  │                 │  │                 │   │
│  │ • State Mgmt    │  │ • Witness Gen   │  │ • ABI Bindings  │   │
│  │ • Merkle Tree   │  │ • Noir Prover   │  │ • TX Builder    │   │
│  │ • Transactions  │  │ • Proof Export  │  │ • Event Listener│   │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘   │
│           │                    │                    │             │
│           └────────────────────┼────────────────────┘             │
│                                │                                  │
│  ┌─────────────────────────────▼─────────────────────────────┐   │
│  │                    Local Storage                           │   │
│  │         SQLite (accounts, txs) + redb (Merkle nodes)       │   │
│  └───────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
                            │
                            │ ZK Proof + State Root
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                    SETTLEMENT LAYER (On-chain)                    │
│                         Mantle L2                                 │
│                                                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐   │
│  │ VeilocityVault  │  │  StateAnchor    │  │ UltraVerifier   │   │
│  │                 │  │                 │  │                 │   │
│  │ • deposit()     │  │ • currentRoot   │  │ • verify()      │   │
│  │ • withdraw()    │  │ • rootHistory   │  │ (auto-generated)│   │
│  │ • locked funds  │  │ • nullifiers    │  │                 │   │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘   │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌──────────────────────────────────────────────────────────────────┐
│                       Ethereum L1                                 │
│                    (Security Anchor)                              │
└──────────────────────────────────────────────────────────────────┘
```

### Data Flow: Deposit

```
User                    CLI                  Mantle               Veilocity State
  │                      │                     │                        │
  │─── deposit 1 ETH ───▶│                     │                        │
  │                      │                     │                        │
  │                      │── generate commitment (hash of secret + amount)
  │                      │                     │                        │
  │                      │─── deposit(commitment) ──▶│                  │
  │                      │                     │                        │
  │                      │◀── DepositEvent ────│                        │
  │                      │                     │                        │
  │                      │─── insert leaf into local Merkle tree ──────▶│
  │                      │                     │                        │
  │◀── deposit complete ─│                     │                        │
```


### Data Flow: Private Transfer

```
Sender                  CLI                  Prover              Veilocity State
  │                      │                     │                        │
  │─ transfer 0.5 to Bob▶│                     │                        │
  │                      │                     │                        │
  │                      │─── compute witness (balances, paths) ───────▶│
  │                      │                     │                        │
  │                      │◀── witness data ─────────────────────────────│
  │                      │                     │                        │
  │                      │─── generate proof ─▶│                        │
  │                      │                     │                        │
  │                      │◀── ZK proof ────────│                        │
  │                      │                     │                        │
  │                      │─── update local state (new balances) ───────▶│
  │                      │                     │                        │
  │◀─ transfer complete ─│                     │                        │
```

### Data Flow: Withdrawal

```
User                    CLI                  Prover              Mantle
  │                      │                     │                   │
  │─── withdraw 0.5 ────▶│                     │                   │
  │                      │                     │                   │
  │                      │─── generate withdrawal proof ──▶│       │
  │                      │                     │                   │
  │                      │◀── ZK proof ────────│                   │
  │                      │                     │                   │
  │                      │─── withdraw(nullifier, amount, proof) ──▶│
  │                      │                     │                   │
  │                      │                     │    verify proof   │
  │                      │                     │    check nullifier│
  │                      │                     │    transfer funds │
  │                      │                     │                   │
  │                      │◀── WithdrawEvent ───────────────────────│
  │                      │                     │                   │
  │◀── withdrawal complete                     │                   │
```

### Off-chain vs On-chain Responsibilities

| Responsibility | Off-chain (Veilocity Engine) | On-chain (Mantle) |
|----------------|------------------------------|-------------------|
| Transaction execution | ✓ | |
| Balance tracking | ✓ (private) | |
| State transitions | ✓ | |
| Proof generation | ✓ | |
| Proof verification | | ✓ |
| Fund custody | | ✓ |
| State root anchoring | | ✓ |
| Nullifier tracking | | ✓ |
| Consensus | | ✓ (inherited) |
| Data availability | ✓ (local) | ✓ (commitments) |

---

## 13. ZK Circuit Specification

### Circuit Overview

Veilocity uses three primary circuits:

1. **Private Transfer** — Proves valid balance transfer between accounts
2. **Deposit** — Proves correct commitment for deposited funds
3. **Withdrawal** — Proves ownership and sufficient balance for withdrawal

### 13.1 Private Transfer Circuit

**Purpose:** Prove that a sender can transfer `amount` to a recipient without revealing balances.

**Public Inputs:**
```noir
pub old_state_root: Field      // Previous Merkle root
pub new_state_root: Field      // New Merkle root after transfer
pub nullifier: Field           // Prevents double-spending
```

**Private Inputs:**
```noir
sender_secret: Field           // Sender's private key
sender_balance: Field          // Sender's current balance
sender_index: Field            // Sender's leaf index in tree
sender_path: [Field; DEPTH]    // Merkle path for sender
sender_indices: [Field; DEPTH] // Path direction bits

recipient_pubkey: Field        // Recipient's public key
recipient_balance: Field       // Recipient's current balance
recipient_index: Field         // Recipient's leaf index
recipient_path: [Field; DEPTH] // Merkle path for recipient
recipient_indices: [Field; DEPTH]

amount: Field                  // Transfer amount
```

**Constraints:**
```
1. sender_balance >= amount
   // Cannot send more than you have

2. sender_pubkey = hash(sender_secret)
   // Proves ownership of sender account

3. sender_leaf = hash(sender_pubkey, sender_balance, sender_nonce)
   // Sender leaf commitment

4. verify_merkle_proof(sender_leaf, sender_path, sender_indices, old_state_root)
   // Sender exists in current state

5. nullifier = hash(sender_secret, sender_index, nonce)
   // Unique nullifier for this spend

6. new_sender_balance = sender_balance - amount
   new_recipient_balance = recipient_balance + amount
   // Balance update

7. new_sender_leaf = hash(sender_pubkey, new_sender_balance, sender_nonce + 1)
   new_recipient_leaf = hash(recipient_pubkey, new_recipient_balance, recipient_nonce)
   // New leaf commitments

8. new_state_root = update_merkle_tree(old_state_root, changes)
   // State transition validity
```

**Estimated Constraints:** ~15,000

### 13.2 Deposit Circuit

**Purpose:** Prove that a deposit commitment is correctly formed.

**Public Inputs:**
```noir
pub commitment: Field          // Pedersen commitment to deposit
pub amount: Field              // Deposit amount (public for verification)
```

**Private Inputs:**
```noir
secret: Field                  // User's secret (blinding factor)
```

**Constraints:**
```
1. commitment = hash(secret, amount)
   // Commitment is correctly formed

2. amount > 0
   // Non-zero deposit
```

**Estimated Constraints:** ~500

### 13.3 Withdrawal Circuit

**Purpose:** Prove ownership and sufficient balance for withdrawal.

**Public Inputs:**
```noir
pub state_root: Field          // Current state root
pub nullifier: Field           // Prevents double-withdrawal
pub amount: Field              // Withdrawal amount
pub recipient: Field           // On-chain recipient address
```

**Private Inputs:**
```noir
secret: Field                  // Account secret
balance: Field                 // Current balance
index: Field                   // Leaf index
path: [Field; DEPTH]           // Merkle path
indices: [Field; DEPTH]        // Path directions
```

**Constraints:**
```
1. pubkey = hash(secret)
   // Ownership proof

2. leaf = hash(pubkey, balance, nonce)
   // Leaf reconstruction

3. verify_merkle_proof(leaf, path, indices, state_root)
   // Membership proof

4. balance >= amount
   // Sufficient funds

5. nullifier = hash(secret, index, "withdraw", nonce)
   // Unique withdrawal nullifier
```

**Estimated Constraints:** ~8,000

### 13.4 Poseidon Hash Parameters

```
Field: BN254 scalar field
Rate: 2
Capacity: 1
Rounds: 63 full rounds
S-box: x^5
```

### 13.5 Merkle Tree Parameters

```
Depth: 20 (supports ~1M accounts)
Hash: Poseidon
Empty leaf: hash(0)
```

---

## 14. Smart Contract Interfaces

### 14.1 VeilocityVault.sol

**Purpose:** Custodial contract for deposited funds.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IVeilocityVault {
    // Events
    event Deposit(
        bytes32 indexed commitment,
        uint256 amount,
        uint256 leafIndex,
        uint256 timestamp
    );

    event Withdrawal(
        bytes32 indexed nullifier,
        address indexed recipient,
        uint256 amount
    );

    // Deposit ETH/MNT into Veilocity
    // commitment = hash(secret, amount)
    function deposit(bytes32 commitment) external payable;

    // Withdraw funds with ZK proof
    // Verifies: ownership, sufficient balance, nullifier uniqueness
    function withdraw(
        bytes32 nullifier,
        address recipient,
        uint256 amount,
        bytes calldata proof
    ) external;

    // View functions
    function isNullifierUsed(bytes32 nullifier) external view returns (bool);
    function getDepositCount() external view returns (uint256);
}
```

### 14.2 StateAnchor.sol

**Purpose:** Anchors Veilocity state roots on Mantle.

```solidity
interface IStateAnchor {
    // Events
    event StateRootUpdated(
        bytes32 indexed oldRoot,
        bytes32 indexed newRoot,
        uint256 batchIndex,
        uint256 timestamp
    );

    // Update state root with validity proof
    function updateStateRoot(
        bytes32 newRoot,
        bytes calldata proof
    ) external;

    // View functions
    function currentRoot() external view returns (bytes32);
    function rootHistory(bytes32 root) external view returns (bool);
    function batchIndex() external view returns (uint256);

    // Check if a root was ever valid (for withdrawal proofs)
    function isValidRoot(bytes32 root) external view returns (bool);
}
```

### 14.3 UltraVerifier.sol

**Purpose:** Verifies Noir UltraPlonk proofs on-chain.

```solidity
// Auto-generated by Noir compiler (nargo codegen-verifier)
interface IUltraVerifier {
    // Verify a proof against public inputs
    function verify(
        bytes calldata proof,
        bytes32[] calldata publicInputs
    ) external view returns (bool);
}
```

### 14.4 Contract Interaction Flow

```
┌─────────────┐     deposit()      ┌─────────────────┐
│    User     │ ─────────────────▶ │ VeilocityVault  │
└─────────────┘                    └────────┬────────┘
                                            │
                                            │ emit Deposit
                                            ▼
┌─────────────┐                    ┌─────────────────┐
│   Indexer   │ ◀───────────────── │   Event Log     │
└──────┬──────┘                    └─────────────────┘
       │
       │ Detect deposit
       ▼
┌─────────────────────────────────────────────────────┐
│              Off-chain Veilocity Engine             │
│                                                     │
│  1. Add leaf to local Merkle tree                   │
│  2. Credit balance to depositor                     │
│  3. Process private transfers                       │
│  4. Generate state transition proof                 │
└──────────────────────┬──────────────────────────────┘
                       │
                       │ updateStateRoot(newRoot, proof)
                       ▼
              ┌─────────────────┐
              │   StateAnchor   │
              └────────┬────────┘
                       │
                       │ verify(proof)
                       ▼
              ┌─────────────────┐
              │  UltraVerifier  │
              └─────────────────┘
```

---

## 15. State Model

### 15.1 Account Structure

```rust
struct PrivateAccount {
    /// Public key: hash(secret)
    pubkey: Field,

    /// Current balance (private)
    balance: Field,

    /// Transaction counter (prevents replay)
    nonce: u64,

    /// Leaf index in Merkle tree
    index: u64,
}

// Leaf commitment
fn compute_leaf(account: &PrivateAccount) -> Field {
    poseidon_hash([
        account.pubkey,
        account.balance,
        Field::from(account.nonce),
    ])
}
```

### 15.2 State Tree Structure

```
                        Root
                       /    \
                     /        \
                   H01         H23
                  /   \       /   \
                H0    H1    H2    H3
                |     |     |     |
              Leaf0 Leaf1 Leaf2 Leaf3
              (Alice)(Bob) (Carol)(Empty)
```

**Parameters:**
```
Tree Depth:     20
Max Accounts:   2^20 = 1,048,576
Hash Function:  Poseidon (BN254)
Empty Leaf:     poseidon_hash([0, 0, 0])
```

### 15.3 Nullifier Scheme

Nullifiers prevent double-spending without revealing which account spent.

```
nullifier = poseidon_hash([
    secret,           // Account secret (private)
    leaf_index,       // Position in tree (private)
    action_type,      // "transfer" or "withdraw"
    nonce,            // Current nonce (private)
])
```

**Properties:**
- **Uniqueness:** Each spend produces unique nullifier
- **Unlinkability:** Cannot link nullifier to account without secret
- **Non-replayable:** Nonce prevents replaying old transactions

### 15.4 Commitment Scheme

Deposits use Pedersen-style commitments:

```
commitment = poseidon_hash([secret, amount])
```

**Properties:**
- **Hiding:** Cannot determine amount without secret
- **Binding:** Cannot open to different amount

### 15.5 Local Storage Schema (SQLite)

```sql
-- Accounts table
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    pubkey BLOB NOT NULL UNIQUE,
    encrypted_balance BLOB NOT NULL,  -- Encrypted with local key
    nonce INTEGER NOT NULL DEFAULT 0,
    leaf_index INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Transactions table
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY,
    tx_type TEXT NOT NULL,  -- 'deposit', 'transfer', 'withdraw'
    nullifier BLOB,
    amount_encrypted BLOB NOT NULL,
    status TEXT NOT NULL,   -- 'pending', 'proven', 'settled'
    proof BLOB,
    created_at INTEGER NOT NULL
);

-- Merkle nodes (or use redb for performance)
CREATE TABLE merkle_nodes (
    level INTEGER NOT NULL,
    index_at_level INTEGER NOT NULL,
    hash BLOB NOT NULL,
    PRIMARY KEY (level, index_at_level)
);

-- Sync state
CREATE TABLE sync_state (
    key TEXT PRIMARY KEY,
    value BLOB NOT NULL
);
```

---

## 16. CLI Commands

### Command Reference

```
veilocity — Private execution layer CLI for Mantle

USAGE:
    veilocity [OPTIONS] <COMMAND>

COMMANDS:
    init        Initialize a new Veilocity wallet
    deposit     Deposit funds from Mantle into Veilocity
    transfer    Send private transfer to another user
    withdraw    Withdraw funds from Veilocity to Mantle
    balance     Display current private balance
    sync        Synchronize with on-chain state
    history     Show transaction history
    help        Print this message or the help of the given subcommand(s)

OPTIONS:
    -c, --config <CONFIG>    Config file path [default: ~/.veilocity/config.toml]
    -n, --network <NETWORK>  Network: mainnet, sepolia [default: sepolia]
    -v, --verbose            Enable verbose output
    -h, --help               Print help
    -V, --version            Print version
```

### Command Details

**`veilocity init`**
```
Initialize a new Veilocity wallet

USAGE:
    veilocity init [OPTIONS]

OPTIONS:
    --recover           Recover from seed phrase
    --seed <PHRASE>     Seed phrase for recovery

EXAMPLE:
    $ veilocity init
    Creating new Veilocity wallet...

    Your recovery phrase (SAVE THIS):
    witch collapse practice feed shame open despair creek road again ice least

    Wallet initialized at ~/.veilocity/
    Address: 0x7a3f...
```

**`veilocity deposit`**
```
Deposit funds from Mantle into Veilocity

USAGE:
    veilocity deposit <AMOUNT>

ARGS:
    <AMOUNT>    Amount to deposit (in ETH)

EXAMPLE:
    $ veilocity deposit 1.5
    Generating commitment...
    Submitting deposit transaction...
    Transaction: 0xabc123...
    Waiting for confirmation...
    ✓ Deposit confirmed at block 12345
    ✓ Balance updated: 1.5 ETH (private)
```

**`veilocity transfer`**
```
Send private transfer to another user

USAGE:
    veilocity transfer <RECIPIENT> <AMOUNT>

ARGS:
    <RECIPIENT>    Recipient's Veilocity public key
    <AMOUNT>       Amount to transfer

EXAMPLE:
    $ veilocity transfer 0x8b2e...f3a1 0.5
    Computing witness...
    Generating ZK proof... (this may take a moment)
    Proof generated ✓
    Updating local state...
    ✓ Transfer complete

    New balance: 1.0 ETH (private)
```

**`veilocity withdraw`**
```
Withdraw funds from Veilocity to Mantle

USAGE:
    veilocity withdraw <AMOUNT> [RECIPIENT]

ARGS:
    <AMOUNT>       Amount to withdraw
    [RECIPIENT]    Mantle address [default: connected wallet]

EXAMPLE:
    $ veilocity withdraw 0.5
    Generating withdrawal proof...
    Submitting withdrawal transaction...
    Transaction: 0xdef456...
    Waiting for confirmation...
    ✓ Withdrawal confirmed
    ✓ 0.5 ETH sent to 0x7a3f...
```

**`veilocity balance`**
```
Display current private balance

USAGE:
    veilocity balance

EXAMPLE:
    $ veilocity balance

    Veilocity Balance
    ─────────────────
    Private:  1.0 ETH
    Pending:  0.0 ETH

    Last synced: 2 minutes ago
```

### Configuration File

```toml
# ~/.veilocity/config.toml

[network]
rpc_url = "https://rpc.sepolia.mantle.xyz"
chain_id = 5003
vault_address = "0x..."
anchor_address = "0x..."

[wallet]
keystore_path = "~/.veilocity/keystore"

[prover]
threads = 4
cache_proofs = true

[sync]
poll_interval_secs = 12
confirmations = 2
```

---

## 17. Security Model

### 17.1 Threat Model

**Adversary Capabilities:**
- Can observe all on-chain transactions
- Can observe network traffic (with encryption)
- Cannot break cryptographic primitives
- Cannot compromise user's local machine

**Assets to Protect:**
- Account balances (confidentiality)
- Transaction amounts (confidentiality)
- Transaction graph (unlinkability)
- Funds (integrity)

### 17.2 Trust Assumptions

| Component | Trust Assumption |
|-----------|------------------|
| **Noir circuits** | Circuits correctly implement constraints |
| **Prover** | Proofs are sound (cannot forge invalid proofs) |
| **Verifier contract** | Correctly verifies proofs |
| **Mantle L2** | Provides consensus, data availability |
| **Poseidon hash** | Collision-resistant, preimage-resistant |
| **User's machine** | Secrets not compromised |
| **Sequencer (MVP)** | Available, processes transactions fairly |

### 17.3 Privacy Guarantees

**What IS hidden:**
- Individual account balances
- Transfer amounts (within Veilocity)
- Sender-recipient relationships
- Transaction frequency per account

**What is NOT hidden:**
- Deposit amounts (public on Mantle)
- Withdrawal amounts (public on Mantle)
- Total value locked in Veilocity
- Number of deposits/withdrawals
- Timing of deposits/withdrawals

### 17.4 Security Properties

| Property | Guarantee | Mechanism |
|----------|-----------|-----------|
| **Balance integrity** | Cannot spend more than owned | ZK range proof |
| **No double-spend** | Each balance spent once | Nullifier tracking |
| **Ownership** | Only owner can spend | Secret key in proof |
| **Withdrawal validity** | Can only withdraw owned funds | Merkle membership + ownership proof |
| **State consistency** | State transitions valid | ZK state transition proof |

### 17.5 Known Limitations (MVP)

| Limitation | Impact | Mitigation Path |
|------------|--------|-----------------|
| **Single sequencer** | Centralized ordering, liveness risk | Decentralized sequencer network |
| **No encrypted memo** | Cannot send messages with transfers | Add encrypted memo field |
| **Fixed denomination** | Deposit/withdrawal amounts visible | Shielded pools with fixed notes |
| **Timing analysis** | Deposit→withdraw correlation | Delayed withdrawals, noise |
| **Local state** | Lost device = lost history | Encrypted cloud backup |

### 17.6 Attack Mitigations

**Front-running:**
- Transfers are private, nothing to front-run
- Deposits/withdrawals use commit-reveal if needed

**Replay attacks:**
- Nonces in nullifier computation
- Nullifier uniqueness enforced on-chain

**Grinding attacks:**
- Poseidon parameters resist grinding
- Proofs bound to specific state roots

**Smart contract risks:**
- Reentrancy guards on withdrawals
- Checks-effects-interactions pattern
- Pausability for emergencies

---

## 18. Project Structure

### Directory Layout

```
veilocity/
│
├── Cargo.toml                    # Rust workspace manifest
├── Nargo.toml                    # Noir workspace manifest
├── veilocity.md                  # This document
├── README.md                     # Quick start guide
├── LICENSE                       # MIT or Apache-2.0
│
├── circuits/                     # Noir ZK circuits
│   ├── Nargo.toml
│   ├── Prover.toml
│   └── src/
│       ├── main.nr               # Private transfer circuit
│       ├── deposit.nr            # Deposit commitment circuit
│       ├── withdraw.nr           # Withdrawal proof circuit
│       ├── merkle.nr             # Merkle tree verification
│       └── poseidon.nr           # Poseidon hash helpers
│
├── contracts/                    # Solidity smart contracts
│   ├── foundry.toml
│   ├── remappings.txt
│   ├── src/
│   │   ├── VeilocityVault.sol    # Main vault: deposit, withdraw, state roots
│   │   ├── interfaces/
│   │   │   └── IVerifier.sol     # Verifier interface
│   │   ├── mocks/
│   │   │   └── MockVerifier.sol  # Mock for testing
│   │   └── verifiers/
│   │       └── UltraVerifier.sol # Auto-generated from Noir (TODO)
│   ├── script/
│   │   └── Deploy.s.sol          # Foundry deployment script
│   └── test/
│       └── VeilocityVault.t.sol  # Comprehensive test suite
│
├── crates/                       # Rust crates
│   │
│   ├── veilocity-cli/            # CLI application
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Entry point with clap
│   │       ├── config.rs         # Configuration handling
│   │       ├── wallet.rs         # Wallet management & encryption
│   │       └── commands/
│   │           ├── mod.rs        # Command module exports
│   │           ├── init.rs       # Wallet initialization
│   │           ├── deposit.rs    # Deposit funds to vault
│   │           ├── transfer.rs   # Private transfer
│   │           ├── withdraw.rs   # Withdraw with ZK proof
│   │           ├── balance.rs    # Display private balance
│   │           ├── sync.rs       # Sync with on-chain state
│   │           └── history.rs    # Transaction history
│   │
│   ├── veilocity-core/           # Core execution engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Module exports
│   │       ├── error.rs          # Error types
│   │       ├── poseidon.rs       # Poseidon hash (BN254)
│   │       ├── merkle.rs         # Incremental Merkle tree
│   │       ├── account.rs        # Account types & secrets
│   │       ├── transaction.rs    # Transaction types
│   │       └── state.rs          # State management (SQLite)
│   │
│   ├── veilocity-prover/         # ZK proof generation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs            # Module exports
│   │       ├── error.rs          # ProverError types
│   │       ├── witness.rs        # Witness computation
│   │       └── prover.rs         # NoirProver (bb CLI integration)
│   │
│   └── veilocity-contracts/      # Contract interaction
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs            # Module exports
│           ├── error.rs          # ContractError types
│           ├── bindings.rs       # Alloy ABI bindings (sol! macro)
│           ├── events.rs         # Event parsing
│           ├── vault.rs          # VaultClient & VaultReader
│           └── anchor.rs         # StateRootHistory management
│
├── scripts/
│   ├── setup.sh                  # Development setup
│   ├── generate-verifier.sh      # Generate Solidity verifier
│   └── deploy.sh                 # Contract deployment
│
└── tests/
    ├── e2e/                      # End-to-end tests
    │   ├── deposit_test.rs
    │   ├── transfer_test.rs
    │   └── withdraw_test.rs
    └── fixtures/                 # Test data
```

### Crate Responsibilities

| Crate | Responsibility |
|-------|----------------|
| `veilocity-cli` | User interface, command parsing, wallet management |
| `veilocity-core` | State management, Merkle trees, account logic |
| `veilocity-prover` | Witness generation, Noir integration, proof creation |
| `veilocity-contracts` | ABI bindings, transaction building, event indexing |

### Build Commands

```bash
# Setup development environment
./scripts/setup.sh

# Build Noir circuits
cd circuits && nargo build

# Generate Solidity verifier
nargo codegen-verifier

# Build Rust workspace
cargo build --release

# Run tests
cargo test

# Build contracts
cd contracts && forge build

# Deploy to testnet
forge script script/Deploy.s.sol --broadcast --rpc-url $MANTLE_RPC

# Run CLI
./target/release/veilocity --help
```

---

## 19. Implementation Roadmap

### Phase 1: Foundation
- Initialize Rust workspace
- Set up Noir project
- Set up Foundry project
- Implement Poseidon hash (Rust, compatible with Noir)
- Implement incremental Merkle tree

### Phase 2: Circuits
- Write Merkle proof verification in Noir
- Write private transfer circuit
- Write deposit commitment circuit
- Write withdrawal proof circuit
- Generate and test Solidity verifier

### Phase 3: Contracts
- Implement VeilocityVault.sol
- Implement StateAnchor.sol
- Write comprehensive tests
- Deploy to Mantle Sepolia

### Phase 4: Execution Engine
- Implement state management
- Implement transaction processing
- Integrate Noir prover
- Implement witness generation

### Phase 5: CLI
- Set up clap CLI structure
- Implement all commands
- Add wallet management
- Add configuration handling

### Phase 6: Integration
- End-to-end deposit flow
- End-to-end transfer flow
- End-to-end withdrawal flow
- Documentation and polish

---

## 20. Summary

Veilocity is a **private execution layer** that:

1. **Executes privately** — Transactions run off-chain with hidden balances
2. **Proves with ZK** — Noir circuits generate validity proofs
3. **Settles on Mantle** — Proofs verified, state anchored on-chain
4. **Preserves security** — Inherits Mantle's consensus and Ethereum's finality

**Tech stack:** Rust + Noir + Foundry

**Architecture:** CLI → Execution Engine → ZK Prover → Mantle Contracts

**MVP scope:** Deposit → Private Transfer → Withdraw (single token)

This design enables **institutional-grade privacy** for OTC settlement, private payroll, confidential DeFi, and more — all while leveraging Mantle's low fees and Ethereum's security.

---

*Document version: 2.0*
*Last updated: December 2024*

---
---

# Part III: Implementation Guide

---

## 21. Implementation Phases

This section provides the step-by-step implementation order with exact file paths and code specifications.

### Phase 1: Project Setup

**Step 1.1: Create Directory Structure**
```
veilocity/
├── Cargo.toml                    # Rust workspace manifest
├── Nargo.toml                    # Noir workspace manifest
├── .gitignore
├── circuits/                     # Noir ZK circuits
│   ├── Nargo.toml
│   └── src/
│       └── lib.nr
├── contracts/                    # Solidity smart contracts
│   ├── foundry.toml
│   ├── remappings.txt
│   └── src/
├── crates/                       # Rust workspace crates
│   ├── veilocity-core/
│   ├── veilocity-prover/
│   ├── veilocity-contracts/
│   └── veilocity-cli/
└── tests/
    └── e2e/
```

**Step 1.2: Install Dependencies**
```bash
# Foundry (Solidity toolchain)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Barretenberg (ZK proving backend for Noir)
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup -nv 1.0.0-beta.16
source ~/.zshrc
```

---

### Phase 2: Noir Circuits Implementation

**Priority Order:**
1. `poseidon_utils.nr` - Hash wrappers (CRITICAL: must match Rust)
2. `merkle.nr` - Merkle tree verification
3. `deposit.nr` - Simplest circuit (~500 constraints)
4. `withdraw.nr` - On-chain verified (~8k constraints)
5. `transfer.nr` - Most complex (~15k constraints)

**Key Parameters:**
- Tree Depth: 20 (supports ~1M accounts)
- Hash Function: Poseidon (BN254 scalar field)
- Poseidon Library: `poseidon` v0.2.0 from noir-lang

**Circuit Compilation:**
```bash
cd circuits
nargo compile
bb write_vk -b ./target/veilocity_circuits.json -o ./target --oracle_hash keccak
bb write_solidity_verifier -k ./target/vk -o ../contracts/src/verifiers/Verifier.sol
```

---

### Phase 3: Smart Contracts Implementation

**Contract Files:**
- `VeilocityVault.sol` - Deposit/withdraw + fund custody
- `interfaces/IVerifier.sol` - Verifier interface
- `verifiers/Verifier.sol` - Auto-generated from Noir

**Key Features:**
- Nullifier tracking (prevents double-spend)
- State root history (100 roots for flexibility)
- Reentrancy protection
- Pausability for emergencies

**Deployment:**
```bash
cd contracts
forge install openzeppelin/openzeppelin-contracts
forge build
forge test
forge script script/Deploy.s.sol --broadcast --rpc-url $MANTLE_SEPOLIA_RPC
```

---

### Phase 4: Rust Crates Implementation

**Implementation Order:**
1. `veilocity-core` - State management, Merkle tree, Poseidon
2. `veilocity-prover` - Witness generation, Noir integration
3. `veilocity-contracts` - ABI bindings, event indexing
4. `veilocity-cli` - User interface

**Critical: Poseidon Compatibility**

The Poseidon hash MUST produce identical outputs in Rust and Noir. Both use:
- BN254 scalar field
- Circom-compatible parameters
- Same round constants

Test vectors must be validated across both implementations before integration.

---

## 22. Rust Workspace Dependencies

```toml
[workspace.dependencies]
# Async runtime
tokio = { version = "1.41", features = ["full"] }

# CLI framework
clap = { version = "4.5", features = ["derive"] }

# Ethereum/Mantle interaction
alloy = { version = "0.15", features = ["full"] }

# Cryptography (using ark 0.5 to match light-poseidon requirements)
ark-ff = "0.5"
ark-bn254 = "0.5"
light-poseidon = "0.3"

# Database
rusqlite = { version = "0.32", features = ["bundled"] }
redb = "2.2"

# Serialization - Pin to a version compatible with alloy's __private usage
serde = { version = "=1.0.219", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Hex encoding
hex = "0.4"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Config
toml = "0.8"
directories = "5.0"

# Security
rand = "0.8"
```

---

## 23. Noir Circuit Dependencies

```toml
# circuits/Nargo.toml
[package]
name = "veilocity_circuits"
type = "lib"
authors = ["Veilocity Team"]
compiler_version = ">=1.0.0-beta.16"

[dependencies]
poseidon = { tag = "v0.2.0", git = "https://github.com/noir-lang/poseidon" }
```

---

## 24. Gas Estimates (Mantle)

| Operation | Gas | Cost @ 0.06 Gwei |
|-----------|-----|------------------|
| Verifier deployment | ~2-3M | ~0.0002 MNT |
| Deposit | ~60k | ~0.000004 MNT |
| Withdrawal | ~300k | ~0.00002 MNT |
| State root update | ~50k | ~0.000003 MNT |

---

## 25. Testing Strategy

### Unit Tests
- `veilocity-core`: Poseidon hashes, Merkle tree operations, account logic
- `veilocity-prover`: Witness serialization, prover CLI integration
- `veilocity-contracts`: ABI encoding, event parsing
- Solidity: Foundry tests with mock verifier

### Integration Tests
1. **Poseidon Compatibility**: Verify Rust/Noir hash equivalence
2. **Deposit Flow**: Commitment → on-chain deposit → state update
3. **Transfer Flow**: Witness → proof creation → state transition
4. **Withdrawal Flow**: Proof generation → contract verification → fund release

### E2E Tests
- Full deposit → transfer → withdraw cycle
- Double-spend prevention
- Invalid proof rejection

---

## 26. Proving Time Estimates

| Circuit | Constraints | Proving Time (M1 Mac) |
|---------|-------------|----------------------|
| Deposit | ~500 | <1 second |
| Withdrawal | ~8,000 | 2-5 seconds |
| Transfer | ~15,000 | 5-10 seconds |

---

## 27. Critical Implementation Notes

### Poseidon Hash Compatibility

This is the **single most critical technical challenge**. If Rust and Noir produce different hash outputs, the entire system fails.

**Verification Steps:**
1. Hash known test vectors in both Noir and Rust
2. Compare outputs byte-by-byte
3. If mismatch, check:
   - Field modulus (must be BN254)
   - Number of rounds
   - Round constants
   - S-box exponent (should be x^5)

### Merkle Tree Path Updates

For transfers, updating two leaves requires careful path recomputation:
1. Update sender leaf → get intermediate root
2. Recompute affected path nodes for recipient
3. Update recipient leaf → get final root

The circuit must verify this state transition is valid.

### Nullifier Security

Nullifiers must be:
- **Unique**: Each spend produces a unique nullifier
- **Unlinkable**: Cannot link nullifier to account without secret
- **Non-replayable**: Nonce prevents replaying old transactions

Formula: `nullifier = poseidon(secret, leaf_index, nonce)`

---

---

## 28. Implementation Status

This section tracks what has been implemented and what remains.

### Completed Components

#### Solidity Contracts (`contracts/src/`)
| File | Status | Description |
|------|--------|-------------|
| `VeilocityVault.sol` | ✅ Complete | Main vault with deposit, withdraw, state root management |
| `interfaces/IVerifier.sol` | ✅ Complete | Verifier interface for ZK proofs |
| `mocks/MockVerifier.sol` | ✅ Complete | Mock verifier for testing |
| `script/Deploy.s.sol` | ✅ Complete | Foundry deployment script |
| `test/VeilocityVault.t.sol` | ✅ Complete | Comprehensive test suite |

#### Rust Crates

**veilocity-core** (`crates/veilocity-core/src/`)
| File | Status | Description |
|------|--------|-------------|
| `lib.rs` | ✅ Complete | Module exports |
| `error.rs` | ✅ Complete | Error types |
| `poseidon.rs` | ✅ Complete | Poseidon hash (BN254 compatible) |
| `merkle.rs` | ✅ Complete | Incremental Merkle tree |
| `account.rs` | ✅ Complete | Account types and secrets |
| `transaction.rs` | ✅ Complete | Transaction types |
| `state.rs` | ✅ Complete | State management with SQLite |

**veilocity-prover** (`crates/veilocity-prover/src/`)
| File | Status | Description |
|------|--------|-------------|
| `lib.rs` | ✅ Complete | Module exports |
| `error.rs` | ✅ Complete | ProverError types |
| `witness.rs` | ✅ Complete | Deposit, Withdraw, Transfer witnesses |
| `prover.rs` | ✅ Complete | NoirProver for proof generation via `bb` CLI |

**veilocity-contracts** (`crates/veilocity-contracts/src/`)
| File | Status | Description |
|------|--------|-------------|
| `lib.rs` | ✅ Complete | Module exports |
| `error.rs` | ✅ Complete | ContractError types |
| `bindings.rs` | ✅ Complete | Alloy ABI bindings via `sol!` macro |
| `events.rs` | ✅ Complete | Event parsing (Deposit, Withdrawal, StateRootUpdated) |
| `vault.rs` | ✅ Complete | VaultClient and VaultReader with generic Provider |
| `anchor.rs` | ✅ Complete | StateRootHistory management |

**veilocity-cli** (`crates/veilocity-cli/src/`)
| File | Status | Description |
|------|--------|-------------|
| `main.rs` | ✅ Complete | CLI entry point with clap |
| `config.rs` | ✅ Complete | Configuration management |
| `wallet.rs` | ✅ Complete | WalletManager, key encryption |
| `commands/mod.rs` | ✅ Complete | Command module exports |
| `commands/init.rs` | ✅ Complete | Wallet initialization |
| `commands/deposit.rs` | ✅ Complete | Deposit funds to vault |
| `commands/transfer.rs` | ✅ Complete | Private transfer between users |
| `commands/withdraw.rs` | ✅ Complete | Withdraw with ZK proof |
| `commands/balance.rs` | ✅ Complete | Display private balance |
| `commands/sync.rs` | ✅ Complete | Synchronize with on-chain state |
| `commands/history.rs` | ✅ Complete | Transaction history |

#### Noir Circuits (`circuits/src/`)
| File | Status | Description |
|------|--------|-------------|
| `lib.nr` | ✅ Complete | Main circuit library |
| `merkle.nr` | ✅ Complete | Merkle tree verification |
| `poseidon_utils.nr` | ✅ Complete | Poseidon hash wrappers |
| `deposit.nr` | ✅ Complete | Deposit commitment circuit |
| `withdraw.nr` | ✅ Complete | Withdrawal proof circuit |
| `transfer.nr` | ✅ Complete | Private transfer circuit |

### Build Status

```
✅ Rust workspace: cargo build --release (successful)
✅ CLI: veilocity --help (working)
⚠️  Foundry contracts: requires `forge` installation to build/test
⚠️  Noir circuits: requires `nargo` and `bb` installation to compile
```

### CLI Commands Available

```
veilocity — Private execution layer CLI for Mantle

USAGE:
    veilocity [OPTIONS] <COMMAND>

COMMANDS:
    init        Initialize a new Veilocity wallet
    deposit     Deposit funds from Mantle into Veilocity
    transfer    Send private transfer to another user
    withdraw    Withdraw funds from Veilocity to Mantle
    balance     Display current private balance
    sync        Synchronize with on-chain state
    history     Show transaction history
    help        Print help information

OPTIONS:
    -c, --config <CONFIG>    Config file path [default: ~/.veilocity/config.toml]
    -n, --network <NETWORK>  Network: mainnet, sepolia [default: sepolia]
    -v, --verbose            Enable verbose output
    -h, --help               Print help
    -V, --version            Print version
```

### Key Technical Decisions

1. **Alloy 0.15**: Using latest alloy-rs for Ethereum/Mantle interaction
   - Generic `Provider` trait for flexibility
   - `connect_http` for HTTP RPC connections
   - `EthereumWallet` for transaction signing

2. **ark-ff 0.5**: Required by light-poseidon for BN254 field arithmetic

3. **Serde 1.0.219**: Pinned version for compatibility with alloy internals

4. **VaultClient/VaultReader Pattern**:
   - `VaultClient<P>` - Full client with signing capability for transactions
   - `VaultReader<P>` - Read-only client for queries
   - Factory functions: `create_vault_client()`, `create_vault_reader()`

### Remaining Work

1. **End-to-end Testing**: Deploy to Mantle Sepolia and test full flows
2. **Verifier Deployment**: Generate Solidity verifier from Noir circuits
3. **Production Hardening**: Improve key encryption, add backup/recovery
4. **Documentation**: Add user guides and API documentation

---

*Document version: 4.0*
*Last updated: December 2024*
