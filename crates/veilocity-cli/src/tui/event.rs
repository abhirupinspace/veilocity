//! TUI Event Loop
//!
//! Handles user input, background task coordination, and UI refresh.

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event};

use super::app::{App, AppEvent};
use super::input::handle_key_event;
use super::tasks::spawn_background_tasks;
use super::ui::render;
use super::Tui;

/// UI tick rate (100ms = 10 FPS)
const TICK_RATE: Duration = Duration::from_millis(100);

/// Run the main event loop
pub async fn run_event_loop(terminal: &mut Tui, app: &mut App) -> Result<()> {
    let mut last_tick = Instant::now();

    // Spawn background tasks
    let event_tx = app.event_tx.clone();
    let config = app.config.clone();
    let _background_handle = spawn_background_tasks(config, event_tx);

    loop {
        // Render UI
        terminal.draw(|frame| render(frame, app))?;

        // Calculate timeout until next tick
        let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());

        // Poll for crossterm events with timeout
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    // Handle key event
                    handle_key_event(app, key)?;
                }
                Event::Resize(_, _) => {
                    // Terminal resized - will be handled on next render
                }
                Event::Mouse(_) => {
                    // Mouse events - could be handled for scrolling etc.
                }
                _ => {}
            }
        }

        // Process background events (non-blocking)
        while let Ok(event) = app.event_rx.try_recv() {
            app.handle_app_event(event);
        }

        // Periodic tick
        if last_tick.elapsed() >= TICK_RATE {
            app.handle_app_event(AppEvent::Tick);
            last_tick = Instant::now();
        }

        // Check for quit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
