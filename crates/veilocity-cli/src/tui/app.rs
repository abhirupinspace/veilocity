//! Veilocity TUI Application State
//!
//! Central state container and mode management for the TUI.

use std::collections::VecDeque;

use alloy::signers::local::PrivateKeySigner;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;

use veilocity_core::account::AccountSecret;
use veilocity_core::poseidon::{field_to_bytes, field_to_hex, PoseidonHasher};
use veilocity_core::state::StateManager;

use crate::config::Config;
use crate::tui::forms::{DepositForm, WithdrawForm};
use crate::wallet::WalletManager;

/// Maximum number of live events to keep in buffer
const MAX_EVENTS: usize = 100;

/// Application events from background tasks and user input
#[derive(Debug, Clone)]
pub enum AppEvent {
    // User input
    Tick,
    Quit,

    // Background updates
    BalanceUpdated(u128),
    NewDeposit(DepositEventInfo),
    NewWithdrawal(WithdrawalEventInfo),
    StateRootUpdated { batch_index: u64 },
    SyncProgress { current: u64, target: u64 },
    SyncMessage(String),
    BlockNumberUpdated(u64),
    ConnectionStatusChanged(ConnectionStatus),

    // ZK Proof Progress
    ZkProgress {
        stage: ZkStage,
        progress: f32,
        message: String,
    },

    // Operation results
    DepositCompleted { tx_hash: String, amount: u128 },
    WithdrawCompleted { tx_hash: String, amount: u128 },
    SyncCompleted { duration_ms: u64 },
    OperationFailed { error: String },

    // Wallet
    WalletUnlocked,
    WalletLocked,
}

/// Deposit event info for display
#[derive(Debug, Clone)]
pub struct DepositEventInfo {
    pub amount_wei: u128,
    pub leaf_index: u64,
    pub is_own: bool,
}

/// Withdrawal event info for display
#[derive(Debug, Clone)]
pub struct WithdrawalEventInfo {
    pub amount_wei: u128,
    pub recipient: String,
    pub is_own: bool,
}

/// Application mode
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Dashboard,
    DepositForm,
    WithdrawForm,
    PasswordPrompt { purpose: PasswordPurpose },
    ZkProofProgress { stage: ZkStage, progress: f32 },
    Confirmation { action: PendingAction },
    Error { message: String },
    Help,
    CreateWallet,
    WalletDetails,
    DeleteWalletConfirm,
}

/// Purpose for password prompt
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordPurpose {
    Unlock,
    Deposit,
    Withdraw,
}

/// ZK proof generation stage
#[derive(Debug, Clone, PartialEq)]
pub enum ZkStage {
    CommitmentGeneration,
    WitnessGeneration,
    ConstraintSatisfaction,
    PolynomialCommitment,
    ProofComputation,
    Verification,
    Submitting,
}

impl ZkStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            ZkStage::CommitmentGeneration => "Generating commitment...",
            ZkStage::WitnessGeneration => "Generating witness...",
            ZkStage::ConstraintSatisfaction => "Satisfying constraints...",
            ZkStage::PolynomialCommitment => "Computing polynomial commitments...",
            ZkStage::ProofComputation => "Computing proof...",
            ZkStage::Verification => "Verifying proof...",
            ZkStage::Submitting => "Submitting transaction...",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            ZkStage::CommitmentGeneration => 0,
            ZkStage::WitnessGeneration => 1,
            ZkStage::ConstraintSatisfaction => 2,
            ZkStage::PolynomialCommitment => 3,
            ZkStage::ProofComputation => 4,
            ZkStage::Verification => 5,
            ZkStage::Submitting => 6,
        }
    }

    pub fn total_stages() -> usize {
        7
    }
}

/// Pending action for confirmation
#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    Deposit { amount: f64 },
    Withdraw { amount: f64, recipient: String },
    Sync,
}

/// Dashboard panel selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Panel {
    Balance,
    Events,
    History,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Balance => Panel::Events,
            Panel::Events => Panel::History,
            Panel::History => Panel::Balance,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Panel::Balance => Panel::History,
            Panel::Events => Panel::Balance,
            Panel::History => Panel::Events,
        }
    }
}

/// Wallet state
#[derive(Default)]
pub struct WalletState {
    pub is_unlocked: bool,
    pub address: Option<String>,
    pub pubkey: Option<String>,
    pub signer: Option<PrivateKeySigner>,
    pub secret: Option<AccountSecret>,
}

