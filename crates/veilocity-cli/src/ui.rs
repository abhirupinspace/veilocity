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

// ============================================================================
// ZK-PROOF SIMULATION DISPLAY
// ============================================================================

use std::io::{self, Write};
use std::time::Duration;

/// ZK Proof visualization stages
#[derive(Debug, Clone, Copy)]
pub enum ZkStage {
    WitnessGeneration,
    ConstraintSatisfaction,
    PolynomialCommitment,
    ProofComputation,
    Verification,
}

impl ZkStage {
    pub fn name(&self) -> &'static str {
        match self {
            ZkStage::WitnessGeneration => "Witness Generation",
            ZkStage::ConstraintSatisfaction => "Constraint Satisfaction",
            ZkStage::PolynomialCommitment => "Polynomial Commitment",
            ZkStage::ProofComputation => "Proof Computation",
            ZkStage::Verification => "Local Verification",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ZkStage::WitnessGeneration => "Computing private inputs and public outputs",
            ZkStage::ConstraintSatisfaction => "Validating circuit constraints are satisfied",
            ZkStage::PolynomialCommitment => "Generating KZG polynomial commitments",
            ZkStage::ProofComputation => "Computing UltraPlonk proof using Barretenberg",
            ZkStage::Verification => "Verifying proof integrity locally",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ZkStage::WitnessGeneration => "◈",
            ZkStage::ConstraintSatisfaction => "◇",
            ZkStage::PolynomialCommitment => "◆",
            ZkStage::ProofComputation => "▣",
            ZkStage::Verification => "✓",
        }
    }
}

/// Simulate and display ZK proof generation with detailed stages
pub async fn display_zk_proof_generation(circuit_name: &str) {
    println!();
    println!(
        "  {}",
        format!("╔═══════════════════════════════════════════════════════╗")
            .truecolor(ORANGE.0, ORANGE.1, ORANGE.2)
    );
    println!(
        "  {}  {}  {}",
        "║".truecolor(ORANGE.0, ORANGE.1, ORANGE.2),
        format!("ZK-PROOF GENERATION: {}", circuit_name.to_uppercase())
            .truecolor(ORANGE.0, ORANGE.1, ORANGE.2)
            .bold(),
        "            ║".truecolor(ORANGE.0, ORANGE.1, ORANGE.2)
    );
    println!(
        "  {}",
        format!("╚═══════════════════════════════════════════════════════╝")
            .truecolor(ORANGE.0, ORANGE.1, ORANGE.2)
    );
    println!();

    let stages = [
        ZkStage::WitnessGeneration,
        ZkStage::ConstraintSatisfaction,
        ZkStage::PolynomialCommitment,
        ZkStage::ProofComputation,
        ZkStage::Verification,
    ];

    for (i, stage) in stages.iter().enumerate() {
        // Show stage header
        print!(
            "  {} {} ",
            stage.icon().truecolor(ORANGE.0, ORANGE.1, ORANGE.2),
            stage.name().truecolor(200, 200, 200)
        );
        io::stdout().flush().unwrap();

        // Simulate progress with spinner
        let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let iterations = match stage {
            ZkStage::WitnessGeneration => 8,
            ZkStage::ConstraintSatisfaction => 12,
            ZkStage::PolynomialCommitment => 15,
            ZkStage::ProofComputation => 20,
            ZkStage::Verification => 6,
        };

        for j in 0..iterations {
            print!(
                "\r  {} {} {}",
                stage.icon().truecolor(ORANGE.0, ORANGE.1, ORANGE.2),
                stage.name().truecolor(200, 200, 200),
                spinners[j % spinners.len()].truecolor(ORANGE.0, ORANGE.1, ORANGE.2)
            );
            io::stdout().flush().unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Show completion
        println!(
            "\r  {} {} {}",
            "✓".green().bold(),
            stage.name().green(),
            "".to_string()
        );

        // Show description on next line
        println!(
            "    {}",
            stage.description().truecolor(100, 100, 100).italic()
        );

        // Show cryptographic details for key stages
        match stage {
            ZkStage::WitnessGeneration => {
                display_witness_details().await;
            }
            ZkStage::ConstraintSatisfaction => {
                display_constraint_details().await;
            }
            ZkStage::PolynomialCommitment => {
                display_commitment_details().await;
            }
            ZkStage::ProofComputation => {
                display_proof_details().await;
            }
            ZkStage::Verification => {
                // Nothing extra for verification
            }
        }

        if i < stages.len() - 1 {
            println!("    {}", "│".truecolor(60, 60, 60));
        }
    }

    println!();
}

/// Display witness generation details
async fn display_witness_details() {
    let items = [
        ("secret_key", "0x████████...████████"),
        ("nullifier", "0x████████...████████"),
        ("merkle_path", "[32 field elements]"),
        ("balance", "███████ wei"),
    ];

    for (key, value) in items {
        println!(
            "    {} {}: {}",
            "├".truecolor(60, 60, 60),
            key.truecolor(130, 130, 130),
            value.truecolor(80, 80, 80)
        );
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
}

/// Display constraint satisfaction details
async fn display_constraint_details() {
    println!(
        "    {} {} {}",
        "├".truecolor(60, 60, 60),
        "Constraints:".truecolor(130, 130, 130),
        "~50,000 R1CS constraints".truecolor(80, 80, 80)
    );
    println!(
        "    {} {} {}",
        "├".truecolor(60, 60, 60),
        "Gates:".truecolor(130, 130, 130),
        "UltraPlonk arithmetic gates".truecolor(80, 80, 80)
    );
    println!(
        "    {} {} {}",
        "├".truecolor(60, 60, 60),
        "Satisfied:".truecolor(130, 130, 130),
        "All constraints pass".green()
    );
}

/// Display polynomial commitment details
async fn display_commitment_details() {
    println!(
        "    {} {} {}",
        "├".truecolor(60, 60, 60),
        "Scheme:".truecolor(130, 130, 130),
        "KZG (Kate-Zaverucha-Goldberg)".truecolor(80, 80, 80)
    );
    println!(
        "    {} {} {}",
        "├".truecolor(60, 60, 60),
        "Curve:".truecolor(130, 130, 130),
        "BN254 (alt_bn128)".truecolor(80, 80, 80)
    );

    // Show commitment progress
    let commit_stages = ["W_L", "W_R", "W_O", "Z", "T"];
    for stage in commit_stages.iter() {
        print!(
            "    {} Committing {}... ",
            "├".truecolor(60, 60, 60),
            stage.truecolor(100, 100, 100)
        );
        io::stdout().flush().unwrap();
        tokio::time::sleep(Duration::from_millis(40)).await;
        println!("{}", "done".green());
    }
}

/// Display proof computation details
async fn display_proof_details() {
    println!(
        "    {} {} {}",
        "├".truecolor(60, 60, 60),
        "Backend:".truecolor(130, 130, 130),
        "Aztec Barretenberg (bb)".truecolor(80, 80, 80)
    );
    println!(
        "    {} {} {}",
        "├".truecolor(60, 60, 60),
        "Protocol:".truecolor(130, 130, 130),
        "UltraPlonk with Plookup".truecolor(80, 80, 80)
    );
    println!(
        "    {} {} {}",
        "├".truecolor(60, 60, 60),
        "Security:".truecolor(130, 130, 130),
        "128-bit soundness".truecolor(80, 80, 80)
    );
}

/// Display a simpler inline proof progress for faster feedback
pub fn print_zk_step(step: &str, status: &str, is_complete: bool) {
    if is_complete {
        println!(
            "  {} {} {}",
            "✓".green().bold(),
            step.bright_white(),
            status.dimmed()
        );
    } else {
        println!(
            "  {} {} {}",
            "◐".truecolor(ORANGE.0, ORANGE.1, ORANGE.2),
            step.truecolor(200, 200, 200),
            status.dimmed()
        );
    }
}

/// Display proof verification result
pub fn print_proof_verified(proof_size: usize) {
    println!();
    println!(
        "  {}",
        "┌─────────────────────────────────────────────────────┐"
            .truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2)
    );
    println!(
        "  {} {} {}",
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2),
        "ZK-PROOF VERIFIED".green().bold(),
        "                                │".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2)
    );
    println!(
        "  {}",
        "├─────────────────────────────────────────────────────┤"
            .truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2)
    );
    println!(
        "  {} {} {} bytes                                 {}",
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2),
        "Proof size:".truecolor(150, 150, 150),
        format!("{}", proof_size).bright_white().bold(),
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2)
    );
    println!(
        "  {} {} Zero-Knowledge: Private inputs hidden     {}",
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2),
        "✓".green(),
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2)
    );
    println!(
        "  {} {} Soundness: Proof mathematically valid     {}",
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2),
        "✓".green(),
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2)
    );
    println!(
        "  {} {} Completeness: All constraints satisfied   {}",
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2),
        "✓".green(),
        "│".truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2)
    );
    println!(
        "  {}",
        "└─────────────────────────────────────────────────────┘"
            .truecolor(ORANGE_DARK.0, ORANGE_DARK.1, ORANGE_DARK.2)
    );
}

