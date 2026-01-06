//! Header Widget
//!
//! Animated header with large gradient logo.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::*,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{App, ConnectionStatus};
use crate::tui::theme;

/// Large ASCII art logo (block style)
const LOGO_ART: &[&str] = &[
    "██╗   ██╗███████╗██╗██╗      ██████╗  ██████╗██╗████████╗██╗   ██╗",
    "██║   ██║██╔════╝██║██║     ██╔═══██╗██╔════╝██║╚══██╔══╝╚██╗ ██╔╝",
    "██║   ██║█████╗  ██║██║     ██║   ██║██║     ██║   ██║    ╚████╔╝ ",
    "╚██╗ ██╔╝██╔══╝  ██║██║     ██║   ██║██║     ██║   ██║     ╚██╔╝  ",
    " ╚████╔╝ ███████╗██║███████╗╚██████╔╝╚██████╗██║   ██║      ██║   ",
    "  ╚═══╝  ╚══════╝╚═╝╚══════╝ ╚═════╝  ╚═════╝╚═╝   ╚═╝      ╚═╝   ",
];

/// Render the header bar with large animated gradient logo
pub fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme::border());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // If we have enough height for the large logo (9+ rows), show it
    if area.height >= 9 {
        render_large_header(frame, app, inner);
    } else {
        render_compact_header(frame, app, inner);
    }
}

/// Render large header with ASCII art logo
fn render_large_header(frame: &mut Frame, app: &App, area: Rect) {
    let tick = app.tick;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Logo
            Constraint::Length(1), // Info bar
        ])
        .split(area);

    // Render gradient logo
    let gradient_colors = theme::logo_gradient();
    let logo_lines: Vec<Line> = LOGO_ART.iter().enumerate().map(|(row_idx, line)| {
        let spans: Vec<Span> = line.chars().enumerate().map(|(col_idx, ch)| {
            let color_idx = (col_idx / 4 + row_idx + tick / 2) % gradient_colors.len();
            Span::styled(
                ch.to_string(),
                Style::default()
                    .fg(gradient_colors[color_idx])
                    .add_modifier(Modifier::BOLD)
            )
        }).collect();
        Line::from(spans)
    }).collect();

    let logo = Paragraph::new(logo_lines)
        .alignment(Alignment::Center);
    frame.render_widget(logo, chunks[0]);

    // Info bar
    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    // Tagline
    let tagline = Line::from(vec![
        Span::styled("Private Execution Layer for ", Style::default().fg(theme::TEXT_DIM)),
        Span::styled("Mantle", Style::default().fg(theme::ORANGE_LIGHT).add_modifier(Modifier::BOLD)),
    ]);

    let tagline_para = Paragraph::new(tagline).alignment(Alignment::Center);
    frame.render_widget(tagline_para, info_chunks[0]);

    // Network info
    render_network_info(frame, app, info_chunks[1]);
}

/// Render compact header for smaller terminals
fn render_compact_header(frame: &mut Frame, app: &App, area: Rect) {
    let tick = app.tick;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Min(20),
            Constraint::Length(40),
        ])
        .split(area);

    // Animated gradient logo (compact)
    let logo_text = "VEILOCITY";
    let gradient_colors = theme::logo_gradient();
    let spans: Vec<Span> = logo_text.chars().enumerate().map(|(i, ch)| {
        let color_idx = (i + tick / 2) % gradient_colors.len();
        Span::styled(
            ch.to_string(),
            Style::default()
                .fg(gradient_colors[color_idx])
                .add_modifier(Modifier::BOLD)
        )
    }).collect();

    let logo = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    frame.render_widget(logo, chunks[0]);

    // Tagline
    let tagline = Line::from(vec![
        Span::styled("Private Execution Layer for ", Style::default().fg(theme::TEXT_DIM)),
        Span::styled("Mantle", Style::default().fg(theme::ORANGE_LIGHT).add_modifier(Modifier::BOLD)),
    ]);

    let paragraph = Paragraph::new(tagline).alignment(Alignment::Center);
    frame.render_widget(paragraph, chunks[1]);

    // Network info
    render_network_info(frame, app, chunks[2]);
}

/// Render network info with visual indicators
fn render_network_info(frame: &mut Frame, app: &App, area: Rect) {
    let (connection_icon, connection_style) = match app.sync_state.connection_status {
        ConnectionStatus::Connected => ("*", theme::success()),
        ConnectionStatus::Disconnected => ("x", theme::error()),
        ConnectionStatus::Reconnecting => ("~", theme::warning()),
    };

    let network_info = Line::from(vec![
        Span::raw("["),
        Span::styled(&app.network_name, Style::default().fg(theme::ORANGE).add_modifier(Modifier::BOLD)),
        Span::raw("] "),
        Span::styled(connection_icon, connection_style),
        Span::raw(" "),
        Span::styled(app.connection_display(), connection_style),
        Span::raw("  Block: "),
        Span::styled(format!("{}", app.current_block), theme::glow()),
    ]);

    let network = Paragraph::new(network_info).alignment(Alignment::Right);
    frame.render_widget(network, area);
}
