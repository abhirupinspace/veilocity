# Veilocity

**A Private Execution Layer on Mantle**

Veilocity enables confidential transactions that run privately off-chain and settle with zero-knowledge proofs on Mantle L2.

## Overview

Veilocity provides:
- **Private Execution**: Transactions execute off-chain with hidden balances
- **ZK Proofs**: Noir circuits generate validity proofs
- **Mantle Settlement**: Proofs verified and state anchored on-chain
- **Inherited Security**: Leverages Mantle's consensus and Ethereum's finality

## Prerequisites

### Required
- **Rust** (1.75+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Git**: For cloning the repository

### Optional (for full functionality)
- **Foundry** (Solidity): `curl -L https://foundry.paradigm.xyz | bash && foundryup`
- **Noir** (ZK circuits): `curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash && noirup`
- **Barretenberg** (Proving backend): `curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash && bbup -nv 1.0.0-beta.16`

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/veilocity/veilocity.git
cd veilocity
```

### 2. Build the Rust Workspace

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release
```

### 3. Run the CLI

```bash
# Using cargo
cargo run --bin veilocity -- --help

# Or directly (after release build)
./target/release/veilocity --help
```

## CLI Usage

### Initialize a Wallet

```bash
veilocity init
```

This creates a new wallet with:
- An Ethereum-compatible private key (for on-chain transactions)
- A Veilocity secret (for private transfers)
- Encrypted storage at `~/.veilocity/`

### Check Balance

```bash
veilocity balance
```

### Deposit Funds

```bash
# Deposit 1 ETH into Veilocity
veilocity deposit 1.0
```

### Private Transfer

```bash
# Transfer 0.5 ETH privately to another user
veilocity transfer <RECIPIENT_PUBKEY> 0.5
```

### Withdraw Funds

```bash
# Withdraw 0.5 ETH back to Mantle
veilocity withdraw 0.5

# Withdraw to a specific address
veilocity withdraw 0.5 --to 0x742d35Cc6634C0532925a3b844Bc9e7595f...
```

### Sync State

```bash
veilocity sync
```

### View History

```bash
veilocity history
```

### CLI Options

```
OPTIONS:
    -c, --config <CONFIG>    Config file path [default: ~/.veilocity/config.toml]
    -n, --network <NETWORK>  Network: mainnet, sepolia [default: sepolia]
    -v, --verbose            Enable verbose output
    -h, --help               Print help
    -V, --version            Print version
```

## Configuration

The CLI uses a TOML configuration file at `~/.veilocity/config.toml`:

```toml
[network]
rpc_url = "https://rpc.sepolia.mantle.xyz"
chain_id = 5003
vault_address = "0x..."  # Deployed VeilocityVault contract
explorer_url = "https://sepolia.mantlescan.xyz"

[prover]
circuits_path = "./circuits"
threads = 4

[storage]
db_path = "~/.veilocity/state.db"
```

## Building Components

### Rust Crates

```bash
# Build all crates
cargo build --release

# Run tests
cargo test

# Check without building
cargo check
```

### Solidity Contracts (requires Foundry)

```bash
cd contracts

# Install dependencies
forge install

# Build contracts
forge build

# Run tests
forge test

# Run tests with verbosity
forge test -vvv

# Run specific test
forge test --match-test testDeposit
```

### Noir Circuits (requires Noir + Barretenberg)

```bash
cd circuits

# Compile circuits
nargo compile

# Run circuit tests
nargo test

# Generate Solidity verifier
bb write_vk -b ./target/veilocity_circuits.json -o ./target --oracle_hash keccak
bb write_solidity_verifier -k ./target/vk -o ../contracts/src/verifiers/Verifier.sol
```

## Testing

### Unit Tests

```bash
# Run all Rust tests
cargo test

# Run tests for a specific crate
cargo test -p veilocity-core
cargo test -p veilocity-prover
cargo test -p veilocity-contracts
cargo test -p veilocity-cli

# Run with output
cargo test -- --nocapture
```

### Contract Tests

```bash
cd contracts

# Run all tests
forge test

# Run with gas reporting
forge test --gas-report

# Fork testing against Mantle Sepolia
forge test --fork-url https://rpc.sepolia.mantle.xyz
```

### Integration Testing

1. **Start a local Anvil node:**
   ```bash
   anvil --fork-url https://rpc.sepolia.mantle.xyz
   ```

2. **Deploy contracts:**
   ```bash
   cd contracts
   forge script script/Deploy.s.sol --broadcast --rpc-url http://localhost:8545
   ```

3. **Update config with deployed addresses:**
   ```bash
   # Edit ~/.veilocity/config.toml with the deployed vault_address
   ```

4. **Test the full flow:**
   ```bash
   veilocity init
   veilocity deposit 1.0
   veilocity balance
   veilocity transfer <RECIPIENT> 0.5
   veilocity withdraw 0.25
   ```

## Project Structure

```
veilocity/
├── Cargo.toml                 # Rust workspace manifest
├── README.md                  # This file
├── veilocity.md              # Technical specification
│
├── circuits/                  # Noir ZK circuits
│   ├── Nargo.toml
│   └── src/
│       ├── lib.nr            # Main library
│       ├── merkle.nr         # Merkle tree verification
│       ├── deposit.nr        # Deposit circuit
│       ├── withdraw.nr       # Withdrawal circuit
│       └── transfer.nr       # Private transfer circuit
│
├── contracts/                 # Solidity smart contracts
│   ├── foundry.toml
│   ├── src/
│   │   ├── VeilocityVault.sol
│   │   ├── interfaces/IVerifier.sol
│   │   └── mocks/MockVerifier.sol
│   ├── script/Deploy.s.sol
│   └── test/VeilocityVault.t.sol
│
└── crates/                    # Rust crates
    ├── veilocity-core/       # State management, Merkle trees, Poseidon
    ├── veilocity-prover/     # Witness generation, proof orchestration
    ├── veilocity-contracts/  # ABI bindings, contract interaction
    └── veilocity-cli/        # Command-line interface
```

## Deployment

### Deploy to Mantle Sepolia

1. **Set environment variables:**
   ```bash
   export PRIVATE_KEY=0x...
   export MANTLE_SEPOLIA_RPC=https://rpc.sepolia.mantle.xyz
   ```

2. **Deploy contracts:**
   ```bash
   cd contracts
   forge script script/Deploy.s.sol \
     --broadcast \
     --rpc-url $MANTLE_SEPOLIA_RPC \
     --private-key $PRIVATE_KEY
   ```

3. **Verify contracts (optional):**
   ```bash
   forge verify-contract <VAULT_ADDRESS> VeilocityVault \
     --chain-id 5003 \
     --verifier blockscout \
     --verifier-url https://sepolia.mantlescan.xyz/api
   ```

## Troubleshooting

### Common Issues

**Build fails with ark-ff version conflict:**
```
The project uses ark-ff 0.5 for light-poseidon compatibility.
Ensure your Cargo.toml has: ark-ff = "0.5"
```

**Serde compilation errors:**
```
Pin serde to version 1.0.219 for alloy compatibility:
serde = { version = "=1.0.219", features = ["derive"] }
```

**CLI can't connect to RPC:**
```
Check your config file at ~/.veilocity/config.toml
Ensure rpc_url is correct and the network is accessible
```

**Proof generation fails:**
```
Ensure Noir and Barretenberg are installed:
- noirup (for nargo)
- bbup -nv 1.0.0-beta.16 (for bb prover)
```

## Security Notes

- **Demo Encryption**: The wallet uses simple XOR encryption for demo purposes. For production, use proper key derivation (scrypt) and encryption (AES-GCM).
- **Local State**: Private balances are stored locally. Back up your `~/.veilocity/` directory.
- **Testnet Only**: This is alpha software. Use only on testnets with test funds.

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit changes: `git commit -am 'Add my feature'`
4. Push to branch: `git push origin feature/my-feature`
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Resources

- [Technical Specification](veilocity.md) - Detailed system design
- [Mantle Documentation](https://docs.mantle.xyz/)
- [Noir Language](https://noir-lang.org/)
- [Alloy-rs](https://alloy.rs/)
- [Foundry Book](https://book.getfoundry.sh/)
