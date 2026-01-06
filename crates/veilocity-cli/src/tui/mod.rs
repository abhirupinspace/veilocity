//! Veilocity TUI - Real-time Terminal User Interface
//!
//! A full-featured dashboard for managing private transactions on Mantle.

pub mod actions;
pub mod app;
pub mod event;
pub mod input;
pub mod tasks;
pub mod theme;
pub mod ui;

pub mod forms;
pub mod widgets;

use std::io::{stdout, Stdout};

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use crate::config::Config;

pub use app::App;
pub use event::run_event_loop;

/// Terminal type alias for the TUI
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI mode
pub fn init_terminal() -> Result<Tui> {
    enable_raw_mode().context("Failed to enable raw mode")?;
    execute!(stdout(), EnterAlternateScreen).context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend).context("Failed to create terminal")?;

    Ok(terminal)
}

/// Restore the terminal to its original state
pub fn restore_terminal() -> Result<()> {
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(stdout(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;
    Ok(())
}

/// Main entry point for the TUI
pub async fn run(config: Config) -> Result<()> {
    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Create app state
    let app_result = App::new(config).await;

    let result = match app_result {
        Ok(mut app) => {
            // Run event loop
            let run_result = run_event_loop(&mut terminal, &mut app).await;

            // Cleanup
            restore_terminal()?;

            run_result
        }
        Err(e) => {
            restore_terminal()?;
            Err(e)
        }
    };

    result
}
