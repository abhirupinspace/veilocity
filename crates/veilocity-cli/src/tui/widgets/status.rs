//! Status Bar Widget
//!
//! Bottom status bar with keybindings and sync status.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{App, ConnectionStatus};
use crate::tui::theme;

/// Render the status bar
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme::ORANGE_DARK));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(50),    // Keybindings
            Constraint::Length(35), // Sync/Connection status
        ])
        .split(inner);

    render_keybindings(frame, app, chunks[0]);
    render_sync_status(frame, app, chunks[1]);
}

/// Render keybindings
fn render_keybindings(frame: &mut Frame, app: &App, area: Rect) {
    let keybindings = if app.wallet_state.is_unlocked {
        Line::from(vec![
            Span::raw(" "),
            Span::styled("D", Style::default().fg(theme::SUCCESS).add_modifier(Modifier::BOLD)),
            Span::styled("eposit  ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled("W", Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("ithdraw  ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled("S", Style::default().fg(theme::ORANGE_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled("ync  ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled("I", Style::default().fg(theme::ORANGE_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled("nfo  ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled("L", Style::default().fg(theme::WARNING).add_modifier(Modifier::BOLD)),
            Span::styled("ock  ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled("Q", Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD)),
            Span::styled("uit", Style::default().fg(theme::TEXT_DIM)),
        ])
    } else {
        Line::from(vec![
            Span::raw(" "),
            Span::styled("U", Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled("nlock  ", Style::default().fg(theme::ORANGE_LIGHT)),
            Span::styled("S", Style::default().fg(theme::ORANGE_LIGHT).add_modifier(Modifier::BOLD)),
            Span::styled("ync  ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled("?", Style::default().fg(theme::TEXT_DIM).add_modifier(Modifier::BOLD)),
            Span::styled("Help  ", Style::default().fg(theme::TEXT_DIM)),
            Span::styled("Q", Style::default().fg(theme::ERROR).add_modifier(Modifier::BOLD)),
            Span::styled("uit", Style::default().fg(theme::TEXT_DIM)),
        ])
    };

    let keys = Paragraph::new(keybindings).alignment(Alignment::Left);
    frame.render_widget(keys, area);
}

/// Render sync and connection status
fn render_sync_status(frame: &mut Frame, app: &App, area: Rect) {
    let tick = app.tick;

    // Connection indicator
    let (conn_icon, conn_style) = match app.sync_state.connection_status {
        ConnectionStatus::Connected => ("*", theme::success()),
        ConnectionStatus::Disconnected => ("!", theme::error()),
        ConnectionStatus::Reconnecting => ("~", theme::warning()),
    };

    let status_line = if app.sync_state.is_syncing {
        let dots = ".".repeat((tick / 3) % 4);
        // Show sync message if available, otherwise show progress
        let sync_text = if !app.sync_message.is_empty() {
            app.sync_message.clone()
        } else {
            format!("Syncing{} {:.0}%", dots, app.sync_state.progress_percent())
        };
        Line::from(vec![
            Span::styled(conn_icon, conn_style),
            Span::raw(" "),
            Span::styled(sync_text, Style::default().fg(theme::WARNING)),
        ])
    } else {
        match app.sync_state.connection_status {
            ConnectionStatus::Connected => {
                Line::from(vec![
                    Span::styled(conn_icon, conn_style),
                    Span::raw(" "),
                    Span::styled("Connected", theme::success()),
                    Span::raw("  "),
                    Span::styled(
                        format!("Block #{}", app.current_block),
                        Style::default().fg(theme::TEXT_DIM),
                    ),
                ])
            }
            ConnectionStatus::Disconnected => {
                Line::from(vec![
                    Span::styled(conn_icon, conn_style),
                    Span::raw(" "),
                    Span::styled("Offline - Check RPC", theme::error()),
                ])
            }
            ConnectionStatus::Reconnecting => {
                let dots = ".".repeat((tick / 3) % 4);
                Line::from(vec![
                    Span::styled(conn_icon, conn_style),
                    Span::raw(" "),
                    Span::styled(format!("Reconnecting{}", dots), theme::warning()),
                ])
            }
        }
    };

    let status_para = Paragraph::new(status_line).alignment(Alignment::Right);
    frame.render_widget(status_para, area);
}
