//! TUI Main Rendering
//!
//! Clean, minimal dashboard layout with real-time updates.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Clear, Gauge, Paragraph, Wrap},
    Frame,
};

use super::app::{App, AppMode, PendingAction, ZkStage};
use super::theme;
use super::widgets::{
    render_balance_panel, render_deposit_form, render_events_panel, render_header,
    render_history_panel, render_status_bar, render_withdraw_form,
};

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Determine header height based on terminal size
    let header_height = if area.height >= 30 { 10 } else { 3 };

    // Main layout: header, content, status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height), // Header
            Constraint::Min(15),               // Content
            Constraint::Length(2),             // Status bar
        ])
        .split(area);

    // Render header
    render_header(frame, app, main_chunks[0]);

    // Render main dashboard content
    render_dashboard(frame, app, main_chunks[1]);

    // Render status bar
    render_status_bar(frame, app, main_chunks[2]);

    // Render modal overlays based on mode
    render_modal_overlay(frame, app, area);
}

/// Render the main dashboard with clean layout
fn render_dashboard(frame: &mut Frame, app: &App, area: Rect) {
    // Three column layout: Balance | Events | History
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Balance & Stats
            Constraint::Percentage(35), // Events
            Constraint::Percentage(40), // History
        ])
        .margin(1)
        .split(area);

    // Left column: Balance on top, Stats below
    let left_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Balance
            Constraint::Percentage(40), // Network Stats
        ])
        .split(columns[0]);

    render_balance_panel(frame, app, left_column[0]);
    render_network_stats(frame, app, left_column[1]);

    // Middle column: Events
    render_events_panel(frame, app, columns[1]);

    // Right column: History
    render_history_panel(frame, app, columns[2]);
}

