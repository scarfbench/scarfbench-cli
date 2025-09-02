use comfy_table::{ presets::UTF8_FULL_CONDENSED, Table };
use console::{ style, Emoji, Term };

// Let's create some shortcuts for emojies
pub const SCARF: Emoji<'_, '_> = Emoji("🧣", "");
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