impl std::fmt::Debug for WalletState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletState")
            .field("is_unlocked", &self.is_unlocked)
            .field("address", &self.address)
            .field("pubkey", &self.pubkey)
            .field("signer", &self.signer.is_some())
            .field("secret", &self.secret.is_some())
            .finish()
    }
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connected,
    Reconnecting,
}

/// Sync state
#[derive(Debug, Default)]
pub struct SyncState {
    pub is_syncing: bool,
    pub last_block: u64,
    pub target_block: u64,
    pub state_root: String,
    pub connection_status: ConnectionStatus,
    pub last_sync: Option<DateTime<Utc>>,
}

impl SyncState {
    pub fn progress_percent(&self) -> f32 {
        if self.target_block == 0 || self.target_block <= self.last_block {
            return 100.0;
        }
        (self.last_block as f32 / self.target_block as f32) * 100.0
    }
}

/// Live event for display
#[derive(Debug, Clone)]
pub struct LiveEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: LiveEventType,
    pub is_own: bool,
}

/// Live event type
#[derive(Debug, Clone)]
pub enum LiveEventType {
    Deposit { amount_mnt: f64, leaf_index: u64 },
    Withdrawal { amount_mnt: f64, recipient: String },
    StateRootUpdate { batch_index: u64 },
    SyncStarted,
    SyncCompleted { duration_ms: u64 },
    Error { message: String },
    Connected,
    Disconnected,
}

/// Transaction record for history display
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub tx_type: TransactionType,
    pub amount_mnt: f64,
    pub status: TransactionStatus,
    pub timestamp: DateTime<Utc>,
    pub tx_hash: Option<String>,
    pub recipient: Option<String>,
}

/// Transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdraw,
    Transfer,
}

impl TransactionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Deposit => "DEPOSIT",
            TransactionType::Withdraw => "WITHDRAW",
            TransactionType::Transfer => "TRANSFER",
        }
    }
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Confirmed => "confirmed",
            TransactionStatus::Failed => "failed",
        }
    }
}

/// Password input buffer
#[derive(Debug, Default)]
pub struct PasswordInput {
    pub buffer: String,
    pub error: Option<String>,
}

/// Create wallet form state
#[derive(Debug, Default)]
pub struct CreateWalletForm {
    pub password: String,
    pub confirm_password: String,
    pub active_field: u8, // 0 = password, 1 = confirm
    pub error: Option<String>,
}

/// Main application state
pub struct App {
    // Configuration
    pub config: Config,

    // UI state
    pub mode: AppMode,
    pub selected_panel: Panel,
    pub scroll_offset: usize,
    pub should_quit: bool,

    // Wallet state
    pub wallet_state: WalletState,
    pub wallet_exists: bool,

    // Data
    pub balance: Option<u128>,
    pub account_index: Option<u64>,
    pub account_nonce: Option<u64>,
    pub events: VecDeque<LiveEvent>,
    pub history: Vec<TransactionRecord>,
    pub sync_state: SyncState,

    // Forms
    pub deposit_form: DepositForm,
    pub withdraw_form: WithdrawForm,
    pub password_input: PasswordInput,
    pub create_wallet_form: CreateWalletForm,

    // Event channels
    pub event_tx: mpsc::Sender<AppEvent>,
    pub event_rx: mpsc::Receiver<AppEvent>,

    // Network info
    pub network_name: String,
    pub current_block: u64,

    // Animation state
    pub tick: usize,

    // ZK progress message for display
    pub zk_message: String,

    // Sync status message for display
    pub sync_message: String,
}

impl App {
    /// Create a new App instance
    pub async fn new(config: Config) -> anyhow::Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(100);

        // Check if wallet exists
        let wallet_path = config.wallet_path();
        let wallet_exists = wallet_path.exists();

        // Determine network name
        let network_name = if config.network.chain_id == 5000 {
            "Mainnet".to_string()
        } else if config.network.chain_id == 5003 {
            "Sepolia".to_string()
        } else {
            format!("Chain {}", config.network.chain_id)
        };

        // Start in appropriate mode based on wallet state
        let initial_mode = if wallet_exists {
            AppMode::PasswordPrompt {
                purpose: PasswordPurpose::Unlock,
            }
        } else {
            AppMode::CreateWallet
        };

