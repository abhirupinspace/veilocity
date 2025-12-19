# Veilocity

**A Private Execution Layer on Mantle**

Veilocity enables confidential transactions that run privately off-chain and settle with zero-knowledge proofs on Mantle L2.

## Overview

Veilocity provides:
- **Private Execution**: Transactions execute off-chain with hidden balances
- **ZK Proofs**: Noir circuits generate validity proofs
- **Mantle Settlement**: Proofs verified and state anchored on-chain
- **Real-time Sync**: Event-driven state synchronization from chain
- **Inherited Security**: Leverages Mantle's consensus and Ethereum's finality

## Test Status

| Component | Tests | Status |
|-----------|-------|--------|
| Noir Circuits | 19 | All Pass |
| Rust Crates | 30+ | All Pass |
| Solidity Contracts | 29 | All Pass |

---

## Prerequisites

### Required
```bash
# 1. Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# 2. Foundry (Solidity toolkit)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# 3. Noir & Barretenberg (ZK circuits)
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.16

curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup -nv 1.0.0-beta.16

# 4. Verify installations
cargo --version      # >= 1.75
nargo --version      # >= 1.0.0-beta.16
forge --version      # >= 0.2
```

---

## Quick Start

### Build Everything

```bash
# Clone the repository
git clone https://github.com/abhirupinspace/veilocity.git
cd veilocity

# Build Rust workspace
cargo build --release

# Compile Noir circuits
cd circuits && nargo compile && cd ..

# Build Solidity contracts
cd contracts && forge build && cd ..
```

### Run All Tests

```bash
# Noir circuit tests (19 tests)
cd circuits && nargo test && cd ..

# Rust crate tests (30+ tests)
cargo test

# Solidity contract tests (29 tests)
cd contracts && forge test && cd ..
```

---

## Complete Testing Flow

### Option 1: Local End-to-End Testing with Anvil

#### Step 1: Start Local Blockchain

```bash
# Terminal 1: Start Anvil (Foundry's local Ethereum node)
anvil --port 8545

# Anvil provides 10 test accounts with 10000 ETH each
# Default test private key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

#### Step 2: Deploy Contracts

```bash
# Terminal 2: Deploy to local Anvil
cd contracts

PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
forge script script/Deploy.s.sol:Deploy \
  --rpc-url http://localhost:8545 \
  --broadcast

# Output:
# MockVerifier deployed at: 0x5FbDB2315678afecb367f032d93F642f64180aa3
# VeilocityVault deployed at: 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
```

#### Step 3: Test Contract Operations with Cast

```bash
VAULT=0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
RPC=http://localhost:8545
PK=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Check initial state
echo "=== Initial State ==="
cast call $VAULT "currentRoot()" --rpc-url $RPC
cast call $VAULT "depositCount()" --rpc-url $RPC
cast call $VAULT "totalValueLocked()" --rpc-url $RPC

# Make a deposit (1 ETH with test commitment)
echo "=== Making Deposit ==="
COMMITMENT=0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
cast send $VAULT "deposit(bytes32)" $COMMITMENT \
  --value 1ether \
  --private-key $PK \
  --rpc-url $RPC

# Verify deposit
echo "=== After Deposit ==="
cast call $VAULT "depositCount()" --rpc-url $RPC
cast call $VAULT "totalValueLocked()" --rpc-url $RPC

# Fetch deposit events
echo "=== Deposit Events ==="
cast logs --address $VAULT --from-block 0 --rpc-url $RPC
```

#### Step 4: Initialize CLI Wallet

```bash
cd ..  # Back to project root

# Initialize a new Veilocity wallet
./target/release/veilocity init

# Follow prompts:
# - Enter password (min 8 characters)
# - Confirm password
# Note your Ethereum address and Veilocity public key
```

#### Step 5: Configure for Local Network

Create or edit `~/.veilocity/config.toml`:

```toml
[network]
name = "local"
rpc_url = "http://localhost:8545"
chain_id = 31337
vault_address = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"
explorer_url = ""
```

#### Step 6: Fund Your Wallet

```bash
# Replace YOUR_WALLET_ADDRESS with the address from 'veilocity init'
cast send YOUR_WALLET_ADDRESS --value 10ether \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --rpc-url http://localhost:8545
```

#### Step 7: Test Full User Flow

```bash
# Deposit 1 ETH into privacy pool
./target/release/veilocity deposit 1.0
# Generates commitment = poseidon(secret, amount)
# Sends to VeilocityVault.deposit(commitment)

