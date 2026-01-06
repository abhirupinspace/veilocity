//! Balance Widget
//!
//! Clean, minimal private balance display.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{App, Panel};
use crate::tui::theme;

/// Render the balance panel with clean design
pub fn render_balance_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.selected_panel == Panel::Balance;

    let border_style = if is_focused {
        Style::default().fg(theme::ORANGE)
    } else {
        theme::border()
    };

    let title_style = if is_focused {
        Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)
    } else {
        theme::title()
    };

    let block = Block::default()
        .title(" BALANCE ")
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Label
            Constraint::Length(2), // Balance amount
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Account info
            Constraint::Min(0),    // Spacer
            Constraint::Length(1), // Status
        ])
        .split(inner);

    let tick = app.tick;

    // Privacy label
    let privacy_label = Paragraph::new(Line::from(vec![
        Span::styled("Private Balance", Style::default().fg(theme::TEXT_DIM)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(privacy_label, chunks[0]);

    // Balance amount with gradient
    let balance_text = if app.wallet_state.is_unlocked {
        app.balance_display()
    } else {
        "******* MNT".to_string()
    };

    let gradient_colors = theme::logo_gradient();
    let balance_spans: Vec<Span> = balance_text.chars().enumerate().map(|(i, ch)| {
        let color_idx = (i + tick / 3) % gradient_colors.len();
        Span::styled(
            ch.to_string(),
            Style::default()
                .fg(gradient_colors[color_idx])
                .add_modifier(Modifier::BOLD)
        )
    }).collect();

    let balance = Paragraph::new(Line::from(balance_spans))
        .alignment(Alignment::Center);
    frame.render_widget(balance, chunks[1]);

    // Account info
    if app.wallet_state.is_unlocked {
        let info_text = if let (Some(idx), Some(nonce)) = (app.account_index, app.account_nonce) {
            format!("Leaf #{} | Nonce {}", idx, nonce)
        } else {
            "No deposits yet".to_string()
        };

        let account_info = Paragraph::new(info_text)
            .style(theme::muted())
            .alignment(Alignment::Center);
        frame.render_widget(account_info, chunks[3]);
    }

    // Wallet status
    let status_text = if !app.wallet_exists {
        Line::from(vec![
            Span::styled("! ", theme::warning()),
            Span::styled("No wallet", theme::warning()),
        ])
    } else if app.wallet_state.is_unlocked {
        Line::from(vec![
            Span::styled("* ", theme::success()),
            Span::styled("Unlocked", theme::success()),
        ])
    } else {
        Line::from(vec![
            Span::styled("Press ", theme::muted()),
            Span::styled("[U]", Style::default().fg(theme::ORANGE)),
            Span::styled(" to unlock", theme::muted()),
        ])
    };

    let status = Paragraph::new(status_text).alignment(Alignment::Center);
    frame.render_widget(status, chunks[5]);
}