/// Render network statistics panel
fn render_network_stats(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" NETWORK ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let tick = app.tick;

    // Status with sync message
    let status_spans = if app.sync_state.is_syncing && !app.sync_message.is_empty() {
        // Truncate long messages
        let msg = if app.sync_message.len() > 20 {
            format!("{}...", &app.sync_message[..17])
        } else {
            app.sync_message.clone()
        };
        vec![Span::styled(msg, Style::default().fg(theme::WARNING))]
    } else if app.sync_state.is_syncing {
        let dots = ".".repeat((tick / 2) % 4);
        vec![Span::styled(format!("Syncing{}", dots), Style::default().fg(theme::WARNING))]
    } else {
        vec![Span::styled("Ready", theme::success())]
    };

    // Build stats content
    let stats_content = vec![
        Line::from(vec![
            Span::styled("Chain: ", theme::label()),
            Span::styled(&app.network_name, Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Block: ", theme::label()),
            Span::styled(format!("#{}", app.current_block), theme::glow()),
        ]),
        Line::from(""),
        Line::from(status_spans),
        Line::from(""),
        Line::from(vec![
            Span::styled("Events: ", theme::label()),
            Span::styled(format!("{}", app.events.len()), theme::text()),
        ]),
    ];

    let stats = Paragraph::new(stats_content)
        .alignment(Alignment::Left)
        .block(Block::default().padding(ratatui::widgets::Padding::horizontal(1)));
    frame.render_widget(stats, inner);
}

/// Render modal overlays
fn render_modal_overlay(frame: &mut Frame, app: &App, area: Rect) {
    match &app.mode {
        AppMode::DepositForm => {
            let modal_area = centered_rect(55, 35, area);
            frame.render_widget(Clear, modal_area);
            render_deposit_form(frame, app, modal_area);
        }
        AppMode::WithdrawForm => {
            let modal_area = centered_rect(55, 45, area);
            frame.render_widget(Clear, modal_area);
            render_withdraw_form(frame, app, modal_area);
        }
        AppMode::PasswordPrompt { purpose } => {
            let modal_area = centered_rect(50, 25, area);
            frame.render_widget(Clear, modal_area);
            render_password_modal(frame, app, modal_area, purpose);
        }
        AppMode::Confirmation { action } => {
            let modal_area = centered_rect(50, 25, area);
            frame.render_widget(Clear, modal_area);
            render_confirmation_modal(frame, app, modal_area, action);
        }
        AppMode::Error { message } => {
            let modal_area = centered_rect(55, 25, area);
            frame.render_widget(Clear, modal_area);
            render_error_modal(frame, message, modal_area);
        }
        AppMode::Help => {
            let modal_area = centered_rect(65, 65, area);
            frame.render_widget(Clear, modal_area);
            render_help_modal(frame, modal_area);
        }
        AppMode::ZkProofProgress { stage, progress } => {
            let modal_area = centered_rect(60, 35, area);
            frame.render_widget(Clear, modal_area);
            render_zk_progress_modal(frame, app, stage, *progress, modal_area);
        }
        AppMode::CreateWallet => {
            let modal_area = centered_rect(55, 45, area);
            frame.render_widget(Clear, modal_area);
            render_create_wallet_modal(frame, app, modal_area);
        }
        AppMode::WalletDetails => {
            let modal_area = centered_rect(60, 55, area);
            frame.render_widget(Clear, modal_area);
            render_wallet_details_modal(frame, app, modal_area);
        }
        AppMode::DeleteWalletConfirm => {
            let modal_area = centered_rect(50, 30, area);
            frame.render_widget(Clear, modal_area);
            render_delete_wallet_confirm_modal(frame, modal_area);
        }
        AppMode::Dashboard => {}
    }
}

/// Render password prompt modal
fn render_password_modal(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    purpose: &super::app::PasswordPurpose,
) {
    let title = match purpose {
        super::app::PasswordPurpose::Unlock => " Unlock Wallet ",
        super::app::PasswordPurpose::Deposit => " Password Required ",
        super::app::PasswordPurpose::Withdraw => " Password Required ",
    };

    let block = Block::default()
        .title(title)
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Label
            Constraint::Length(3), // Input
            Constraint::Length(2), // Error
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Hint
        ])
        .split(inner);

    let label = Paragraph::new("Password:").style(theme::label());
    frame.render_widget(label, chunks[0]);

    let masked: String = "*".repeat(app.password_input.buffer.len());
    let input = Paragraph::new(masked)
        .style(theme::input_active())
        .block(Block::default().borders(Borders::ALL).border_style(theme::border()));
    frame.render_widget(input, chunks[1]);

    if let Some(ref error) = app.password_input.error {
        let error_text = Paragraph::new(error.as_str()).style(theme::error());
        frame.render_widget(error_text, chunks[2]);
    }

    let hint = Paragraph::new("[Enter] Submit  [Esc] Cancel").style(theme::muted());
    frame.render_widget(hint, chunks[4]);
}

/// Render confirmation modal
fn render_confirmation_modal(
    frame: &mut Frame,
    _app: &App,
    area: Rect,
    action: &PendingAction,
) {
    let (title, message) = match action {
        PendingAction::Deposit { amount } => (
            " Confirm Deposit ",
            format!("Deposit {} MNT to privacy pool?", amount),
        ),
        PendingAction::Withdraw { amount, recipient } => (
            " Confirm Withdrawal ",
            format!("Withdraw {} MNT to {}?", amount, truncate_address(recipient)),
        ),
        PendingAction::Sync => (" Confirm Sync ", "Sync state with network?".to_string()),
    };

    let block = Block::default()
        .title(title)
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(3),    // Message
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons hint
        ])
        .split(inner);

    let message_widget = Paragraph::new(message)
        .style(theme::text())
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);
    frame.render_widget(message_widget, chunks[0]);

    let hint = Paragraph::new("[Y] Yes  [N] No  [Esc] Cancel")
        .style(theme::muted())
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[2]);
}

/// Render error modal
fn render_error_modal(frame: &mut Frame, message: &str, area: Rect) {
    let block = Block::default()
        .title(" Error ")
        .title_style(theme::error())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ERROR));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Min(3),    // Message
            Constraint::Length(1), // Hint
        ])
        .split(inner);

    let message_widget = Paragraph::new(message)
        .style(theme::text())
        .wrap(Wrap { trim: true });
    frame.render_widget(message_widget, chunks[0]);

    let hint = Paragraph::new("[Enter] or [Esc] to dismiss")
        .style(theme::muted())
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[1]);
}

