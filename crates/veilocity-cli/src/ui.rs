//! UI module for Veilocity CLI
//!
//! Provides consistent styling, ASCII art logo, and color helpers.

use colored::{ColoredString, Colorize};

/// Veilocity brand orange color (RGB: 255, 140, 0)
pub const ORANGE: (u8, u8, u8) = (255, 140, 0);

/// Lighter orange for accents
pub const ORANGE_LIGHT: (u8, u8, u8) = (255, 180, 60);

/// Dark orange for emphasis
pub const ORANGE_DARK: (u8, u8, u8) = (230, 115, 0);

/// Mantle purple accent
pub const PURPLE: (u8, u8, u8) = (138, 99, 210);

/// ASCII art logo for Veilocity
pub const LOGO: &str = r#"
██╗   ██╗███████╗██╗██╗      ██████╗  ██████╗██╗████████╗██╗   ██╗
██║   ██║██╔════╝██║██║     ██╔═══██╗██╔════╝██║╚══██╔══╝╚██╗ ██╔╝
██║   ██║█████╗  ██║██║     ██║   ██║██║     ██║   ██║    ╚████╔╝
╚██╗ ██╔╝██╔══╝  ██║██║     ██║   ██║██║     ██║   ██║     ╚██╔╝
 ╚████╔╝ ███████╗██║███████╗╚██████╔╝╚██████╗██║   ██║      ██║
  ╚═══╝  ╚══════╝╚═╝╚══════╝ ╚═════╝  ╚═════╝╚═╝   ╚═╝      ╚═╝
"#;

/// Compact logo for smaller outputs
pub const LOGO_COMPACT: &str = r#"
 ╦  ╦┌─┐┬┬  ┌─┐┌─┐┬┌┬┐┬ ┬
 ╚╗╔╝├┤ ││  │ ││  │ │ └┬┘
  ╚╝ └─┘┴┴─┘└─┘└─┘┴ ┴  ┴
"#;

/// Mini logo for inline use
pub const LOGO_MINI: &str = "◈ Veilocity";

/// Print the main logo with gradient coloring
pub fn print_logo() {
    let lines: Vec<&str> = LOGO.trim().lines().collect();

    for (i, line) in lines.iter().enumerate() {
        // Gradient from orange to darker orange
        let ratio = i as f32 / (lines.len() - 1) as f32;
        let r = (ORANGE.0 as f32 * (1.0 - ratio * 0.3)) as u8;
        let g = (ORANGE.1 as f32 * (1.0 - ratio * 0.2)) as u8;
        let b = (ORANGE.2 as f32) as u8;

        println!("{}", line.truecolor(r, g, b).bold());
    }
}

/// Print the compact logo
pub fn print_logo_compact() {
    for line in LOGO_COMPACT.trim().lines() {
        println!("{}", line.truecolor(ORANGE.0, ORANGE.1, ORANGE.2).bold());
    }
}

/// Print a section header with orange styling
pub fn header(text: &str) -> ColoredString {
    format!("═══ {} ═══", text)
        .truecolor(ORANGE.0, ORANGE.1, ORANGE.2)
        .bold()
}

/// Print a subheader
pub fn subheader(text: &str) -> ColoredString {
    format!("─── {} ───", text)
        .truecolor(ORANGE_LIGHT.0, ORANGE_LIGHT.1, ORANGE_LIGHT.2)
}

/// Style text with brand orange
pub fn orange(text: &str) -> ColoredString {
    text.truecolor(ORANGE.0, ORANGE.1, ORANGE.2)
}

/// Style text with brand orange (bold)
pub fn orange_bold(text: &str) -> ColoredString {
    text.truecolor(ORANGE.0, ORANGE.1, ORANGE.2).bold()
}

/// Style text with light orange (for accents)
pub fn accent(text: &str) -> ColoredString {
    text.truecolor(ORANGE_LIGHT.0, ORANGE_LIGHT.1, ORANGE_LIGHT.2)
}