        Ok(Self {
            config,
            mode: initial_mode,
            selected_panel: Panel::Balance,
            scroll_offset: 0,
            should_quit: false,
            wallet_state: WalletState::default(),
            wallet_exists,
            balance: None,
            account_index: None,
            account_nonce: None,
            events: VecDeque::with_capacity(MAX_EVENTS),
            history: Vec::new(),
            sync_state: SyncState::default(),
            deposit_form: DepositForm::default(),
            withdraw_form: WithdrawForm::default(),
            password_input: PasswordInput::default(),
            create_wallet_form: CreateWalletForm::default(),
            event_tx,
            event_rx,
            network_name,
            current_block: 0,
            tick: 0,
            zk_message: String::new(),
            sync_message: String::new(),
        })
    }

    /// Handle an application event
    pub fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Tick => {
                // Periodic tick - update animations
                self.tick = self.tick.wrapping_add(1);
            }
            AppEvent::Quit => {
                self.should_quit = true;
            }
            AppEvent::BalanceUpdated(balance) => {
                self.balance = Some(balance);
            }
            AppEvent::NewDeposit(info) => {
                self.add_live_event(LiveEvent {
                    timestamp: Utc::now(),
                    event_type: LiveEventType::Deposit {
                        amount_mnt: wei_to_mnt(info.amount_wei),
                        leaf_index: info.leaf_index,
                    },
                    is_own: info.is_own,
                });
            }
            AppEvent::NewWithdrawal(info) => {
                self.add_live_event(LiveEvent {
                    timestamp: Utc::now(),
                    event_type: LiveEventType::Withdrawal {
                        amount_mnt: wei_to_mnt(info.amount_wei),
                        recipient: info.recipient,
                    },
                    is_own: info.is_own,
                });
            }
            AppEvent::StateRootUpdated { batch_index } => {
                self.add_live_event(LiveEvent {
                    timestamp: Utc::now(),
                    event_type: LiveEventType::StateRootUpdate { batch_index },
                    is_own: false,
                });
            }
            AppEvent::SyncProgress { current, target } => {
                self.sync_state.last_block = current;
                self.sync_state.target_block = target;
                self.sync_state.is_syncing = current < target;
            }
            AppEvent::SyncMessage(message) => {
                self.sync_message = message;
            }
            AppEvent::BlockNumberUpdated(block) => {
                self.current_block = block;
                self.sync_state.target_block = block;
            }
            AppEvent::ZkProgress {
                stage,
                progress,
                message,
            } => {
                self.zk_message = message;
                self.mode = AppMode::ZkProofProgress { stage, progress };
            }
            AppEvent::ConnectionStatusChanged(status) => {
                let prev_status = self.sync_state.connection_status;
                self.sync_state.connection_status = status;

                // Add connection event
                if prev_status != status {
                    match status {
                        ConnectionStatus::Connected => {
                            self.add_live_event(LiveEvent {
                                timestamp: Utc::now(),
                                event_type: LiveEventType::Connected,
                                is_own: false,
                            });
                        }
                        ConnectionStatus::Disconnected => {
                            self.add_live_event(LiveEvent {
                                timestamp: Utc::now(),
                                event_type: LiveEventType::Disconnected,
                                is_own: false,
                            });
                        }
                        _ => {}
                    }
                }
            }
            AppEvent::DepositCompleted { tx_hash, amount } => {
                self.mode = AppMode::Dashboard;
                self.add_live_event(LiveEvent {
                    timestamp: Utc::now(),
                    event_type: LiveEventType::Deposit {
                        amount_mnt: wei_to_mnt(amount),
                        leaf_index: 0, // Will be updated on sync
                    },
                    is_own: true,
                });
                self.history.insert(
                    0,
                    TransactionRecord {
                        tx_type: TransactionType::Deposit,
                        amount_mnt: wei_to_mnt(amount),
                        status: TransactionStatus::Pending,
                        timestamp: Utc::now(),
                        tx_hash: Some(tx_hash),
                        recipient: None,
                    },
                );
            }
            AppEvent::WithdrawCompleted { tx_hash, amount } => {
                self.mode = AppMode::Dashboard;
                self.add_live_event(LiveEvent {
                    timestamp: Utc::now(),
                    event_type: LiveEventType::Withdrawal {
                        amount_mnt: wei_to_mnt(amount),
                        recipient: "self".to_string(),
                    },
                    is_own: true,
                });
                self.history.insert(
                    0,
                    TransactionRecord {
                        tx_type: TransactionType::Withdraw,
                        amount_mnt: wei_to_mnt(amount),
                        status: TransactionStatus::Pending,
                        timestamp: Utc::now(),
                        tx_hash: Some(tx_hash),
                        recipient: None,
                    },
                );
            }
            AppEvent::SyncCompleted { duration_ms } => {
                self.sync_state.is_syncing = false;
                self.sync_state.last_sync = Some(Utc::now());
                self.add_live_event(LiveEvent {
                    timestamp: Utc::now(),
                    event_type: LiveEventType::SyncCompleted { duration_ms },
                    is_own: false,
                });
            }
            AppEvent::OperationFailed { error } => {
                self.mode = AppMode::Error { message: error.clone() };
                self.add_live_event(LiveEvent {
                    timestamp: Utc::now(),
                    event_type: LiveEventType::Error { message: error },
                    is_own: false,
                });
            }
            AppEvent::WalletUnlocked => {
                self.wallet_state.is_unlocked = true;
            }
            AppEvent::WalletLocked => {
                self.wallet_state.is_unlocked = false;
                self.wallet_state.signer = None;
                self.wallet_state.secret = None;
            }
        }
    }

    /// Add a live event to the buffer
    fn add_live_event(&mut self, event: LiveEvent) {
        self.events.push_front(event);
        if self.events.len() > MAX_EVENTS {
            self.events.pop_back();
        }
    }

    /// Format balance for display
    pub fn balance_display(&self) -> String {
        match self.balance {
            Some(wei) => format!("{:.6} MNT", wei_to_mnt(wei)),
            None => "-- MNT".to_string(),
        }
    }

    /// Check if wallet is ready for operations
    pub fn is_wallet_ready(&self) -> bool {
        self.wallet_state.is_unlocked
            && self.wallet_state.signer.is_some()
            && self.wallet_state.secret.is_some()
    }

    /// Get sync progress string
    pub fn sync_status_display(&self) -> String {
        if self.sync_state.is_syncing {
            format!("Syncing: {:.0}%", self.sync_state.progress_percent())
        } else {
            "Synced".to_string()
        }
    }

    /// Get connection status string
    pub fn connection_display(&self) -> &'static str {
        match self.sync_state.connection_status {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Disconnected => "Offline",
            ConnectionStatus::Reconnecting => "Reconnecting...",
        }
    }

    /// Reset to dashboard mode
    pub fn go_to_dashboard(&mut self) {
        self.mode = AppMode::Dashboard;
        self.password_input = PasswordInput::default();
    }

    /// Start deposit flow
    pub fn start_deposit(&mut self) {
        if !self.wallet_state.is_unlocked {
            self.mode = AppMode::PasswordPrompt {
                purpose: PasswordPurpose::Deposit,
            };
        } else {
            self.deposit_form = DepositForm::default();
            self.mode = AppMode::DepositForm;
        }
    }

    /// Start withdraw flow
    pub fn start_withdraw(&mut self) {
        if !self.wallet_state.is_unlocked {
            self.mode = AppMode::PasswordPrompt {
                purpose: PasswordPurpose::Withdraw,
            };
        } else {
            self.withdraw_form = WithdrawForm::default();
            self.mode = AppMode::WithdrawForm;
        }
    }

    /// Toggle help modal
    pub fn toggle_help(&mut self) {
        if self.mode == AppMode::Help {
            self.mode = AppMode::Dashboard;
        } else {
            self.mode = AppMode::Help;
        }
    }

    /// Attempt to unlock wallet with password
    pub fn unlock_wallet(&mut self, password: &str) -> Result<(), String> {
        let wallet_manager = WalletManager::new(self.config.clone());

        // Load wallet
        let wallet = wallet_manager
            .load_wallet()
            .map_err(|e| format!("Failed to load wallet: {}", e))?;

        // Try to unlock (decrypt) the Ethereum signer
        let signer = wallet_manager
            .unlock(&wallet, password)
            .map_err(|e| format!("Wrong password or corrupted wallet: {}", e))?;

        // Get Veilocity secret
        let secret = wallet_manager
            .get_veilocity_secret(&wallet, password)
            .map_err(|e| format!("Failed to decrypt secret: {}", e))?;

        // Derive public key
        let mut hasher = PoseidonHasher::new();
        let pubkey_field = secret.derive_pubkey(&mut hasher);
        let pubkey_hex = field_to_hex(&pubkey_field);

        // Update wallet state
        self.wallet_state.is_unlocked = true;
        self.wallet_state.address = Some(wallet.address.clone());
        self.wallet_state.pubkey = Some(pubkey_hex);
        self.wallet_state.signer = Some(signer);
        self.wallet_state.secret = Some(secret);

        // Load balance from local state
        self.load_balance_from_state();

        Ok(())
    }

    /// Load balance from local state database
    pub fn load_balance_from_state(&mut self) {
        let db_path = self.config.db_path();

        // If no state DB exists, balance is 0
        if !db_path.exists() {
            self.balance = Some(0);
            self.account_index = None;
            self.account_nonce = None;
            return;
        }

        // Try to load state
        let state = match StateManager::new(&db_path) {
            Ok(s) => s,
            Err(_) => {
                self.balance = Some(0);
                return;
            }
        };

        // Get secret to derive pubkey
        let secret = match &self.wallet_state.secret {
            Some(s) => s,
            None => {
                self.balance = Some(0);
                return;
            }
        };

        // Derive pubkey and look up account
        let mut hasher = PoseidonHasher::new();
        let pubkey_field = secret.derive_pubkey(&mut hasher);
        let pubkey_bytes = field_to_bytes(&pubkey_field);

        match state.get_account(&pubkey_bytes) {
            Ok(Some(account)) => {
                self.balance = Some(account.balance);
                self.account_index = Some(account.index);
                self.account_nonce = Some(account.nonce);
            }
            Ok(None) => {
                // Account not found - no deposits yet
                self.balance = Some(0);
                self.account_index = None;
                self.account_nonce = None;
            }
            Err(_) => {
                self.balance = Some(0);
            }
        }
    }

    /// Lock the wallet (clear sensitive data)
    pub fn lock_wallet(&mut self) {
        self.wallet_state.is_unlocked = false;
        self.wallet_state.signer = None;
        self.wallet_state.secret = None;
        self.balance = None;
        self.account_index = None;
        self.account_nonce = None;
    }

    /// Prompt for password to unlock wallet
    pub fn prompt_unlock(&mut self) {
        self.password_input = PasswordInput::default();
        self.mode = AppMode::PasswordPrompt {
            purpose: PasswordPurpose::Unlock,
        };
    }

    /// Create a new wallet with the given password
    pub fn create_wallet(&mut self, password: &str) -> Result<(), String> {
        let wallet_manager = WalletManager::new(self.config.clone());

        // Generate new wallet
        let (wallet, signer, secret) = wallet_manager
            .generate(password)
            .map_err(|e| format!("Failed to generate wallet: {}", e))?;

        // Save wallet
        wallet_manager
            .save_wallet(&wallet)
            .map_err(|e| format!("Failed to save wallet: {}", e))?;

        // Derive public key
        let mut hasher = PoseidonHasher::new();
        let pubkey_field = secret.derive_pubkey(&mut hasher);
        let pubkey_hex = field_to_hex(&pubkey_field);

        // Update state
        self.wallet_exists = true;
        self.wallet_state.is_unlocked = true;
        self.wallet_state.address = Some(wallet.address.clone());
        self.wallet_state.pubkey = Some(pubkey_hex);
        self.wallet_state.signer = Some(signer);
        self.wallet_state.secret = Some(secret);
        self.balance = Some(0);

        Ok(())
    }

    /// Show wallet details
    pub fn show_wallet_details(&mut self) {
        self.mode = AppMode::WalletDetails;
    }

    /// Show delete wallet confirmation
    pub fn confirm_delete_wallet(&mut self) {
        self.mode = AppMode::DeleteWalletConfirm;
    }

    /// Delete the wallet file
    pub fn delete_wallet(&mut self) -> Result<(), String> {
        let wallet_path = self.config.wallet_path();

        // Lock wallet first
        self.lock_wallet();

        // Delete the wallet file
        if wallet_path.exists() {
            std::fs::remove_file(&wallet_path)
                .map_err(|e| format!("Failed to delete wallet: {}", e))?;
        }

        // Also delete state database if it exists
        let db_path = self.config.db_path();
        if db_path.exists() {
            let _ = std::fs::remove_file(&db_path);
        }

        // Reset state
        self.wallet_exists = false;
        self.wallet_state = WalletState::default();
        self.balance = None;
        self.account_index = None;
        self.account_nonce = None;
        self.history.clear();
        self.events.clear();

        Ok(())
    }

    /// Start creating a new wallet (replacing existing)
    pub fn start_create_wallet(&mut self) {
        self.create_wallet_form = CreateWalletForm::default();
        self.mode = AppMode::CreateWallet;
    }
}

/// Convert wei to MNT (18 decimals)
pub fn wei_to_mnt(wei: u128) -> f64 {
    wei as f64 / 1_000_000_000_000_000_000.0
}

/// Convert MNT to wei (18 decimals)
pub fn mnt_to_wei(mnt: f64) -> u128 {
    (mnt * 1_000_000_000_000_000_000.0) as u128
}
