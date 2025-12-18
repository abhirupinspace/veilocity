# Veilocity: From Hackathon to Production

> A comprehensive roadmap for building a production-grade private execution layer on Mantle L2

---

## Table of Contents

1. [Current State Assessment](#1-current-state-assessment)
2. [Phase 1: Security Hardening](#2-phase-1-security-hardening)
3. [Phase 2: Decentralization](#3-phase-2-decentralization)
4. [Phase 3: Scalability](#4-phase-3-scalability)
5. [Phase 4: User Experience](#5-phase-4-user-experience)
6. [Phase 5: Privacy Enhancements](#6-phase-5-privacy-enhancements)
7. [Phase 6: Ecosystem Integration](#7-phase-6-ecosystem-integration)
8. [Phase 7: Governance & Sustainability](#8-phase-7-governance--sustainability)
9. [Implementation Timeline](#9-implementation-timeline)
10. [Resource Requirements](#10-resource-requirements)
11. [Competitive Analysis](#11-competitive-analysis)
12. [Risk Assessment](#12-risk-assessment)

---

## 1. Current State Assessment

### What We Have (MVP)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        HACKATHON MVP STATUS                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  COMPLETED                           │  GAPS                            │
│  ─────────                           │  ────                            │
│  ✅ Core ZK circuits (Noir)          │  ⚠️  MockVerifier (not real)     │
│  ✅ Transfer, Deposit, Withdraw      │  ⚠️  Centralized state updates   │
│  ✅ Rust execution engine            │  ⚠️  Single-node architecture    │
│  ✅ StateManager + Merkle tree       │  ⚠️  No SDK for developers       │
│  ✅ Basic VeilocityVault contract    │  ⚠️  No relayer network          │
│  ✅ CLI interface                    │  ⚠️  Basic encryption only       │
│  ✅ Poseidon hashing (BN254)         │  ⚠️  No audits                   │
│  ✅ alloy-rs RPC integration         │  ⚠️  No monitoring/analytics     │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Technical Debt

| Component | Issue | Risk Level |
|-----------|-------|------------|
| MockVerifier | Accepts all proofs | **CRITICAL** |
| Owner-only state updates | Centralization | **HIGH** |
| XOR encryption | Not production-grade | **MEDIUM** |
| No rate limiting | DoS vulnerability | **MEDIUM** |
| Single sequencer | Single point of failure | **HIGH** |

### Maturity Assessment

```
                    MVP        Beta       Production
                     │          │            │
Circuit Design    ████████████░░░░░░░░░░░░░░░░  40%
Smart Contracts   ██████████░░░░░░░░░░░░░░░░░░  35%
Rust Engine       ████████████████░░░░░░░░░░░░  55%
Security          ████░░░░░░░░░░░░░░░░░░░░░░░░  15%
Decentralization  ██░░░░░░░░░░░░░░░░░░░░░░░░░░  5%
User Experience   ████████░░░░░░░░░░░░░░░░░░░░  25%
Documentation     ████████████████░░░░░░░░░░░░  55%
```

---

## 2. Phase 1: Security Hardening

**Timeline: 4-8 weeks**
**Priority: CRITICAL - Must complete before any mainnet deployment**

### 2.1 Deploy Real Verifier

Replace the MockVerifier with the auto-generated UltraVerifier from Barretenberg.

```bash
# Step 1: Compile circuits
cd circuits
nargo compile

# Step 2: Generate verification key
bb write_vk -b ./target/veilocity_circuits.json -o ./target/vk

# Step 3: Generate Solidity verifier
bb write_solidity_verifier -k ./target/vk -o ../contracts/src/verifiers/UltraVerifier.sol

# Step 4: Update deployment script to use real verifier
```

**Contract Changes:**

```solidity
// Before (INSECURE)
contract MockVerifier is IVerifier {
    function verify(bytes calldata, bytes32[] calldata)
        external pure returns (bool) {
        return true;  // DANGER: Accepts everything
    }
}

// After (SECURE)
// Use auto-generated UltraVerifier.sol from Barretenberg
// Contains actual cryptographic verification logic
```

### 2.2 Security Audits

| Audit Type | Scope | Recommended Firms | Est. Cost |
|------------|-------|-------------------|-----------|
| **Circuit Audit** | Noir circuits, constraint soundness | Zellic, Veridise | $50-100k |
| **Smart Contract Audit** | Solidity, access control, reentrancy | OpenZeppelin, Trail of Bits, Spearbit | $50-150k |
| **Cryptographic Review** | Poseidon params, key derivation | Academic partners, NCC Group | $30-50k |
| **Rust Security Review** | Memory safety, secret handling | Cure53, Include Security | $30-50k |

**Audit Preparation Checklist:**

```
□ Complete test coverage (>90% for critical paths)
□ Document all assumptions and invariants
□ Create threat model document
□ Prepare known issues list
□ Set up private audit repo
□ Assign internal point of contact
□ Budget for remediation time (2-4 weeks post-audit)
```

### 2.3 Formal Verification

**Circuit Verification Goals:**

```
Prove mathematically:
1. Soundness: Invalid state transitions cannot produce valid proofs
2. Completeness: Valid state transitions always produce valid proofs
3. Zero-Knowledge: Proofs reveal nothing about private inputs

Specific Properties:
□ Balance conservation: sum(inputs) == sum(outputs)
□ Nullifier uniqueness: each (secret, index, nonce) → unique nullifier
□ Merkle correctness: computed root matches expected root
□ No underflow: balance >= withdrawal_amount always checked
```

**Tools:**

| Tool | Purpose | Integration |
|------|---------|-------------|
| **Ecne** | Noir circuit verification | Direct Noir support |
| **Certora** | Solidity formal verification | Contract invariants |
| **Halmos** | Symbolic testing | Property-based |

### 2.4 Enhanced Contract Security

```solidity
// security/VeilocityVaultV2.sol

contract VeilocityVaultV2 is
    ReentrancyGuard,
    Pausable,
    AccessControl,
    UUPSUpgradeable
{
    // ═══════════════════════════════════════════════════════════════
    //                      NEW SECURITY FEATURES
    // ═══════════════════════════════════════════════════════════════

    bytes32 public constant SEQUENCER_ROLE = keccak256("SEQUENCER_ROLE");
    bytes32 public constant GUARDIAN_ROLE = keccak256("GUARDIAN_ROLE");

    // Rate limiting
    uint256 public constant MAX_WITHDRAWAL_PER_BLOCK = 100 ether;
    uint256 public withdrawalsThisBlock;
    uint256 public lastWithdrawalBlock;

    // Root freshness
    uint256 public constant MAX_ROOT_AGE = 100; // blocks
    mapping(bytes32 => uint256) public rootSubmissionBlock;

    // Timelocked admin functions
    uint256 public constant TIMELOCK_DURATION = 2 days;
    mapping(bytes32 => uint256) public pendingActions;

    // ═══════════════════════════════════════════════════════════════
    //                      ENHANCED WITHDRAW
    // ═══════════════════════════════════════════════════════════════

    function withdraw(
        bytes32 nullifier,
        address payable recipient,
        uint256 amount,
        bytes32 root,
        bytes calldata proof
    ) external nonReentrant whenNotPaused {
        // Rate limiting check
        if (block.number != lastWithdrawalBlock) {
            withdrawalsThisBlock = 0;
            lastWithdrawalBlock = block.number;
        }
        require(
            withdrawalsThisBlock + amount <= MAX_WITHDRAWAL_PER_BLOCK,
            "Rate limit exceeded"
        );
        withdrawalsThisBlock += amount;

        // Root freshness check
        require(
            block.number - rootSubmissionBlock[root] <= MAX_ROOT_AGE,
            "Root too old"
        );

        // ... rest of withdrawal logic
    }

    // ═══════════════════════════════════════════════════════════════
    //                      EMERGENCY FUNCTIONS
    // ═══════════════════════════════════════════════════════════════

    /// @notice Guardian can pause in emergency
    function emergencyPause() external onlyRole(GUARDIAN_ROLE) {
        _pause();
        emit EmergencyPaused(msg.sender, block.timestamp);
    }

    /// @notice Unpause requires timelock
    function initiateUnpause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        bytes32 actionId = keccak256("unpause");
        pendingActions[actionId] = block.timestamp + TIMELOCK_DURATION;
        emit UnpauseInitiated(block.timestamp + TIMELOCK_DURATION);
    }

    function executeUnpause() external onlyRole(DEFAULT_ADMIN_ROLE) {
        bytes32 actionId = keccak256("unpause");
        require(
            pendingActions[actionId] != 0 &&
            block.timestamp >= pendingActions[actionId],
            "Timelock not elapsed"
        );
        delete pendingActions[actionId];
        _unpause();
    }
}
```

### 2.5 Testing Infrastructure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         TESTING PYRAMID                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                         ┌─────────────┐                                │
│                         │   E2E Tests │  ← Full flow tests             │
│                         │  (10 tests) │    Deposit → Transfer →        │
│                         └──────┬──────┘    Withdraw                    │
│                                │                                        │
│                    ┌───────────┴───────────┐                           │
│                    │  Integration Tests    │  ← Cross-component        │
│                    │     (50 tests)        │    Circuit + Contract     │
│                    └───────────┬───────────┘                           │
│                                │                                        │
│           ┌────────────────────┴────────────────────┐                  │
│           │           Unit Tests                    │  ← Per function  │
│           │          (200+ tests)                   │    Isolated      │
│           └─────────────────────────────────────────┘                  │
│                                                                         │
│  Coverage Targets:                                                      │
│  • Circuits: 100% constraint coverage                                  │
│  • Contracts: 95% line coverage, 100% branch coverage                 │
│  • Rust: 90% line coverage                                            │
└─────────────────────────────────────────────────────────────────────────┘
```

**Fuzz Testing Setup:**

```rust
// tests/fuzz/transfer_fuzz.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_transfer_balance_conservation(
        sender_balance in 1u128..u128::MAX,
        amount in 1u128..u128::MAX,
    ) {
        prop_assume!(sender_balance >= amount);

        let sender = Account::new_with_balance(sender_balance);
        let recipient = Account::new_with_balance(0);

        let (new_sender, new_recipient) = execute_transfer(
            &sender, &recipient, amount
        )?;

        // Conservation invariant
        prop_assert_eq!(
            sender.balance + recipient.balance,
            new_sender.balance + new_recipient.balance
        );
    }
}
```

### 2.6 Bug Bounty Program

**Launch on Immunefi after audits complete**

| Severity | Reward | Examples |
|----------|--------|----------|
| **Critical** | $50,000 - $100,000 | Proof forgery, fund theft |
| **High** | $10,000 - $50,000 | Double-spend, privacy leak |
| **Medium** | $2,000 - $10,000 | DoS, griefing |
| **Low** | $500 - $2,000 | Minor issues |

---

## 3. Phase 2: Decentralization

**Timeline: 8-12 weeks**
**Priority: HIGH - Required for trustless operation**

### 3.1 Decentralized Sequencer Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    SEQUENCER NETWORK ARCHITECTURE                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                         USER TRANSACTIONS                               │
│                               │                                         │
│                               ▼                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                      MEMPOOL LAYER                                │  │
│  │                                                                   │  │
│  │   Encrypted transactions broadcast to all sequencers             │  │
│  │   Transactions include: encrypted_data, fee_commitment           │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                               │                                         │
│                               ▼                                         │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    SEQUENCER NETWORK                              │  │
│  │                                                                   │  │
│  │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐           │  │
│  │   │ Sequencer 1 │   │ Sequencer 2 │   │ Sequencer 3 │   ...     │  │
│  │   │  (Staked)   │   │  (Staked)   │   │  (Staked)   │           │  │
│  │   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘           │  │
│  │          │                 │                 │                    │  │
│  │          └─────────────────┼─────────────────┘                    │  │
│  │                            │                                      │  │
│  │                            ▼                                      │  │
│  │                  ┌─────────────────┐                              │  │
│  │                  │    CONSENSUS    │                              │  │
│  │                  │   (BFT / HotStuff)                             │  │
│  │                  └────────┬────────┘                              │  │
│  │                           │                                       │  │
│  └───────────────────────────┼───────────────────────────────────────┘  │
│                              │                                          │
│                              ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    BATCH PROCESSING                               │  │
│  │                                                                   │  │
│  │   1. Order transactions                                          │  │
│  │   2. Execute state transitions                                   │  │
│  │   3. Generate aggregated proof                                   │  │
│  │   4. Submit to Mantle L2                                         │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                              │                                          │
│                              ▼                                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    SETTLEMENT (Mantle L2)                         │  │
│  │                                                                   │  │
│  │   VeilocityVault.updateStateRoot(newRoot, aggregatedProof)       │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Sequencer Options Analysis

| Option | Pros | Cons | Effort |
|--------|------|------|--------|
| **Custom BFT** | Full control, optimized for privacy | Complex, needs 3f+1 nodes | 16-20 weeks |
| **Espresso Sequencer** | Battle-tested, shared security | Less customization | 6-8 weeks |
| **Astria** | Modular, Cosmos-based | Newer, less proven | 8-10 weeks |
| **Based Sequencing** | Inherits L1 security | Less control over ordering | 4-6 weeks |

**Recommended: Start with Espresso, migrate to custom later**

### 3.3 Sequencer Node Implementation

```rust
// sequencer/src/node.rs

pub struct SequencerNode {
    // Identity
    pub node_id: NodeId,
    pub stake: U256,

    // State
    state_manager: StateManager,
    mempool: Mempool,

    // Networking
    p2p: P2PNetwork,

    // Consensus
    consensus: BftConsensus,

    // Proving
    prover_pool: ProverPool,
}

impl SequencerNode {
    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                // Receive transactions from users
                tx = self.p2p.receive_transaction() => {
                    self.mempool.add(tx?)?;
                }

                // Participate in consensus
                proposal = self.consensus.receive_proposal() => {
                    self.handle_proposal(proposal?).await?;
                }

                // Check if we should propose
                _ = self.consensus.should_propose() => {
                    self.propose_batch().await?;
                }

                // Finalize batches
                finalized = self.consensus.receive_finalized() => {
                    self.finalize_batch(finalized?).await?;
                }
            }
        }
    }

    async fn propose_batch(&mut self) -> Result<()> {
        // 1. Select transactions from mempool
        let txs = self.mempool.select_batch(MAX_BATCH_SIZE)?;

        // 2. Execute and compute new state root
        let (new_root, state_diff) = self.state_manager
            .execute_batch(&txs)?;

        // 3. Create proposal
        let proposal = BatchProposal {
            transactions: txs,
            old_root: self.state_manager.current_root(),
            new_root,
            state_diff,
            sequencer: self.node_id,
        };

        // 4. Broadcast to network
        self.consensus.propose(proposal).await
    }

    async fn finalize_batch(&mut self, batch: FinalizedBatch) -> Result<()> {
        // 1. Generate aggregated proof
        let proof = self.prover_pool
            .generate_batch_proof(&batch)
            .await?;

        // 2. Submit to Mantle
        self.submit_to_l2(batch.new_root, proof).await?;

        // 3. Update local state
        self.state_manager.apply_batch(&batch)?;

        Ok(())
    }
}
```

### 3.4 Staking & Slashing

```solidity
// staking/SequencerStaking.sol

contract SequencerStaking {
    uint256 public constant MINIMUM_STAKE = 10_000 ether;  // 10k VEIL or MNT
    uint256 public constant SLASH_FRACTION = 10;  // 10% slashed for misbehavior
    uint256 public constant UNBONDING_PERIOD = 7 days;

    struct Sequencer {
        address operator;
        uint256 stake;
        uint256 unbondingTime;
        bool active;
    }

    mapping(address => Sequencer) public sequencers;
    address[] public activeSequencerSet;

    event SequencerRegistered(address indexed sequencer, uint256 stake);
    event SequencerSlashed(address indexed sequencer, uint256 amount, bytes32 reason);
    event SequencerUnbonding(address indexed sequencer, uint256 unbondingTime);

    function register() external payable {
        require(msg.value >= MINIMUM_STAKE, "Insufficient stake");
        require(!sequencers[msg.sender].active, "Already registered");

        sequencers[msg.sender] = Sequencer({
            operator: msg.sender,
            stake: msg.value,
            unbondingTime: 0,
            active: true
        });

        activeSequencerSet.push(msg.sender);
        emit SequencerRegistered(msg.sender, msg.value);
    }

    function slash(
        address sequencer,
        bytes32 reason,
        bytes calldata proof
    ) external {
        // Verify misbehavior proof (e.g., signed conflicting batches)
        require(verifyMisbehavior(sequencer, reason, proof), "Invalid proof");

        uint256 slashAmount = sequencers[sequencer].stake / SLASH_FRACTION;
        sequencers[sequencer].stake -= slashAmount;

        // Burn or redistribute slashed stake
        // ...

        emit SequencerSlashed(sequencer, slashAmount, reason);
    }
}
```

### 3.5 Data Availability

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DATA AVAILABILITY OPTIONS                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Option 1: Mantle DA (Default)                                         │
│  ─────────────────────────────                                         │
│  • Post encrypted transaction data to Mantle                           │
│  • Inherit Mantle's DA guarantees                                      │
│  • Cost: ~0.001 ETH per KB                                             │
│                                                                         │
│  Option 2: Celestia                                                    │
│  ──────────────────                                                    │
│  • Separate DA layer                                                   │
│  • Lower cost for high throughput                                      │
│  • Requires bridge trust assumptions                                   │
│                                                                         │
│  Option 3: EigenDA                                                     │
│  ─────────────────                                                     │
│  • Ethereum-aligned security                                           │
│  • Restaking-based                                                     │
│  • Good for cross-chain scenarios                                      │
│                                                                         │
│  Recommendation: Start with Mantle DA, add Celestia for scale          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Phase 3: Scalability

**Timeline: 8-12 weeks**
**Priority: HIGH - Required for production throughput**

### 4.1 Proof Aggregation

**Batch multiple proofs into one on-chain verification**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      PROOF AGGREGATION                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  WITHOUT AGGREGATION:                                                  │
│  ────────────────────                                                  │
│  TX₁ → Proof₁ → Verify (300k gas)                                     │
│  TX₂ → Proof₂ → Verify (300k gas)                                     │
│  TX₃ → Proof₃ → Verify (300k gas)                                     │
│  ...                                                                    │
│  TX₁₀₀ → Proof₁₀₀ → Verify (300k gas)                                 │
│  Total: 30M gas for 100 transactions                                   │
│                                                                         │
│  WITH AGGREGATION:                                                     │
│  ─────────────────                                                     │
│  TX₁ → Proof₁ ─┐                                                       │
│  TX₂ → Proof₂ ─┼──▶ Aggregate ──▶ AggProof ──▶ Verify (400k gas)     │
│  ...          ─┤                                                       │
│  TX₁₀₀ → Proof₁₀₀─┘                                                    │
│  Total: 400k gas for 100 transactions (75x improvement!)               │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Implementation:**

```noir
// circuits/src/aggregator.nr

// Recursive proof verification circuit
fn verify_batch(
    // Previous aggregated state
    prev_aggregate_root: pub Field,
    prev_proof: [Field; PROOF_SIZE],

    // New proofs to aggregate
    new_proofs: [Proof; BATCH_SIZE],
    new_transitions: [StateTransition; BATCH_SIZE],

    // Output
    new_aggregate_root: pub Field,
) {
    // 1. Verify previous aggregate (recursive)
    if prev_aggregate_root != GENESIS_ROOT {
        verify_recursive_proof(prev_proof);
    }

    // 2. Verify each new proof
    let mut current_root = prev_aggregate_root;
    for i in 0..BATCH_SIZE {
        // Verify state transition proof
        assert(new_transitions[i].old_root == current_root);
        verify_proof(new_proofs[i], new_transitions[i]);
        current_root = new_transitions[i].new_root;
    }

    // 3. Output new aggregate root
    assert(current_root == new_aggregate_root);
}
```

### 4.2 Proof Generation Performance

| Optimization | Speedup | Implementation |
|--------------|---------|----------------|
| **Multi-threading** | 4-8x | Rayon parallel proving |
| **GPU Acceleration** | 10-50x | CUDA/Metal Barretenberg |
| **Proof Caching** | 2-5x | Cache partial computations |
| **Circuit Optimization** | 1.5-2x | Reduce constraint count |

**GPU Proving Setup:**

```bash
# Install CUDA-enabled Barretenberg
bbup --version 1.0.0-beta.16 --cuda

# Or Metal for macOS
bbup --version 1.0.0-beta.16 --metal
```

```rust
// prover/src/gpu_prover.rs

pub struct GpuProver {
    device: GpuDevice,
    proving_key: ProvingKey,
}

impl GpuProver {
    pub async fn prove(&self, witness: &Witness) -> Result<Proof> {
        // Offload MSM and FFT to GPU
        let commitment = self.device.msm(&witness.values, &self.proving_key)?;
        let evaluation = self.device.fft(&commitment)?;

        // Generate proof
        Ok(Proof::from_evaluation(evaluation))
    }
}
```

### 4.3 Proof Outsourcing (Proof Market)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      PROOF MARKET INTEGRATION                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  USER                    VEILOCITY              PROOF MARKET            │
│                                                                         │
│  Request   ──────────▶  Create witness  ──────▶  Bid for proof         │
│                         (private data)          (Succinct/Bonsai)       │
│                                                        │                │
│                                                        ▼                │
│                                              Prover generates proof     │
│                                                        │                │
│  Verify    ◀──────────  Submit proof   ◀──────────────┘                │
│  (on-chain)                                                            │
│                                                                         │
│  Benefits:                                                              │
│  • No local proving hardware needed                                    │
│  • Competitive pricing through market                                  │
│  • Faster proofs (specialized hardware)                                │
│                                                                         │
│  Providers:                                                             │
│  • Succinct (SP1)                                                      │
│  • RiscZero (Bonsai)                                                   │
│  • Gevulot                                                             │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.4 State Compression

**Verkle Trees (Future Upgrade)**

```
Current (Merkle):
• Proof size: 20 × 32 bytes = 640 bytes
• Verification: 20 hashes

Verkle Trees:
• Proof size: ~100-150 bytes
• Verification: 1 pairing check

Benefits:
• 4-6x smaller proofs
• Faster verification
• Better for recursive proofs
```

### 4.5 Throughput Targets

| Phase | TPS | Latency | Gas/TX |
|-------|-----|---------|--------|
| **MVP** | 1-5 | 30-60s | 300k |
| **Aggregated** | 50-100 | 10-30s | 4k |
| **Optimized** | 500-1000 | 5-10s | 1k |
| **Future** | 10,000+ | 1-2s | <500 |

---

## 5. Phase 4: User Experience

**Timeline: 6-8 weeks**
**Priority: MEDIUM - Required for adoption**

### 5.1 TypeScript/JavaScript SDK

```typescript
// packages/sdk/src/index.ts

export class VeilocityClient {
    private provider: Provider;
    private vault: VeilocityVault;
    private stateManager: StateManager;
    private prover: RemoteProver;

    constructor(config: VeilocityConfig) {
        this.provider = new Provider(config.rpcUrl);
        this.vault = new VeilocityVault(config.vaultAddress, this.provider);
        this.stateManager = new StateManager(config.storageBackend);
        this.prover = new RemoteProver(config.proverUrl);
    }

    // ═══════════════════════════════════════════════════════════════
    //                         WALLET
    // ═══════════════════════════════════════════════════════════════

    /**
     * Create a new Veilocity wallet
     */
    static createWallet(): Wallet {
        const secret = randomBytes(32);
        const publicKey = poseidonHash([secret]);
        return new Wallet(secret, publicKey);
    }

    /**
     * Import wallet from secret
     */
    static importWallet(secret: Uint8Array): Wallet {
        const publicKey = poseidonHash([secret]);
        return new Wallet(secret, publicKey);
    }

    // ═══════════════════════════════════════════════════════════════
    //                         DEPOSIT
    // ═══════════════════════════════════════════════════════════════

    /**
     * Deposit ETH into the privacy pool
     * @param wallet - User's Veilocity wallet
     * @param amount - Amount in ETH (e.g., "1.5")
     * @returns Transaction receipt
     */
    async deposit(wallet: Wallet, amount: string): Promise<TransactionReceipt> {
        const amountWei = parseEther(amount);
        const commitment = poseidonHash([wallet.secret, amountWei]);

        // Send deposit transaction
        const tx = await this.vault.deposit(commitment, { value: amountWei });
        const receipt = await tx.wait();

        // Parse leaf index from event
        const event = receipt.events?.find(e => e.event === 'Deposit');
        const leafIndex = event?.args?.leafIndex.toNumber();

        // Update local state
        await this.stateManager.insertLeaf(commitment, leafIndex);
        await this.stateManager.creditBalance(wallet.publicKey, amountWei);

        return receipt;
    }

    // ═══════════════════════════════════════════════════════════════
    //                      PRIVATE TRANSFER
    // ═══════════════════════════════════════════════════════════════

    /**
     * Transfer privately to another Veilocity account
     * @param wallet - Sender's wallet
     * @param recipientPubKey - Recipient's public key (hex)
     * @param amount - Amount in ETH
     * @returns Transfer result with proof
     */
    async transfer(
        wallet: Wallet,
        recipientPubKey: string,
        amount: string
    ): Promise<TransferResult> {
        const amountWei = parseEther(amount);

        // 1. Get sender account and proof
        const senderAccount = await this.stateManager.getAccount(wallet.publicKey);
        const senderProof = await this.stateManager.getMerkleProof(senderAccount.index);

        // 2. Get or create recipient account
        let recipientAccount = await this.stateManager.getAccount(recipientPubKey);
        if (!recipientAccount) {
            recipientAccount = await this.stateManager.createAccount(recipientPubKey);
        }
        const recipientProof = await this.stateManager.getMerkleProof(recipientAccount.index);

        // 3. Build witness
        const witness: TransferWitness = {
            oldStateRoot: await this.stateManager.getRoot(),
            senderSecret: wallet.secret,
            senderBalance: senderAccount.balance,
            senderNonce: senderAccount.nonce,
            senderIndex: senderAccount.index,
            senderPath: senderProof,
            recipientPubKey: recipientPubKey,
            recipientBalance: recipientAccount.balance,
            recipientNonce: recipientAccount.nonce,
            recipientIndex: recipientAccount.index,
            recipientPath: recipientProof,
            amount: amountWei,
        };

        // 4. Generate proof
        const proof = await this.prover.prove('transfer', witness);

        // 5. Compute nullifier
        const nullifier = poseidonHash([
            wallet.secret,
            senderAccount.index,
            senderAccount.nonce
        ]);

        // 6. Update local state
        await this.stateManager.debitBalance(wallet.publicKey, amountWei);
        await this.stateManager.creditBalance(recipientPubKey, amountWei);
        await this.stateManager.useNullifier(nullifier);

        return {
            nullifier,
            proof,
            newSenderBalance: senderAccount.balance - amountWei,
        };
    }

    // ═══════════════════════════════════════════════════════════════
    //                        WITHDRAW
    // ═══════════════════════════════════════════════════════════════

    /**
     * Withdraw from privacy pool to public address
     * @param wallet - User's wallet
     * @param amount - Amount in ETH
     * @param recipient - Destination address (defaults to connected wallet)
     * @returns Transaction receipt
     */
    async withdraw(
        wallet: Wallet,
        amount: string,
        recipient?: string
    ): Promise<TransactionReceipt> {
        const amountWei = parseEther(amount);
        const recipientAddress = recipient || await this.provider.getAddress();

        // 1. Get account and proof
        const account = await this.stateManager.getAccount(wallet.publicKey);
        const merkleProof = await this.stateManager.getMerkleProof(account.index);
        const stateRoot = await this.stateManager.getRoot();

        // 2. Compute nullifier
        const nullifier = poseidonHash([
            wallet.secret,
            account.index,
            account.nonce
        ]);

        // 3. Build witness
        const witness: WithdrawWitness = {
            stateRoot,
            nullifier,
            amount: amountWei,
            recipient: recipientAddress,
            secret: wallet.secret,
            balance: account.balance,
            nonce: account.nonce,
            index: account.index,
            path: merkleProof,
        };

        // 4. Generate proof
        const proof = await this.prover.prove('withdraw', witness);

        // 5. Submit on-chain
        const tx = await this.vault.withdraw(
            nullifier,
            recipientAddress,
            amountWei,
            stateRoot,
            proof
        );
        const receipt = await tx.wait();

        // 6. Update local state
        await this.stateManager.debitBalance(wallet.publicKey, amountWei);
        await this.stateManager.useNullifier(nullifier);

        return receipt;
    }

    // ═══════════════════════════════════════════════════════════════
    //                         QUERIES
    // ═══════════════════════════════════════════════════════════════

    /**
     * Get private balance
     */
    async getBalance(wallet: Wallet): Promise<bigint> {
        const account = await this.stateManager.getAccount(wallet.publicKey);
        return account?.balance ?? 0n;
    }

    /**
     * Sync local state with on-chain events
     */
    async sync(): Promise<SyncResult> {
        const lastBlock = await this.stateManager.getLastSyncedBlock();
        const currentBlock = await this.provider.getBlockNumber();

        // Get deposit events
        const events = await this.vault.queryFilter(
            this.vault.filters.Deposit(),
            lastBlock + 1,
            currentBlock
        );

        // Update local tree
        for (const event of events) {
            await this.stateManager.insertLeaf(
                event.args.commitment,
                event.args.leafIndex.toNumber()
            );
        }

        await this.stateManager.setLastSyncedBlock(currentBlock);

        return {
            blocksProcessed: currentBlock - lastBlock,
            depositsFound: events.length,
        };
    }
}

// ═══════════════════════════════════════════════════════════════
//                         EXPORTS
// ═══════════════════════════════════════════════════════════════

export { Wallet } from './wallet';
export { poseidonHash } from './crypto';
export type {
    VeilocityConfig,
    TransferResult,
    SyncResult,
    TransferWitness,
    WithdrawWitness
} from './types';
```

**Usage Example:**

```typescript
import { VeilocityClient, Wallet } from '@veilocity/sdk';

// Initialize client
const client = new VeilocityClient({
    rpcUrl: 'https://rpc.mantle.xyz',
    vaultAddress: '0x...',
    proverUrl: 'https://prover.veilocity.xyz',
});

// Create wallet
const wallet = VeilocityClient.createWallet();
console.log('Your public key:', wallet.publicKey);

// Deposit
await client.deposit(wallet, '1.0');
console.log('Deposited 1.0 ETH');

// Check balance
const balance = await client.getBalance(wallet);
console.log('Private balance:', formatEther(balance));

// Transfer
const recipientPubKey = '0x...';
await client.transfer(wallet, recipientPubKey, '0.5');
console.log('Transferred 0.5 ETH privately');

// Withdraw
await client.withdraw(wallet, '0.3', '0xMyPublicAddress');
console.log('Withdrawn 0.3 ETH');
```

### 5.2 React Hooks Library

```typescript
// packages/react/src/hooks.ts

import { useCallback, useEffect, useState } from 'react';
import { VeilocityClient, Wallet } from '@veilocity/sdk';

export function useVeilocity(config: VeilocityConfig) {
    const [client] = useState(() => new VeilocityClient(config));
    const [wallet, setWallet] = useState<Wallet | null>(null);
    const [balance, setBalance] = useState<bigint>(0n);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<Error | null>(null);

    // Load wallet from storage
    useEffect(() => {
        const stored = localStorage.getItem('veilocity_wallet');
        if (stored) {
            const secret = new Uint8Array(JSON.parse(stored));
            setWallet(VeilocityClient.importWallet(secret));
        }
    }, []);

    // Sync balance
    useEffect(() => {
        if (!wallet) return;

        const syncBalance = async () => {
            const bal = await client.getBalance(wallet);
            setBalance(bal);
        };

        syncBalance();
        const interval = setInterval(syncBalance, 10000);
        return () => clearInterval(interval);
    }, [wallet, client]);

    const createWallet = useCallback(() => {
        const newWallet = VeilocityClient.createWallet();
        localStorage.setItem(
            'veilocity_wallet',
            JSON.stringify(Array.from(newWallet.secret))
        );
        setWallet(newWallet);
        return newWallet;
    }, []);

    const deposit = useCallback(async (amount: string) => {
        if (!wallet) throw new Error('No wallet');
        setLoading(true);
        setError(null);
        try {
            const receipt = await client.deposit(wallet, amount);
            const newBalance = await client.getBalance(wallet);
            setBalance(newBalance);
            return receipt;
        } catch (e) {
            setError(e as Error);
            throw e;
        } finally {
            setLoading(false);
        }
    }, [wallet, client]);

    const transfer = useCallback(async (recipient: string, amount: string) => {
        if (!wallet) throw new Error('No wallet');
        setLoading(true);
        setError(null);
        try {
            const result = await client.transfer(wallet, recipient, amount);
            setBalance(result.newSenderBalance);
            return result;
        } catch (e) {
            setError(e as Error);
            throw e;
        } finally {
            setLoading(false);
        }
    }, [wallet, client]);

    const withdraw = useCallback(async (amount: string, recipient?: string) => {
        if (!wallet) throw new Error('No wallet');
        setLoading(true);
        setError(null);
        try {
            const receipt = await client.withdraw(wallet, amount, recipient);
            const newBalance = await client.getBalance(wallet);
            setBalance(newBalance);
            return receipt;
        } catch (e) {
            setError(e as Error);
            throw e;
        } finally {
            setLoading(false);
        }
    }, [wallet, client]);

    return {
        wallet,
        balance,
        loading,
        error,
        createWallet,
        deposit,
        transfer,
        withdraw,
    };
}
```

**React Component Example:**

```tsx
// Example app
function PrivateWallet() {
    const {
        wallet,
        balance,
        loading,
        createWallet,
        deposit,
        withdraw
    } = useVeilocity({
        rpcUrl: 'https://rpc.mantle.xyz',
        vaultAddress: '0x...',
    });

    if (!wallet) {
        return (
            <button onClick={createWallet}>
                Create Private Wallet
            </button>
        );
    }

    return (
        <div>
            <h2>Private Balance: {formatEther(balance)} ETH</h2>

            <button
                onClick={() => deposit('1.0')}
                disabled={loading}
            >
                Deposit 1 ETH
            </button>

            <button
                onClick={() => withdraw('0.5')}
                disabled={loading}
            >
                Withdraw 0.5 ETH
            </button>

            {loading && <p>Processing...</p>}
        </div>
    );
}
```

### 5.3 Relayer Network

**Enable gasless transactions**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      RELAYER ARCHITECTURE                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  USER                                                                   │
│    │                                                                    │
│    │ 1. Generate withdrawal proof                                      │
│    │    (includes relayer fee in proof)                                │
│    │                                                                    │
│    ▼                                                                    │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    RELAYER NETWORK                               │   │
│  │                                                                  │   │
│  │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐          │   │
│  │   │  Relayer 1  │   │  Relayer 2  │   │  Relayer 3  │          │   │
│  │   │  Fee: 0.1%  │   │  Fee: 0.15% │   │  Fee: 0.2%  │          │   │
│  │   └─────────────┘   └─────────────┘   └─────────────┘          │   │
│  │          │                                                       │   │
│  │          │ 2. Cheapest relayer picks up request                 │   │
│  │          │                                                       │   │
│  └──────────┼───────────────────────────────────────────────────────┘   │
│             │                                                           │
│             │ 3. Submit on-chain (relayer pays gas)                    │
│             ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    VEILOCITY VAULT                               │   │
│  │                                                                  │   │
│  │   withdrawWithRelayer(                                          │   │
│  │       nullifier,                                                 │   │
│  │       recipient,        // User gets (amount - fee)             │   │
│  │       amount,                                                    │   │
│  │       relayerFee,       // Relayer gets fee                     │   │
│  │       relayer,                                                   │   │
│  │       root,                                                      │   │
│  │       proof             // Proof commits to fee amount          │   │
│  │   )                                                              │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  4. Funds distributed:                                                  │
│     • recipient receives (amount - relayerFee)                         │
│     • relayer receives relayerFee                                      │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Contract Update:**

```solidity
function withdrawWithRelayer(
    bytes32 nullifier,
    address payable recipient,
    uint256 amount,
    uint256 relayerFee,
    address payable relayer,
    bytes32 root,
    bytes calldata proof
) external nonReentrant whenNotPaused {
    require(!nullifiers[nullifier], "Nullifier used");
    require(rootHistory[root], "Invalid root");
    require(relayerFee <= amount / 10, "Fee too high");  // Max 10%

    // Public inputs include relayer fee commitment
    bytes32[] memory publicInputs = new bytes32[](6);
    publicInputs[0] = root;
    publicInputs[1] = nullifier;
    publicInputs[2] = bytes32(amount);
    publicInputs[3] = bytes32(uint256(uint160(recipient)));
    publicInputs[4] = bytes32(relayerFee);
    publicInputs[5] = bytes32(uint256(uint160(relayer)));

    require(verifier.verify(proof, publicInputs), "Invalid proof");

    nullifiers[nullifier] = true;
    totalValueLocked -= amount;

    // Split payment
    uint256 userAmount = amount - relayerFee;

    (bool s1, ) = recipient.call{value: userAmount}("");
    require(s1, "User transfer failed");

    (bool s2, ) = relayer.call{value: relayerFee}("");
    require(s2, "Relayer transfer failed");

    emit WithdrawalWithRelayer(nullifier, recipient, userAmount, relayer, relayerFee);
}
```

### 5.4 Mobile Wallet (React Native)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     MOBILE WALLET FEATURES                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Security:                                                              │
│  • Secure Enclave / Keystore for secret storage                        │
│  • Biometric authentication (Face ID / Fingerprint)                    │
│  • PIN backup                                                          │
│                                                                         │
│  Core Features:                                                         │
│  • QR code for receiving (encode public key)                           │
│  • Push notifications for incoming transfers                           │
│  • Transaction history                                                  │
│  • Fiat value display                                                  │
│                                                                         │
│  Advanced:                                                              │
│  • WalletConnect integration                                           │
│  • Hardware wallet support (Ledger)                                    │
│  • Multi-account management                                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Phase 5: Privacy Enhancements

**Timeline: Ongoing**
**Priority: HIGH - Core differentiator**

### 6.1 Anonymity Set Optimization

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ANONYMITY SET GROWTH                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Anonymity Set = Number of indistinguishable accounts                  │
│                                                                         │
│  CURRENT (MVP):                                                         │
│  • 1 pool, all denominations                                           │
│  • Anonymity set = total deposits                                      │
│  • Problem: 1.0 ETH deposit → 1.0 ETH withdrawal is linkable          │
│                                                                         │
│  ENHANCED (Fixed Denominations):                                        │
│  ┌─────────────────────────────────────────────────────────────┐       │
│  │  Pool 0.1 ETH    │    Pool 1.0 ETH    │    Pool 10 ETH    │       │
│  │  ──────────────  │    ─────────────   │    ────────────   │       │
│  │  500 deposits    │    200 deposits    │    50 deposits    │       │
│  │  Anon set: 500   │    Anon set: 200   │    Anon set: 50   │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│  User deposits 1.5 ETH:                                                │
│  → 1 × 1.0 ETH pool deposit                                           │
│  → 5 × 0.1 ETH pool deposits                                          │
│  → Each withdrawal anonymous within its pool                           │
│                                                                         │
│  ADVANCED (Unified Pool):                                              │
│  • Multiple protocols share one Merkle tree                            │
│  • Veilocity + Protocol B + Protocol C = Larger anonymity set         │
│  • Requires standardization and coordination                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 Timing Attack Mitigations

```
PROBLEM: Deposit at T₀, withdraw at T₁ → correlation by timing

MITIGATIONS:

1. MINIMUM WAIT TIME
   ─────────────────
   Enforce minimum time between deposit and withdrawal
   → Increases anonymity set of "recent deposits"

2. RANDOM DELAYS
   ──────────────
   User specifies max delay
   Withdrawal happens at random time within window

3. SCHEDULED WITHDRAWALS
   ──────────────────────
   Batch withdrawals at fixed intervals (e.g., every hour)
   All withdrawals in batch are indistinguishable by timing

4. DECOY TRANSACTIONS
   ──────────────────
   Generate fake deposit/withdrawal events
   Adds noise to timing analysis
```

### 6.3 Amount Privacy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    AMOUNT PRIVACY TECHNIQUES                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  PROBLEM:                                                               │
│  Deposit 1.234 ETH → Withdraw 1.234 ETH = Easily correlated            │
│                                                                         │
│  SOLUTIONS:                                                             │
│                                                                         │
│  1. FIXED DENOMINATIONS (Tornado-style)                                │
│     • Pools: 0.1, 1.0, 10.0 ETH                                       │
│     • User splits deposits/withdrawals                                 │
│     • Best privacy, less flexibility                                   │
│                                                                         │
│  2. SPLIT WITHDRAWALS                                                  │
│     • Deposit 1.234 ETH                                                │
│     • Withdraw 0.5 + 0.4 + 0.334 at different times                   │
│     • More flexibility, weaker than fixed denominations                │
│                                                                         │
│  3. CHANGE ADDRESSES                                                   │
│     • Withdraw different amount than deposited                        │
│     • Keep "change" in pool                                            │
│     • Requires multiple accounts                                       │
│                                                                         │
│  4. CONFIDENTIAL TRANSACTIONS (Future)                                 │
│     • Hide amounts on-chain using Pedersen commitments                │
│     • Even deposits/withdrawals are hidden                            │
│     • Much more complex, higher gas                                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.4 Viewing Keys (Compliance Option)

```rust
// For users who need transaction transparency (tax, audit)

pub struct ViewingKeySet {
    /// See all incoming transactions
    pub incoming_key: [u8; 32],

    /// See all outgoing transactions
    pub outgoing_key: [u8; 32],

    /// Full visibility (incoming + outgoing + balances)
    pub full_view_key: [u8; 32],
}

impl ViewingKeySet {
    /// Derive viewing keys from master secret
    pub fn derive(secret: &[u8; 32]) -> Self {
        Self {
            incoming_key: poseidon_hash(&[secret, b"incoming"]),
            outgoing_key: poseidon_hash(&[secret, b"outgoing"]),
            full_view_key: poseidon_hash(&[secret, b"full"]),
        }
    }
}

// User shares viewing key with auditor
// Auditor can see transactions but CANNOT spend funds
```

### 6.5 Private Smart Contract Calls (Future)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    PRIVATE CONTRACT EXECUTION                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  CURRENT: Only private transfers                                        │
│                                                                         │
│  FUTURE: Private DeFi interactions                                      │
│                                                                         │
│  Example: Private Uniswap Swap                                         │
│  ─────────────────────────────                                         │
│  1. User has private ETH balance                                       │
│  2. User generates proof: "I have ≥ X ETH"                            │
│  3. Shielded swap contract executes trade                             │
│  4. User receives private USDC balance                                 │
│  5. On-chain: only sees "some swap happened"                          │
│                                                                         │
│  Technical Approach:                                                    │
│  • Private input: balance proof, swap parameters                       │
│  • Public output: commitment to new balances                          │
│  • Contract verifies proof + executes swap logic                      │
│                                                                         │
│  Challenges:                                                            │
│  • Circuit complexity                                                   │
│  • AMM state integration                                               │
│  • MEV considerations                                                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Phase 6: Ecosystem Integration

**Timeline: 12-24 weeks**
**Priority: MEDIUM - Growth driver**

### 7.1 DeFi Integrations

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DEFI INTEGRATION ROADMAP                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  TIER 1: Basic Integrations                                            │
│  ─────────────────────────────                                         │
│  □ DEX Aggregators (1inch, Paraswap)                                   │
│    → Deposit from swap output                                          │
│    → Withdraw to swap input                                            │
│                                                                         │
│  □ Lending Protocols (Aave, Compound forks)                            │
│    → Private collateral deposits                                       │
│    → Shielded borrowing                                                │
│                                                                         │
│  TIER 2: Native Integrations                                           │
│  ───────────────────────────                                           │
│  □ Private AMM (Penumbra-style)                                        │
│    → Swap within shielded pool                                         │
│    → No public trade history                                           │
│                                                                         │
│  □ Private Yield Aggregator                                            │
│    → Stake privately                                                   │
│    → Earn yield in shielded balance                                    │
│                                                                         │
│  TIER 3: Advanced                                                       │
│  ────────────────                                                       │
│  □ Private NFT Marketplace                                             │
│    → Anonymous NFT purchases                                           │
│                                                                         │
│  □ DAO Voting                                                          │
│    → Private voting with public tally                                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Cross-Chain Privacy Bridge

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CROSS-CHAIN ARCHITECTURE                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│       MANTLE L2                              ETHEREUM L1                │
│                                                                         │
│  ┌─────────────────┐                    ┌─────────────────┐            │
│  │ VeilocityVault  │                    │ VeilocityVault  │            │
│  │    (Mantle)     │◀──── Bridge ─────▶│   (Ethereum)    │            │
│  │                 │                    │                 │            │
│  │ Private Pool    │                    │ Private Pool    │            │
│  └─────────────────┘                    └─────────────────┘            │
│                                                                         │
│  FLOW: Private L2 → Private L1                                         │
│  ─────────────────────────────                                         │
│                                                                         │
│  1. User has private balance on Mantle                                 │
│  2. Generate withdrawal proof for bridge                               │
│  3. Burn on Mantle (nullifier recorded)                                │
│  4. Bridge relays proof to Ethereum                                    │
│  5. Mint on Ethereum (new commitment)                                  │
│  6. User has private balance on Ethereum                               │
│                                                                         │
│  PRIVACY PRESERVED:                                                     │
│  • Bridge only sees: "X ETH moved L2 → L1"                            │
│  • Cannot link Mantle account to Ethereum account                     │
│                                                                         │
│  TRUST ASSUMPTIONS:                                                     │
│  • Bridge operators (can be decentralized)                            │
│  • Message relay correctness                                           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.3 Explorer & Analytics

```
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│                      VEILOCITY EXPLORER                                 │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  NETWORK OVERVIEW                                                       │
│  ════════════════                                                       │
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │
│  │     TVL      │  │   Deposits   │  │  Withdrawals │  │   Users    │ │
│  │   $12.5M     │  │    1,234     │  │     456      │  │   ~500     │ │
│  │   ▲ +5.2%    │  │   ▲ +12      │  │   ▲ +8       │  │  ▲ +23     │ │
│  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘ │
│                                                                         │
│  PRIVACY METRICS                                                        │
│  ═══════════════                                                        │
│                                                                         │
│  Anonymity Set Size        │████████████████████░░░░│ 1,234 accounts   │
│  Avg Time in Pool          │██████████████░░░░░░░░░░│ 4.2 days         │
│  Unique Depositors (24h)   │████████░░░░░░░░░░░░░░░░│ 45 users         │
│                                                                         │
│  RECENT PUBLIC EVENTS                                                   │
│  ════════════════════                                                   │
│                                                                         │
│  Block     │ Type       │ Amount    │ Commitment/Nullifier             │
│  ──────────┼────────────┼───────────┼─────────────────────────────────  │
│  12345     │ Deposit    │ 1.0 ETH   │ 0x1234...5678                    │
│  12344     │ Withdrawal │ 0.5 ETH   │ 0xabcd...ef01                    │
│  12343     │ Deposit    │ 0.1 ETH   │ 0x9876...5432                    │
│  12342     │ Withdrawal │ 2.0 ETH   │ 0xfedc...ba98                    │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  NOTE: Private transfer amounts and recipients are not visible  │   │
│  │        Only deposits and withdrawals appear in public events    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  CHARTS                                                                 │
│  ══════                                                                 │
│                                                                         │
│  TVL Over Time                    Daily Deposits/Withdrawals           │
│  ┌────────────────────────┐       ┌────────────────────────┐           │
│  │           ___.-──      │       │    ▓▓  ▓▓              │           │
│  │      _.──'             │       │ ▓▓ ▓▓▓▓▓▓▓▓  ▓▓ ▓▓    │           │
│  │   _─'                  │       │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  │           │
│  │ _/                     │       │░░░░░░░░░░░░░░░░░░░░░░░  │           │
│  └────────────────────────┘       └────────────────────────┘           │
│    Jan Feb Mar Apr May Jun         Mon Tue Wed Thu Fri Sat Sun         │
│                                    ▓ Deposits  ░ Withdrawals           │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 7.4 API Service

```yaml
# openapi.yaml
openapi: 3.0.0
info:
  title: Veilocity API
  version: 1.0.0

paths:
  /v1/status:
    get:
      summary: Get network status
      responses:
        200:
          content:
            application/json:
              schema:
                type: object
                properties:
                  tvl: { type: string }
                  depositCount: { type: integer }
                  currentRoot: { type: string }

  /v1/proof/generate:
    post:
      summary: Generate ZK proof (remote proving)
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                circuitType: { enum: [deposit, withdraw, transfer] }
                witness: { type: object }
      responses:
        200:
          content:
            application/json:
              schema:
                type: object
                properties:
                  proof: { type: string }
                  publicInputs: { type: array }

  /v1/relayer/quote:
    post:
      summary: Get relayer fee quote
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                amount: { type: string }
      responses:
        200:
          content:
            application/json:
              schema:
                type: object
                properties:
                  fee: { type: string }
                  relayerAddress: { type: string }
```

---

## 8. Phase 7: Governance & Sustainability

**Timeline: 12+ weeks**
**Priority: MEDIUM - Long-term viability**

### 8.1 Token Model (Optional)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         VEIL TOKEN MODEL                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  TOKEN: VEIL                                                            │
│  Total Supply: 100,000,000                                              │
│                                                                         │
│  UTILITY                                                                │
│  ───────                                                                │
│  1. Sequencer Staking                                                  │
│     • Minimum stake: 10,000 VEIL                                       │
│     • Earn transaction fees                                            │
│     • Slashable for misbehavior                                        │
│                                                                         │
│  2. Governance                                                          │
│     • Vote on protocol upgrades                                        │
│     • Vote on fee parameters                                           │
│     • Vote on compliance policies                                      │
│                                                                         │
│  3. Fee Discounts                                                       │
│     • Hold VEIL → reduced transaction fees                             │
│     • Tiered discount structure                                        │
│                                                                         │
│  4. Relayer Bonds                                                       │
│     • Relayers stake VEIL                                              │
│     • Slashed for failed transactions                                  │
│                                                                         │
│  DISTRIBUTION                                                           │
│  ────────────                                                           │
│  ┌─────────────────────────────────────────────────────────────┐       │
│  │  Team & Advisors    │████████████░░░░░░░░░░░░░░│ 20%        │       │
│  │  Treasury           │██████████░░░░░░░░░░░░░░░░│ 25%        │       │
│  │  Community/Airdrop  │████████░░░░░░░░░░░░░░░░░░│ 15%        │       │
│  │  Ecosystem Fund     │██████████░░░░░░░░░░░░░░░░│ 20%        │       │
│  │  Investors          │████████░░░░░░░░░░░░░░░░░░│ 15%        │       │
│  │  Liquidity          │██░░░░░░░░░░░░░░░░░░░░░░░░│  5%        │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
│  VESTING                                                                │
│  ───────                                                                │
│  • Team: 4-year vest, 1-year cliff                                     │
│  • Investors: 2-year vest, 6-month cliff                               │
│  • Community: No vesting (immediate)                                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Fee Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           FEE STRUCTURE                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  TRANSACTION FEES                                                       │
│  ────────────────                                                       │
│                                                                         │
│  │ Operation     │ Fee              │ Notes                    │       │
│  ├───────────────┼──────────────────┼──────────────────────────┤       │
│  │ Deposit       │ 0.1% of amount   │ Goes to protocol         │       │
│  │ Withdrawal    │ 0.1% of amount   │ Goes to protocol         │       │
│  │ Transfer      │ 0.01 ETH flat    │ Goes to sequencers       │       │
│  │ Relayer       │ Market-based     │ Set by relayers          │       │
│                                                                         │
│  FEE DISTRIBUTION                                                       │
│  ────────────────                                                       │
│                                                                         │
│       Protocol Fees (Deposit/Withdrawal)                               │
│       ──────────────────────────────────                               │
│       │                                                                │
│       ├── 50% → Protocol Treasury                                      │
│       │         (Development, audits, marketing)                       │
│       │                                                                │
│       ├── 30% → Sequencer Rewards                                      │
│       │         (Distributed by stake weight)                          │
│       │                                                                │
│       └── 20% → VEIL Stakers                                          │
│                 (Distributed to token stakers)                         │
│                                                                         │
│  PROJECTED REVENUE                                                      │
│  ─────────────────                                                      │
│                                                                         │
│  │ TVL          │ Monthly Volume │ Monthly Revenue │                   │
│  ├──────────────┼────────────────┼─────────────────┤                   │
│  │ $1M          │ $5M            │ $10k            │                   │
│  │ $10M         │ $50M           │ $100k           │                   │
│  │ $100M        │ $500M          │ $1M             │                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Governance Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       GOVERNANCE FRAMEWORK                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  GOVERNANCE SCOPE                                                       │
│  ────────────────                                                       │
│                                                                         │
│  ✓ Protocol upgrades (contract changes)                                │
│  ✓ Fee parameter adjustments                                           │
│  ✓ Sequencer requirements                                              │
│  ✓ Treasury spending                                                   │
│  ✓ Emergency actions (pause, unpause)                                  │
│  ✓ Compliance policy changes                                           │
│                                                                         │
│  PROPOSAL PROCESS                                                       │
│  ────────────────                                                       │
│                                                                         │
│  1. DISCUSSION (Forum)                                                  │
│     │  Duration: 3-7 days                                              │
│     │  Requirement: None                                               │
│     ▼                                                                   │
│  2. TEMPERATURE CHECK (Snapshot)                                        │
│     │  Duration: 3 days                                                │
│     │  Requirement: 10k VEIL to propose                                │
│     │  Threshold: >50% approval                                        │
│     ▼                                                                   │
│  3. ON-CHAIN VOTE (Governor)                                           │
│     │  Duration: 5 days                                                │
│     │  Quorum: 4% of supply                                            │
│     │  Threshold: >50% approval                                        │
│     ▼                                                                   │
│  4. TIMELOCK                                                            │
│     │  Duration: 2 days                                                │
│     │  Can be vetoed by Guardian (security)                            │
│     ▼                                                                   │
│  5. EXECUTION                                                           │
│                                                                         │
│  ROLES                                                                  │
│  ─────                                                                  │
│  • Guardian (Multisig): Emergency pause, veto malicious proposals      │
│  • Timelock: Executes passed proposals                                 │
│  • Governor: Vote counting, proposal lifecycle                         │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 9. Implementation Timeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ROADMAP TIMELINE                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  2024                                                                   │
│  ────                                                                   │
│                                                                         │
│  Q1: SECURITY & FOUNDATION                                              │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Week 1-2   │ Deploy real UltraVerifier                           │  │
│  │ Week 3-4   │ Comprehensive test coverage                         │  │
│  │ Week 5-8   │ Security audits (circuits + contracts)              │  │
│  │ Week 9-10  │ Audit remediation                                   │  │
│  │ Week 11-12 │ Bug bounty launch                                   │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  Q2: DECENTRALIZATION & SDK                                            │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Week 1-4   │ TypeScript SDK development                          │  │
│  │ Week 5-8   │ Sequencer network design                            │  │
│  │ Week 9-12  │ Sequencer implementation + testing                  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  Q3: SCALING & LAUNCH                                                  │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Week 1-4   │ Proof aggregation                                   │  │
│  │ Week 5-6   │ Relayer network                                     │  │
│  │ Week 7-8   │ Explorer & analytics                                │  │
│  │ Week 9-10  │ Testnet launch (Mantle Sepolia)                     │  │
│  │ Week 11-12 │ Mainnet launch (Mantle)                             │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  Q4: ECOSYSTEM & GROWTH                                                │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Week 1-4   │ DeFi integrations                                   │  │
│  │ Week 5-8   │ Cross-chain bridge                                  │  │
│  │ Week 9-12  │ Mobile wallet                                       │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
│  2025                                                                   │
│  ────                                                                   │
│                                                                         │
│  Q1: GOVERNANCE & SUSTAINABILITY                                       │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ Token launch (if applicable)                                      │  │
│  │ Governance deployment                                             │  │
│  │ Community transition                                              │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Milestone Checklist

**Phase 1: Security (Weeks 1-12)**
```
□ Deploy UltraVerifier (replace MockVerifier)
□ Achieve 90%+ test coverage
□ Complete circuit audit
□ Complete contract audit
□ Remediate all critical/high findings
□ Launch bug bounty on Immunefi
□ Formal verification of core invariants
```

**Phase 2: Decentralization (Weeks 13-24)**
```
□ Design sequencer architecture
□ Implement sequencer node
□ Deploy 3+ node testnet
□ Implement staking contracts
□ Test consensus under adversarial conditions
□ Document operator requirements
```

**Phase 3: Scalability (Weeks 17-28)**
```
□ Implement proof aggregation circuit
□ Achieve 100+ TPS on testnet
□ GPU proving support
□ Proof market integration (optional)
□ Benchmark and optimize
```

**Phase 4: User Experience (Weeks 13-20)**
```
□ Publish @veilocity/sdk to npm
□ React hooks library
□ Documentation site
□ Example applications
□ Relayer network (3+ relayers)
```

**Phase 5: Launch (Weeks 25-30)**
```
□ Testnet deployment (Mantle Sepolia)
□ Testnet incentivized program
□ Security review of deployment
□ Mainnet deployment (Mantle)
□ Launch marketing campaign
```

---

## 10. Resource Requirements

### 10.1 Team Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         TEAM STRUCTURE                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  CORE TEAM (4-6 FTE)                                                   │
│  ────────────────────                                                  │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │ Role                 │ Responsibilities              │ Count   │   │
│  ├──────────────────────┼───────────────────────────────┼─────────┤   │
│  │ Protocol Lead        │ Architecture, research        │ 1       │   │
│  │ ZK Engineer          │ Circuit development, proving  │ 1-2     │   │
│  │ Smart Contract Dev   │ Solidity, security            │ 1       │   │
│  │ Backend Engineer     │ Rust, sequencer, infra        │ 1-2     │   │
│  │ Frontend/SDK Dev     │ TypeScript, React             │ 1       │   │
│  └──────────────────────┴───────────────────────────────┴─────────┘   │
│                                                                         │
│  EXTENDED TEAM (Part-time / Contract)                                  │
│  ────────────────────────────────────                                  │
│                                                                         │
│  • DevOps Engineer (0.5 FTE)                                          │
│  • Security Researcher (0.5 FTE)                                       │
│  • Technical Writer (0.25 FTE)                                         │
│  • Community Manager (0.5 FTE)                                         │
│                                                                         │
│  ADVISORS                                                               │
│  ────────                                                               │
│                                                                         │
│  • Cryptography advisor                                                │
│  • Legal/compliance advisor                                            │
│  • DeFi ecosystem advisor                                              │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 10.2 Budget Estimate

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         BUDGET BREAKDOWN                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  CATEGORY                          │ 12-MONTH ESTIMATE                 │
│  ──────────────────────────────────┼───────────────────────────────────│
│                                    │                                   │
│  PERSONNEL                         │                                   │
│  ├─ Core Team (5 FTE × $150k avg)  │ $750,000                         │
│  ├─ Extended Team                  │ $150,000                         │
│  └─ Advisors                       │ $50,000                          │
│  Subtotal                          │ $950,000                         │
│                                    │                                   │
│  SECURITY                          │                                   │
│  ├─ Circuit Audit                  │ $75,000                          │
│  ├─ Contract Audit                 │ $100,000                         │
│  ├─ Cryptographic Review           │ $40,000                          │
│  ├─ Bug Bounty Pool                │ $100,000                         │
│  └─ Ongoing Security               │ $35,000                          │
│  Subtotal                          │ $350,000                         │
│                                    │                                   │
│  INFRASTRUCTURE                    │                                   │
│  ├─ Cloud/Hosting                  │ $60,000                          │
│  ├─ RPC Nodes                      │ $24,000                          │
│  ├─ Monitoring/Alerting            │ $12,000                          │
│  └─ Development Tools              │ $24,000                          │
│  Subtotal                          │ $120,000                         │
│                                    │                                   │
│  LEGAL & COMPLIANCE                │                                   │
│  ├─ Legal Counsel                  │ $75,000                          │
│  └─ Regulatory Analysis            │ $25,000                          │
│  Subtotal                          │ $100,000                         │
│                                    │                                   │
│  MARKETING & GROWTH                │                                   │
│  ├─ Developer Relations            │ $50,000                          │
│  ├─ Events/Conferences             │ $30,000                          │
│  └─ Content/Documentation          │ $20,000                          │
│  Subtotal                          │ $100,000                         │
│                                    │                                   │
│  CONTINGENCY (10%)                 │ $162,000                         │
│                                    │                                   │
│  ════════════════════════════════════════════════════════════════════ │
│  TOTAL (12 months)                 │ $1,782,000                       │
│  ════════════════════════════════════════════════════════════════════ │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 10.3 Funding Sources

| Source | Amount | Stage | Notes |
|--------|--------|-------|-------|
| **Grants** | $100-500k | Early | Mantle Ecosystem, EF, Aztec |
| **Pre-seed** | $500k-1M | Security phase | Angels, small funds |
| **Seed** | $2-5M | Post-audit | Crypto VCs |
| **Ecosystem funds** | Variable | Ongoing | Integration incentives |

**Grant Opportunities:**

| Program | Focus | Amount |
|---------|-------|--------|
| Mantle Grants | Ecosystem projects | $50-500k |
| Ethereum Foundation | Privacy research | $50-200k |
| Aztec Grants | Noir ecosystem | $25-100k |
| Protocol Labs | ZK infrastructure | $50-200k |

---

## 11. Competitive Analysis

### 11.1 Landscape Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    PRIVACY PROTOCOL LANDSCAPE                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│                        HIGH PRIVACY                                     │
│                            ▲                                            │
│                            │                                            │
│         Zcash ●            │           ● Aztec                         │
│                            │                                            │
│                            │                                            │
│    Tornado ●               │              ● Penumbra                   │
│    (Sanctioned)            │                                            │
│                            │                                            │
│              ● Railgun     │        ● VEILOCITY                        │
│                            │          (Target Position)                 │
│                            │                                            │
│ ◀────────────────────────────────────────────────────────────────────▶ │
│ COMPLEX                                                      SIMPLE    │
│ (Own Chain)                                              (EVM Native)   │
│                            │                                            │
│                            │                                            │
│                            │                                            │
│        Secret ●            │                                            │
│        Network             │                                            │
│                            │                                            │
│                            ▼                                            │
│                       LOW PRIVACY                                       │
│                    (Mixers, Tumblers)                                  │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 11.2 Detailed Comparison

| Feature | Veilocity | Aztec | Railgun | Tornado |
|---------|-----------|-------|---------|---------|
| **Chain** | Mantle L2 | Aztec L2 | Multi-chain | Ethereum |
| **Privacy Model** | Account-based | UTXO | UTXO | Fixed denom |
| **Proof System** | UltraPlonk | Honk | Groth16 | Groth16 |
| **Circuit Language** | Noir | Noir | Circom | Circom |
| **EVM Compatible** | Yes | No (Noir contracts) | Yes | Yes |
| **Status** | Development | Testnet | Live | Sanctioned |
| **Compliance Option** | Planned | No | Optional | No |
| **Gas Cost** | Low (L2) | Medium | High | High |

### 11.3 Veilocity Differentiation

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    VEILOCITY COMPETITIVE ADVANTAGES                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  1. MANTLE-NATIVE                                                       │
│     ─────────────                                                       │
│     • First privacy solution on Mantle                                 │
│     • Benefit from Mantle's low fees                                   │
│     • Deep ecosystem integration                                        │
│                                                                         │
│  2. MODERN TECH STACK                                                   │
│     ─────────────────                                                   │
│     • Noir (latest ZK DSL, better than Circom)                        │
│     • UltraPlonk (efficient proofs)                                    │
│     • Rust execution (performance + safety)                            │
│                                                                         │
│  3. COMPLIANCE-READY                                                    │
│     ────────────────                                                    │
│     • Optional viewing keys                                            │
│     • Designed with regulatory considerations                          │
│     • Not a "mixer" but a "private execution layer"                   │
│                                                                         │
│  4. DEVELOPER EXPERIENCE                                                │
│     ────────────────────                                                │
│     • TypeScript SDK                                                   │
│     • Comprehensive documentation                                       │
│     • Clear API boundaries                                             │
│                                                                         │
│  5. ACCOUNT-BASED MODEL                                                 │
│     ────────────────────                                                │
│     • Familiar mental model (vs UTXO)                                  │
│     • Simpler integration                                              │
│     • Better UX for recurring payments                                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 12. Risk Assessment

### 12.1 Risk Matrix

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         RISK MATRIX                                     │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  IMPACT                                                                 │
│    ▲                                                                    │
│    │                                                                    │
│ HIGH│  ┌─────────────┐     ┌─────────────┐                             │
│    │  │ Regulatory  │     │ Smart       │                             │
│    │  │ Action      │     │ Contract    │                             │
│    │  │             │     │ Exploit     │                             │
│    │  └─────────────┘     └─────────────┘                             │
│    │                                                                    │
│ MED │        ┌─────────────┐     ┌─────────────┐                       │
│    │        │ Circuit     │     │ Sequencer   │                       │
│    │        │ Soundness   │     │ Centralization                      │
│    │        └─────────────┘     └─────────────┘                       │
│    │                                                                    │
│ LOW │              ┌─────────────┐     ┌─────────────┐                 │
│    │              │ Key         │     │ Low         │                 │
│    │              │ Management  │     │ Adoption    │                 │
│    │              └─────────────┘     └─────────────┘                 │
│    │                                                                    │
│    └────────────────────────────────────────────────────────────▶      │
│         LOW              MEDIUM              HIGH                       │
│                       LIKELIHOOD                                        │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 12.2 Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Smart Contract Exploit** | Medium | Critical | Multiple audits, formal verification, bug bounty, gradual TVL increase |
| **Circuit Soundness Bug** | Low | Critical | Specialized ZK audit, extensive testing, formal verification |
| **Regulatory Action** | Medium | High | Compliance features, legal counsel, transparent operation, optional viewing keys |
| **Sequencer Centralization** | High (MVP) | Medium | Clear decentralization roadmap, interim multisig |
| **Low Adoption** | Medium | Medium | Developer grants, DeFi integrations, marketing |
| **Key Management Failure** | Medium | Medium | Hardware wallet support, social recovery, education |
| **Proving Performance** | Low | Low | GPU support, proof markets, optimization |

### 12.3 Regulatory Considerations

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    REGULATORY STRATEGY                                  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  POSITIONING                                                            │
│  ───────────                                                            │
│  • "Private execution layer" not "mixer"                               │
│  • Legitimate use cases: payroll, donations, business transactions     │
│  • Optional compliance features from day 1                             │
│                                                                         │
│  COMPLIANCE FEATURES                                                    │
│  ───────────────────                                                    │
│  ✓ Viewing keys for authorized auditors                                │
│  ✓ Optional KYC for larger withdrawals                                 │
│  ✓ Sanctions screening integration (optional)                          │
│  ✓ Transaction reporting for users                                     │
│                                                                         │
│  LEGAL STRUCTURE                                                        │
│  ───────────────                                                        │
│  • Foundation in privacy-friendly jurisdiction                         │
│  • Clear terms of service                                              │
│  • Proactive regulator engagement                                      │
│                                                                         │
│  PRECEDENTS TO MONITOR                                                  │
│  ────────────────────                                                   │
│  • Tornado Cash sanctions and litigation                               │
│  • Railgun regulatory response                                         │
│  • EU MiCA implementation                                              │
│  • US stablecoin regulation                                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Appendix A: Immediate Action Items

### Week 1-2 Checklist

```
CRITICAL PATH
═════════════

□ 1. Deploy Real Verifier
     ├─ Compile circuits: nargo compile
     ├─ Generate VK: bb write_vk
     ├─ Generate Solidity: bb write_solidity_verifier
     ├─ Test verification gas costs
     └─ Update deployment scripts

□ 2. Security Foundations
     ├─ Set up CI/CD with security checks
     ├─ Add slither + mythril to pipeline
     ├─ Configure fuzzing (foundry fuzz)
     └─ Document threat model

□ 3. Testing Infrastructure
     ├─ Unit tests for all circuits
     ├─ Integration tests for full flows
     ├─ Gas benchmarks
     └─ Coverage reporting

□ 4. Documentation
     ├─ API documentation
     ├─ Architecture diagrams
     └─ Deployment runbooks
```

### Contacts & Resources

**Audit Firms:**
- Zellic: hello@zellic.io
- OpenZeppelin: contact@openzeppelin.com
- Trail of Bits: contact@trailofbits.com
- Spearbit: info@spearbit.com

**Grants:**
- Mantle: grants@mantle.xyz
- Ethereum Foundation: grants@ethereum.org
- Aztec/Noir: grants@aztec.network

**Community:**
- Noir Discord: discord.gg/aztec
- Mantle Discord: discord.gg/mantle
- ZK Research: zkresearch.io

---

## Appendix B: Technical Specifications

### Circuit Constraints Summary

| Circuit | Constraints | Proving Time (CPU) | Proving Time (GPU) |
|---------|-------------|--------------------|--------------------|
| Deposit | ~500 | <1s | <0.1s |
| Withdraw | ~8,000 | ~5s | ~0.5s |
| Transfer | ~15,000 | ~10s | ~1s |
| Aggregator (100 tx) | ~500,000 | ~5min | ~30s |

### Gas Costs (Estimated)

| Operation | Gas | Cost @ 1 gwei | Cost @ 10 gwei |
|-----------|-----|---------------|----------------|
| Deposit | 80,000 | $0.08 | $0.80 |
| Withdraw (single) | 350,000 | $0.35 | $3.50 |
| Withdraw (batched) | 5,000 | $0.005 | $0.05 |
| State root update | 100,000 | $0.10 | $1.00 |

### Storage Requirements

| Component | Size per Account | Size for 1M Accounts |
|-----------|------------------|----------------------|
| Merkle leaf | 32 bytes | 32 MB |
| Merkle path | 640 bytes | N/A (computed) |
| Account data | ~100 bytes | 100 MB |
| Transaction history | ~200 bytes/tx | Variable |

---

*Document Version: 1.0.0*
*Last Updated: 2024*
*Project: Veilocity - From Hackathon to Production*