# Sync state from chain
./target/release/veilocity sync
# Fetches Deposit events
# Updates local Merkle tree
# Stores sync checkpoint

# Check balance
./target/release/veilocity balance

# Private transfer (if you have another user's pubkey)
./target/release/veilocity transfer RECIPIENT_PUBKEY 0.5

# Withdraw back to public address
./target/release/veilocity withdraw 0.5
```

---

### Option 2: Foundry Contract Tests Only

```bash
cd contracts

# Run all 29 tests
forge test

# Run with verbosity (see failing assertions)
forge test -vvv

# Run specific test
forge test --match-test test_Deposit

# Run with gas report
forge test --gas-report

# Test categories:
# - Constructor tests (2)
# - Deposit tests (6)
# - Withdrawal tests (6)
# - State root tests (4)
# - Pause/Unpause tests (4)
# - Emergency withdraw tests (2)
# - View function tests (3)
# - Fuzz tests (1)
# - Receive tests (1)
```

---

### Option 3: Noir Circuit Tests Only

```bash
cd circuits

# Run all 19 tests
nargo test

# Test categories:
# - Poseidon hash tests (6)
# - Merkle tree tests (2)
# - Deposit circuit tests (4)
# - Withdrawal circuit tests (3)
# - Transfer circuit tests (4)
```

---

### Option 4: Rust Crate Tests Only

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p veilocity-core      # State, Merkle, Poseidon
cargo test -p veilocity-prover    # Witness generation
cargo test -p veilocity-contracts # ABI bindings

# Run with output
cargo test -- --nocapture
```

---

## Deploy to Mantle Sepolia

### 1. Get Test MNT

Visit https://faucet.sepolia.mantle.xyz to get testnet MNT.

### 2. Configure Environment

```bash
cd contracts

# Create .env file with your private key
echo "PRIVATE_KEY=your_private_key_here" > .env
source .env
```

### 3. Deploy Contracts

```bash
forge script script/DeployTestnet.s.sol:DeployTestnet \
  --rpc-url https://rpc.sepolia.mantle.xyz \
  --broadcast

# Note the deployed addresses from output
```

### 4. Configure CLI for Mantle Sepolia

Edit `~/.veilocity/config.toml`:

```toml
[network]
name = "mantle-sepolia"
rpc_url = "https://rpc.sepolia.mantle.xyz"
chain_id = 5003
vault_address = "YOUR_DEPLOYED_VAULT_ADDRESS"
explorer_url = "https://sepolia.mantlescan.xyz"
```

### 5. Test on Testnet

```bash
./target/release/veilocity init
./target/release/veilocity deposit 0.1
./target/release/veilocity sync
./target/release/veilocity balance
```

---

## CLI Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `veilocity init` | `i` | Create a new encrypted wallet |
| `veilocity deposit <amount>` | `d`, `dep` | Deposit ETH into privacy pool |
| `veilocity transfer <pubkey> <amount>` | `t`, `send` | Private transfer to another user |
| `veilocity withdraw <amount>` | `w` | Withdraw to public address |
| `veilocity balance` | `b`, `bal` | Show private balance |
| `veilocity sync` | `s` | Sync with on-chain state |
| `veilocity history` | `h`, `hist` | Show transaction history |

### CLI Options

```
OPTIONS:
    -c, --config <CONFIG>    Config file path [default: ~/.veilocity/config.toml]
    -n, --network <NETWORK>  Network: mainnet, sepolia [default: sepolia]
    -v, --verbose            Enable verbose output
    -h, --help               Print help
    -V, --version            Print version

COMMAND OPTIONS:
    --dry-run                Preview transaction without executing (deposit, transfer, withdraw)
    -r, --recipient <ADDR>   Recipient address for withdrawals
```

### Quick Examples

```bash
# Short form commands
veilocity d 0.1              # Deposit 0.1 ETH
veilocity b                  # Check balance
veilocity s                  # Sync state
veilocity w 0.05             # Withdraw 0.05 ETH

# Preview before executing
veilocity deposit 1.0 --dry-run
veilocity withdraw 0.5 --dry-run
```

---

## Web Demo

The `demo/` folder contains a Next.js web application for interacting with Veilocity.

### Features
- **Wallet Connection**: Privy integration for easy wallet connection
- **Private Deposits**: Deposit MNT with automatic commitment generation
- **Balance Tracking**: Local private balance display
- **Transaction History**: View your deposit and withdrawal history
- **Secret Management**: Export/import secret backups
- **Real-time Updates**: Live event feed from the contract