/// Render help modal
fn render_help_modal(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Help ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        Line::from(vec![
            Span::styled("VEILOCITY", theme::title()),
            Span::raw(" - Private Execution Layer"),
        ]),
        Line::from(""),
        Line::styled("--- Navigation ---", theme::dim()),
        Line::from(vec![
            Span::styled("  Tab      ", theme::keybind()),
            Span::raw("  Next panel"),
        ]),
        Line::from(vec![
            Span::styled("  j/k      ", theme::keybind()),
            Span::raw("  Scroll"),
        ]),
        Line::from(""),
        Line::styled("--- Wallet ---", theme::dim()),
        Line::from(vec![
            Span::styled("  U        ", theme::keybind()),
            Span::raw("  Unlock wallet"),
        ]),
        Line::from(vec![
            Span::styled("  L        ", theme::keybind()),
            Span::raw("  Lock wallet"),
        ]),
        Line::from(vec![
            Span::styled("  I        ", theme::keybind()),
            Span::raw("  Wallet info (delete/new)"),
        ]),
        Line::from(""),
        Line::styled("--- Actions ---", theme::dim()),
        Line::from(vec![
            Span::styled("  D        ", theme::keybind()),
            Span::raw("  Deposit MNT"),
        ]),
        Line::from(vec![
            Span::styled("  W        ", theme::keybind()),
            Span::raw("  Withdraw MNT"),
        ]),
        Line::from(vec![
            Span::styled("  S        ", theme::keybind()),
            Span::raw("  Sync with network"),
        ]),
        Line::from(""),
        Line::styled("--- General ---", theme::dim()),
        Line::from(vec![
            Span::styled("  ?        ", theme::keybind()),
            Span::raw("  Toggle help"),
        ]),
        Line::from(vec![
            Span::styled("  Q        ", theme::keybind()),
            Span::raw("  Quit"),
        ]),
        Line::from(vec![
            Span::styled("  Esc      ", theme::keybind()),
            Span::raw("  Cancel/Back"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default().padding(ratatui::widgets::Padding::horizontal(2)));
    frame.render_widget(help, inner);
}

/// Render ZK proof progress modal
fn render_zk_progress_modal(
    frame: &mut Frame,
    app: &App,
    stage: &ZkStage,
    progress: f32,
    area: Rect,
) {
    let block = Block::default()
        .title(" GENERATING ZK PROOF ")
        .title_style(Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ORANGE_DARK));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Stage pipeline
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Current operation
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Progress bar
            Constraint::Length(1), // Spacer
            Constraint::Min(1),    // Status message
        ])
        .split(inner);

    let tick = app.tick;

    // Stage pipeline visualization
    render_stage_pipeline(frame, stage, tick, chunks[0]);

    // Current operation with spinner
    let spinner_chars = ['-', '\\', '|', '/'];
    let spinner = spinner_chars[tick % spinner_chars.len()];

    let operation_text = if !app.zk_message.is_empty() {
        &app.zk_message
    } else {
        stage.as_str()
    };

    let operation = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} ", spinner),
            Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)
        ),
        Span::styled(
            operation_text,
            Style::default().fg(theme::TEXT).add_modifier(Modifier::BOLD)
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(operation, chunks[2]);

    // Progress bar
    let progress_pct = (progress * 100.0).min(100.0) as u16;
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(theme::ORANGE).bg(theme::BG_PANEL))
        .percent(progress_pct)
        .label(format!("{}%", progress_pct));
    frame.render_widget(gauge, chunks[4]);

    // Status message
    let status_msg = Line::from(vec![
        Span::styled(
            "Your transaction details are cryptographically hidden",
            Style::default().fg(theme::TEXT_DIM).add_modifier(Modifier::ITALIC)
        ),
    ]);
    let status = Paragraph::new(status_msg).alignment(Alignment::Center);
    frame.render_widget(status, chunks[6]);
}

