# Veilocity Documentation

> **Private Execution Layer for Mantle**

---

## Table of Contents

- [Introduction](#introduction)
- [Getting Started](#getting-started)
  - [Installation](#installation)
  - [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
  - [Private Execution Layer](#private-execution-layer)
  - [Zero-Knowledge Proofs](#zero-knowledge-proofs)
  - [State Management](#state-management)
- [CLI Reference](#cli-reference)
  - [init](#init)
  - [deposit](#deposit)
  - [transfer](#transfer)
  - [withdraw](#withdraw)
  - [balance](#balance)
  - [sync](#sync)
  - [history](#history)
  - [config](#config)
- [Architecture](#architecture)
  - [System Overview](#system-overview)
  - [Components](#components)
  - [Data Flow](#data-flow)
- [Smart Contracts](#smart-contracts)
  - [VeilocityVault](#veilocityvault)
  - [Contract Addresses](#contract-addresses)
- [ZK Circuits](#zk-circuits)
  - [Deposit Circuit](#deposit-circuit)
  - [Withdrawal Circuit](#withdrawal-circuit)
  - [Transfer Circuit](#transfer-circuit)
- [Security](#security)
  - [Threat Model](#threat-model)
  - [Privacy Guarantees](#privacy-guarantees)
- [API Reference](#api-reference)
- [FAQ](#faq)

---

## Introduction

**Veilocity** is a private execution layer that enables confidential transactions on Mantle L2 using zero-knowledge proofs.

### What is Veilocity?

Veilocity allows users to:
- **Deposit** MNT into a shielded pool
- **Transfer** privately between users (hidden amounts, hidden balances)
- **Withdraw** back to Mantle with ZK proof verification

All private operations execute off-chain while settlement happens securely on Mantle.

### Key Features

| Feature | Description |
|---------|-------------|
| **Private Balances** | Your balance is never revealed on-chain |
| **Private Transfers** | Send MNT without revealing amounts or recipients |
| **ZK Proofs** | Cryptographic guarantees without trusted parties |
| **Mantle Settlement** | Final state anchored on Mantle L2 |
| **Low Fees** | Leverage Mantle's efficient L2 transactions |

---

## Getting Started

### Installation

#### Prerequisites

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Foundry**: `curl -L https://foundry.paradigm.xyz | bash && foundryup`
- **Noir** (optional, for circuit development): `curl -L https://raw.githubusercontent.com/noir-lang/noirup/refs/heads/main/install | bash && noirup`

#### Build from Source

```bash
# Clone the repository
git clone https://github.com/abhirupinspace/veilocity
cd veilocity

# Build the CLI
cargo build --release

# The binary is at ./target/release/veilocity
```

#### Verify Installation

```bash
./target/release/veilocity --version
# veilocity 0.1.4
```

### Quick Start

#### 1. Initialize Your Wallet

```bash
veilocity init
```

This creates a new wallet with:
- A 12-word recovery phrase (save this securely!)
- An encrypted keystore at `~/.veilocity/`
- A Veilocity public key for receiving private transfers

#### 2. Deposit MNT

```bash
veilocity deposit 1.0
```

This:
- Generates a deposit commitment
- Sends MNT to the VeilocityVault contract
- Credits your private balance

#### 3. Check Your Balance

```bash
veilocity balance
```

#### 4. Send a Private Transfer

```bash
veilocity transfer 0x<recipient_pubkey> 0.5
```

#### 5. Withdraw to Mantle

```bash
veilocity withdraw 0.5
```

---

## Core Concepts

### Private Execution Layer

Veilocity is **not** a new blockchain. It's an execution layer that:

```
┌─────────────────────────────────────────┐
│           Applications                   │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│      Veilocity (Private Execution)       │
│  • Execute private transactions          │
│  • Generate ZK proofs                    │
│  • Maintain encrypted state              │
└────────────────┬────────────────────────┘
                 │ ZK Proofs + State Roots
┌────────────────▼────────────────────────┐
│         Mantle L2 (Settlement)           │
│  • Verify proofs on-chain               │
│  • Custody funds in vault               │
│  • Anchor state roots                   │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│         Ethereum L1 (Security)           │
└─────────────────────────────────────────┘
```

### Zero-Knowledge Proofs

Veilocity uses **Noir** circuits compiled to **UltraPlonk** proofs:

| Property | What it Proves |
|----------|----------------|
| **Ownership** | You control the account (know the secret) |
| **Sufficient Balance** | You have enough to transfer/withdraw |
| **Valid State Transition** | The new state is correctly computed |
| **No Double-Spend** | Each spend uses a unique nullifier |

**Proof Sizes:**
- Deposit: ~2 KB
- Withdrawal: ~2 KB
- Transfer: ~2 KB

### State Management

Veilocity maintains a **Merkle tree** of account states:

```
                    Root
                   /    \
                 H01     H23
                /   \   /   \
              L0    L1 L2   L3
              │     │  │    │
           Alice  Bob Carol Empty
```

Each leaf contains: `hash(pubkey, balance, nonce)`

- **Tree Depth**: 20 (supports ~1M accounts)
- **Hash Function**: Poseidon (BN254-compatible)
- **Storage**: Local SQLite database

---

## CLI Reference

### Global Options

```bash
veilocity [OPTIONS] <COMMAND>

OPTIONS:
  -c, --config <PATH>    Config file [default: ~/.veilocity/config.toml]
  -n, --network <NET>    Network: mainnet, sepolia [default: sepolia]
  -v, --verbose          Enable verbose logging
  -h, --help             Print help
  -V, --version          Print version
```

### init

Initialize a new Veilocity wallet.

```bash
veilocity init [--recover]
```

**Options:**
- `--recover`: Recover from an existing seed phrase

**Example:**
```bash
$ veilocity init

Creating new Veilocity wallet...

Your recovery phrase (SAVE THIS SECURELY):
  witch collapse practice feed shame open despair creek road again ice least

Wallet Address: 0x7a3f...
Veilocity PubKey: 0x1234...

Wallet saved to ~/.veilocity/wallet.json
```

### deposit

Deposit MNT from Mantle into Veilocity.

```bash
veilocity deposit <AMOUNT> [--dry-run]
```

**Arguments:**
- `<AMOUNT>`: Amount in MNT (e.g., `1.5`)

**Options:**
- `--dry-run`: Preview without executing

**Example:**
```bash
$ veilocity deposit 1.0

═══ Shield Deposit ═══

  ↓ 1.000000 MNT  (entering shielded pool)

  Commitment: 0x8b2e...f3a1

Submitting transaction...
  TX Hash: 0xabc123...

✓ Deposit confirmed at block 12345
✓ New private balance: 1.000000 MNT
```

### transfer

Send a private transfer to another Veilocity user.

```bash
veilocity transfer <RECIPIENT> <AMOUNT> [--dry-run]
```

**Arguments:**
- `<RECIPIENT>`: Recipient's Veilocity public key (0x...)
- `<AMOUNT>`: Amount in MNT

**Example:**
```bash
$ veilocity transfer 0x8b2e...f3a1 0.5

═══ Private Transfer ═══

  ◈ 0.500000 MNT  (shielded transaction)

  To: 0x8b2e...f3a1

Generating ZK proof...
  [████████████████████] 100%

✓ Transfer complete
✓ New balance: 0.500000 MNT
```

### withdraw

Withdraw MNT from Veilocity back to Mantle.

```bash
veilocity withdraw <AMOUNT> [--to <ADDRESS>] [--dry-run]
```

**Arguments:**
- `<AMOUNT>`: Amount in MNT

**Options:**
- `--to <ADDRESS>`: Destination address (defaults to your wallet)
- `--dry-run`: Preview without executing

**Example:**
```bash
$ veilocity withdraw 0.5

═══ Withdraw ═══

  ↑ 0.500000 MNT  (exiting shielded pool)

  To: 0x7a3f...1234

Generating withdrawal proof...
Submitting transaction...

✓ Withdrawal confirmed
✓ 0.5 MNT sent to 0x7a3f...1234
✓ New private balance: 0.500000 MNT
```

### balance

Display your current private balance.

```bash
veilocity balance
```

**Example:**
```bash
$ veilocity balance

═══ Balance ═══

  ◈ Private Balance

    1.000000 MNT

═══ State Info ═══

  State Root:   0x19df90ec...
  Total Leaves: 16
  Network:      https://rpc.sepolia.mantle.xyz
```

### sync

Synchronize local state with on-chain events.

```bash
veilocity sync
```

This fetches:
- New deposit events
- Withdrawal events
- State root updates

**Example:**
```bash
$ veilocity sync

═══ Syncing with Network ═══

  Network: https://rpc.sepolia.mantle.xyz

═══ On-chain State ═══

  Block Number:  32341094
  State Root:    0x2098f5fb...
  Deposit Count: 16
  TVL:           0.464000 MNT

Fetching events...
  [████████████████████] 100%

✓ Sync complete
  Processed 3 new events
```

### history

View your transaction history.

```bash
veilocity history
```

**Example:**
```bash
$ veilocity history

═══ Transaction History ═══

  Type         Amount           Status     Time
  ─────────────────────────────────────────────────
  DEPOSIT      1.000000 MNT     confirmed  2h ago
  TRANSFER     0.500000 MNT     confirmed  1h ago
  WITHDRAW     0.250000 MNT     confirmed  30m ago
```

### config

View or update configuration.

```bash
veilocity config [set <KEY> <VALUE>]
```

**Subcommands:**
- `veilocity config`: Show current config
- `veilocity config set vault <ADDRESS>`: Set vault address
- `veilocity config set rpc <URL>`: Set RPC URL

---

## Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                         USER LAYER                                │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    Veilocity CLI                            │  │
│  │   veilocity deposit | transfer | withdraw | balance         │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
                                │
┌───────────────────────────────▼──────────────────────────────────┐
│                    EXECUTION LAYER (Off-chain)                    │
│                                                                   │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐   │
│  │  veilocity-core │  │veilocity-prover │  │veilocity-contracts│ │
│  │                 │  │                 │  │                 │   │
│  │ • State Mgmt    │  │ • Witness Gen   │  │ • ABI Bindings  │   │
│  │ • Merkle Tree   │  │ • Noir Prover   │  │ • TX Builder    │   │
│  │ • Poseidon Hash │  │ • Proof Export  │  │ • Event Parsing │   │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘   │
│                                                                   │
│  ┌───────────────────────────────────────────────────────────┐   │
│  │              Local Storage (SQLite + redb)                 │   │
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
│  │ VeilocityVault  │  │  UltraVerifier  │  │   State Root    │   │
│  │                 │  │                 │  │    History      │   │
│  │ • deposit()     │  │ • verify()      │  │ • 100 roots     │   │
│  │ • withdraw()    │  │ (auto-generated)│  │ • nullifiers    │   │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘   │
└──────────────────────────────────────────────────────────────────┘
```

### Components

| Crate | Purpose |
|-------|---------|
| `veilocity-cli` | Command-line interface, wallet management |
| `veilocity-core` | Poseidon hash, Merkle tree, state management |
| `veilocity-prover` | Witness generation, Noir proof creation |
| `veilocity-contracts` | Alloy bindings, event indexing, TX building |

### Data Flow

#### Deposit Flow

```
User                    CLI                  Mantle
  │                      │                     │
  │─── deposit 1 MNT ───▶│                     │
  │                      │                     │
  │                      │── commitment ───────▶│
  │                      │                     │
  │                      │◀── DepositEvent ────│
  │                      │                     │
  │◀── balance updated ──│                     │
```

#### Transfer Flow

```
Sender                  CLI                  Local State
  │                      │                        │
  │─ transfer 0.5 ──────▶│                        │
  │                      │                        │
  │                      │─── generate proof ────▶│
  │                      │                        │
  │                      │◀── update state ───────│
  │                      │                        │
  │◀── transfer done ────│                        │
```

#### Withdrawal Flow

```
User                    CLI                  Mantle
  │                      │                     │
  │─── withdraw 0.5 ────▶│                     │
  │                      │                     │
  │                      │── proof + nullifier ─▶│
  │                      │                     │
  │                      │   verify proof       │
  │                      │   check nullifier    │
  │                      │   transfer MNT       │
  │                      │                     │
  │◀── withdrawal done ──│◀─ WithdrawEvent ────│
```

---

## Smart Contracts

### VeilocityVault

The main contract handling deposits, withdrawals, and state management.

**Address (Sepolia):** `0x94Ad4197120E06cFDC3E98AE18e6219269C4b727`

#### Functions

```solidity
// Deposit MNT with a commitment
function deposit(bytes32 commitment) external payable;

// Withdraw with ZK proof
function withdraw(
    bytes32 nullifier,
    address recipient,
    uint256 amount,
    bytes32 root,
    bytes calldata proof
) external;

// Update state root (owner only)
function updateStateRoot(bytes32 newRoot, bytes calldata proof) external;
```

#### Events

```solidity
event Deposit(bytes32 indexed commitment, uint256 amount, uint256 leafIndex, uint256 timestamp);
event Withdrawal(bytes32 indexed nullifier, address indexed recipient, uint256 amount);
event StateRootUpdated(bytes32 indexed oldRoot, bytes32 indexed newRoot, uint256 batchIndex, uint256 timestamp);
```

### Contract Addresses

| Network | Contract | Address |
|---------|----------|---------|
| Mantle Sepolia | VeilocityVault | `0x94Ad4197120E06cFDC3E98AE18e6219269C4b727` |
| Mantle Sepolia | Verifier | `0x...` (pending deployment) |

---

## ZK Circuits

All circuits are written in **Noir** and compiled to UltraPlonk proofs.

### Deposit Circuit

**Purpose:** Prove commitment is correctly formed.

```noir
// Public inputs
commitment: Field   // The deposit commitment
amount: Field       // Deposit amount (public)

// Private inputs
secret: Field       // User's secret

// Constraints
assert(commitment == poseidon([secret, amount]));
assert(amount > 0);
```

**Constraints:** ~500

### Withdrawal Circuit

**Purpose:** Prove ownership and balance for withdrawal.

```noir
// Public inputs
state_root: Field   // Current Merkle root
nullifier: Field    // Unique spend identifier
amount: Field       // Withdrawal amount
recipient: Field    // Destination address

// Private inputs
secret: Field
balance: Field
nonce: Field
index: Field
path: [Field; 20]

// Constraints
1. pubkey = hash(secret)           // Ownership
2. leaf = hash(pubkey, balance, nonce)
3. verify_merkle_proof(leaf, path, state_root)
4. balance >= amount               // Sufficient funds
5. nullifier = hash(secret, index, nonce)
```

**Constraints:** ~8,000

### Transfer Circuit

**Purpose:** Prove valid balance transfer between accounts.

```noir
// Public inputs
old_root: Field     // Previous state root
new_root: Field     // New state root
nullifier: Field    // Sender's nullifier

// Private inputs
sender_secret, sender_balance, sender_nonce, sender_index, sender_path
recipient_pubkey, recipient_balance, recipient_nonce, recipient_index, recipient_path
amount: Field

// Constraints
1. sender_balance >= amount
2. verify sender membership
3. compute new sender leaf (balance - amount, nonce + 1)
4. compute new recipient leaf (balance + amount)
5. verify state transition
```

**Constraints:** ~15,000

---

## Security

### Threat Model

**Adversary Capabilities:**
- Can observe all on-chain transactions
- Can observe network traffic
- Cannot break cryptographic primitives
- Cannot compromise user's local machine

**Protected Assets:**
- Account balances (confidentiality)
- Transaction amounts (confidentiality)
- Transaction graph (unlinkability)
- Funds (integrity)

### Privacy Guarantees

| What IS Hidden | What is NOT Hidden |
|----------------|-------------------|
| Individual balances | Deposit amounts |
| Transfer amounts | Withdrawal amounts |
| Sender-recipient links | Total value locked |
| Per-account activity | Number of deposits/withdrawals |

### Security Properties

| Property | Guarantee | Mechanism |
|----------|-----------|-----------|
| Balance Integrity | Cannot spend more than owned | ZK range proof |
| No Double-Spend | Each balance spent once | Nullifier tracking |
| Ownership | Only owner can spend | Secret key in proof |
| Withdrawal Validity | Can only withdraw owned funds | Merkle + ownership proof |

---

## API Reference

### Configuration File

Location: `~/.veilocity/config.toml`

```toml
[network]
rpc_url = "https://rpc.sepolia.mantle.xyz"
chain_id = 5003
vault_address = "0x94Ad4197120E06cFDC3E98AE18e6219269C4b727"

[wallet]
path = "~/.veilocity/wallet.json"

[sync]
deployment_block = 24880000
blocks_per_batch = 10000

[prover]
circuit_path = "~/.veilocity/circuits"
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `VEILOCITY_CONFIG` | Override config file path |
| `VEILOCITY_NETWORK` | Override network (mainnet/sepolia) |
| `RUST_LOG` | Set log level (debug, info, warn, error) |

---

## FAQ

### General

**Q: Is Veilocity a new blockchain?**
A: No. Veilocity is an execution layer that runs on top of Mantle. It handles private computation while Mantle handles settlement and security.

**Q: What token does Veilocity use?**
A: MNT (Mantle's native token). You deposit MNT, transfer MNT privately, and withdraw MNT.

**Q: Is my balance visible on-chain?**
A: No. Only commitments are stored on-chain. Your actual balance is encrypted locally.

### Technical

**Q: How long do proofs take to generate?**
A:
- Deposit: <1 second
- Withdrawal: 2-5 seconds
- Transfer: 5-10 seconds

**Q: What happens if I lose my device?**
A: Your recovery phrase can restore your wallet, but transaction history is stored locally. Consider backing up `~/.veilocity/`.

**Q: Can I run my own node?**
A: Currently, Veilocity uses a single sequencer (MVP). Decentralized sequencing is on the roadmap.

### Troubleshooting

**Q: "Wallet not found" error**
A: Run `veilocity init` to create a new wallet.

**Q: "Account not found" during withdrawal**
A: Run `veilocity sync` to synchronize with on-chain state.

**Q: Sync is slow**
A: The sync needs to scan blocks from the deployment block. This is a one-time operation.

---

## Links

- **GitHub:** https://github.com/abhirupinspace/veilocity
- **Mantle:** https://mantle.xyz
- **Noir Language:** https://noir-lang.org

---

*Documentation Version: 1.0*
*Last Updated: December 2024*
