# Veilocity ZK Circuits

Zero-knowledge circuits for the Veilocity private execution layer on Mantle L2.

## Overview

This project contains Noir circuits that enable private transactions on the Veilocity protocol. The circuits use Poseidon hashing (BN254 compatible) for efficient on-chain verification.

### Architecture

```
circuits/
├── Nargo.toml              # Project configuration (binary package)
├── Prover.toml             # Witness inputs for proof generation
└── src/
    ├── main.nr             # Binary entry point (deposit circuit)
    ├── lib.nr              # Library exports (for external use)
    ├── poseidon_utils.nr   # Poseidon hash utilities
    ├── merkle.nr           # Merkle tree verification (depth 20, ~1M accounts)
    ├── deposit.nr          # Deposit circuit (~500 constraints)
    ├── withdraw.nr         # Withdrawal circuit (~8,000 constraints)
    └── transfer.nr         # Transfer circuit (~15,000 constraints)
```

## Circuits

### 1. Deposit Circuit (Main Entry Point)

Proves that a deposit commitment is correctly formed. This is the default circuit compiled as the binary entry point.

**Public Inputs:**
- `amount` - The deposit amount (public for contract value verification)

**Private Inputs:**
- `secret` - User's secret for this deposit

**Public Outputs:**
- `commitment` - The computed deposit commitment (returned by circuit)

**Constraints:**
- Commitment = hash(secret, amount)
- Amount is positive and within u64 range

### 2. Withdrawal Circuit

Proves ownership of funds and generates a nullifier for withdrawal.

**Public Inputs:**
- `state_root` - Current Merkle tree state root
- `nullifier` - Unique identifier preventing double-withdrawal
- `amount` - Withdrawal amount
- `recipient` - On-chain address receiving funds

**Private Inputs:**
- `secret` - Account secret key
- `balance` - Current account balance
- `nonce` - Account nonce
- `index` - Leaf index in tree
- `path` - Merkle proof path (20 elements)

**Constraints:**
- Account exists in Merkle tree
- Sufficient balance for withdrawal
- Nullifier is correctly derived
- Recipient is valid (non-zero)

### 3. Transfer Circuit

Proves a valid balance transfer between two accounts.

**Full Transfer Public Inputs:**
- `old_state_root` - State root before transfer
- `new_state_root` - State root after transfer
- `nullifier` - Sender's nullifier

**Full Transfer Private Inputs:**
- Sender: secret, balance, nonce, index, path_old (20), path_new (20)
- Recipient: pubkey, balance, nonce, index, path_old (20), path_new (20)
- Amount: transfer amount

The full transfer requires 4 Merkle paths (80 field elements total) to properly verify the sequential state transition:
```
old_root → (update sender) → intermediate_root → (update recipient) → new_root
```

**Simple Transfer (MVP) Public Inputs:**
- `old_state_root` - Current state root
- `nullifier` - Sender's nullifier

**Simple Transfer Private Inputs:**
- Sender: secret, balance, nonce, index, path (20)
- Recipient: pubkey
- Amount: transfer amount

**Constraints:**
- Sender owns account with sufficient balance
- Nullifier prevents double-spending
- Balances conserved and state transition valid (full transfer only)
- Cannot transfer to self

## Prerequisites

### Install Noir

```bash
# Using noirup (recommended)
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup

# Verify installation
nargo --version
```

**Tested Versions:**
- Noir: 1.0.0-beta.16
- Barretenberg: 3.0.0-nightly.20251104

### Dependencies