/// Render stage pipeline (simplified: 4 stages)
fn render_stage_pipeline(frame: &mut Frame, current_stage: &ZkStage, _tick: usize, area: Rect) {
    // Simplified stages for display
    let stages = [
        ("Witness", 0, 2),    // Covers CommitmentGeneration, WitnessGeneration, ConstraintSatisfaction
        ("Prove", 3, 4),      // Covers PolynomialCommitment, ProofComputation
        ("Verify", 5, 5),     // Verification
        ("Submit", 6, 6),     // Submitting
    ];

    let current_idx = current_stage.index();

    let mut spans = vec![Span::raw("    ")];

    for (i, (name, start_idx, end_idx)) in stages.iter().enumerate() {
        let style = if current_idx > *end_idx {
            // Completed
            Style::default().fg(theme::SUCCESS)
        } else if current_idx >= *start_idx && current_idx <= *end_idx {
            // Current
            Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)
        } else {
            // Pending
            Style::default().fg(theme::MUTED)
        };

        let indicator = if current_idx > *end_idx {
            "[*]"
        } else if current_idx >= *start_idx && current_idx <= *end_idx {
            "[>]"
        } else {
            "[ ]"
        };

        spans.push(Span::styled(format!("{} {}", indicator, name), style));

        if i < stages.len() - 1 {
            spans.push(Span::styled(" -> ", Style::default().fg(theme::MUTED)));
        }
    }

    let pipeline = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(pipeline, area);
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Truncate an address for display
fn truncate_address(addr: &str) -> String {
    if addr.len() <= 13 {
        addr.to_string()
    } else {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    }
}

/// Render create wallet modal
fn render_create_wallet_modal(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.wallet_exists {
        " Create New Wallet "
    } else {
        " Welcome to Veilocity "
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Welcome text
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Password label
            Constraint::Length(3), // Password input
            Constraint::Length(1), // Confirm label
            Constraint::Length(3), // Confirm input
            Constraint::Length(2), // Error
            Constraint::Min(1),    // Spacer
            Constraint::Length(1), // Hint
        ])
        .split(inner);

    // Welcome text
    let welcome_line = if app.wallet_exists {
        "Create a new wallet (will replace existing)"
    } else {
        "Create a secure password for your wallet"
    };

    let welcome = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Welcome to ", theme::text()),
            Span::styled("Veilocity", Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled(welcome_line, theme::dim())),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(welcome, chunks[0]);

    // Password label
    let pwd_style = if app.create_wallet_form.active_field == 0 {
        Style::default().fg(theme::ORANGE)
    } else {
        theme::label()
    };
    let pwd_label = Paragraph::new("Password:").style(pwd_style);
    frame.render_widget(pwd_label, chunks[2]);

    // Password input
    let pwd_border = if app.create_wallet_form.active_field == 0 {
        theme::border_focused()
    } else {
        theme::border()
    };
    let pwd_masked: String = "*".repeat(app.create_wallet_form.password.len());
    let pwd_input = Paragraph::new(pwd_masked)
        .style(theme::input_active())
        .block(Block::default().borders(Borders::ALL).border_style(pwd_border));
    frame.render_widget(pwd_input, chunks[3]);

    // Confirm label
    let confirm_style = if app.create_wallet_form.active_field == 1 {
        Style::default().fg(theme::ORANGE)
    } else {
        theme::label()
    };
    let confirm_label = Paragraph::new("Confirm Password:").style(confirm_style);
    frame.render_widget(confirm_label, chunks[4]);

    // Confirm input
    let confirm_border = if app.create_wallet_form.active_field == 1 {
        theme::border_focused()
    } else {
        theme::border()
    };
    let confirm_masked: String = "*".repeat(app.create_wallet_form.confirm_password.len());
    let confirm_input = Paragraph::new(confirm_masked)
        .style(theme::input_active())
        .block(Block::default().borders(Borders::ALL).border_style(confirm_border));
    frame.render_widget(confirm_input, chunks[5]);

    // Error message
    if let Some(ref error) = app.create_wallet_form.error {
        let error_text = Paragraph::new(error.as_str())
            .style(theme::error())
            .alignment(Alignment::Center);
        frame.render_widget(error_text, chunks[6]);
    }

    // Hint (shows Quit if no wallet exists, otherwise shows nothing for Esc)
    let hint_text = if app.wallet_exists {
        "[Tab] Switch field  [Enter] Create"
    } else {
        "[Tab] Switch field  [Enter] Create  [Esc] Quit"
    };
    let hint = Paragraph::new(hint_text)
        .style(theme::muted())
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[8]);
}

