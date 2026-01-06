//! Events Widget
//!
//! Live event feed with clean design.

use ratatui::{
    layout::{Alignment, Constraint, Rect},
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::tui::app::{App, LiveEvent, LiveEventType, Panel};
use crate::tui::theme;

/// Render the events panel
pub fn render_events_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.selected_panel == Panel::Events;

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
        .title(" LIVE EVENTS ")
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.events.is_empty() {
        let tick = app.tick;
        let waiting_chars = ['·', '·', '·', ' '];
        let dots: String = (0..3).map(|i| waiting_chars[(tick / 2 + i) % 4]).collect();

        let empty = Paragraph::new(Line::from(vec![
            Span::styled("Listening", theme::muted()),
            Span::styled(dots, theme::muted()),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    let rows: Vec<Row> = app
        .events
        .iter()
        .skip(app.scroll_offset)
        .take(inner.height.saturating_sub(1) as usize)
        .map(|event| event_to_row(event))
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(9),  // Time
            Constraint::Length(3),  // Marker
            Constraint::Min(20),    // Description
        ],
    )
    .column_spacing(1);

    frame.render_widget(table, inner);
}

/// Convert a live event to a table row
fn event_to_row(event: &LiveEvent) -> Row<'static> {
    let time = event.timestamp.format("%H:%M:%S").to_string();

    let (marker, marker_style) = if event.is_own {
        (" * ".to_string(), Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD))
    } else {
        ("   ".to_string(), Style::default().fg(theme::MUTED))
    };

    let (icon, description, style) = match &event.event_type {
        LiveEventType::Deposit { amount_mnt, leaf_index } => {
            (
                "+".to_string(),
                format!("{:.4} MNT (#{}) ", amount_mnt, leaf_index),
                Style::default().fg(theme::SUCCESS),
            )
        }
        LiveEventType::Withdrawal { amount_mnt, recipient } => {
            let short_addr = if recipient.len() > 10 {
                format!("{}...{}", &recipient[..6], &recipient[recipient.len()-4..])
            } else {
                recipient.clone()
            };
            (
                "-".to_string(),
                format!("{:.4} MNT > {}", amount_mnt, short_addr),
                Style::default().fg(theme::ORANGE),
            )
        }
        LiveEventType::StateRootUpdate { batch_index } => {
            (
                "~".to_string(),
                format!("Root update (batch {})", batch_index),
                Style::default().fg(theme::TEXT_DIM),
            )
        }
        LiveEventType::SyncStarted => {
            (
                ">".to_string(),
                "Syncing...".to_string(),
                Style::default().fg(theme::ORANGE_LIGHT),
            )
        }
        LiveEventType::SyncCompleted { duration_ms } => {
            (
                "v".to_string(),
                format!("Synced in {}ms", duration_ms),
                Style::default().fg(theme::SUCCESS),
            )
        }
        LiveEventType::Error { message } => {
            let short_msg = if message.len() > 30 {
                format!("{}...", &message[..27])
            } else {
                message.clone()
            };
            (
                "!".to_string(),
                short_msg,
                Style::default().fg(theme::ERROR),
            )
        }
        LiveEventType::Connected => {
            (
                "*".to_string(),
                "Connected".to_string(),
                Style::default().fg(theme::SUCCESS),
            )
        }
        LiveEventType::Disconnected => {
            (
                "x".to_string(),
                "Disconnected".to_string(),
                Style::default().fg(theme::ERROR),
            )
        }
    };

    let time_cell = Line::from(vec![
        Span::styled(icon, style),
        Span::raw(" "),
        Span::styled(time, Style::default().fg(theme::TEXT_DIM)),
    ]);

    Row::new(vec![
        Cell::from(time_cell),
        Cell::from(marker).style(marker_style),
        Cell::from(description).style(style),
    ])
}
