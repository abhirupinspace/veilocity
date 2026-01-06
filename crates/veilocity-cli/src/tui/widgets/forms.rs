//! Form Widgets
//!
//! Deposit and withdraw form rendering.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::App;
use crate::tui::forms::WithdrawField;
use crate::tui::theme;

/// Render the deposit form modal
pub fn render_deposit_form(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Deposit to Privacy Pool ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1), // Description
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Label
            Constraint::Length(3), // Input
            Constraint::Length(2), // Error
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Balance hint
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Buttons
        ])
        .split(inner);

    // Description
    let desc = Paragraph::new("Enter the amount of MNT to deposit into the privacy pool.")
        .style(theme::text())
        .alignment(Alignment::Center);
    frame.render_widget(desc, chunks[0]);

    // Label
    let label = Paragraph::new("Amount (MNT):").style(theme::label());
    frame.render_widget(label, chunks[2]);

    // Input field
    let input_text = format!("{}_", app.deposit_form.amount_input);
    let input = Paragraph::new(input_text)
        .style(theme::input_active())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border_focused()),
        );
    frame.render_widget(input, chunks[3]);

    // Validation error
    if let Some(ref error) = app.deposit_form.validation_error {
        let error_text = Paragraph::new(error.as_str()).style(theme::error());
        frame.render_widget(error_text, chunks[4]);
    }

    // Balance hint
    if let Some(balance) = app.balance {
        let balance_mnt = balance as f64 / 1_000_000_000_000_000_000.0;
        let hint = Paragraph::new(format!("Available balance: {:.6} MNT", balance_mnt))
            .style(theme::muted())
            .alignment(Alignment::Center);
        frame.render_widget(hint, chunks[6]);
    }

    // Buttons hint
    let buttons = Paragraph::new("[Enter] Confirm  [Esc] Cancel")
        .style(theme::muted())
        .alignment(Alignment::Center);
    frame.render_widget(buttons, chunks[8]);
}

/// Render the withdraw form modal
pub fn render_withdraw_form(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Withdraw from Privacy Pool ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(1), // Description
            Constraint::Length(2), // Spacer
            Constraint::Length(1), // Amount label
            Constraint::Length(3), // Amount input
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Recipient label
            Constraint::Length(3), // Recipient input
            Constraint::Length(2), // Error
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Hint
            Constraint::Length(1), // Buttons
        ])
        .split(inner);

    // Description
    let desc = Paragraph::new("Withdraw MNT with zero-knowledge proof verification.")
        .style(theme::text())
        .alignment(Alignment::Center);
    frame.render_widget(desc, chunks[0]);

    // Amount label
    let amount_label = Paragraph::new("Amount (MNT):").style(theme::label());
    frame.render_widget(amount_label, chunks[2]);

    // Amount input
    let amount_focused = app.withdraw_form.active_field == WithdrawField::Amount;
    let amount_border_style = if amount_focused {
        theme::border_focused()
    } else {
        theme::border()
    };
    let amount_input_style = if amount_focused {
        theme::input_active()
    } else {
        theme::input_inactive()
    };

    let amount_text = if amount_focused {
        format!("{}_", app.withdraw_form.amount_input)
    } else {
        app.withdraw_form.amount_input.clone()
    };

    let amount_input = Paragraph::new(amount_text)
        .style(amount_input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(amount_border_style),
        );
    frame.render_widget(amount_input, chunks[3]);

    // Recipient label
    let recipient_label = Paragraph::new("Recipient address (leave empty for self):")
        .style(theme::label());
    frame.render_widget(recipient_label, chunks[5]);

    // Recipient input
    let recipient_focused = app.withdraw_form.active_field == WithdrawField::Recipient;
    let recipient_border_style = if recipient_focused {
        theme::border_focused()
    } else {
        theme::border()
    };
    let recipient_input_style = if recipient_focused {
        theme::input_active()
    } else {
        theme::input_inactive()
    };

    let recipient_text = if recipient_focused {
        format!("{}_", app.withdraw_form.recipient_input)
    } else if app.withdraw_form.recipient_input.is_empty() {
        app.wallet_state
            .address
            .as_ref()
            .map(|a| format!("{} (self)", truncate_address(a)))
            .unwrap_or_else(|| "self".to_string())
    } else {
        app.withdraw_form.recipient_input.clone()
    };

    let recipient_input = Paragraph::new(recipient_text)
        .style(recipient_input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(recipient_border_style),
        );
    frame.render_widget(recipient_input, chunks[6]);

    // Validation error
    if let Some(ref error) = app.withdraw_form.validation_error {
        let error_text = Paragraph::new(error.as_str()).style(theme::error());
        frame.render_widget(error_text, chunks[7]);
    }

    // Hint
    let hint = Paragraph::new("[Tab] Switch field  [Enter] Confirm  [Esc] Cancel")
        .style(theme::muted())
        .alignment(Alignment::Center);
    frame.render_widget(hint, chunks[10]);
}

/// Truncate an address for display
fn truncate_address(addr: &str) -> String {
    if addr.len() <= 13 {
        addr.to_string()
    } else {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    }
}
