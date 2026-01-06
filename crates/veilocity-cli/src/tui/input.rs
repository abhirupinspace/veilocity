//! TUI Input Handling
//!
//! Keyboard navigation and input processing.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use alloy::primitives::Address;

use super::actions::{execute_deposit, execute_withdraw, execute_sync};
use super::app::{App, AppEvent, AppMode, PasswordPurpose, PendingAction, ZkStage};
use super::forms::{validate_address, validate_amount};

/// Handle a key event based on current mode
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Result<()> {
    // Global quit shortcut (Ctrl+C or Ctrl+Q)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') | KeyCode::Char('q') => {
                app.should_quit = true;
                return Ok(());
            }
            _ => {}
        }
    }

    match &app.mode {
        AppMode::Dashboard => handle_dashboard_key(app, key),
        AppMode::DepositForm => handle_deposit_form_key(app, key),
        AppMode::WithdrawForm => handle_withdraw_form_key(app, key),
        AppMode::PasswordPrompt { purpose } => {
            let purpose = purpose.clone();
            handle_password_key(app, key, purpose)
        }
        AppMode::Confirmation { action } => {
            let action = action.clone();
            handle_confirmation_key(app, key, action)
        }
        AppMode::Error { .. } => handle_error_key(app, key),
        AppMode::Help => handle_help_key(app, key),
        AppMode::ZkProofProgress { .. } => {
            // No input during proof generation
            Ok(())
        }
        AppMode::CreateWallet => handle_create_wallet_key(app, key),
        AppMode::WalletDetails => handle_wallet_details_key(app, key),
        AppMode::DeleteWalletConfirm => handle_delete_wallet_confirm_key(app, key),
    }
}

/// Handle dashboard mode keys
fn handle_dashboard_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        // Quit
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }

        // Help
        KeyCode::Char('?') => {
            app.toggle_help();
        }

        // Actions
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.start_deposit();
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            app.start_withdraw();
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            // Trigger sync
            app.mode = AppMode::Confirmation {
                action: PendingAction::Sync,
            };
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            // Unlock wallet
            if !app.wallet_state.is_unlocked && app.wallet_exists {
                app.prompt_unlock();
            }
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            // Lock wallet
            if app.wallet_state.is_unlocked {
                app.lock_wallet();
            }
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            // Show wallet info/details (available even when locked for delete option)
            if app.wallet_exists {
                app.show_wallet_details();
            }
        }

        // Panel navigation
        KeyCode::Tab => {
            app.selected_panel = app.selected_panel.next();
            app.scroll_offset = 0;
        }
        KeyCode::BackTab => {
            app.selected_panel = app.selected_panel.prev();
            app.scroll_offset = 0;
        }

        // Scroll within panel
        KeyCode::Up | KeyCode::Char('k') => {
            if app.scroll_offset > 0 {
                app.scroll_offset -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_offset += 1;
        }

        // Page up/down
        KeyCode::PageUp => {
            app.scroll_offset = app.scroll_offset.saturating_sub(10);
        }
        KeyCode::PageDown => {
            app.scroll_offset += 10;
        }

        // Home/End
        KeyCode::Home => {
            app.scroll_offset = 0;
        }

        _ => {}
    }
    Ok(())
}

/// Handle deposit form keys
fn handle_deposit_form_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.go_to_dashboard();
        }
        KeyCode::Enter => {
            // Validate and confirm
            match validate_amount(&app.deposit_form.amount_input) {
                Ok(amount) => {
                    app.mode = AppMode::Confirmation {
                        action: PendingAction::Deposit { amount },
                    };
                }
                Err(e) => {
                    app.deposit_form.validation_error = Some(e.to_string());
                }
            }
        }
        KeyCode::Char(c) => {
            // Only allow numeric input and decimal point
            if c.is_ascii_digit() || c == '.' {
                app.deposit_form.amount_input.push(c);
                app.deposit_form.validation_error = None;
            }
        }
        KeyCode::Backspace => {
            app.deposit_form.amount_input.pop();
            app.deposit_form.validation_error = None;
        }
        _ => {}
    }
    Ok(())
}