/// Style text with purple (Mantle color)
pub fn purple(text: &str) -> ColoredString {
    text.truecolor(PURPLE.0, PURPLE.1, PURPLE.2)
}

/// Style text as a success message
pub fn success(text: &str) -> ColoredString {
    text.green().bold()
}

/// Style text as a warning
pub fn warning(text: &str) -> ColoredString {
    text.yellow()
}

/// Style text as an error
pub fn error(text: &str) -> ColoredString {
    text.red().bold()
}

/// Style a value for display
pub fn value(text: &str) -> ColoredString {
    text.bright_white().bold()
}

/// Style a label
pub fn label(text: &str) -> ColoredString {
    text.truecolor(180, 180, 180)
}

/// Style a hint/note
pub fn hint(text: &str) -> ColoredString {
    text.dimmed().italic()
}

/// Style a command example
pub fn command(text: &str) -> ColoredString {
    text.truecolor(ORANGE.0, ORANGE.1, ORANGE.2)
}

/// Print a horizontal divider
pub fn divider(width: usize) {
    println!("{}", "─".repeat(width).truecolor(80, 80, 80));
}

/// Print a double-line divider
pub fn divider_double(width: usize) {
    println!("{}", "═".repeat(width).truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2));
}

/// Print a box around text
pub fn print_box(lines: &[&str], width: usize) {
    let border_color = (ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2);

    // Top border
    println!(
        "{}{}{}",
        "╭".truecolor(border_color.0, border_color.1, border_color.2),
        "─".repeat(width - 2).truecolor(border_color.0, border_color.1, border_color.2),
        "╮".truecolor(border_color.0, border_color.1, border_color.2)
    );

    // Content
    for line in lines {
        let padding = width - 4 - line.chars().count();
        println!(
            "{} {}{} {}",
            "│".truecolor(border_color.0, border_color.1, border_color.2),
            line,
            " ".repeat(padding.max(0)),
            "│".truecolor(border_color.0, border_color.1, border_color.2)
        );
    }

    // Bottom border
    println!(
        "{}{}{}",
        "╰".truecolor(border_color.0, border_color.1, border_color.2),
        "─".repeat(width - 2).truecolor(border_color.0, border_color.1, border_color.2),
        "╯".truecolor(border_color.0, border_color.1, border_color.2)
    );
}

/// Print important notice box
pub fn print_notice(title: &str, message: &str) {
    println!();
    println!(
        "{} {}",
        "⚠".truecolor(ORANGE.0, ORANGE.1, ORANGE.2),
        title.truecolor(ORANGE.0, ORANGE.1, ORANGE.2).bold()
    );
    println!("  {}", message.dimmed());
}

/// Print success box
pub fn print_success(message: &str) {
    println!();
    println!("{} {}", "✓".green().bold(), message.green().bold());
}

/// Print step indicator
pub fn step(number: u8, text: &str) -> String {
    format!(
        "{} {}",
        format!("{}.", number).truecolor(ORANGE.0, ORANGE.1, ORANGE.2).bold(),
        text
    )
}

/// Format an address for display (shortened)
pub fn format_address(addr: &str) -> String {
    if addr.len() > 16 {
        format!(
            "{}...{}",
            &addr[..10],
            &addr[addr.len() - 6..]
        )
    } else {
        addr.to_string()
    }
}

/// Format a hash for display (shortened)
pub fn format_hash(hash: &str) -> String {
    if hash.len() > 20 {
        format!("{}...", &hash[..16])
    } else {
        hash.to_string()
    }
}

/// Spinner characters for loading animations
pub const SPINNER: &[char] = &['◐', '◓', '◑', '◒'];

/// Loading indicator characters
pub const LOADING: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_address() {
        assert_eq!(
            format_address("0x1234567890abcdef1234567890abcdef12345678"),
            "0x12345678...345678"
        );
    }

    #[test]
    fn test_format_hash() {
        assert_eq!(
            format_hash("0x1234567890abcdef1234567890abcdef"),
            "0x12345678901234..."
        );
    }
}
