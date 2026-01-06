//! Withdraw Form State
//!
//! State management for the withdraw form.

/// Active field in withdraw form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WithdrawField {
    #[default]
    Amount,
    Recipient,
}

/// Withdraw form state
#[derive(Debug, Default, Clone)]
pub struct WithdrawForm {
    /// Amount input string
    pub amount_input: String,
    /// Recipient address input
    pub recipient_input: String,
    /// Currently active field
    pub active_field: WithdrawField,
    /// Cursor position in active input
    pub cursor_position: usize,
    /// Validation error message
    pub validation_error: Option<String>,
    /// Whether the form is currently processing
    pub is_processing: bool,
}

impl WithdrawForm {
    /// Create a new withdraw form
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the form to initial state
    pub fn reset(&mut self) {
        self.amount_input.clear();
        self.recipient_input.clear();
        self.active_field = WithdrawField::Amount;
        self.cursor_position = 0;
        self.validation_error = None;
        self.is_processing = false;
    }

    /// Toggle between fields
    pub fn toggle_field(&mut self) {
        match self.active_field {
            WithdrawField::Amount => {
                self.active_field = WithdrawField::Recipient;
                self.cursor_position = self.recipient_input.len();
            }
            WithdrawField::Recipient => {
                self.active_field = WithdrawField::Amount;
                self.cursor_position = self.amount_input.len();
            }
        }
    }

    /// Get the currently active input buffer
    fn active_buffer(&self) -> &String {
        match self.active_field {
            WithdrawField::Amount => &self.amount_input,
            WithdrawField::Recipient => &self.recipient_input,
        }
    }

    /// Get the currently active input buffer mutably
    fn active_buffer_mut(&mut self) -> &mut String {
        match self.active_field {
            WithdrawField::Amount => &mut self.amount_input,
            WithdrawField::Recipient => &mut self.recipient_input,
        }
    }

    /// Input a character
    pub fn input_char(&mut self, c: char) {
        match self.active_field {
            WithdrawField::Amount => {
                // Only allow digits and decimal point for amount
                if c.is_ascii_digit() || (c == '.' && !self.amount_input.contains('.')) {
                    self.amount_input.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    self.validation_error = None;
                }
            }
            WithdrawField::Recipient => {
                // Allow hex characters and 'x' for addresses
                if c.is_ascii_hexdigit() || c == 'x' || c == 'X' {
                    self.recipient_input.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    self.validation_error = None;
                }
            }
        }
    }

    /// Handle backspace
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            let pos = self.cursor_position;
            self.active_buffer_mut().remove(pos);
            self.validation_error = None;
        }
    }

    /// Handle delete
    pub fn delete(&mut self) {
        let len = self.active_buffer().len();
        let pos = self.cursor_position;
        if pos < len {
            self.active_buffer_mut().remove(pos);
            self.validation_error = None;
        }
    }

    /// Move cursor left
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right
    pub fn cursor_right(&mut self) {
        let len = self.active_buffer().len();
        if self.cursor_position < len {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.active_buffer().len();
    }

    /// Get the parsed amount if valid
    pub fn get_amount(&self) -> Option<f64> {
        self.amount_input.parse().ok()
    }

    /// Check if the form has valid input
    pub fn is_valid(&self) -> bool {
        // Amount must be valid
        let amount_valid = if let Ok(amount) = self.amount_input.parse::<f64>() {
            amount > 0.0
        } else {
            false
        };

        // Recipient can be empty (uses self) or must be valid address
        let recipient_valid = self.recipient_input.is_empty()
            || (self.recipient_input.starts_with("0x") && self.recipient_input.len() == 42);

        amount_valid && recipient_valid
    }
}