/// Handle withdraw form keys
fn handle_withdraw_form_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.go_to_dashboard();
        }
        KeyCode::Tab => {
            // Switch between fields
            app.withdraw_form.toggle_field();
        }
        KeyCode::Enter => {
            // Validate and confirm
            let amount_result = validate_amount(&app.withdraw_form.amount_input);
            let addr_result = if app.withdraw_form.recipient_input.is_empty() {
                // Use own address
                Ok(())
            } else {
                validate_address(&app.withdraw_form.recipient_input).map(|_| ())
            };

            match (amount_result, addr_result) {
                (Ok(amount), Ok(())) => {
                    let recipient = if app.withdraw_form.recipient_input.is_empty() {
                        app.wallet_state
                            .address
                            .clone()
                            .unwrap_or_else(|| "self".to_string())
                    } else {
                        app.withdraw_form.recipient_input.clone()
                    };
                    app.mode = AppMode::Confirmation {
                        action: PendingAction::Withdraw { amount, recipient },
                    };
                }
                (Err(e), _) => {
                    app.withdraw_form.validation_error = Some(format!("Amount: {}", e));
                }
                (_, Err(e)) => {
                    app.withdraw_form.validation_error = Some(format!("Address: {}", e));
                }
            }
        }
        KeyCode::Char(c) => {
            app.withdraw_form.input_char(c);
            app.withdraw_form.validation_error = None;
        }
        KeyCode::Backspace => {
            app.withdraw_form.backspace();
            app.withdraw_form.validation_error = None;
        }
        _ => {}
    }
    Ok(())
}

/// Handle password prompt keys
fn handle_password_key(app: &mut App, key: KeyEvent, purpose: PasswordPurpose) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.go_to_dashboard();
        }
        KeyCode::Enter => {
            // Attempt to unlock wallet
            let password = app.password_input.buffer.clone();
            if password.is_empty() {
                app.password_input.error = Some("Password cannot be empty".to_string());
                return Ok(());
            }

            // Actually unlock the wallet
            match app.unlock_wallet(&password) {
                Ok(()) => {
                    // Clear password from memory
                    app.password_input.buffer.clear();
                    app.password_input.error = None;

                    // Proceed based on the purpose
                    match purpose {
                        PasswordPurpose::Deposit => {
                            app.deposit_form = super::forms::DepositForm::default();
                            app.mode = AppMode::DepositForm;
                        }
                        PasswordPurpose::Withdraw => {
                            app.withdraw_form = super::forms::WithdrawForm::default();
                            app.mode = AppMode::WithdrawForm;
                        }
                        PasswordPurpose::Unlock => {
                            app.go_to_dashboard();
                        }
                    }
                }
                Err(e) => {
                    app.password_input.error = Some(e);
                }
            }
        }
        KeyCode::Char(c) => {
            app.password_input.buffer.push(c);
            app.password_input.error = None;
        }
        KeyCode::Backspace => {
            app.password_input.buffer.pop();
            app.password_input.error = None;
        }
        _ => {}
    }
    Ok(())
}

/// Handle confirmation dialog keys
fn handle_confirmation_key(app: &mut App, key: KeyEvent, action: PendingAction) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.go_to_dashboard();
        }
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Execute the confirmed action
            match action {
                PendingAction::Deposit { amount } => {
                    // Start deposit execution
                    if let (Some(signer), Some(secret)) = (
                        app.wallet_state.signer.clone(),
                        app.wallet_state.secret.clone(),
                    ) {
                        let config = app.config.clone();
                        let event_tx = app.event_tx.clone();

                        // Set initial progress state
                        app.mode = AppMode::ZkProofProgress {
                            stage: ZkStage::CommitmentGeneration,
                            progress: 0.0,
                        };

                        // Spawn async deposit task
                        tokio::spawn(async move {
                            match execute_deposit(config, amount, signer, secret, event_tx.clone())
                                .await
                            {
                                Ok(tx_hash) => {
                                    let _ = event_tx
                                        .send(AppEvent::DepositCompleted {
                                            tx_hash,
                                            amount: (amount * 1e18) as u128,
                                        })
                                        .await;
                                }
                                Err(e) => {
                                    let _ = event_tx
                                        .send(AppEvent::OperationFailed { error: e })
                                        .await;
                                }
                            }
                        });
                    } else {
                        app.mode = AppMode::Error {
                            message: "Wallet not unlocked".to_string(),
                        };
                    }
                }
                PendingAction::Withdraw { amount, recipient } => {
                    // Start withdraw execution
                    if let (Some(signer), Some(secret)) = (
                        app.wallet_state.signer.clone(),
                        app.wallet_state.secret.clone(),
                    ) {
                        // Parse recipient address
                        let recipient_addr: Address = if recipient == "self" {
                            app.wallet_state
                                .address
                                .as_ref()
                                .and_then(|a| a.parse().ok())
                                .unwrap_or_default()
                        } else {
                            recipient.parse().unwrap_or_default()
                        };

                        let config = app.config.clone();
                        let event_tx = app.event_tx.clone();

                        // Set initial progress state
                        app.mode = AppMode::ZkProofProgress {
                            stage: ZkStage::WitnessGeneration,
                            progress: 0.0,
                        };

                        // Spawn async withdraw task
                        tokio::spawn(async move {
                            match execute_withdraw(
                                config,
                                amount,
                                recipient_addr,
                                signer,
                                secret,
                                event_tx.clone(),
                            )
                            .await
                            {
                                Ok(tx_hash) => {
                                    let _ = event_tx
                                        .send(AppEvent::WithdrawCompleted {
                                            tx_hash,
                                            amount: (amount * 1e18) as u128,
                                        })
                                        .await;
                                }
                                Err(e) => {
                                    let _ = event_tx
                                        .send(AppEvent::OperationFailed { error: e })
                                        .await;
                                }
                            }
                        });
                    } else {
                        app.mode = AppMode::Error {
                            message: "Wallet not unlocked".to_string(),
                        };
                    }
                }
                PendingAction::Sync => {
                    // Trigger sync
                    let config = app.config.clone();
                    let event_tx = app.event_tx.clone();
                    let secret = app.wallet_state.secret.clone();

                    app.sync_state.is_syncing = true;

                    tokio::spawn(async move {
                        match execute_sync(config, secret, event_tx.clone()).await {
                            Ok(duration_ms) => {
                                let _ = event_tx
                                    .send(AppEvent::SyncCompleted { duration_ms })
                                    .await;
                            }
                            Err(e) => {
                                let _ = event_tx
                                    .send(AppEvent::OperationFailed { error: e })
                                    .await;
                            }
                        }
                    });

                    app.go_to_dashboard();
                }
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handle error dialog keys
fn handle_error_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
            app.go_to_dashboard();
        }
        _ => {}
    }
    Ok(())
}

