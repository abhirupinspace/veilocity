//! TUI Forms
//!
//! Form state management for deposit and withdraw operations.

pub mod deposit;
pub mod validation;
pub mod withdraw;

pub use deposit::DepositForm;
pub use validation::{validate_address, validate_amount};
pub use withdraw::{WithdrawField, WithdrawForm};
