#![allow(dead_code)]

use colored::Colorize;

/// Print a dimmed tip with "Tip:" prefix.
pub fn print_tip(msg: &str) {
    println!("  {} {}", "Tip:".cyan().bold(), msg.dimmed());
}

/// Print a green checkmark + bold message.
pub fn print_success(msg: &str) {
    println!("  {} {}", "✓".green(), msg.bold());
}

/// Print a yellow warning prefix + message.
pub fn print_warning(msg: &str) {
    println!("  {} {}", "Warning:".yellow().bold(), msg);
}

/// Print a red error + cyan hint on the next line.
pub fn print_error_with_hint(error: &str, hint: &str) {
    eprintln!("{} {}", "Error:".red().bold(), error);
    eprintln!("  {} {}", "Hint:".cyan().bold(), hint);
}

/// Print a bold underlined section header.
pub fn print_section_header(title: &str) {
    println!();
    println!("  {}", title.bold().underline());
    println!();
}

/// Format a table as aligned columns. Returns the formatted string.
pub fn format_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    if headers.is_empty() {
        return String::new();
    }

    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let mut out = String::new();

    // Header row
    let header_line: Vec<String> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:<width$}", h, width = widths[i]))
        .collect();
    out.push_str(&format!("  {}\n", header_line.join("  ")));

    // Separator
    let sep: Vec<String> = widths.iter().map(|w| "─".repeat(*w)).collect();
    out.push_str(&format!("  {}\n", sep.join("  ")));

    // Data rows
    for row in rows {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let w = widths.get(i).copied().unwrap_or(cell.len());
                format!("{:<width$}", cell, width = w)
            })
            .collect();
        out.push_str(&format!("  {}\n", cells.join("  ")));
    }

    out
}

/// Print a table with aligned columns using padding.
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    print!("{}", format_table(headers, rows));
}

/// Print a cyan command + dimmed description.
pub fn print_command_hint(cmd: &str, description: &str) {
    println!("  {}  {}", cmd.cyan(), description.dimmed());
}

/// Print the forja banner with version and tagline.
pub fn print_banner() {
    println!();
    println!(
        "  {}",
        format!("forja v{}", env!("CARGO_PKG_VERSION")).bold()
    );
    println!("  {}", "Skills marketplace for Claude Code".dimmed());
    print_divider();
}

/// Print a dimmed double-line divider.
pub fn print_divider() {
    println!("  {}", "══════════════════════════════════════".dimmed());
}

/// Print a step indicator like `[1/3] Setup mode`.
pub fn print_step(current: usize, total: usize, label: &str) {
    println!();
    println!(
        "  {} {}",
        format!("[{}/{}]", current, total).cyan().bold(),
        label.bold()
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_table_empty_headers() {
        let result = format_table(&[], &[]);
        assert_eq!(result, "");
    }

    #[test]
    fn format_table_single_row() {
        let headers = &["Name", "Phase"];
        let rows = vec![vec!["feature".to_string(), "code".to_string()]];
        let result = format_table(headers, &rows);

        assert!(result.contains("Name"));
        assert!(result.contains("Phase"));
        assert!(result.contains("feature"));
        assert!(result.contains("code"));
    }

    #[test]
    fn format_table_aligns_columns() {
        let headers = &["ID", "Description"];
        let rows = vec![
            vec!["short".to_string(), "A short one".to_string()],
            vec!["very-long-id".to_string(), "Another".to_string()],
        ];
        let result = format_table(headers, &rows);
        let lines: Vec<&str> = result.lines().collect();

        // Header and separator and 2 data rows
        assert_eq!(lines.len(), 4);

        // All data columns should start at the same position
        let col2_pos_row1 = lines[2].find("A short one");
        let col2_pos_row2 = lines[3].find("Another");
        assert!(col2_pos_row1.is_some());
        assert!(col2_pos_row2.is_some());
        assert_eq!(col2_pos_row1.unwrap(), col2_pos_row2.unwrap());
    }

    #[test]
    fn format_table_handles_unicode() {
        let headers = &["Name"];
        let rows = vec![vec!["café".to_string()], vec!["naïve".to_string()]];
        let result = format_table(headers, &rows);
        assert!(result.contains("café"));
        assert!(result.contains("naïve"));
    }

    #[test]
    fn format_table_long_strings() {
        let long = "a".repeat(200);
        let headers = &["Col"];
        let rows = vec![vec![long.clone()]];
        let result = format_table(headers, &rows);
        assert!(result.contains(&long));
    }

    #[test]
    fn format_command_hint_pattern() {
        // Verify the function doesn't panic — output goes to stdout
        // so we just test it runs without error
        print_command_hint("forja install", "Install a skill");
    }

    #[test]
    fn print_banner_does_not_panic() {
        print_banner();
    }

    #[test]
    fn print_divider_does_not_panic() {
        print_divider();
    }

    #[test]
    fn print_step_does_not_panic() {
        print_step(1, 3, "Setup mode");
        print_step(2, 3, "Workflow phases");
        print_step(3, 3, "Model profile");
    }
}