### Run the Demo

```bash
cd demo
npm install
cp .env.example .env.local
# Edit .env.local with your Privy app ID

npm run dev
# Open http://localhost:3000
```

### Environment Variables

```bash
# Required: Get from https://dashboard.privy.io
NEXT_PUBLIC_PRIVY_APP_ID=your_privy_app_id
```

---

## Project Structure

```
veilocity/
├── demo/                    # Next.js Web Demo
│   ├── app/                 # Next.js app router
│   ├── components/          # React components
│   │   ├── deposit-form.tsx # Deposit UI
│   │   ├── withdraw-form.tsx # Withdrawal UI
│   │   ├── private-balance.tsx # Balance display
│   │   └── ...
│   └── lib/                 # Utilities
│       ├── crypto.ts        # Poseidon hashing
│       └── abi.ts           # Contract ABIs
│
├── circuits/                 # Noir ZK Circuits (19 tests)
│   ├── src/
│   │   ├── main.nr          # Entry points for all circuits
│   │   ├── deposit.nr       # Deposit circuit (~500 constraints)
│   │   ├── withdraw.nr      # Withdrawal circuit (~8k constraints)
│   │   ├── transfer.nr      # Transfer circuit (~15k constraints)
│   │   ├── merkle.nr        # Merkle tree (depth 20, ~1M accounts)
│   │   └── poseidon_utils.nr # Poseidon hash wrappers
│   └── Nargo.toml
│
├── contracts/               # Solidity Contracts (29 tests)
│   ├── src/
│   │   ├── VeilocityVault.sol    # Main vault contract
│   │   ├── interfaces/           # IVerifier, IVeilocityVault
│   │   └── mocks/               # MockVerifier for testing
│   ├── test/                    # Foundry tests
│   └── script/                  # Deployment scripts
│
├── crates/                  # Rust Workspace (30+ tests)
│   ├── veilocity-core/      # Poseidon, Merkle, State management
│   ├── veilocity-prover/    # Witness generation, Noir integration
│   ├── veilocity-contracts/ # ABI bindings, Event fetching
│   └── veilocity-cli/       # CLI application
│
├── tech.md                  # Technical documentation
├── veilocity.md            # Project overview
└── README.md               # This file
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER CLI                                 │
│  veilocity init | deposit | transfer | withdraw | balance | sync│
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    EXECUTION LAYER (Off-chain)                   │
│                                                                  │
│  ┌──────────────┐  ┌───────────────┐  ┌───────────────────────┐ │
│  │ veilocity-   │  │ veilocity-    │  │ veilocity-contracts   │ │
│  │ core         │  │ prover        │  │                       │ │
│  │              │  │               │  │ • Event fetching      │ │
│  │ • State      │  │ • Witness     │  │ • Real-time sync      │ │
│  │ • Merkle     │  │ • ZK Proofs   │  │ • TX submission       │ │
│  │ • Poseidon   │  │ • Noir/BB     │  │ • ABI bindings        │ │
│  └──────────────┘  └───────────────┘  └───────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    SQLite Database                          │ │
│  │  accounts | transactions | nullifiers | sync_state          │ │
│  └────────────────────────────────────────────────────────────┘ │
└────────────────────────────────┬────────────────────────────────┘
                                 │ ZK Proofs + Events
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                   SETTLEMENT LAYER (Mantle L2)                   │
│                                                                  │
│  ┌─────────────────────────┐    ┌─────────────────────────────┐ │
│  │    VeilocityVault.sol   │    │     UltraVerifier.sol       │ │
│  │                         │    │                             │ │
│  │  • deposit(commitment)  │◄──►│  • verify(proof, inputs)    │ │
│  │  • withdraw(proof,...)  │    │  • ~300k gas/verification   │ │
│  │  • updateStateRoot(...) │    │                             │ │
│  │  • Nullifier tracking   │    │                             │ │
│  │  • Event emission       │    │                             │ │
│  └─────────────────────────┘    └─────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## Protocol Flow

### Deposit Flow
```
User → veilocity deposit 1.0
    → Generate commitment = poseidon(secret, amount)
    → TX to VeilocityVault.deposit(commitment) {value: 1 ETH}
    → Event: Deposit(commitment, amount, leafIndex, timestamp)
    → veilocity sync fetches event, updates local Merkle tree
