//! Deposit Form State
//!
//! State management for the deposit form.

/// Deposit form state
#[derive(Debug, Default, Clone)]
pub struct DepositForm {
    /// Amount input string
    pub amount_input: String,
    /// Cursor position in input
    pub cursor_position: usize,
    /// Validation error message
    pub validation_error: Option<String>,
    /// Whether the form is currently processing
    pub is_processing: bool,
}

impl DepositForm {
    /// Create a new deposit form
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the form to initial state
    pub fn reset(&mut self) {
        self.amount_input.clear();
        self.cursor_position = 0;
        self.validation_error = None;
        self.is_processing = false;
    }

    /// Input a character
    pub fn input_char(&mut self, c: char) {
        // Only allow digits and decimal point
        if c.is_ascii_digit() || (c == '.' && !self.amount_input.contains('.')) {
            self.amount_input.insert(self.cursor_position, c);
            self.cursor_position += 1;
            self.validation_error = None;
        }
    }

    /// Handle backspace
    pub fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.amount_input.remove(self.cursor_position);
            self.validation_error = None;
        }
    }

    /// Handle delete
    pub fn delete(&mut self) {
        if self.cursor_position < self.amount_input.len() {
            self.amount_input.remove(self.cursor_position);
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
        if self.cursor_position < self.amount_input.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start
    pub fn cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end
    pub fn cursor_end(&mut self) {
        self.cursor_position = self.amount_input.len();
    }

    /// Get the parsed amount if valid
    pub fn get_amount(&self) -> Option<f64> {
        self.amount_input.parse().ok()
    }

    /// Check if the form has valid input
    pub fn is_valid(&self) -> bool {
        if let Ok(amount) = self.amount_input.parse::<f64>() {
            amount > 0.0
        } else {
            false
        }
    }
}