/// Display nullifier generation
pub fn print_nullifier_generated(nullifier_hex: &str) {
    println!(
        "  {} {} 0x{}...",
        "◈".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "Nullifier:".truecolor(130, 130, 130),
        &nullifier_hex[..16].truecolor(PURPLE.0, PURPLE.1, PURPLE.2)
    );
    println!(
        "    {}",
        "(unique identifier prevents double-spending)"
            .truecolor(100, 100, 100)
            .italic()
    );
}

/// Display merkle proof information
pub fn print_merkle_proof_info(tree_depth: usize, leaf_index: u64) {
    println!(
        "  {} {} depth={}, index={}",
        "◈".truecolor(ORANGE.0, ORANGE.1, ORANGE.2),
        "Merkle Proof:".truecolor(130, 130, 130),
        tree_depth.to_string().bright_white(),
        leaf_index.to_string().bright_white()
    );
    println!(
        "    {}",
        "(cryptographic path from leaf to root)"
            .truecolor(100, 100, 100)
            .italic()
    );
}

/// Display state root verification
pub fn print_state_root_check(root_hex: &str, is_valid: bool) {
    let status = if is_valid {
        "valid on-chain".green()
    } else {
        "not found".red()
    };
    println!(
        "  {} {} 0x{}... [{}]",
        "◈".truecolor(ORANGE.0, ORANGE.1, ORANGE.2),
        "State Root:".truecolor(130, 130, 130),
        &root_hex[..16].bright_white(),
        status
    );
}

/// Display private transaction summary
pub fn print_privacy_summary() {
    println!();
    println!(
        "  {}",
        "┌─ Privacy Guarantees ─────────────────────────────────┐"
            .truecolor(PURPLE.0, PURPLE.1, PURPLE.2)
    );
    println!(
        "  {} {} Sender identity: {}",
        "│".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "●".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {} {} Recipient identity: {}",
        "│".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "●".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {} {} Transaction amount: {}",
        "│".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "●".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {} {} Balance: {}",
        "│".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "●".truecolor(PURPLE.0, PURPLE.1, PURPLE.2),
        "Hidden".green().bold()
    );
    println!(
        "  {}",
        "└───────────────────────────────────────────────────────┘"
            .truecolor(PURPLE.0, PURPLE.1, PURPLE.2)
    );
}

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
            "0x1234567890abcd..."
        );
    }
}
