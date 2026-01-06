//! TUI Widgets
//!
//! Custom Ratatui widgets for the Veilocity dashboard.

pub mod balance;
pub mod events;
pub mod forms;
pub mod header;
pub mod history;
pub mod status;

pub use balance::render_balance_panel;
pub use events::render_events_panel;
pub use forms::{render_deposit_form, render_withdraw_form};
pub use header::render_header;
pub use history::render_history_panel;
pub use status::render_status_bar;