The project uses:
- [noir-lang/poseidon](https://github.com/noir-lang/poseidon) v0.1.1 - BN254-compatible Poseidon hash

## Commands

### Build/Compile

```bash
# Compile the circuits
nargo compile

# Check for errors without full compilation
nargo check
```

### Run Tests

```bash
# Run all tests
nargo test

# Run tests with verbose output
nargo test --show-output

# Run specific test
nargo test test_valid_deposit
```

### Execute Circuit

```bash
# Execute circuit with inputs (from Prover.toml)
nargo execute

# Execute with witness name
nargo execute witness_name
```

### Generate Proofs (requires backend)

Noir 1.x uses external backends for proof generation. Install Barretenberg:

```bash
# Install bb (Barretenberg backend)
curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
bbup

# Verify installation
bb --version  # Should show 3.0.0 or compatible
```

**Complete Proof Generation Workflow:**

```bash
# 1. Set witness inputs in Prover.toml
cat > Prover.toml << EOF
amount = "1000000000000000000"
secret = "123456789"
EOF

# 2. Execute circuit (compiles + generates witness)
nargo execute

# Note: Output files are in ../target/ (parent project's target directory)
# - ../target/veilocity_circuits.json  (compiled circuit)
# - ../target/veilocity_circuits.gz    (witness)

# 3. Generate verification key
bb write_vk -b ../target/veilocity_circuits.json -o ./target/vk

# 4. Generate proof
bb prove \
  -b ../target/veilocity_circuits.json \
  -w ../target/veilocity_circuits.gz \
  -k ./target/vk/vk \
  -o ./target/proof

# 5. Verify proof
bb verify \
  -k ./target/vk/vk \
  -p ./target/proof/proof \
  -i ./target/proof/public_inputs
```

**Output Files:**
- `./target/vk/vk` - Verification key
- `./target/vk/vk_hash` - Verification key hash
- `./target/proof/proof` - The ZK proof
- `./target/proof/public_inputs` - Public inputs for verification

### Code Formatting

```bash
# Format code
nargo fmt

# Check formatting
nargo fmt --check
```

### Documentation

```bash
# Generate documentation
nargo doc
```

## Cryptographic Primitives

### Poseidon Hash Functions

| Function | Inputs | Usage |
|----------|--------|-------|
| `hash1(a)` | 1 Field | Derive pubkey from secret |
| `hash2(a, b)` | 2 Fields | Merkle nodes, deposit commitments |
| `hash3(a, b, c)` | 3 Fields | Account leaves, nullifiers |

### Key Derivations

```
pubkey = hash1(secret)
nullifier = hash3(secret, leaf_index, nonce)
leaf = hash3(pubkey, balance, nonce)
deposit_commitment = hash2(secret, amount)
```

### Merkle Tree

- **Depth:** 20 levels
- **Capacity:** ~1,048,576 accounts
- **Hash:** Poseidon2 (BN254)

## Test Cases

The circuits include comprehensive test coverage (19 tests total):

| Module | Tests |
|--------|-------|
| `deposit.nr` | Valid deposit, commitment determinism, wrong secret/amount failures |
| `withdraw.nr` | Valid withdrawal, insufficient balance, wrong nullifier failures |
| `transfer.nr` | Valid simple transfer, valid full transfer (state transition), insufficient balance, transfer to self rejection |
| `merkle.nr` | Single leaf tree, index-root uniqueness |
| `poseidon_utils.nr` | Hash determinism, pubkey derivation, nullifier uniqueness, leaf computation |

## Integration

### Solidity Contract Integration

The circuits generate proofs verified by the `VeilocityVault.sol` contract. Public inputs must match between proof generation and on-chain verification.

### Rust/TypeScript Integration

Ensure your off-chain implementation uses identical Poseidon parameters:
- Curve: BN254
- Field: Fr (scalar field)
- Hash rate: Standard Poseidon sponge

## Security Considerations

1. **Secret Generation:** Use cryptographically secure random number generation
2. **Nullifier Uniqueness:** Nullifiers prevent double-spending; never reuse secrets
3. **Merkle Paths:** Ensure paths are validated against current state root
4. **Amount Bounds:** Amounts must fit in u64 to prevent overflow attacks

## References

- [Noir Documentation](https://noir-lang.org/docs)
- [Poseidon Hash Paper](https://eprint.iacr.org/2019/458.pdf)
- [BN254 Curve](https://eips.ethereum.org/EIPS/eip-197)
- [Merkle Tree Proofs](https://en.wikipedia.org/wiki/Merkle_tree)
- [Nargo CLI Reference](https://noir-lang.org/docs/reference/nargo_commands)

## License

Part of the Veilocity Protocol.
