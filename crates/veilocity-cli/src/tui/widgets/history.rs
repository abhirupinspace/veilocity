//! History Widget
//!
//! Transaction history panel.

use chrono::Utc;
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Row, Table},
    Frame,
};

use crate::tui::app::{App, Panel, TransactionRecord, TransactionStatus, TransactionType};
use crate::tui::theme;

/// Render the history panel
pub fn render_history_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.selected_panel == Panel::History;

    let border_style = if is_focused {
        theme::border_focused()
    } else {
        theme::border()
    };

    let block = Block::default()
        .title(" TRANSACTION HISTORY ")
        .title_style(theme::title())
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.history.is_empty() {
        let empty = ratatui::widgets::Paragraph::new(
            "No transactions yet. Deposit some MNT to get started!",
        )
        .style(theme::muted())
        .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(empty, inner);
        return;
    }

    // Header row
    let header = Row::new(vec![
        Cell::from("Type").style(theme::label()),
        Cell::from("Amount").style(theme::label()),
        Cell::from("Status").style(theme::label()),
        Cell::from("Time").style(theme::label()),
    ])
    .height(1)
    .bottom_margin(1);

    // Data rows
    let rows: Vec<Row> = app
        .history
        .iter()
        .skip(app.scroll_offset)
        .take(inner.height.saturating_sub(2) as usize)
        .map(|tx| transaction_to_row(tx))
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(10), // Type
            Constraint::Length(18), // Amount
            Constraint::Length(12), // Status
            Constraint::Min(10),    // Time
        ],
    )
    .header(header)
    .column_spacing(2);

    frame.render_widget(table, inner);
}

/// Convert a transaction record to a table row
fn transaction_to_row(tx: &TransactionRecord) -> Row<'static> {
    let type_style = match tx.tx_type {
        TransactionType::Deposit => theme::event_deposit(),
        TransactionType::Withdraw => theme::event_withdraw(),
        TransactionType::Transfer => theme::accent(),
    };

    let status_style = match tx.status {
        TransactionStatus::Pending => theme::status_pending(),
        TransactionStatus::Confirmed => theme::status_confirmed(),
        TransactionStatus::Failed => theme::status_failed(),
    };

    let time_ago = format_time_ago(tx.timestamp);

    Row::new(vec![
        Cell::from(tx.tx_type.as_str()).style(type_style),
        Cell::from(format!("{:.6} MNT", tx.amount_mnt)).style(theme::text()),
        Cell::from(tx.status.as_str()).style(status_style),
        Cell::from(time_ago).style(theme::dim()),
    ])
}

/// Format a timestamp as relative time
fn format_time_ago(timestamp: chrono::DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(timestamp);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        timestamp.format("%Y-%m-%d").to_string()
    }
}