/// Render wallet details modal
fn render_wallet_details_modal(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Wallet Details ")
        .title_style(Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Address section
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Pubkey section
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Balance section
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Account info
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Wallet actions
            Constraint::Min(1),    // Spacer
            Constraint::Length(1), // Hint
        ])
        .split(inner);

    // Ethereum Address
    let address = app.wallet_state.address.as_deref().unwrap_or("Not available");
    let address_section = Paragraph::new(vec![
        Line::from(Span::styled("Ethereum Address", theme::label())),
        Line::from(Span::styled(address, Style::default().fg(theme::TEXT))),
    ]);
    frame.render_widget(address_section, chunks[0]);

    // Veilocity Public Key
    let pubkey = app.wallet_state.pubkey.as_deref().unwrap_or("Not available");
    let pubkey_display = if pubkey.len() > 40 {
        format!("{}...{}", &pubkey[..20], &pubkey[pubkey.len()-16..])
    } else {
        pubkey.to_string()
    };
    let pubkey_section = Paragraph::new(vec![
        Line::from(Span::styled("Veilocity Public Key", theme::label())),
        Line::from(Span::styled(pubkey_display, Style::default().fg(theme::TEXT))),
    ]);
    frame.render_widget(pubkey_section, chunks[2]);

    // Balance
    let balance_display = app.balance_display();
    let balance_section = Paragraph::new(vec![
        Line::from(Span::styled("Private Balance", theme::label())),
        Line::from(Span::styled(balance_display, Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD))),
    ]);
    frame.render_widget(balance_section, chunks[4]);

    // Account Info
    let account_info = if let (Some(idx), Some(nonce)) = (app.account_index, app.account_nonce) {
        format!("Leaf Index: {}  |  Nonce: {}", idx, nonce)
    } else {
        "No deposits yet".to_string()
    };
    let account_section = Paragraph::new(vec![
        Line::from(Span::styled("Account Info", theme::label())),
        Line::from(Span::styled(account_info, theme::dim())),
    ]);
    frame.render_widget(account_section, chunks[6]);

    // Wallet Actions
    let actions_section = Paragraph::new(vec![
        Line::from(Span::styled("Wallet Actions", theme::label())),
        Line::from(vec![
            Span::styled("[X] ", Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD)),
            Span::styled("Delete Wallet  ", theme::dim()),
            Span::styled("[N] ", Style::default().fg(theme::WARNING).add_modifier(Modifier::BOLD)),
            Span::styled("New Wallet", theme::dim()),
        ]),
    ]);
    frame.render_widget(actions_section, chunks[8]);

    // Hint
    let hint = Paragraph::new("[Esc] Close")
        .style(theme::muted())
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[10]);
}

/// Render delete wallet confirmation modal
fn render_delete_wallet_confirm_modal(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Delete Wallet ")
        .title_style(Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::ERROR));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(2), // Warning icon
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Message
            Constraint::Min(1),    // Spacer
            Constraint::Length(1), // Buttons
        ])
        .split(inner);

    // Warning
    let warning = Paragraph::new(Line::from(vec![
        Span::styled("! ", Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD)),
        Span::styled("WARNING", Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(warning, chunks[0]);

    // Message
    let message = Paragraph::new(vec![
        Line::from("This will permanently delete your wallet"),
        Line::from("and all associated data."),
        Line::from(Span::styled("This action cannot be undone!", Style::default().fg(theme::ERROR))),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true });
    frame.render_widget(message, chunks[2]);

    // Buttons hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("[Y] ", Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD)),
        Span::styled("Delete  ", theme::dim()),
        Span::styled("[N] ", Style::default().fg(theme::SUCCESS).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme::dim()),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[4]);
}
