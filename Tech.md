# Veilocity Technical Documentation

> A Private Execution Layer on Mantle L2 with Zero-Knowledge Proof Settlement

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Technology Stack & Prerequisites](#2-technology-stack--prerequisites)
3. [System Architecture](#3-system-architecture)
4. [Zero-Knowledge Circuits (Noir)](#4-zero-knowledge-circuits-noir)
5. [Cryptographic Foundations](#5-cryptographic-foundations)
6. [Core Modules (Rust)](#6-core-modules-rust)
7. [Smart Contracts (Solidity)](#7-smart-contracts-solidity)
8. [Data Flows & Transaction Lifecycle](#8-data-flows--transaction-lifecycle)
9. [State Management](#9-state-management)
10. [Build, Test & Deployment](#10-build-test--deployment)
11. [Security Model](#11-security-model)
12. [Design Decisions & Rationale](#12-design-decisions--rationale)
13. [Learning Resources](#13-learning-resources)

---

## 1. Executive Summary

### What is Veilocity?

Veilocity is a **private execution layer** built on Mantle L2 that enables confidential transactions with cryptographic guarantees. Users can:

- **Deposit** funds into a privacy pool
- **Transfer** privately (off-chain) with hidden balances and recipients
- **Withdraw** back to public addresses with ZK proof verification

### Core Principles

```
┌─────────────────────────────────────────────────────────────────┐
│                     PRIVACY GUARANTEE                           │
│                                                                 │
│  • Balances are NEVER revealed on-chain                        │
│  • Transfer amounts and recipients are HIDDEN                  │
│  • Only the user knows their account state                     │
│  • Mathematical proofs verify correctness without revealing    │
│    underlying data                                             │
└─────────────────────────────────────────────────────────────────┘
```

### Security Model

```
Ethereum L1 ──▶ Mantle L2 ──▶ Veilocity Private Layer
    │               │                    │
    │               │                    │~
    ▼               ▼                    ▼
Security       Settlement          Privacy +
Anchor         Layer               Execution
```

### Implementation Status

| Component | Status | Tests | Notes |
|-----------|--------|-------|-------|
| **Noir Circuits** | Complete | 19/19 pass | Deposit, Withdraw, Transfer (simple & full) |
| **Solidity Contracts** | Complete | 29/29 pass | VeilocityVault with HonkVerifier |
| **Rust Core** | Complete | 23+ pass | State, Merkle, Poseidon |
| **Rust Prover** | Complete | 7 pass | Witness generation for all circuits |
| **Rust Contracts** | Complete | - | Event fetching, real-time sync |
| **Rust Indexer** | Complete | - | Background sync, REST API |
| **CLI** | Complete | - | All commands functional |
| **Demo Web App** | Complete | - | Next.js 15 with Privy auth |

**Key Features Implemented:**
- Real-time event fetching from chain (not hardcoded)
- Sync checkpoint tracking in SQLite
- Full transfer circuit with 4 Merkle paths for state transition
- Deposit commitment verification
- Withdrawal proof generation and verification
- Nullifier tracking for double-spend prevention

---

## 2. Technology Stack & Prerequisites

### Core Technologies

| Layer | Technology | Version | Purpose | Documentation |
|-------|------------|---------|---------|---------------|
| **ZK Circuits** | Noir (Aztec) | ≥1.0.0-beta.16 | Privacy proofs, state transitions | [noir-lang.org](https://noir-lang.org) |
| **Proving Backend** | Barretenberg | ≥1.0.0-beta.16 | UltraPlonk proof generation | [aztec-barretenberg](https://github.com/AztecProtocol/aztec-packages) |
| **Execution Engine** | Rust | 2021 Edition | CLI, state management, orchestration | [rust-lang.org](https://www.rust-lang.org) |
| **Smart Contracts** | Solidity | 0.8.27 | Mantle L2 settlement | [soliditylang.org](https://soliditylang.org) |
| **Contract Framework** | Foundry | Latest | Build, test, deploy | [getfoundry.sh](https://getfoundry.sh) |
| **RPC Client** | alloy-rs | 0.15 | Ethereum/Mantle interaction | [alloy.rs](https://alloy.rs) |
| **State Storage** | SQLite + redb | 0.32 / 2.2 | Local encrypted state | - |
| **Curve Arithmetic** | ark-bn254 | 0.5 | BN254 field operations | [arkworks.rs](https://arkworks.rs) |
| **Hash Function** | Poseidon | - | ZK-friendly hashing | See [§5.2](#52-poseidon-hash-function) |

### Rust Dependencies (Key Crates)

```toml
# Workspace Cargo.toml - Pinned versions for compatibility
[workspace.dependencies]
# Async Runtime
tokio = { version = "1.41", features = ["full"] }

# CLI Framework
clap = { version = "4.5", features = ["derive"] }

# Ethereum/Mantle Interaction
alloy = { version = "0.15", features = ["full"] }

# Database
rusqlite = { version = "0.32", features = ["bundled"] }
redb = "2.2"

# Cryptography
ark-ff = "0.5"
ark-bn254 = "0.5"
light-poseidon = "0.3"

# Serialization (PINNED - alloy compatibility)
serde = { version = "=1.0.219", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Error Handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utilities
rand = "0.8"
hex = "0.4"
```

### Installation Prerequisites

```bash
# 1. Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# 2. Noir & Barretenberg
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.16
bbup -v 1.0.0-beta.16

# 3. Foundry (Solidity toolkit)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# 4. Verify installations
cargo --version      # >= 1.75
nargo --version      # >= 1.0.0-beta.16
bb --version         # >= 1.0.0-beta.16
forge --version      # >= 0.2
```

---

## 3. System Architecture

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           USER LAYER                                    │
│                                                                         │
│    $ veilocity init     $ veilocity deposit 1.0                        │
│    $ veilocity transfer <pubkey> 0.5    $ veilocity withdraw 0.3       │
│    $ veilocity balance  $ veilocity sync $ veilocity history           │
│                                                                         │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      EXECUTION LAYER (Off-chain)                        │
│                                                                         │
│  ┌─────────────────┐  ┌──────────────────┐  ┌──────────────────────┐  │
│  │ veilocity-core  │  │ veilocity-prover │  │ veilocity-contracts  │  │
│  │                 │  │                  │  │                      │  │
│  │ • StateManager  │  │ • Witness Gen    │  │ • VaultClient        │  │
│  │ • MerkleTree    │  │ • Noir Compile   │  │ • Event Listener     │  │
│  │ • PoseidonHash  │  │ • BB Prove       │  │ • RPC Interaction    │  │
│  │ • Accounts      │  │ • Proof Export   │  │ • ABI Bindings       │  │
│  └─────────────────┘  └──────────────────┘  └──────────────────────┘  │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────────┐ │
│  │                     LOCAL STORAGE                                  │ │
│  │   SQLite: accounts, transactions, nullifiers, sync_state          │ │
│  │   redb: Merkle tree nodes (sparse storage)                        │ │
│  └───────────────────────────────────────────────────────────────────┘ │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 │ ZK Proofs + State Roots
                                 │ (via alloy-rs RPC)
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      SETTLEMENT LAYER (On-chain)                        │
│                             Mantle L2                                   │
│                                                                         │
│  ┌─────────────────────────────┐    ┌────────────────────────────────┐ │
│  │      VeilocityVault.sol     │    │       UltraVerifier.sol        │ │
│  │                             │    │    (Auto-generated by bb)      │ │
│  │  • deposit(commitment)      │    │                                │ │
│  │  • withdraw(proof, ...)     │◀──▶│  • verify(proof, publicInputs) │ │
│  │  • updateStateRoot(...)     │    │  • ~300k gas per verification  │ │
│  │  • Nullifier tracking       │    │                                │ │
│  │  • TVL accounting           │    │                                │ │
│  └─────────────────────────────┘    └────────────────────────────────┘ │
│                                                                         │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      SECURITY LAYER                                     │
│                       Ethereum L1                                       │
│                                                                         │
│  Mantle posts state commitments to Ethereum for security inheritance   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Project Directory Structure

```
veilocity/
├── Cargo.toml                    # Rust workspace manifest
├── Nargo.toml                    # Noir workspace config
├── README.md                     # Quick start guide
├── Tech.md                       # This document
├── veilocity.md                  # Detailed specification
│
├── circuits/                     # Noir ZK Circuits
│   ├── Nargo.toml               # Circuit package config
│   └── src/
│       ├── main.nr              # Circuit entry points
│       ├── transfer.nr          # Private transfer (~15k constraints)
│       ├── deposit.nr           # Deposit proof (~500 constraints)
│       ├── withdraw.nr          # Withdrawal proof (~8k constraints)
│       ├── merkle.nr            # Merkle verification (depth=20)
│       ├── poseidon_utils.nr    # Hash function wrappers
│       └── lib.nr               # Library exports
│
├── contracts/                    # Solidity Smart Contracts
│   ├── foundry.toml             # Foundry config (solc 0.8.27)
│   ├── remappings.txt           # Import remappings
│   ├── src/
│   │   ├── VeilocityVault.sol   # Main custody contract
│   │   ├── interfaces/
│   │   │   └── IVerifier.sol    # Verifier interface
│   │   └── mocks/
│   │       └── MockVerifier.sol # Testing mock
│   ├── script/
│   │   └── Deploy.s.sol         # Deployment script
│   └── test/
│       └── VeilocityVault.t.sol # Contract tests
│
├── demo/                         # Web Demo Application
│   ├── package.json             # Next.js 15 + Privy + Wagmi
│   ├── app/                     # Next.js app router
│   │   ├── layout.tsx           # Root layout with providers
│   │   └── page.tsx             # Main dashboard page
│   ├── components/              # React components
│   │   ├── deposit-form.tsx     # Deposit with commitment
│   │   └── withdraw-form.tsx    # Withdraw with ZK proof
│   └── lib/
│       └── crypto.ts            # Poseidon hash in browser
│
└── crates/                       # Rust Implementation
    ├── veilocity-core/          # State, Merkle, Poseidon
    │   └── src/
    │       ├── lib.rs
    │       ├── poseidon.rs      # BN254 Poseidon hasher
    │       ├── merkle.rs        # Incremental Merkle tree
    │       ├── account.rs       # Private account struct
    │       ├── state.rs         # StateManager (SQLite + tree)
    │       ├── transaction.rs   # Transaction types
    │       └── error.rs         # Error definitions
    │
    ├── veilocity-prover/        # Proof Generation
    │   └── src/
    │       ├── lib.rs
    │       ├── prover.rs        # Noir/BB CLI orchestration
    │       ├── witness.rs       # Witness structures
    │       └── error.rs         # Prover errors
    │
    ├── veilocity-contracts/     # On-chain Interaction
    │   └── src/
    │       ├── lib.rs
    │       ├── vault.rs         # VaultClient RPC wrapper
    │       ├── anchor.rs        # StateAnchor interface
    │       ├── bindings.rs      # alloy ABI bindings
    │       ├── events.rs        # Event parsing
    │       └── error.rs         # Contract errors
    │
    ├── veilocity-indexer/       # Background Indexer Service
    │   └── src/
    │       ├── main.rs          # Entry point with Axum server
    │       ├── indexer.rs       # Background sync loop
    │       └── api.rs           # REST API endpoints
    │
    └── veilocity-cli/           # Command Line Interface
        └── src/
            ├── main.rs          # Entry point (clap)
            ├── config.rs        # TOML configuration
            ├── ui.rs            # Terminal UI helpers
            ├── wallet.rs        # Key generation/loading
            └── commands/        # Subcommands
                ├── init.rs      # Wallet initialization
                ├── deposit.rs   # Deposit flow
                ├── transfer.rs  # Private transfer
                ├── withdraw.rs  # Withdrawal flow
                ├── balance.rs   # Balance query
                ├── sync.rs      # Chain sync
                ├── history.rs   # Transaction history
                └── config.rs    # Config view/set
```

### 3.3 Component Interaction Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                         veilocity-cli                                │
│                                                                      │
│  main.rs ──▶ Commands ──▶ Orchestration                             │
└──────┬───────────────────────────────────────────────┬───────────────┘
       │                                               │
       │ State Queries                                 │ Proof Requests
       ▼                                               ▼
┌──────────────────────┐                    ┌─────────────────────────┐
│   veilocity-core     │                    │   veilocity-prover      │
│                      │                    │                         │
│  StateManager        │◀──── Witness ─────▶│  NoirProver             │
│    │                 │      Data          │    │                    │
│    ├─ MerkleTree     │                    │    ├─ nargo execute     │
│    ├─ AccountStore   │                    │    ├─ bb prove          │
│    └─ NullifierSet   │                    │    └─ Proof bytes       │
│                      │                    │                         │
│  PoseidonHasher      │                    │  Witness Serializer     │
│    │                 │                    │    │                    │
│    ├─ hash2()        │                    │    ├─ DepositWitness    │
│    ├─ hash3()        │                    │    ├─ WithdrawWitness   │
│    └─ derive_pubkey()│                    │    └─ TransferWitness   │
└──────────────────────┘                    └─────────────────────────┘
       │                                               │
       │ Account Data                                  │ Proof Bytes
       ▼                                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      veilocity-contracts                            │
│                                                                     │
│  VaultClient                                                        │
│    │                                                                │
│    ├─ deposit(commitment) ──────────────▶ VeilocityVault.deposit()  │
│    ├─ withdraw(nullifier, proof, ...) ──▶ VeilocityVault.withdraw() │
│    ├─ current_root() ───────────────────▶ VeilocityVault.view       │
│    └─ is_nullifier_used() ──────────────▶ VeilocityVault.view       │
│                                                                     │
│  EventListener                                                      │
│    │                                                                │
│    └─ DepositEvent ─────────────────────▶ Update local Merkle tree  │
└─────────────────────────────────────────────────────────────────────┘

### 3.4 Background Indexer Service

```
┌─────────────────────────────────────────────────────────────────────┐
│                      veilocity-indexer                               │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                    Background Sync Loop                        │ │
│  │                                                                │ │
│  │  1. Poll RPC for new blocks (every N seconds)                  │ │
│  │  2. Fetch Deposit/Withdrawal events in batches (9000 blocks)   │ │
│  │  3. Insert deposit commitments into local Merkle tree          │ │
│  │  4. Track used nullifiers                                      │ │
│  │  5. Compute and cache current state root                       │ │
│  │  6. Update sync progress                                       │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                      REST API (Axum)                           │ │
│  │                                                                │ │
│  │  GET /health      → { status, is_syncing, progress, block }    │ │
│  │  GET /sync        → { state_root, leaves, nullifiers, ... }    │ │
│  │  GET /deposits    → [ { commitment, amount, leaf_index } ]     │ │
│  │  GET /withdrawals → [ { nullifier, recipient, amount } ]       │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 3.5 Demo Web Application

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Demo Web App (Next.js 15)                         │
│                                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐   │
│  │    Privy     │  │    Wagmi     │  │    circomlibjs           │   │
│  │  Auth Layer  │  │   RPC/TX     │  │  Poseidon in Browser     │   │
│  └──────────────┘  └──────────────┘  └──────────────────────────┘   │
│                                                                     │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Components                                                     │ │
│  │  • DepositForm: Generate commitment, submit deposit tx         │ │
│  │  • WithdrawForm: Generate ZK proof (mock), submit withdraw tx  │ │
│  │  • ProofVisualization: Display ZK proof generation steps       │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. Zero-Knowledge Circuits (Noir)

### 4.1 What is Noir?

**Noir** is a domain-specific language (DSL) for writing zero-knowledge circuits, developed by Aztec. It compiles to an intermediate representation (ACIR) that can be proved using various backends, most commonly **Barretenberg** (UltraPlonk).

**Why Noir for Veilocity?**

| Feature | Benefit |
|---------|---------|
| Rust-like syntax | Familiar to systems programmers |
| Privacy-native | Designed for confidential applications |
| Universal setup | Not per-circuit (reusable) |
| Solidity verifier generation | Auto-generates on-chain verifiers |
| Active ecosystem | Good tooling and documentation |

### 4.2 Circuit Overview

| Circuit | File | Constraints | Purpose |
|---------|------|-------------|---------|
| Transfer | `transfer.nr` | ~15,000 | Private balance transfer between accounts |
| Deposit | `deposit.nr` | ~500 | Verify deposit commitment formation |
| Withdraw | `withdraw.nr` | ~8,000 | Prove ownership and sufficient balance |

### 4.3 Transfer Circuit

**Purpose**: Prove a valid balance transfer between two accounts without revealing balances, amounts, or identities.

```noir
// circuits/src/transfer.nr

// PUBLIC INPUTS - Visible on-chain
// • old_state_root: Merkle root before transfer
// • new_state_root: Merkle root after transfer
// • nullifier: Unique spend identifier (prevents double-spend)

// PRIVATE INPUTS - Known only to prover
// • Sender: secret, balance, nonce, index, merkle_path[20]
// • Recipient: pubkey, balance, nonce, index, merkle_path[20]
// • amount: Transfer amount

fn main(
    // Public inputs
    old_state_root: pub Field,
    new_state_root: pub Field,
    nullifier: pub Field,

    // Sender private inputs
    sender_secret: Field,
    sender_balance: Field,
    sender_nonce: Field,
    sender_index: Field,
    sender_path: [Field; TREE_DEPTH],

    // Recipient private inputs
    recipient_pubkey: Field,
    recipient_balance: Field,
    recipient_nonce: Field,
    recipient_index: Field,
    recipient_path: [Field; TREE_DEPTH],

    // Transfer amount (private)
    amount: Field,
) {
    // CONSTRAINT 1: Derive sender pubkey from secret
    let sender_pubkey = poseidon::hash_1([sender_secret]);

    // CONSTRAINT 2: Compute sender leaf commitment
    let sender_leaf = poseidon::hash_3([sender_pubkey, sender_balance, sender_nonce]);

    // CONSTRAINT 3: Verify sender exists in old state (Merkle proof)
    let computed_root = compute_root(sender_leaf, sender_index, sender_path);
    assert(computed_root == old_state_root);

    // CONSTRAINT 4: Sender has sufficient balance
    assert(sender_balance >= amount);

    // CONSTRAINT 5: Verify nullifier is correctly computed
    let expected_nullifier = poseidon::hash_3([sender_secret, sender_index, sender_nonce]);
    assert(nullifier == expected_nullifier);

    // CONSTRAINT 6: Compute new sender state
    let new_sender_balance = sender_balance - amount;
    let new_sender_nonce = sender_nonce + 1;
    let new_sender_leaf = poseidon::hash_3([sender_pubkey, new_sender_balance, new_sender_nonce]);

    // CONSTRAINT 7: Compute new recipient state
    let new_recipient_balance = recipient_balance + amount;
    let new_recipient_leaf = poseidon::hash_3([recipient_pubkey, new_recipient_balance, recipient_nonce]);

    // CONSTRAINT 8: Verify new state root
    // (This would involve updating both leaves in the tree)
    // ...
}
```

**Constraint Breakdown**:

```
┌─────────────────────────────────────────────────────────────────┐
│                    TRANSFER CIRCUIT FLOW                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  INPUT: sender_secret                                           │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────────┐                                       │
│  │ pubkey = hash(secret) │  ◀── Proves knowledge of secret     │
│  └─────────────────────┘                                       │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────────────────────┐                           │
│  │ leaf = hash(pubkey, balance, nonce) │                       │
│  └─────────────────────────────────┘                           │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────────────────────────────┐                   │
│  │ verify_merkle_proof(leaf, path, root)   │ ◀── Proves account│
│  │ assert(computed_root == old_state_root) │     exists        │
│  └─────────────────────────────────────────┘                   │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────────────┐                                   │
│  │ assert(balance >= amount) │ ◀── Proves sufficient funds     │
│  └─────────────────────────┘                                   │
│           │                                                     │
│           ▼                                                     │
│  ┌────────────────────────────────────────┐                    │
│  │ nullifier = hash(secret, index, nonce) │ ◀── Unique spend ID│
│  └────────────────────────────────────────┘                    │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────────────────────┐                           │
│  │ Update balances, compute new root │                         │
│  │ assert(new_root == new_state_root)│ ◀── Correct transition  │
│  └─────────────────────────────────┘                           │
│                                                                 │
│  OUTPUT: Proof that all constraints satisfied                   │
└─────────────────────────────────────────────────────────────────┘
```

### 4.4 Deposit Circuit

**Purpose**: Verify that a deposit commitment is correctly formed.

```noir
// circuits/src/deposit.nr

// PUBLIC INPUTS
// • commitment: The deposit commitment (published on-chain)
// • amount: Deposit amount (verified on-chain via msg.value)

// PRIVATE INPUTS
// • secret: User's secret key

fn main(
    commitment: pub Field,
    amount: pub Field,
    secret: Field,
) {
    // CONSTRAINT: commitment == hash(secret, amount)
    let expected = poseidon::hash_2([secret, amount]);
    assert(commitment == expected);

    // CONSTRAINT: amount > 0
    assert(amount != 0);
}
```

**Why a deposit proof?**
- Proves the user knows the secret that generated the commitment
- Prevents front-running attacks where someone could claim another's deposit
- Binds the amount to the commitment

### 4.5 Withdrawal Circuit

**Purpose**: Prove account ownership and sufficient balance for withdrawal.

```noir
// circuits/src/withdraw.nr

// PUBLIC INPUTS
// • state_root: Current Merkle root (verifiable on-chain)
// • nullifier: Withdrawal identifier (tracked on-chain)
// • amount: Withdrawal amount (verified on-chain)
// • recipient: Destination address (bound to proof)

// PRIVATE INPUTS
// • secret, balance, nonce, index, path[20]

fn main(
    state_root: pub Field,
    nullifier: pub Field,
    amount: pub Field,
    recipient: pub Field,

    secret: Field,
    balance: Field,
    nonce: Field,
    index: Field,
    path: [Field; TREE_DEPTH],
) {
    // 1. Derive public key
    let pubkey = poseidon::hash_1([secret]);

    // 2. Compute leaf
    let leaf = poseidon::hash_3([pubkey, balance, nonce]);

    // 3. Verify Merkle membership
    let computed_root = compute_root(leaf, index, path);
    assert(computed_root == state_root);

    // 4. Verify sufficient balance
    assert(balance >= amount);

    // 5. Verify nullifier
    let expected_nullifier = poseidon::hash_3([secret, index, nonce]);
    assert(nullifier == expected_nullifier);

    // 6. Recipient binding (prevents front-running)
    assert(recipient != 0);
}
```

### 4.6 Merkle Tree Verification

**File**: `circuits/src/merkle.nr`

**Parameters**:
- **Depth**: 20 levels (supports 2²⁰ = 1,048,576 accounts)
- **Hash**: Poseidon
- **Empty leaf**: `hash(0, 0)`

```noir
// circuits/src/merkle.nr

global TREE_DEPTH: u32 = 20;

// Compute Merkle root from leaf and path
fn compute_root(leaf: Field, index: Field, path: [Field; TREE_DEPTH]) -> Field {
    let mut current = leaf;
    let index_bits = index.to_be_bits(TREE_DEPTH);

    for i in 0..TREE_DEPTH {
        let sibling = path[i];

        // If bit is 0: current is left child
        // If bit is 1: current is right child
        if index_bits[TREE_DEPTH - 1 - i] == 0 {
            current = poseidon::hash_2([current, sibling]);
        } else {
            current = poseidon::hash_2([sibling, current]);
        }
    }

    current
}

// Assert membership (common pattern)
fn assert_merkle_proof(
    leaf: Field,
    index: Field,
    path: [Field; TREE_DEPTH],
    expected_root: Field
) {
    let computed = compute_root(leaf, index, path);
    assert(computed == expected_root);
}
```

**Merkle Path Traversal Visualization**:

```
                          Root (Level 20)
                        /                \
                      /                    \
            H(0,1)                              H(2,3)      Level 19
           /      \                            /      \
         H(0)    H(1)                        H(2)    H(3)   Level 18
         / \     / \                         / \     / \
        ...........................                         ...

        Leaf[0]  Leaf[1]  ...  Leaf[2^20-1]                 Level 0


Path for Leaf[5] (binary: 00000000000000000101):
─────────────────────────────────────────────────

Step 0: bit=1 → Leaf is RIGHT child, sibling is path[0]
        current = hash(path[0], current)

Step 1: bit=0 → Current is LEFT child, sibling is path[1]
        current = hash(current, path[1])

Step 2: bit=1 → Current is RIGHT child, sibling is path[2]
        current = hash(path[2], current)

... continue for 20 levels ...

Final: current == root
```

### 4.7 Poseidon Hash Utilities

**File**: `circuits/src/poseidon_utils.nr`

```noir
// circuits/src/poseidon_utils.nr

use dep::std::hash::poseidon;

// Hash 2 elements (Merkle nodes)
fn hash2(left: Field, right: Field) -> Field {
    poseidon::bn254::hash_2([left, right])
}

// Hash 3 elements (account leaves, nullifiers)
fn hash3(a: Field, b: Field, c: Field) -> Field {
    poseidon::bn254::hash_3([a, b, c])
}

// Derive public key from secret
fn derive_pubkey(secret: Field) -> Field {
    poseidon::bn254::hash_1([secret])
}

// Compute nullifier (unique spend identifier)
fn compute_nullifier(secret: Field, index: Field, nonce: Field) -> Field {
    hash3(secret, index, nonce)
}

// Compute account leaf commitment
fn compute_leaf(pubkey: Field, balance: Field, nonce: Field) -> Field {
    hash3(pubkey, balance, nonce)
}

// Compute deposit commitment
fn compute_deposit_commitment(secret: Field, amount: Field) -> Field {
    poseidon::bn254::hash_2([secret, amount])
}
```

---

## 5. Cryptographic Foundations

### 5.1 BN254 Elliptic Curve

**What is BN254?**

BN254 (also called alt_bn128 or bn128) is a pairing-friendly elliptic curve used extensively in zero-knowledge proof systems. It's the curve used by Ethereum's precompiled contracts for pairing operations.

**Parameters**:

```
Field Order (r): 21888242871839275222246405745257275088548364400416034343698204186575808495617
                 (approximately 2^254)

Curve Equation: y² = x³ + 3 (over the base field)
```

**Why BN254?**

| Reason | Explanation |
|--------|-------------|
| Ethereum compatibility | Precompiles at addresses 0x06, 0x07, 0x08 |
| Efficient pairings | Enables succinct proof verification |
| Wide adoption | Used by Zcash, Tornado Cash, Aztec, etc. |
| Noir default | Built-in support in Barretenberg |

### 5.2 Poseidon Hash Function

**What is Poseidon?**

Poseidon is an arithmetic-friendly hash function designed specifically for zero-knowledge proof systems. Unlike SHA-256 which operates on bits, Poseidon operates on field elements, making it ~8x more efficient in ZK circuits.

**Parameters (Circom-compatible)**:

```
┌──────────────────────────────────────────────┐
│           POSEIDON PARAMETERS                │
├──────────────────────────────────────────────┤
│ Field:           BN254 scalar field          │
│ S-box:           x^5 (fifth power)           │
│ Full rounds:     8                           │
│ Partial rounds:  depending on rate           │
│ State size:      rate + 1 (capacity = 1)     │
│ Output:          Single field element        │
└──────────────────────────────────────────────┘
```

**Rate configurations used**:

| Function | Rate | Input | Use Case |
|----------|------|-------|----------|
| hash_1 | 1 | 1 element | Public key derivation |
| hash_2 | 2 | 2 elements | Merkle nodes, deposit commitments |
| hash_3 | 3 | 3 elements | Account leaves, nullifiers |

**Rust Implementation** (`veilocity-core/src/poseidon.rs`):

```rust
use light_poseidon::{Poseidon, PoseidonHasher, PoseidonBytesHasher};
use ark_bn254::Fr;

pub struct PoseidonHasher {
    // Pre-initialized hashers for different rates
    hasher_1: Poseidon<Fr, 2, 1>,  // rate 1
    hasher_2: Poseidon<Fr, 3, 2>,  // rate 2
    hasher_3: Poseidon<Fr, 4, 3>,  // rate 3
}

impl PoseidonHasher {
    pub fn hash2(&self, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        let left_fr = Fr::from_be_bytes_mod_order(&left);
        let right_fr = Fr::from_be_bytes_mod_order(&right);

        let result = self.hasher_2.hash(&[left_fr, right_fr]).unwrap();
        result.into_bigint().to_bytes_be().try_into().unwrap()
    }

    pub fn derive_pubkey(&self, secret: [u8; 32]) -> [u8; 32] {
        let secret_fr = Fr::from_be_bytes_mod_order(&secret);
        let result = self.hasher_1.hash(&[secret_fr]).unwrap();
        result.into_bigint().to_bytes_be().try_into().unwrap()
    }

    pub fn compute_nullifier(
        &self,
        secret: [u8; 32],
        index: u64,
        nonce: u64
    ) -> [u8; 32] {
        let secret_fr = Fr::from_be_bytes_mod_order(&secret);
        let index_fr = Fr::from(index);
        let nonce_fr = Fr::from(nonce);

        let result = self.hasher_3.hash(&[secret_fr, index_fr, nonce_fr]).unwrap();
        result.into_bigint().to_bytes_be().try_into().unwrap()
    }
}
```

### 5.3 Key Derivation

```
┌────────────────────────────────────────────────────────────────┐
│                    KEY DERIVATION FLOW                         │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  STEP 1: Generate Secret                                       │
│  ────────────────────────                                      │
│  secret = random_bytes(32)     # 256-bit random value          │
│  (Stored encrypted locally)                                    │
│                                                                │
│  STEP 2: Derive Public Key                                     │
│  ─────────────────────────                                     │
│  pubkey = poseidon_1(secret)   # One-way function              │
│                                                                │
│  ┌─────────┐     poseidon_1     ┌─────────┐                   │
│  │ secret  │ ─────────────────▶ │ pubkey  │                   │
│  └─────────┘                    └─────────┘                   │
│      │                              │                          │
│      │ Private                      │ Public                   │
│      │ (never shared)               │ (shared with senders)    │
│                                                                │
│  SECURITY PROPERTIES:                                          │
│  • Collision resistance: Infeasible to find two secrets        │
│    with the same pubkey                                        │
│  • Preimage resistance: Cannot derive secret from pubkey       │
│  • Deterministic: Same secret always gives same pubkey         │
└────────────────────────────────────────────────────────────────┘
```

### 5.4 Nullifier System

**Purpose**: Prevent double-spending by creating a unique, one-time identifier for each spend.

```
┌────────────────────────────────────────────────────────────────┐
│                    NULLIFIER MECHANISM                         │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  FORMULA: nullifier = poseidon_3(secret, leaf_index, nonce)   │
│                                                                │
│  COMPONENTS:                                                   │
│  ┌──────────────┬────────────────────────────────────────────┐│
│  │ secret       │ Only account owner knows this              ││
│  ├──────────────┼────────────────────────────────────────────┤│
│  │ leaf_index   │ Account's position in Merkle tree          ││
│  ├──────────────┼────────────────────────────────────────────┤│
│  │ nonce        │ Increments with each spend                 ││
│  └──────────────┴────────────────────────────────────────────┘│
│                                                                │
│  LIFECYCLE:                                                    │
│                                                                │
│  Account Created: nonce = 0                                    │
│       │                                                        │
│       ▼                                                        │
│  First Spend: nullifier_0 = hash(secret, index, 0)            │
│       │       ─────────────────────────────────────           │
│       │       Published on-chain, stored in nullifier set     │
│       │       nonce incremented to 1                          │
│       ▼                                                        │
│  Second Spend: nullifier_1 = hash(secret, index, 1)           │
│       │        ─────────────────────────────────────          │
│       │        Different from nullifier_0 (nonce changed)     │
│       ▼                                                        │
│  ... and so on                                                 │
│                                                                │
│  DOUBLE-SPEND PREVENTION:                                      │
│  ─────────────────────────                                     │
│  On-chain contract maintains: Set<nullifier>                   │
│                                                                │
│  Before withdrawal:                                            │
│  if nullifiers.contains(submitted_nullifier) {                 │
│      REJECT  // Already spent                                  │
│  } else {                                                      │
│      nullifiers.insert(submitted_nullifier)                    │
│      PROCEED                                                   │
│  }                                                             │
└────────────────────────────────────────────────────────────────┘
```

### 5.5 Account Commitment (Leaf)

```
leaf = poseidon_3(pubkey, balance, nonce)
```

**Why include each component?**

| Component | Reason |
|-----------|--------|
| `pubkey` | Binds leaf to account owner |
| `balance` | Committed balance (hidden) |
| `nonce` | Prevents replay attacks, enables unique nullifiers |

### 5.6 Deposit Commitment

```
commitment = poseidon_2(secret, amount)
```

**Purpose**: Binds the deposit amount to the user's secret, preventing:
- Front-running (someone else claiming the deposit)
- Amount manipulation

---

## 6. Core Modules (Rust)

### 6.1 veilocity-core

The foundational crate containing state management, cryptographic primitives, and data structures.

#### 6.1.1 StateManager (`state.rs`)

```rust
pub struct StateManager {
    db: Connection,           // SQLite connection
    tree: MerkleTree,         // In-memory Merkle tree
    hasher: PoseidonHasher,   // Poseidon hash instance
}

impl StateManager {
    // Create new account in the system
    pub fn create_account(&mut self, pubkey: [u8; 32]) -> Result<u64> {
        // 1. Insert into database
        // 2. Add leaf to Merkle tree
        // 3. Return leaf index
    }

    // Get account by its tree index
    pub fn get_account_by_index(&self, index: u64) -> Result<PrivateAccount> {
        // Query from database
    }

    // Update account balance/nonce
    pub fn update_account(&mut self, account: &PrivateAccount) -> Result<()> {
        // 1. Update database
        // 2. Recompute and update Merkle leaf
    }

    // Get Merkle proof for an account
    pub fn get_merkle_proof(&self, index: u64) -> Result<Vec<[u8; 32]>> {
        self.tree.get_proof(index)
    }

    // Check if nullifier has been used
    pub fn is_nullifier_used(&self, nullifier: [u8; 32]) -> Result<bool> {
        // Query nullifiers table
    }

    // Mark nullifier as used
    pub fn use_nullifier(&mut self, nullifier: [u8; 32]) -> Result<()> {
        // Insert into nullifiers table
    }

    // Get current Merkle root
    pub fn current_root(&self) -> [u8; 32] {
        self.tree.root()
    }
}
```

#### 6.1.2 MerkleTree (`merkle.rs`)

```rust
pub struct MerkleTree {
    depth: usize,                              // Tree depth (20)
    nodes: redb::Database,                     // Sparse node storage
    empty_hashes: Vec<[u8; 32]>,              // Precomputed empty subtree hashes
    hasher: PoseidonHasher,
    next_index: u64,                           // Next available leaf index
}

impl MerkleTree {
    // Insert new leaf at next available index
    pub fn insert(&mut self, leaf: [u8; 32]) -> u64 {
        let index = self.next_index;
        self.update_leaf(index, leaf);
        self.next_index += 1;
        index
    }

    // Update existing leaf and recalculate path to root
    pub fn update_leaf(&mut self, index: u64, new_leaf: [u8; 32]) {
        let mut current = new_leaf;
        let mut current_index = index;

        for level in 0..self.depth {
            // Store current node
            self.set_node(level, current_index, current);

            // Get sibling
            let sibling_index = current_index ^ 1;
            let sibling = self.get_node(level, sibling_index);

            // Hash parent
            let parent_index = current_index / 2;
            if current_index % 2 == 0 {
                current = self.hasher.hash2(current, sibling);
            } else {
                current = self.hasher.hash2(sibling, current);
            }

            current_index = parent_index;
        }

        // Store root
        self.set_node(self.depth, 0, current);
    }

    // Get proof (siblings along path to root)
    pub fn get_proof(&self, index: u64) -> Vec<[u8; 32]> {
        let mut proof = Vec::with_capacity(self.depth);
        let mut current_index = index;

        for level in 0..self.depth {
            let sibling_index = current_index ^ 1;
            let sibling = self.get_node(level, sibling_index);
            proof.push(sibling);
            current_index /= 2;
        }

        proof
    }

    // Verify proof externally
    pub fn verify_proof(
        &self,
        leaf: [u8; 32],
        index: u64,
        proof: &[[u8; 32]],
        expected_root: [u8; 32]
    ) -> bool {
        let computed_root = self.compute_root_from_proof(leaf, index, proof);
        computed_root == expected_root
    }
}
```

**Sparse Storage Optimization**:

```
┌────────────────────────────────────────────────────────────────┐
│                   SPARSE MERKLE TREE                           │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Problem: Full tree with 2^20 leaves = millions of nodes       │
│  Solution: Only store non-empty nodes                          │
│                                                                │
│  EMPTY HASH PRECOMPUTATION:                                    │
│  ──────────────────────────                                    │
│  empty_hash[0] = hash(0, 0)              // Empty leaf         │
│  empty_hash[1] = hash(empty_hash[0], empty_hash[0])            │
│  empty_hash[2] = hash(empty_hash[1], empty_hash[1])            │
│  ...                                                           │
│  empty_hash[20] = hash(empty_hash[19], empty_hash[19]) // Root │
│                                                                │
│  When requesting a node that doesn't exist in storage:         │
│  → Return empty_hash[level]                                    │
│                                                                │
│  Result: O(n log n) storage for n accounts instead of O(2^20)  │
└────────────────────────────────────────────────────────────────┘
```

#### 6.1.3 PrivateAccount (`account.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateAccount {
    pub pubkey: [u8; 32],
    pub balance: u128,
    pub nonce: u64,
    pub index: u64,  // Position in Merkle tree
}

impl PrivateAccount {
    pub fn new(pubkey: [u8; 32], index: u64) -> Self {
        Self {
            pubkey,
            balance: 0,
            nonce: 0,
            index,
        }
    }

    // Credit funds to account
    pub fn credit(&mut self, amount: u128) {
        self.balance = self.balance.checked_add(amount)
            .expect("Balance overflow");
    }

    // Debit funds from account (panics if insufficient)
    pub fn debit(&mut self, amount: u128) {
        self.balance = self.balance.checked_sub(amount)
            .expect("Insufficient balance");
        self.nonce += 1;  // Increment nonce on spend
    }

    // Check if has sufficient balance
    pub fn has_balance(&self, amount: u128) -> bool {
        self.balance >= amount
    }

    // Compute leaf commitment
    pub fn compute_leaf(&self, hasher: &PoseidonHasher) -> [u8; 32] {
        hasher.hash3(
            self.pubkey,
            self.balance.to_be_bytes()[16..].try_into().unwrap(),
            self.nonce.to_be_bytes().try_into().unwrap()
        )
    }
}

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct AccountSecret {
    secret: [u8; 32],
}

impl AccountSecret {
    pub fn generate() -> Self {
        let mut secret = [0u8; 32];
        rand::thread_rng().fill(&mut secret);
        Self { secret }
    }

    pub fn derive_pubkey(&self, hasher: &PoseidonHasher) -> [u8; 32] {
        hasher.derive_pubkey(self.secret)
    }

    pub fn compute_nullifier(
        &self,
        hasher: &PoseidonHasher,
        index: u64,
        nonce: u64
    ) -> [u8; 32] {
        hasher.compute_nullifier(self.secret, index, nonce)
    }
}
```

#### 6.1.4 Database Schema

```sql
-- accounts table
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pubkey BLOB NOT NULL UNIQUE,
    balance_encrypted BLOB NOT NULL,  -- Encrypted with local key
    nonce INTEGER DEFAULT 0,
    leaf_index INTEGER UNIQUE,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX idx_accounts_pubkey ON accounts(pubkey);
CREATE INDEX idx_accounts_leaf_index ON accounts(leaf_index);

-- transactions table
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tx_type TEXT NOT NULL,  -- "deposit", "transfer", "withdraw"
    nullifier BLOB,
    data BLOB NOT NULL,     -- JSON serialized transaction data
    status TEXT NOT NULL,   -- "pending", "confirmed", "failed"
    block_number INTEGER,
    tx_hash BLOB,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE INDEX idx_transactions_nullifier ON transactions(nullifier);
CREATE INDEX idx_transactions_status ON transactions(status);

-- nullifiers table
CREATE TABLE nullifiers (
    nullifier BLOB PRIMARY KEY,
    created_at INTEGER NOT NULL
);

-- sync_state table
CREATE TABLE sync_state (
    key TEXT PRIMARY KEY,
    value BLOB NOT NULL
);
-- Keys: "last_synced_block", "current_root"
```

### 6.2 veilocity-prover

Handles zero-knowledge proof generation by orchestrating Noir and Barretenberg CLI tools.

#### 6.2.1 NoirProver (`prover.rs`)

```rust
pub struct NoirProver {
    circuits_path: PathBuf,
    bb_path: PathBuf,
    nargo_path: PathBuf,
}

impl NoirProver {
    pub async fn prove(
        &self,
        circuit_type: CircuitType,
        witness: impl Serialize
    ) -> Result<ProofData> {
        // 1. Write witness to Prover.toml
        let prover_toml = self.circuits_path.join("Prover.toml");
        let witness_str = toml::to_string(&witness)?;
        fs::write(&prover_toml, witness_str)?;

        // 2. Execute nargo to generate witness
        let output = Command::new(&self.nargo_path)
            .current_dir(&self.circuits_path)
            .args(["execute", &circuit_type.function_name()])
            .output()
            .await?;

        if !output.status.success() {
            return Err(ProverError::NargoExecution(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // 3. Generate proof using Barretenberg
        let witness_path = self.circuits_path
            .join("target")
            .join(format!("{}.gz", circuit_type.function_name()));

        let circuit_path = self.circuits_path
            .join("target")
            .join("veilocity_circuits.json");

        let proof_path = self.circuits_path.join("target/proof");

        let output = Command::new(&self.bb_path)
            .args([
                "prove",
                "-b", circuit_path.to_str().unwrap(),
                "-w", witness_path.to_str().unwrap(),
                "-o", proof_path.to_str().unwrap(),
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(ProverError::BBProve(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // 4. Read proof bytes
        let proof_bytes = fs::read(&proof_path)?;

        Ok(ProofData {
            proof: proof_bytes,
            circuit_type,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CircuitType {
    Deposit,
    Withdraw,
    Transfer,
}

impl CircuitType {
    fn function_name(&self) -> &str {
        match self {
            Self::Deposit => "verify_deposit",
            Self::Withdraw => "verify_withdraw",
            Self::Transfer => "verify_transfer",
        }
    }
}
```

#### 6.2.2 Witness Structures (`witness.rs`)

```rust
#[derive(Debug, Serialize)]
pub struct DepositWitness {
    pub commitment: String,  // 0x-prefixed hex
    pub amount: String,
    pub secret: String,
}

#[derive(Debug, Serialize)]
pub struct WithdrawWitness {
    // Public inputs
    pub state_root: String,
    pub nullifier: String,
    pub amount: String,
    pub recipient: String,

    // Private inputs
    pub secret: String,
    pub balance: String,
    pub nonce: String,
    pub index: String,
    pub path: Vec<String>,  // 20 elements
}

#[derive(Debug, Serialize)]
pub struct TransferWitness {
    // Public inputs
    pub old_state_root: String,
    pub new_state_root: String,
    pub nullifier: String,

    // Sender private inputs
    pub sender_secret: String,
    pub sender_balance: String,
    pub sender_nonce: String,
    pub sender_index: String,
    pub sender_path: Vec<String>,

    // Recipient private inputs
    pub recipient_pubkey: String,
    pub recipient_balance: String,
    pub recipient_nonce: String,
    pub recipient_index: String,
    pub recipient_path: Vec<String>,

    // Amount
    pub amount: String,
}

impl DepositWitness {
    pub fn new(
        commitment: [u8; 32],
        amount: u128,
        secret: [u8; 32]
    ) -> Self {
        Self {
            commitment: format!("0x{}", hex::encode(commitment)),
            amount: format!("0x{:064x}", amount),
            secret: format!("0x{}", hex::encode(secret)),
        }
    }
}

// Similar constructors for other witness types...
```

### 6.3 veilocity-contracts

Handles interaction with on-chain contracts via alloy-rs.

#### 6.3.1 VaultClient (`vault.rs`)

```rust
use alloy::{
    network::EthereumWallet,
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    primitives::{Address, U256, Bytes, B256},
};

pub struct VaultClient {
    provider: Box<dyn Provider>,
    vault_address: Address,
    wallet: Option<EthereumWallet>,
}

impl VaultClient {
    pub async fn new(
        rpc_url: &str,
        vault_address: Address,
        private_key: Option<&str>
    ) -> Result<Self> {
        let provider = ProviderBuilder::new()
            .on_builtin(rpc_url)
            .await?;

        let wallet = private_key.map(|pk| {
            let signer: PrivateKeySigner = pk.parse().unwrap();
            EthereumWallet::from(signer)
        });

        Ok(Self {
            provider: Box::new(provider),
            vault_address,
            wallet,
        })
    }

    // Query current Merkle root
    pub async fn current_root(&self) -> Result<B256> {
        let vault = IVeilocityVault::new(self.vault_address, &self.provider);
        Ok(vault.currentRoot().call().await?._0)
    }

    // Query deposit count (next leaf index)
    pub async fn deposit_count(&self) -> Result<u64> {
        let vault = IVeilocityVault::new(self.vault_address, &self.provider);
        Ok(vault.depositCount().call().await?._0.try_into()?)
    }

    // Check if nullifier has been used
    pub async fn is_nullifier_used(&self, nullifier: B256) -> Result<bool> {
        let vault = IVeilocityVault::new(self.vault_address, &self.provider);
        Ok(vault.nullifiers(nullifier).call().await?._0)
    }

    // Send deposit transaction
    pub async fn deposit(&self, commitment: B256, amount: U256) -> Result<B256> {
        let wallet = self.wallet.as_ref()
            .ok_or(ContractError::NoWallet)?;

        let provider = ProviderBuilder::new()
            .wallet(wallet.clone())
            .on_provider(&self.provider);

        let vault = IVeilocityVault::new(self.vault_address, &provider);

        let tx = vault.deposit(commitment)
            .value(amount)
            .send()
            .await?;

        let receipt = tx.get_receipt().await?;
        Ok(receipt.transaction_hash)
    }

    // Send withdrawal transaction
    pub async fn withdraw(
        &self,
        nullifier: B256,
        recipient: Address,
        amount: U256,
        root: B256,
        proof: Bytes,
    ) -> Result<B256> {
        let wallet = self.wallet.as_ref()
            .ok_or(ContractError::NoWallet)?;

        let provider = ProviderBuilder::new()
            .wallet(wallet.clone())
            .on_provider(&self.provider);

        let vault = IVeilocityVault::new(self.vault_address, &provider);

        let tx = vault.withdraw(nullifier, recipient, amount, root, proof)
            .send()
            .await?;

        let receipt = tx.get_receipt().await?;
        Ok(receipt.transaction_hash)
    }
}
```

#### 6.3.2 Event Listener (`events.rs`)

```rust
use alloy::primitives::{B256, U256, Log};

#[derive(Debug, Clone)]
pub struct DepositEvent {
    pub commitment: B256,
    pub amount: U256,
    pub leaf_index: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct WithdrawalEvent {
    pub nullifier: B256,
    pub recipient: Address,
    pub amount: U256,
}

pub async fn listen_for_deposits(
    client: &VaultClient,
    from_block: u64,
) -> Result<Vec<DepositEvent>> {
    let filter = Filter::new()
        .address(client.vault_address)
        .event("Deposit(bytes32,uint256,uint256,uint256)")
        .from_block(from_block);

    let logs = client.provider.get_logs(&filter).await?;

    logs.iter()
        .map(|log| parse_deposit_log(log))
        .collect()
}

fn parse_deposit_log(log: &Log) -> Result<DepositEvent> {
    // Parse indexed and non-indexed parameters from log
    let commitment = B256::from_slice(&log.topics[1].0);
    let data = &log.data;

    let amount = U256::from_be_slice(&data[0..32]);
    let leaf_index = U256::from_be_slice(&data[32..64]).to::<u64>();
    let timestamp = U256::from_be_slice(&data[64..96]).to::<u64>();

    Ok(DepositEvent {
        commitment,
        amount,
        leaf_index,
        timestamp,
    })
}
```

### 6.4 veilocity-cli

Command-line interface for user interaction.

#### 6.4.1 Main Entry Point (`main.rs`)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "veilocity")]
#[command(about = "Private execution layer CLI for Mantle L2")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "~/.veilocity/config.toml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize wallet and configuration
    Init {
        #[arg(long)]
        force: bool,
    },

    /// Deposit funds into privacy pool
    Deposit {
        /// Amount in MNT (e.g., "1.5")
        amount: String,
    },

    /// Transfer privately to another account
    Transfer {
        /// Recipient's public key (hex)
        recipient: String,
        /// Amount in MNT
        amount: String,
    },

    /// Withdraw funds to public address
    Withdraw {
        /// Amount in MNT
        amount: String,
        /// Destination address (defaults to own address)
        #[arg(long)]
        to: Option<String>,
    },

    /// Check private balance
    Balance,

    /// Sync with on-chain state
    Sync {
        #[arg(long)]
        full: bool,
    },

    /// View transaction history
    History {
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::init();

    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;

    match cli.command {
        Commands::Init { force } => commands::init::run(force).await,
        Commands::Deposit { amount } => commands::deposit::run(&config, &amount).await,
        Commands::Transfer { recipient, amount } => {
            commands::transfer::run(&config, &recipient, &amount).await
        }
        Commands::Withdraw { amount, to } => {
            commands::withdraw::run(&config, &amount, to.as_deref()).await
        }
        Commands::Balance => commands::balance::run(&config).await,
        Commands::Sync { full } => commands::sync::run(&config, full).await,
        Commands::History { limit } => commands::history::run(&config, limit).await,
    }
}
```

#### 6.4.2 Configuration (`config.rs`)

```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
    pub prover: ProverConfig,
    pub sync: SyncConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub rpc_url: String,
    pub chain_id: u64,
    pub vault_address: String,
    pub verifier_address: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProverConfig {
    pub circuits_path: PathBuf,
    pub threads: Option<usize>,
    pub cache_proofs: bool,
}

#[derive(Debug, Deserialize)]
pub struct SyncConfig {
    pub poll_interval_secs: u64,
    pub confirmations: u64,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub db_path: PathBuf,
    pub encrypted: bool,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
```

**Example config.toml**:

```toml
[network]
rpc_url = "https://rpc.sepolia.mantle.xyz"
chain_id = 5003
vault_address = "0x..."

[prover]
circuits_path = "~/.veilocity/circuits"
threads = 4
cache_proofs = true

[sync]
poll_interval_secs = 12
confirmations = 6

[storage]
db_path = "~/.veilocity/state.db"
encrypted = true
```

### 6.5 veilocity-indexer

Background indexer service that syncs with the chain and provides a REST API.

#### 6.5.1 Indexer State

```rust
/// Current indexer state - served via API
pub struct IndexerState {
    /// Current Merkle root
    pub state_root: [u8; 32],
    /// All deposit commitments (leaves)
    pub leaves: Vec<[u8; 32]>,
    /// All deposits indexed
    pub deposits: Vec<IndexedDeposit>,
    /// All withdrawals indexed
    pub withdrawals: Vec<IndexedWithdrawal>,
    /// Used nullifiers
    pub nullifiers: Vec<[u8; 32]>,
    /// Last synced block
    pub last_block: u64,
    /// On-chain deposit count
    pub deposit_count: u64,
    /// Total value locked
    pub tvl_wei: String,
    /// Is currently syncing
    pub is_syncing: bool,
    /// Sync progress (0-100)
    pub sync_progress: u8,
}
```

#### 6.5.2 REST API Endpoints

```rust
// Health check
GET /health -> { status, is_syncing, sync_progress, last_block }

// Full sync state (main endpoint for CLI)
GET /sync -> {
    state_root,      // Current Merkle root (hex)
    leaves,          // All leaves (hex array)
    nullifiers,      // Used nullifiers (hex array)
    last_block,      // Last synced block
    deposit_count,   // Number of deposits
    tvl_wei,         // Total value locked
    is_syncing,      // Sync status
    sync_progress    // Progress percentage
}

// Deposit history
GET /deposits -> {
    deposits: [{ commitment, amount_wei, amount_mnt, leaf_index, block_number, tx_hash }],
    total: usize
}

// Withdrawal history
GET /withdrawals -> {
    withdrawals: [{ nullifier, recipient, amount_wei, amount_mnt, block_number, tx_hash }],
    total: usize
}
```

#### 6.5.3 Running the Indexer

```bash
# Start the indexer
veilocity-indexer --rpc-url https://rpc.sepolia.mantle.xyz \
                  --vault-address 0x... \
                  --port 8080 \
                  --deployment-block 12345678

# The indexer will:
# 1. Start syncing from deployment block
# 2. Serve REST API on http://localhost:8080
# 3. Poll for new blocks every 12 seconds
```

---

## 7. Smart Contracts (Solidity)

### 7.1 VeilocityVault.sol

The main on-chain contract that holds deposits and processes withdrawals.

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {ReentrancyGuard} from "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {IVerifier} from "./interfaces/IVerifier.sol";

contract VeilocityVault is ReentrancyGuard, Pausable, Ownable {
    // ═══════════════════════════════════════════════════════════════
    //                           CONSTANTS
    // ═══════════════════════════════════════════════════════════════

    uint256 public constant MINIMUM_DEPOSIT = 0.001 ether;
    uint256 public constant ROOT_HISTORY_SIZE = 100;

    // ═══════════════════════════════════════════════════════════════
    //                           STATE
    // ═══════════════════════════════════════════════════════════════

    /// @notice The ZK proof verifier contract
    IVerifier public immutable verifier;

    /// @notice Current Merkle state root
    bytes32 public currentRoot;

    /// @notice Maps root to validity (for historical roots)
    mapping(bytes32 => bool) public rootHistory;

    /// @notice Tracks used nullifiers to prevent double-spending
    mapping(bytes32 => bool) public nullifiers;

    /// @notice Number of deposits (next leaf index)
    uint256 public depositCount;

    /// @notice Total value locked in the contract
    uint256 public totalValueLocked;

    // ═══════════════════════════════════════════════════════════════
    //                           EVENTS
    // ═══════════════════════════════════════════════════════════════

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

    event StateRootUpdated(
        bytes32 indexed oldRoot,
        bytes32 indexed newRoot,
        uint256 batchIndex,
        uint256 timestamp
    );

    // ═══════════════════════════════════════════════════════════════
    //                         CONSTRUCTOR
    // ═══════════════════════════════════════════════════════════════

    constructor(
        IVerifier _verifier,
        bytes32 _initialRoot
    ) Ownable(msg.sender) {
        verifier = _verifier;
        currentRoot = _initialRoot;
        rootHistory[_initialRoot] = true;
    }

    // ═══════════════════════════════════════════════════════════════
    //                      EXTERNAL FUNCTIONS
    // ═══════════════════════════════════════════════════════════════

    /// @notice Deposit MNT into the privacy pool
    /// @param commitment The deposit commitment hash(secret, amount)
    function deposit(bytes32 commitment) external payable whenNotPaused {
        require(msg.value >= MINIMUM_DEPOSIT, "Deposit too small");
        require(commitment != bytes32(0), "Invalid commitment");

        uint256 leafIndex = depositCount;
        depositCount++;
        totalValueLocked += msg.value;

        emit Deposit(commitment, msg.value, leafIndex, block.timestamp);
    }

    /// @notice Withdraw funds with a valid ZK proof
    /// @param nullifier Unique withdrawal identifier
    /// @param recipient Destination address for funds
    /// @param amount Amount to withdraw
    /// @param root Merkle root used in proof
    /// @param proof ZK proof bytes
    function withdraw(
        bytes32 nullifier,
        address payable recipient,
        uint256 amount,
        bytes32 root,
        bytes calldata proof
    ) external nonReentrant whenNotPaused {
        // Validation
        require(!nullifiers[nullifier], "Nullifier already used");
        require(rootHistory[root], "Invalid root");
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be positive");
        require(amount <= totalValueLocked, "Insufficient TVL");

        // Construct public inputs for verifier
        bytes32[] memory publicInputs = new bytes32[](4);
        publicInputs[0] = root;
        publicInputs[1] = nullifier;
        publicInputs[2] = bytes32(amount);
        publicInputs[3] = bytes32(uint256(uint160(recipient)));

        // Verify ZK proof
        require(
            verifier.verify(proof, publicInputs),
            "Invalid proof"
        );

        // Update state
        nullifiers[nullifier] = true;
        totalValueLocked -= amount;

        // Transfer funds
        (bool success, ) = recipient.call{value: amount}("");
        require(success, "Transfer failed");

        emit Withdrawal(nullifier, recipient, amount);
    }

    /// @notice Update state root (sequencer/owner only for MVP)
    /// @param newRoot New Merkle state root
    /// @param proof Validity proof for state transition
    function updateStateRoot(
        bytes32 newRoot,
        bytes calldata proof
    ) external onlyOwner {
        bytes32 oldRoot = currentRoot;

        // TODO: Verify state transition proof in production
        // For MVP, owner is trusted

        currentRoot = newRoot;
        rootHistory[newRoot] = true;

        emit StateRootUpdated(oldRoot, newRoot, depositCount, block.timestamp);
    }

    // ═══════════════════════════════════════════════════════════════
    //                      VIEW FUNCTIONS
    // ═══════════════════════════════════════════════════════════════

    /// @notice Check if a root is valid (current or historical)
    function isValidRoot(bytes32 root) external view returns (bool) {
        return rootHistory[root];
    }

    /// @notice Check if a nullifier has been used
    function isNullifierUsed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }

    // ═══════════════════════════════════════════════════════════════
    //                      ADMIN FUNCTIONS
    // ═══════════════════════════════════════════════════════════════

    /// @notice Pause the contract in case of emergency
    function pause() external onlyOwner {
        _pause();
    }

    /// @notice Unpause the contract
    function unpause() external onlyOwner {
        _unpause();
    }
}
```

### 7.2 IVerifier Interface

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

interface IVerifier {
    /// @notice Verify a ZK proof
    /// @param proof The proof bytes
    /// @param publicInputs Array of public inputs
    /// @return True if proof is valid
    function verify(
        bytes calldata proof,
        bytes32[] calldata publicInputs
    ) external view returns (bool);
}
```

### 7.3 MockVerifier (Testing)

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {IVerifier} from "../interfaces/IVerifier.sol";

/// @notice Mock verifier that always returns true (for testing only)
contract MockVerifier is IVerifier {
    function verify(
        bytes calldata,
        bytes32[] calldata
    ) external pure override returns (bool) {
        return true;
    }
}
```

### 7.4 HonkVerifier (Production)

The production verifier is auto-generated from Noir circuits using Barretenberg. It implements the Honk proving system.

```solidity
// Generated by: bb write_solidity_verifier
// File: contracts/src/HonkVerifier.sol

// Key parameters from the generated verifier:
uint256 constant N = 65536;                    // Circuit size
uint256 constant LOG_N = 16;                   // Log of circuit size
uint256 constant NUMBER_OF_PUBLIC_INPUTS = 20; // Public inputs count

// The verifier includes:
// - Verification key with G1 points for all constraint polynomials
// - Pairing-based proof verification
// - Poseidon2 gates for efficient hashing
```

### 7.5 Deployment Script

```solidity
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import {Script} from "forge-std/Script.sol";
import {VeilocityVault} from "../src/VeilocityVault.sol";
import {MockVerifier} from "../src/mocks/MockVerifier.sol";
import {IVerifier} from "../src/interfaces/IVerifier.sol";

contract Deploy is Script {
    // Empty Merkle tree root (depth 20)
    bytes32 constant INITIAL_ROOT =
        0x2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address verifierAddress = vm.envOr("VERIFIER_ADDRESS", address(0));

        vm.startBroadcast(deployerPrivateKey);

        // Deploy verifier if not provided
        IVerifier verifier;
        if (verifierAddress == address(0)) {
            MockVerifier mockVerifier = new MockVerifier();
            verifier = IVerifier(address(mockVerifier));
        } else {
            verifier = IVerifier(verifierAddress);
        }

        // Deploy vault
        VeilocityVault vault = new VeilocityVault(verifier, INITIAL_ROOT);

        vm.stopBroadcast();

        // Log addresses
        console.log("Verifier:", address(verifier));
        console.log("Vault:", address(vault));
    }
}
```

### 7.6 Contract Security Features

| Feature | Implementation | Protection Against |
|---------|----------------|-------------------|
| **ReentrancyGuard** | OpenZeppelin modifier | Reentrancy attacks on withdraw |
| **Pausable** | Emergency pause mechanism | Exploit mitigation |
| **Nullifier tracking** | `mapping(bytes32 => bool)` | Double-spending |
| **Root history** | Valid root whitelist | Invalid state proofs |
| **Zero-address check** | `require(recipient != 0)` | Fund loss |
| **Amount validation** | `require(amount <= TVL)` | Overdraw |

---

## 8. Data Flows & Transaction Lifecycle

### 8.1 Deposit Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         DEPOSIT FLOW                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  USER                     CLI                        ON-CHAIN           │
│                                                                         │
│  $ veilocity deposit 1.0                                               │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐                        │
│  │ 1. Load account secret from wallet         │                        │
│  │ 2. Parse amount: 1.0 MNT = 1e18 wei       │                        │
│  │ 3. Compute commitment:                     │                        │
│  │    commitment = poseidon(secret, amount)   │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐    ┌─────────────────┐ │
│  │ 4. Build transaction:                      │───▶│ VeilocityVault  │ │
│  │    vault.deposit(commitment)               │    │    .deposit()   │ │
│  │    value: 1.0 MNT                         │    └─────────────────┘ │
│  └────────────────────────────────────────────┘             │          │
│                                                              ▼          │
│                                                    ┌─────────────────┐ │
│                                                    │ Emit Deposit(   │ │
│                                                    │   commitment,   │ │
│                                                    │   amount,       │ │
│       ┌──────────────────────────────────────────◀│   leafIndex,    │ │
│       │                                            │   timestamp)    │ │
│       ▼                                            └─────────────────┘ │
│  ┌────────────────────────────────────────────┐                        │
│  │ 5. Listen for event confirmation           │                        │
│  │ 6. Parse leafIndex from event              │                        │
│  │ 7. Insert commitment into local tree       │                        │
│  │ 8. Credit balance to local account         │                        │
│  │ 9. Store transaction in history            │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  "Deposit confirmed. New balance: 1.0 MNT"                             │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Private Transfer Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      PRIVATE TRANSFER FLOW                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  USER                     CLI                         LOCAL STATE       │
│                                                                         │
│  $ veilocity transfer 0xABC... 0.5                                     │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐    ┌─────────────────┐ │
│  │ 1. Parse recipient pubkey                  │    │   StateManager  │ │
│  │ 2. Parse amount                            │◀──▶│                 │ │
│  │ 3. Load sender account & Merkle proof      │    │   MerkleTree    │ │
│  │ 4. Load/create recipient account           │    └─────────────────┘ │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐                        │
│  │ 5. Build TransferWitness:                  │                        │
│  │    • old_state_root (current root)         │                        │
│  │    • sender_secret, balance, nonce, path   │                        │
│  │    • recipient_pubkey, balance, nonce, path│                        │
│  │    • amount                                │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐    ┌─────────────────┐ │
│  │ 6. Generate ZK proof:                      │    │   NoirProver    │ │
│  │    a. Write Prover.toml                    │───▶│                 │ │
│  │    b. nargo execute                        │    │   nargo + bb    │ │
│  │    c. bb prove                             │◀───│                 │ │
│  │    d. Read proof bytes                     │    └─────────────────┘ │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐    ┌─────────────────┐ │
│  │ 7. Update local state:                     │    │   StateManager  │ │
│  │    • Debit sender (balance -= amount)      │───▶│                 │ │
│  │    • Credit recipient (balance += amount)  │    │   SQLite + Tree │ │
│  │    • Increment sender nonce                │    └─────────────────┘ │
│  │    • Record nullifier as used              │                        │
│  │    • Update Merkle tree leaves             │                        │
│  │    • Log transaction                       │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  "Transfer complete. New balance: 0.5 MNT"                             │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ NOTE: No on-chain transaction!                                   │   │
│  │       Transfer is purely off-chain with local state update.     │   │
│  │       The proof can be verified later if needed.                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Withdrawal Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        WITHDRAWAL FLOW                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  USER                     CLI                        ON-CHAIN           │
│                                                                         │
│  $ veilocity withdraw 0.3 --to 0xDEF...                                │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐    ┌─────────────────┐ │
│  │ 1. Load account from StateManager          │◀──▶│   StateManager  │ │
│  │ 2. Get Merkle proof for account            │    └─────────────────┘ │
│  │ 3. Compute nullifier:                      │                        │
│  │    nullifier = hash(secret, index, nonce)  │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐                        │
│  │ 4. Build WithdrawWitness:                  │                        │
│  │    • state_root (public)                   │                        │
│  │    • nullifier (public)                    │                        │
│  │    • amount (public)                       │                        │
│  │    • recipient (public)                    │                        │
│  │    • secret, balance, nonce, path (private)│                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐    ┌─────────────────┐ │
│  │ 5. Generate ZK proof                       │───▶│   NoirProver    │ │
│  │    (proves balance >= amount, ownership)   │◀───│                 │ │
│  └────────────────────────────────────────────┘    └─────────────────┘ │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐    ┌─────────────────┐ │
│  │ 6. Submit withdrawal transaction:          │───▶│ VeilocityVault  │ │
│  │    vault.withdraw(                         │    │   .withdraw()   │ │
│  │      nullifier,                            │    │                 │ │
│  │      recipient,                            │    │ Verifier.verify │ │
│  │      amount,                               │    │   ↓             │ │
│  │      root,                                 │    │ MNT transfer    │ │
│  │      proof                                 │    └─────────────────┘ │
│  │    )                                       │             │          │
│  └────────────────────────────────────────────┘             ▼          │
│                                                    ┌─────────────────┐ │
│                                                    │ Emit Withdrawal │ │
│       ┌──────────────────────────────────────────◀│ (nullifier,     │ │
│       │                                            │  recipient,     │ │
│       ▼                                            │  amount)        │ │
│  ┌────────────────────────────────────────────┐   └─────────────────┘ │
│  │ 7. Update local state:                     │                        │
│  │    • Debit balance                         │                        │
│  │    • Mark nullifier used                   │                        │
│  │    • Log transaction                       │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  "Withdrawal complete. 0.3 MNT sent to 0xDEF..."                       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 9. State Management

### 9.1 State Components

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       LOCAL STATE                                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                        SQLite Database                           │   │
│  │                                                                  │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │   │
│  │  │  accounts   │  │transactions │  │ nullifiers  │             │   │
│  │  │  ─────────  │  │ ─────────── │  │ ────────────│             │   │
│  │  │ • pubkey    │  │ • tx_type   │  │ • nullifier │             │   │
│  │  │ • balance*  │  │ • nullifier │  │ • timestamp │             │   │
│  │  │ • nonce     │  │ • data      │  └─────────────┘             │   │
│  │  │ • leaf_index│  │ • status    │                               │   │
│  │  └─────────────┘  │ • tx_hash   │  ┌─────────────┐             │   │
│  │                   └─────────────┘  │ sync_state  │             │   │
│  │  * balance is encrypted locally    │ ────────────│             │   │
│  │                                    │ • last_block│             │   │
│  │                                    │ • root      │             │   │
│  │                                    └─────────────┘             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                     Merkle Tree (redb)                           │   │
│  │                                                                  │   │
│  │  ┌──────────────────────────────────────────────────────────┐   │   │
│  │  │ Key: (level, index)    Value: node_hash                  │   │   │
│  │  │                                                          │   │   │
│  │  │ (0, 0) → leaf_0      (0, 1) → leaf_1    ...             │   │   │
│  │  │ (1, 0) → H(leaf_0, leaf_1)   (1, 1) → H(leaf_2, leaf_3) │   │   │
│  │  │ ...                                                      │   │   │
│  │  │ (20, 0) → ROOT                                          │   │   │
│  │  └──────────────────────────────────────────────────────────┘   │   │
│  │                                                                  │   │
│  │  Sparse storage: only non-empty nodes stored                    │   │
│  │  Missing nodes use precomputed empty_hash[level]                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                       ON-CHAIN STATE                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  VeilocityVault Contract                                               │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ currentRoot: bytes32          // Latest Merkle root             │   │
│  │ rootHistory: mapping          // Valid historical roots         │   │
│  │ nullifiers: mapping           // Used nullifiers (spent)        │   │
│  │ depositCount: uint256         // Number of deposits (leaves)    │   │
│  │ totalValueLocked: uint256     // MNT held in contract          │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 9.2 State Synchronization

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      STATE SYNC PROCESS                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  $ veilocity sync                                                       │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐                        │
│  │ 1. Load last_synced_block from sync_state  │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐    ┌─────────────────┐ │
│  │ 2. Query RPC for events:                   │───▶│   Mantle RPC    │ │
│  │    Filter: Deposit events                  │    │                 │ │
│  │    From: last_synced_block + 1             │◀───│   eth_getLogs   │ │
│  │    To: latest - confirmations              │    └─────────────────┘ │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐                        │
│  │ 3. For each Deposit event:                 │                        │
│  │    a. Parse commitment, amount, leafIndex  │                        │
│  │    b. Check if this is our deposit         │                        │
│  │       (can we derive commitment?)          │                        │
│  │    c. If ours:                             │                        │
│  │       - Insert into local tree             │                        │
│  │       - Update account balance             │                        │
│  │    d. If not ours:                         │                        │
│  │       - Insert empty/dummy leaf            │                        │
│  │       - (Maintains tree structure)         │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  ┌────────────────────────────────────────────┐                        │
│  │ 4. Update sync_state:                      │                        │
│  │    last_synced_block = latest - confs      │                        │
│  └────────────────────────────────────────────┘                        │
│       │                                                                 │
│       ▼                                                                 │
│  "Synced to block 12345. Found 3 deposits."                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 10. Build, Test & Deployment

### 10.1 Building the Project

```bash
# Clone the repository
git clone https://github.com/yourorg/veilocity.git
cd veilocity

# Build Rust workspace
cargo build --release

# Compile Noir circuits
cd circuits
nargo compile

# Build Solidity contracts
cd ../contracts
forge build
```

### 10.2 Running Tests

```bash
# Rust unit tests
cargo test

# Noir circuit tests
cd circuits
nargo test

# Solidity tests (with gas report)
cd contracts
forge test -vvv --gas-report
```

### 10.3 Generating Solidity Verifier

```bash
cd circuits

# Compile circuits
nargo compile

# Generate proving/verification keys
bb write_vk -b ./target/veilocity_circuits.json -o ./target/vk

# Generate Solidity verifier
bb write_solidity_verifier -k ./target/vk -o ../contracts/src/verifiers/UltraVerifier.sol
```

### 10.4 Deploying to Mantle

```bash
# Set environment variables
export PRIVATE_KEY="0x..."

# Deploy to Mantle Sepolia (testnet)
cd contracts
forge script script/Deploy.s.sol \
    --broadcast \
    --rpc-url https://rpc.sepolia.mantle.xyz \
    --verify

# Deploy to Mantle Mainnet
forge script script/Deploy.s.sol \
    --broadcast \
    --rpc-url https://rpc.mantle.xyz \
    --verify
```

### 10.5 Network Configuration

| Network | RPC URL | Chain ID | Explorer |
|---------|---------|----------|----------|
| Mantle Sepolia | `https://rpc.sepolia.mantle.xyz` | 5003 | `https://explorer.sepolia.mantle.xyz` |
| Mantle Mainnet | `https://rpc.mantle.xyz` | 5000 | `https://explorer.mantle.xyz` |

---

## 11. Security Model

### 11.1 Threat Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         THREAT MODEL                                    │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  WHAT WE PROTECT:                                                       │
│  ────────────────                                                       │
│  • Account balances (hidden on-chain)                                  │
│  • Transfer amounts (private, off-chain)                               │
│  • Transfer recipients (private, off-chain)                            │
│  • Transaction graph (who sends to whom)                               │
│                                                                         │
│  WHAT WE DON'T HIDE:                                                   │
│  ──────────────────                                                    │
│  • Deposit amounts (visible on-chain)                                  │
│  • Withdrawal amounts (visible on-chain)                               │
│  • Withdrawal recipients (visible on-chain)                            │
│  • Total value locked (public contract state)                          │
│  • Number of deposits (public counter)                                 │
│                                                                         │
│  ADVERSARY CAPABILITIES:                                               │
│  ────────────────────────                                              │
│  • Full view of blockchain (all transactions)                          │
│  • Statistical analysis of deposit/withdrawal patterns                 │
│  • Timing correlation attacks                                          │
│                                                                         │
│  SECURITY GUARANTEES:                                                  │
│  ────────────────────                                                  │
│  ✓ Cannot forge proofs without secret                                  │
│  ✓ Cannot double-spend (nullifier tracking)                            │
│  ✓ Cannot withdraw more than deposited                                 │
│  ✓ Cannot front-run withdrawals (recipient binding)                    │
│  ✓ Cannot link deposits to withdrawals (privacy set)                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 11.2 Security Properties

| Property | Mechanism | Verification |
|----------|-----------|--------------|
| **Soundness** | ZK proof system | Mathematically proven |
| **Double-spend prevention** | Nullifier set | On-chain tracking |
| **Balance integrity** | Circuit constraints | `balance >= amount` check |
| **Ownership proof** | Secret knowledge | `pubkey = hash(secret)` |
| **Replay protection** | Nonce increment | Part of nullifier |
| **Front-running protection** | Recipient binding | Recipient in public inputs |

### 11.3 Known Limitations (MVP)

| Limitation | Description | Mitigation Path |
|------------|-------------|-----------------|
| Centralized sequencer | Owner controls state updates | Decentralize with consensus |
| Mock verifier | Testing only, accepts all proofs | Deploy real UltraVerifier |
| Single-node execution | No redundancy | Distributed sequencer network |
| Basic encryption | XOR for demo | Use proper AEAD (AES-GCM) |

### 11.4 Audit Checklist

**Circuits**:
- [ ] Constraint completeness (no underconstrained values)
- [ ] Range checks on balances
- [ ] Field overflow protection
- [ ] Merkle path verification correctness

**Contracts**:
- [ ] Reentrancy protection
- [ ] Access control verification
- [ ] Integer overflow (Solidity 0.8+ has built-in checks)
- [ ] Proper event emission
- [ ] Emergency pause functionality

**Rust Implementation**:
- [ ] Secret zeroization on drop
- [ ] Constant-time comparisons
- [ ] Input validation
- [ ] Error handling (no panics in production paths)

---

## 12. Design Decisions & Rationale

### 12.1 Why Noir?

| Alternative | Reason Not Chosen |
|-------------|-------------------|
| Circom | Older, less ergonomic DSL |
| Halo2 | More complex, no auto-verifier generation |
| Cairo | StarkNet-specific, different proof system |
| Miden | Less mature ecosystem |

**Noir advantages**:
- Rust-like syntax
- Integrated with Barretenberg (battle-tested prover)
- Automatic Solidity verifier generation
- Universal trusted setup

### 12.2 Why Poseidon?

| Alternative | Reason Not Chosen |
|-------------|-------------------|
| SHA-256 | ~10x more constraints in ZK |
| Pedersen | Requires elliptic curve operations |
| MiMC | Less studied, fewer rounds |

**Poseidon advantages**:
- ~8x more efficient than SHA-256 in circuits
- Well-studied security
- Native field arithmetic

### 12.3 Why Depth 20 Merkle Tree?

| Depth | Capacity | Trade-off |
|-------|----------|-----------|
| 16 | ~65K | Too small for production |
| 20 | ~1M | Good balance |
| 24 | ~16M | Longer proofs, slower |

### 12.4 Why alloy-rs over ethers-rs?

| Aspect | alloy | ethers |
|--------|-------|--------|
| Maintenance | Active | Deprecated |
| Typing | Better generics | More runtime checks |
| Performance | Faster | Slower |
| Ecosystem | Growing | Stable but legacy |

### 12.5 Why SQLite + redb?

| Component | Storage | Reason |
|-----------|---------|--------|
| Accounts, TXs | SQLite | Relational queries, ACID |
| Merkle nodes | redb | Fast key-value, embeddable |

**Why not just SQLite?**
- redb is faster for simple key-value lookups
- Merkle tree nodes don't need relational features
- Separation of concerns

---

## 13. Learning Resources

### 13.1 Zero-Knowledge Proofs

| Resource | Type | Link |
|----------|------|------|
| ZK Whiteboard Sessions | Video Series | [youtube.com/zkwhiteboard](https://www.youtube.com/c/zkwhiteboard) |
| ZK Book | Online Book | [zk-learning.org](https://zk-learning.org) |
| Proofs, Arguments, and Zero-Knowledge | Textbook | [people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf](https://people.cs.georgetown.edu/jthaler/ProofsArgsAndZK.pdf) |
| Vitalik's ZK-SNARKs Blog | Blog | [vitalik.ca/general/2021/01/26/snarks.html](https://vitalik.ca/general/2021/01/26/snarks.html) |

### 13.2 Noir Language

| Resource | Type | Link |
|----------|------|------|
| Noir Documentation | Official Docs | [noir-lang.org/docs](https://noir-lang.org/docs) |
| Noir Examples | GitHub | [github.com/noir-lang/noir/tree/master/examples](https://github.com/noir-lang/noir/tree/master/examples) |
| Noir by Example | Tutorial | [noir-by-example.org](https://noir-by-example.org) |

### 13.3 Cryptography

| Resource | Type | Link |
|----------|------|------|
| BN254 Curve | Paper | [eprint.iacr.org/2010/354](https://eprint.iacr.org/2010/354.pdf) |
| Poseidon Hash | Paper | [eprint.iacr.org/2019/458](https://eprint.iacr.org/2019/458.pdf) |
| Merkle Trees | Wikipedia | [en.wikipedia.org/wiki/Merkle_tree](https://en.wikipedia.org/wiki/Merkle_tree) |

### 13.4 Rust Development

| Resource | Type | Link |
|----------|------|------|
| The Rust Book | Online Book | [doc.rust-lang.org/book](https://doc.rust-lang.org/book/) |
| Rust by Example | Tutorial | [doc.rust-lang.org/rust-by-example](https://doc.rust-lang.org/rust-by-example/) |
| Tokio Tutorial | Async Guide | [tokio.rs/tokio/tutorial](https://tokio.rs/tokio/tutorial) |

### 13.5 Solidity & Foundry

| Resource | Type | Link |
|----------|------|------|
| Solidity Docs | Official Docs | [docs.soliditylang.org](https://docs.soliditylang.org/) |
| Foundry Book | Official Docs | [book.getfoundry.sh](https://book.getfoundry.sh/) |
| OpenZeppelin Contracts | Library | [docs.openzeppelin.com/contracts](https://docs.openzeppelin.com/contracts/) |

### 13.6 Mantle L2

| Resource | Type | Link |
|----------|------|------|
| Mantle Documentation | Official Docs | [docs.mantle.xyz](https://docs.mantle.xyz/) |
| Mantle Network | Website | [mantle.xyz](https://mantle.xyz/) |

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **BN254** | A pairing-friendly elliptic curve used for ZK proofs |
| **Commitment** | A cryptographic hash that binds a value without revealing it |
| **Constraint** | A mathematical equation that must be satisfied in a ZK circuit |
| **Leaf** | A terminal node in a Merkle tree, representing an account |
| **Merkle Proof** | A path of sibling hashes proving leaf membership |
| **Noir** | A DSL for writing zero-knowledge circuits |
| **Nullifier** | A unique identifier that prevents double-spending |
| **Poseidon** | A ZK-friendly hash function operating on field elements |
| **Public Input** | Data visible to verifiers (on-chain) |
| **Private Input** | Data known only to the prover (off-chain) |
| **Sequencer** | Entity that orders and batches transactions |
| **State Root** | The Merkle root representing the entire account state |
| **TVL** | Total Value Locked in the contract |
| **UltraPlonk** | A SNARK proving system used by Barretenberg |
| **Witness** | The set of all inputs (public + private) to a circuit |

---

## Appendix B: Quick Reference

### CLI Commands

```bash
veilocity init                        # Initialize wallet (alias: i)
veilocity deposit <amount>            # Deposit MNT (alias: d, dep)
veilocity transfer <pubkey> <amount>  # Private transfer (alias: t, send)
veilocity withdraw <amount> [--to]    # Withdraw to address (alias: w)
veilocity balance                     # Check balance (alias: b, bal)
veilocity sync                        # Sync with chain (alias: s)
veilocity history                     # View transactions (alias: h, hist)
veilocity config                      # View configuration (alias: cfg)
veilocity config set <key> <value>    # Set configuration value
```

### Key File Locations

```
~/.veilocity/
├── config.toml          # Configuration
├── wallet.json          # Encrypted wallet
├── state.db             # SQLite database
├── merkle.redb          # Merkle tree storage
└── circuits/            # Compiled circuits
```

### Environment Variables

```bash
VEILOCITY_RPC_URL=https://rpc.sepolia.mantle.xyz
VEILOCITY_VAULT_ADDRESS=0x...
VEILOCITY_PRIVATE_KEY=0x...  # For deployments only
```

---

*Document Version: 2.0.0*
*Last Updated: December 2024*
*Project: Veilocity v0.1.4 - Private Execution Layer on Mantle L2*
