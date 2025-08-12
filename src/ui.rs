use comfy_table::{ presets::UTF8_FULL_CONDENSED, Table };
use console::{ style, Emoji, Term };
use indicatif::{ ProgressBar, ProgressStyle };
use std::time::Duration;

// Let's create some shortcuts for emojies
pub const ROCKET: Emoji<'_, '_> = Emoji("🚀", "↗");
pub const CHECK: Emoji<'_, '_> = Emoji("✅", "✓");
pub const CROSS: Emoji<'_, '_> = Emoji("❌", "x");
pub const WARN: Emoji<'_, '_> = Emoji("⚠️", "!");
pub const INFO: Emoji<'_, '_> = Emoji("ℹ️", "i");

/// Prints a header with a title in a styled format
/// # Arguments
/// * `title` - The title to display in the header
pub fn header(title: &str) {
    let term = Term::stdout();
    let width = term.size().1 as usize;
    let bar: String = "─".repeat(width.saturating_sub(1).min(60));
    println!("{} {}", style(bar).dim(), style(format!(" {} ", title)).bold().magenta());
}

pub fn success(msg: &str) {
    println!("{} {}", CHECK, style(msg).green().bold());
}

pub fn info(msg: &str) {
    println!("{} {}", INFO, style(msg).green().bold());
}

pub fn warning(msg: &str) {
    println!("{} {}", WARN, style(msg).yellow().bold());
}

pub fn error(msg: &str) {
    println!("{} {}", CROSS, style(msg).red().bold());
}

/// Pretty progress bar that can be manually advance
pub fn pbar(len: u64, message: &str) -> ProgressBar {
    let pb: ProgressBar = ProgressBar::new(len);
    pb.set_message(message.to_string());
    let template: String =
        "{spinner} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3} {msg}".to_string();
    let style: ProgressStyle = ProgressStyle::with_template(&template)
        .unwrap()
        .progress_chars("█▓░");
    pb.set_style(style);
    return pb;
}

/// Spin and carry computation in f()
pub fn spin<F, T>(msg: &str, f: F) -> T where F: FnOnce() -> T {
    let pb: ProgressBar = ProgressBar::new_spinner();
    let style: ProgressStyle = ProgressStyle::with_template("{spinner} {msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ");
    pb.set_style(style);
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message(msg.to_string());
    let res: T = f();
    pb.finish_with_message("done");
    return res;
}

/// Pretty Table
pub fn table(headers: &[&str], rows: &[Vec<String>]) -> Table {
    let mut table: Table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(headers);
    for row in rows {
        table.add_row(row.clone());
    }
    return table;
}