```

### Sync Flow
```
User → veilocity sync
    → Fetch current block & state root from chain
    → Query Deposit events from last_synced_block to now
    → For each deposit: insert commitment into local Merkle tree
    → Update sync checkpoint in SQLite
    → Verify local root matches on-chain root
```

### Withdraw Flow
```
User → veilocity withdraw 0.5
    → Load account from local state
    → Compute nullifier = poseidon(secret, index, nonce)
    → Generate ZK proof (Noir circuit + Barretenberg)
    → TX to VeilocityVault.withdraw(nullifier, recipient, amount, root, proof)
    → Contract verifies proof via UltraVerifier
    → Funds transferred to recipient
```

---

## Security Notes

- **Balances**: Never revealed on-chain
- **Transfers**: Amounts and recipients are hidden
- **Nullifiers**: Prevent double-spending
- **Merkle Proofs**: Verify account existence without revealing position
- **Wallet Encryption**: AES-256-GCM with Argon2id key derivation
- **Password Requirements**: Minimum 8 chars with uppercase, lowercase, and digits
- **Memory Safety**: Sensitive keys are zeroized after use
- **Testnet Only**: This is alpha software. Use only on testnets.

---

## Troubleshooting

### Build fails with ark-ff version conflict
The project uses ark-ff 0.5 for light-poseidon compatibility. Ensure your Cargo.toml has: `ark-ff = "0.5"`

### Serde compilation errors
Pin serde for alloy compatibility: `serde = { version = "=1.0.219", features = ["derive"] }`

### CLI can't connect to RPC
Check `~/.veilocity/config.toml` - ensure rpc_url is correct and network is accessible.

### Proof generation fails
Ensure Noir and Barretenberg are installed:
- `nargo --version` should show >= 1.0.0-beta.16
- `bb --version` should show >= 1.0.0-beta.16

---

## Resources

- [Technical Specification](tech.md) - Detailed system design
- [Project Overview](veilocity.md) - Framing and use cases
- [Circuits README](circuits/README.md) - ZK circuit documentation
- [Mantle Documentation](https://docs.mantle.xyz/)
- [Noir Language](https://noir-lang.org/)
- [Foundry Book](https://book.getfoundry.sh/)

---

## Roadmap & Future Improvements

### MVP Complete
- [x] Noir circuits for deposit, withdraw, transfer
- [x] Solidity vault contract with ZK verification
- [x] Rust CLI with full wallet management
- [x] Next.js web demo with Privy integration
- [x] Real-time event synchronization
- [x] Local private state management
- [x] 78+ comprehensive tests

### Near-term Improvements
- [ ] **Production Verifier**: Generate and deploy real Solidity verifier from Noir circuits
- [ ] **State Root Proofs**: Verify state transitions with ZK proofs (not just owner trust)
- [ ] **Hardware Wallet Support**: Integrate Ledger/Trezor for key management
- [ ] **Encrypted Cloud Backup**: Optional encrypted backup of secrets

### Medium-term Features
- [ ] **Multi-user Transfers**: Full private transfer UI in web demo
- [ ] **Mobile App**: React Native app for iOS/Android
- [ ] **Batch Withdrawals**: Aggregate multiple withdrawals for gas efficiency
- [ ] **Relayer Network**: Gasless withdrawals via meta-transactions
- [ ] **Cross-chain Bridges**: Private bridges to other L2s

### Long-term Vision
- [ ] **Decentralized Sequencer**: Replace owner-based state root updates
- [ ] **Compliance Mode**: Optional auditability for institutional users
- [ ] **Private DEX**: Integrate with Mantle DeFi protocols
- [ ] **Recursive Proofs**: Aggregate proofs for scalability

---

## Additional Integration Ideas

### DeFi Protocols
- **Private Lending**: Borrow/lend with hidden positions
- **Private Swaps**: Trade without revealing trade size
- **Private Staking**: Stake MNT with hidden amounts

### Enterprise Use Cases
- **Payroll Privacy**: Private salary payments
- **Supply Chain**: Confidential B2B payments
- **Treasury Management**: Hide corporate holdings

### Gaming & NFTs
- **Private NFT Ownership**: Hide which NFTs you own
- **In-game Currency**: Private game economies
- **Prediction Markets**: Private betting

---

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Run all tests (`nargo test`, `forge test`, `cargo test`)
4. Submit a pull request

---

## License

MIT License
