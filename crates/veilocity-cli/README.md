# Veilocity CLI

Private execution layer CLI for Mantle. Deposit, transfer, and withdraw funds with zero-knowledge proofs.

## Installation

### From crates.io (Recommended)

```bash
cargo install veilocity-cli
```

### From GitHub Releases

Download the prebuilt binary for your platform from [GitHub Releases](https://github.com/abhirupinspace/veilocity/releases).

```bash
# macOS (Apple Silicon)
curl -L https://github.com/abhirupinspace/veilocity/releases/latest/download/veilocity-darwin-arm64 -o veilocity
chmod +x veilocity
sudo mv veilocity /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/abhirupinspace/veilocity/releases/latest/download/veilocity-darwin-x64 -o veilocity
chmod +x veilocity
sudo mv veilocity /usr/local/bin/

# Linux (x64)
curl -L https://github.com/abhirupinspace/veilocity/releases/latest/download/veilocity-linux-x64 -o veilocity
chmod +x veilocity
sudo mv veilocity /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/abhirupinspace/veilocity.git
cd veilocity
cargo build --release
cp target/release/veilocity /usr/local/bin/
```

## Quick Start

```bash
# 1. Create a new wallet
veilocity init

# 2. Fund your wallet address with MNT (shown after init)

# 3. Deposit into the privacy pool
veilocity deposit 0.1

# 4. Sync with the network
veilocity sync

# 5. Check your private balance
veilocity balance

# 6. View transaction history
veilocity history
```

## Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `veilocity init` | `i` | Create a new encrypted wallet |
| `veilocity deposit <amount>` | `d`, `dep` | Deposit ETH into privacy pool |
| `veilocity transfer <pubkey> <amount>` | `t`, `send` | Private transfer to another user |
| `veilocity withdraw <amount>` | `w` | Withdraw to public address |
| `veilocity balance` | `b`, `bal` | Show private balance |
| `veilocity sync` | `s` | Sync with on-chain state |
| `veilocity history` | `h`, `hist` | Show transaction history |

## Options

```
-c, --config <PATH>     Config file path [default: ~/.veilocity/config.toml]
-n, --network <NETWORK> Network: mainnet, sepolia [default: sepolia]
-v, --verbose           Enable debug logging
    --dry-run           Preview transaction without executing
-h, --help              Print help
-V, --version           Print version
```

## Examples

```bash
# Short form commands
veilocity d 0.1              # Deposit 0.1 ETH
veilocity b                  # Check balance
veilocity s                  # Sync state
veilocity w 0.05             # Withdraw 0.05 ETH

# Preview before executing
veilocity deposit 1.0 --dry-run
veilocity withdraw 0.5 --dry-run

# Use specific network
veilocity -n mainnet balance
veilocity -n sepolia deposit 0.1
```

## Configuration

The CLI stores data in `~/.veilocity/`:

```
~/.veilocity/
├── config.toml    # Network and prover settings
├── wallet.json    # Encrypted wallet (AES-256-GCM)
└── state.db       # Local state database (SQLite)
```

### config.toml

```toml
[network]
rpc_url = "https://rpc.sepolia.mantle.xyz"
chain_id = 5003
vault_address = "0x..."
explorer_url = "https://explorer.sepolia.mantle.xyz"

[sync]
poll_interval_secs = 12
confirmations = 2
deployment_block = 1000000  # Optional: skip scanning old blocks

[prover]
threads = 4
cache_proofs = true
```

## Security

- **Wallet Encryption**: AES-256-GCM with Argon2id key derivation
- **Password Requirements**: 8+ characters, uppercase, lowercase, and digits
- **Memory Safety**: Sensitive keys are zeroized after use
- **Local Storage**: All private data stays on your machine

## Requirements

For proof generation, you need Noir and Barretenberg installed:

```bash
# Install Noir
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.16

# Install Barretenberg
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/refs/heads/master/barretenberg/bbup/install | bash
bbup -nv 1.0.0-beta.16
```

## License

MIT
