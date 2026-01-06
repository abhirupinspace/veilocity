//! Veilocity TUI Theme
//!
//! Minimal, cohesive color palette focused on orange brand identity.

use ratatui::style::{Color, Modifier, Style};

// Primary brand colors - Orange
pub const ORANGE: Color = Color::Rgb(255, 140, 0);
pub const ORANGE_LIGHT: Color = Color::Rgb(255, 180, 60);
pub const ORANGE_DARK: Color = Color::Rgb(180, 100, 0);

// Neutral tones
pub const TEXT: Color = Color::Rgb(220, 220, 225);
pub const TEXT_DIM: Color = Color::Rgb(120, 120, 130);
pub const MUTED: Color = Color::Rgb(80, 80, 90);

// Essential semantic colors only
pub const SUCCESS: Color = Color::Rgb(80, 200, 120);
pub const ERROR: Color = Color::Rgb(220, 80, 80);
pub const WARNING: Color = Color::Rgb(220, 180, 60);

// Background
pub const BG_PANEL: Color = Color::Rgb(25, 25, 30);

// Styles

pub fn title() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn highlight() -> Style {
    Style::default().fg(ORANGE_LIGHT).add_modifier(Modifier::BOLD)
}

pub fn balance() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn success() -> Style {
    Style::default().fg(SUCCESS)
}

pub fn warning() -> Style {
    Style::default().fg(WARNING)
}

pub fn error() -> Style {
    Style::default().fg(ERROR)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

pub fn text() -> Style {
    Style::default().fg(TEXT)
}

pub fn dim() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn border() -> Style {
    Style::default().fg(MUTED)
}

pub fn border_focused() -> Style {
    Style::default().fg(ORANGE)
}

pub fn keybind() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn event_deposit() -> Style {
    Style::default().fg(SUCCESS)
}

pub fn event_withdraw() -> Style {
    Style::default().fg(ORANGE)
}

pub fn event_state() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn event_error() -> Style {
    Style::default().fg(ERROR)
}

pub fn status_confirmed() -> Style {
    Style::default().fg(SUCCESS)
}

pub fn status_pending() -> Style {
    Style::default().fg(WARNING)
}

pub fn status_failed() -> Style {
    Style::default().fg(ERROR)
}

pub fn input_active() -> Style {
    Style::default().fg(TEXT).bg(BG_PANEL)
}

pub fn input_inactive() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn label() -> Style {
    Style::default().fg(TEXT_DIM)
}

pub fn network_connected() -> Style {
    Style::default().fg(SUCCESS)
}

pub fn network_disconnected() -> Style {
    Style::default().fg(ERROR)
}

pub fn logo() -> Style {
    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
}

pub fn accent() -> Style {
    Style::default().fg(ORANGE_LIGHT)
}

pub fn glow() -> Style {
    Style::default().fg(ORANGE_LIGHT).add_modifier(Modifier::BOLD)
}

/// Logo gradient - orange tones only
pub fn logo_gradient() -> Vec<Color> {
    vec![
        Color::Rgb(255, 100, 0),
        Color::Rgb(255, 130, 0),
        Color::Rgb(255, 160, 30),
        Color::Rgb(255, 180, 60),
        Color::Rgb(255, 160, 30),
        Color::Rgb(255, 130, 0),
    ]
}