/// Handle help modal keys
fn handle_help_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter | KeyCode::Char(' ') => {
            app.go_to_dashboard();
        }
        _ => {}
    }
    Ok(())
}

/// Handle create wallet form keys
fn handle_create_wallet_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            // Can escape to quit if wallet was deleted, otherwise go to dashboard
            if !app.wallet_exists {
                // No wallet - user can quit entirely
                app.should_quit = true;
            }
        }
        KeyCode::Tab => {
            // Toggle between password fields
            app.create_wallet_form.active_field = 1 - app.create_wallet_form.active_field;
        }
        KeyCode::Enter => {
            // Validate and create wallet
            let password = app.create_wallet_form.password.clone();
            let confirm = app.create_wallet_form.confirm_password.clone();

            if password.is_empty() {
                app.create_wallet_form.error = Some("Password cannot be empty".to_string());
                return Ok(());
            }
            if password.len() < 8 {
                app.create_wallet_form.error = Some("Password must be at least 8 characters".to_string());
                return Ok(());
            }
            if password != confirm {
                app.create_wallet_form.error = Some("Passwords do not match".to_string());
                return Ok(());
            }

            // Create the wallet
            match app.create_wallet(&password) {
                Ok(()) => {
                    app.create_wallet_form = super::app::CreateWalletForm::default();
                    app.go_to_dashboard();
                }
                Err(e) => {
                    app.create_wallet_form.error = Some(e);
                }
            }
        }
        KeyCode::Char(c) => {
            app.create_wallet_form.error = None;
            if app.create_wallet_form.active_field == 0 {
                app.create_wallet_form.password.push(c);
            } else {
                app.create_wallet_form.confirm_password.push(c);
            }
        }
        KeyCode::Backspace => {
            app.create_wallet_form.error = None;
            if app.create_wallet_form.active_field == 0 {
                app.create_wallet_form.password.pop();
            } else {
                app.create_wallet_form.confirm_password.pop();
            }
        }
        _ => {}
    }
    Ok(())
}

/// Handle wallet details modal keys
fn handle_wallet_details_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Char('i') | KeyCode::Char('I') => {
            app.go_to_dashboard();
        }
        // Delete wallet
        KeyCode::Char('x') | KeyCode::Char('X') => {
            app.confirm_delete_wallet();
        }
        // Create new wallet (replace)
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.confirm_delete_wallet(); // Delete first, then create
        }
        _ => {}
    }
    Ok(())
}

/// Handle delete wallet confirmation keys
fn handle_delete_wallet_confirm_key(app: &mut App, key: KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.go_to_dashboard();
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Delete the wallet
            match app.delete_wallet() {
                Ok(()) => {
                    // Show create wallet screen
                    app.start_create_wallet();
                }
                Err(e) => {
                    app.mode = AppMode::Error { message: e };
                }
            }
        }
        _ => {}
    }
    Ok(())
}
