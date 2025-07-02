//! Diff visualization and formatting
//!
//! This module provides various output formats for displaying diffs including
//! colored terminal output, side-by-side diffs, unified diff format, and HTML reports.

use crate::diff::{DiffChange, DiffResult, DiffType};
use std::fmt::Write;

/// Terminal colors for diff output
pub mod colors {
    /// ANSI color codes for terminal output
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const CYAN: &str = "\x1b[36m";
    pub const GRAY: &str = "\x1b[90m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
}

/// Configuration for diff display options
#[derive(Debug, Clone)]
pub struct DisplayOptions {
    /// Whether to use colors in terminal output
    pub use_colors: bool,
    /// Number of columns for side-by-side display
    pub terminal_width: usize,
    /// Whether to show line numbers
    pub show_line_numbers: bool,
    /// Whether to show context lines
    pub show_context: bool,
    /// Maximum lines to display (0 for unlimited)
    pub max_lines: usize,
    /// Whether to compact unchanged regions
    pub compact_unchanged: bool,
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self {
            use_colors: true,
            terminal_width: 120,
            show_line_numbers: true,
            show_context: true,
            max_lines: 0,
            compact_unchanged: true,
        }
    }
}

/// Trait for formatting diff output in various styles
pub trait DiffFormatter {
    /// Format diff result as a string
    fn format(&self, diff: &DiffResult, options: &DisplayOptions) -> String;
}

/// Colored terminal diff formatter
pub struct ColoredTerminalFormatter;

impl DiffFormatter for ColoredTerminalFormatter {
    fn format(&self, diff: &DiffResult, options: &DisplayOptions) -> String {
        let mut output = String::new();

        // Header with summary
        if options.use_colors {
            writeln!(
                output,
                "{}{}Diff Summary{}{}",
                colors::BOLD,
                colors::CYAN,
                colors::RESET,
                colors::RESET
            )
            .unwrap();
        } else {
            writeln!(output, "Diff Summary").unwrap();
        }

        writeln!(output, "  Total changes: {}", diff.summary.total_changes).unwrap();
        writeln!(output, "  Additions: {}", diff.summary.additions).unwrap();
        writeln!(output, "  Deletions: {}", diff.summary.deletions).unwrap();
        writeln!(output, "  Modifications: {}", diff.summary.modifications).unwrap();
        writeln!(output, "  Risk level: {}", diff.summary.highest_risk).unwrap();
        writeln!(output).unwrap();

        // Format text diff changes
        self.format_text_changes(&diff.text_diff.changes, options, &mut output);

        output
    }
}

impl ColoredTerminalFormatter {
    fn format_text_changes(
        &self,
        changes: &[DiffChange],
        options: &DisplayOptions,
        output: &mut String,
    ) {
        let mut unchanged_count = 0;

        for change in changes {
            if change.change_type == DiffType::Unchanged {
                if options.compact_unchanged {
                    unchanged_count += 1;
                    continue;
                }
                self.format_unchanged_line(change, options, output);
            } else {
                // Show compacted unchanged lines if any
                if unchanged_count > 0 {
                    self.format_unchanged_separator(unchanged_count, options, output);
                    unchanged_count = 0;
                }
                self.format_changed_line(change, options, output);
            }
        }

        // Handle remaining unchanged lines
        if unchanged_count > 0 {
            self.format_unchanged_separator(unchanged_count, options, output);
        }
    }

    fn format_changed_line(
        &self,
        change: &DiffChange,
        options: &DisplayOptions,
        output: &mut String,
    ) {
        let (prefix, color) = if options.use_colors {
            match change.change_type {
                DiffType::Addition => ("+", colors::GREEN),
                DiffType::Deletion => ("-", colors::RED),
                DiffType::Modification => ("~", colors::YELLOW),
                DiffType::Unchanged => (" ", ""),
            }
        } else {
            match change.change_type {
                DiffType::Addition => ("+", ""),
                DiffType::Deletion => ("-", ""),
                DiffType::Modification => ("~", ""),
                DiffType::Unchanged => (" ", ""),
            }
        };

        if options.show_line_numbers {
            let old_num = change
                .old_line_number
                .map_or("   ".to_string(), |n| format!("{n:3}"));
            let new_num = change
                .new_line_number
                .map_or("   ".to_string(), |n| format!("{n:3}"));

            if options.use_colors {
                writeln!(
                    output,
                    "{}{} {}:{} {}{}{}",
                    color,
                    prefix,
                    old_num,
                    new_num,
                    change
                        .new_line
                        .as_ref()
                        .or(change.old_line.as_ref())
                        .unwrap_or(&String::new()),
                    colors::RESET,
                    colors::RESET
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "{} {}:{} {}",
                    prefix,
                    old_num,
                    new_num,
                    change
                        .new_line
                        .as_ref()
                        .or(change.old_line.as_ref())
                        .unwrap_or(&String::new())
                )
                .unwrap();
            }
        } else if options.use_colors {
            writeln!(
                output,
                "{}{} {}{}{}",
                color,
                prefix,
                change
                    .new_line
                    .as_ref()
                    .or(change.old_line.as_ref())
                    .unwrap_or(&String::new()),
                colors::RESET,
                colors::RESET
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "{} {}",
                prefix,
                change
                    .new_line
                    .as_ref()
                    .or(change.old_line.as_ref())
                    .unwrap_or(&String::new())
            )
            .unwrap();
        }
    }

    fn format_unchanged_line(
        &self,
        change: &DiffChange,
        options: &DisplayOptions,
        output: &mut String,
    ) {
        if !options.show_context {
            return;
        }

        if options.show_line_numbers {
            let old_num = change
                .old_line_number
                .map_or("   ".to_string(), |n| format!("{n:3}"));
            let new_num = change
                .new_line_number
                .map_or("   ".to_string(), |n| format!("{n:3}"));

            if options.use_colors {
                writeln!(
                    output,
                    "{}  {}:{} {}{}",
                    colors::DIM,
                    old_num,
                    new_num,
                    change.old_line.as_ref().unwrap_or(&String::new()),
                    colors::RESET
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "  {}:{} {}",
                    old_num,
                    new_num,
                    change.old_line.as_ref().unwrap_or(&String::new())
                )
                .unwrap();
            }
        } else if options.use_colors {
            writeln!(
                output,
                "{}  {}{}",
                colors::DIM,
                change.old_line.as_ref().unwrap_or(&String::new()),
                colors::RESET
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "  {}",
                change.old_line.as_ref().unwrap_or(&String::new())
            )
            .unwrap();
        }
    }

    fn format_unchanged_separator(
        &self,
        count: usize,
        options: &DisplayOptions,
        output: &mut String,
    ) {
        if options.use_colors {
            writeln!(
                output,
                "{}... {} unchanged lines ...{}",
                colors::GRAY,
                count,
                colors::RESET
            )
            .unwrap();
        } else {
            writeln!(output, "... {count} unchanged lines ...").unwrap();
        }
    }
}

/// Side-by-side diff formatter
pub struct SideBySideFormatter;

impl DiffFormatter for SideBySideFormatter {
    fn format(&self, diff: &DiffResult, options: &DisplayOptions) -> String {
        let mut output = String::new();

        // Header
        writeln!(output, "Side-by-Side Diff").unwrap();
        writeln!(output, "{}", "=".repeat(options.terminal_width)).unwrap();

        let half_width = (options.terminal_width - 10) / 2; // Account for separators and margins

        // Column headers
        writeln!(
            output,
            "{:<width$} | {:<width$}",
            "Old Configuration",
            "New Configuration",
            width = half_width
        )
        .unwrap();
        writeln!(output, "{}", "-".repeat(options.terminal_width)).unwrap();

        // Format changes side by side
        for change in &diff.text_diff.changes {
            self.format_side_by_side_line(change, options, half_width, &mut output);
        }

        output
    }
}

impl SideBySideFormatter {
    fn format_side_by_side_line(
        &self,
        change: &DiffChange,
        options: &DisplayOptions,
        half_width: usize,
        output: &mut String,
    ) {
        let old_line = change.old_line.as_deref().unwrap_or("");
        let new_line = change.new_line.as_deref().unwrap_or("");

        // Truncate lines if too long
        let old_display = if old_line.len() > half_width {
            format!("{}...", &old_line[..half_width.saturating_sub(3)])
        } else {
            old_line.to_string()
        };

        let new_display = if new_line.len() > half_width {
            format!("{}...", &new_line[..half_width.saturating_sub(3)])
        } else {
            new_line.to_string()
        };

        let (old_color, new_color, separator) = if options.use_colors {
            match change.change_type {
                DiffType::Addition => ("", colors::GREEN, "|"),
                DiffType::Deletion => (colors::RED, "", "|"),
                DiffType::Modification => (colors::RED, colors::GREEN, "|"),
                DiffType::Unchanged => (colors::DIM, colors::DIM, "|"),
            }
        } else {
            ("", "", "|")
        };

        if options.use_colors {
            writeln!(
                output,
                "{}{:<width$}{} {} {}{:<width$}{}",
                old_color,
                old_display,
                colors::RESET,
                separator,
                new_color,
                new_display,
                colors::RESET,
                width = half_width
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "{old_display:<half_width$} {separator} {new_display:<half_width$}"
            )
            .unwrap();
        }
    }
}

/// Unified diff formatter (Git-style)
pub struct UnifiedFormatter;

impl DiffFormatter for UnifiedFormatter {
    fn format(&self, diff: &DiffResult, _options: &DisplayOptions) -> String {
        let mut output = String::new();

        // Unified diff header
        writeln!(output, "--- old").unwrap();
        writeln!(output, "+++ new").unwrap();

        // Process changes in chunks
        let chunks =
            self.group_changes_into_chunks(&diff.text_diff.changes, diff.text_diff.context_lines);

        for chunk in chunks {
            self.format_chunk(&chunk, &mut output);
        }

        output
    }
}

impl UnifiedFormatter {
    fn group_changes_into_chunks<'a>(
        &self,
        changes: &'a [DiffChange],
        context_lines: usize,
    ) -> Vec<Vec<&'a DiffChange>> {
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        let mut last_change_index = None;

        for (i, change) in changes.iter().enumerate() {
            if let Some(last_idx) = last_change_index {
                // If there's a gap larger than context_lines * 2, start a new chunk
                if i - last_idx > context_lines * 2
                    && change.change_type == DiffType::Unchanged
                    && !current_chunk.is_empty()
                {
                    chunks.push(current_chunk);
                    current_chunk = Vec::new();
                }
            }

            current_chunk.push(change);

            if change.change_type != DiffType::Unchanged {
                last_change_index = Some(i);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    fn format_chunk(&self, chunk: &[&DiffChange], output: &mut String) {
        if chunk.is_empty() {
            return;
        }

        // Calculate chunk header
        let first_old = chunk.first().and_then(|c| c.old_line_number).unwrap_or(1);
        let first_new = chunk.first().and_then(|c| c.new_line_number).unwrap_or(1);
        let old_count = chunk.iter().filter(|c| c.old_line.is_some()).count();
        let new_count = chunk.iter().filter(|c| c.new_line.is_some()).count();

        writeln!(
            output,
            "@@ -{first_old},{old_count} +{first_new},{new_count} @@"
        )
        .unwrap();

        // Format chunk lines
        for change in chunk {
            match change.change_type {
                DiffType::Addition => {
                    writeln!(
                        output,
                        "+{}",
                        change.new_line.as_ref().unwrap_or(&String::new())
                    )
                    .unwrap();
                }
                DiffType::Deletion => {
                    writeln!(
                        output,
                        "-{}",
                        change.old_line.as_ref().unwrap_or(&String::new())
                    )
                    .unwrap();
                }
                DiffType::Modification => {
                    writeln!(
                        output,
                        "-{}",
                        change.old_line.as_ref().unwrap_or(&String::new())
                    )
                    .unwrap();
                    writeln!(
                        output,
                        "+{}",
                        change.new_line.as_ref().unwrap_or(&String::new())
                    )
                    .unwrap();
                }
                DiffType::Unchanged => {
                    writeln!(
                        output,
                        " {}",
                        change.old_line.as_ref().unwrap_or(&String::new())
                    )
                    .unwrap();
                }
            }
        }
    }
}

/// HTML diff report formatter
pub struct HtmlFormatter;

impl DiffFormatter for HtmlFormatter {
    fn format(&self, diff: &DiffResult, _options: &DisplayOptions) -> String {
        let mut output = String::new();

        // HTML header and styles
        writeln!(output, r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Configuration Diff Report</title>
    <style>
        body {{ font-family: 'Courier New', monospace; margin: 20px; }}
        .header {{ background: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }}
        .summary {{ margin-bottom: 20px; }}
        .diff-container {{ border: 1px solid #ddd; border-radius: 5px; }}
        .diff-line {{ padding: 4px 8px; border-left: 3px solid transparent; }}
        .addition {{ background: #e8f5e8; border-left-color: #28a745; }}
        .deletion {{ background: #ffeaea; border-left-color: #dc3545; }}
        .modification {{ background: #fff3cd; border-left-color: #ffc107; }}
        .unchanged {{ background: #f8f9fa; color: #6c757d; }}
        .line-number {{ color: #6c757d; margin-right: 10px; min-width: 40px; display: inline-block; }}
        .compact-separator {{ background: #e9ecef; color: #6c757d; text-align: center; padding: 8px; }}
        .risk-high {{ color: #dc3545; font-weight: bold; }}
        .risk-medium {{ color: #fd7e14; font-weight: bold; }}
        .risk-low {{ color: #28a745; }}
        .stats {{ display: flex; gap: 20px; flex-wrap: wrap; }}
        .stat-item {{ background: white; padding: 10px; border-radius: 3px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
    </style>
</head>
<body>"#).unwrap();

        // Header and summary
        writeln!(
            output,
            r#"    <div class="header">
        <h1>Configuration Diff Report</h1>
        <p>Generated at: {}</p>
    </div>"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        )
        .unwrap();

        writeln!(
            output,
            r#"    <div class="summary">
        <h2>Summary</h2>
        <div class="stats">
            <div class="stat-item">
                <strong>Total Changes:</strong> {}
            </div>
            <div class="stat-item">
                <strong>Additions:</strong> <span style="color: #28a745;">{}</span>
            </div>
            <div class="stat-item">
                <strong>Deletions:</strong> <span style="color: #dc3545;">{}</span>
            </div>
            <div class="stat-item">
                <strong>Modifications:</strong> <span style="color: #ffc107;">{}</span>
            </div>
            <div class="stat-item">
                <strong>Risk Level:</strong> <span class="risk-{}">{}</span>
            </div>
        </div>
    </div>"#,
            diff.summary.total_changes,
            diff.summary.additions,
            diff.summary.deletions,
            diff.summary.modifications,
            diff.summary.highest_risk.to_string().to_lowercase(),
            diff.summary.highest_risk
        )
        .unwrap();

        // Diff content
        writeln!(
            output,
            r#"    <div class="diff-container">
        <h3>Changes</h3>"#
        )
        .unwrap();

        let mut unchanged_count = 0;

        for change in &diff.text_diff.changes {
            if change.change_type == DiffType::Unchanged {
                unchanged_count += 1;
                continue;
            }
            if unchanged_count > 0 {
                writeln!(
                    output,
                    r#"        <div class="diff-line compact-separator">
                ... {unchanged_count} unchanged lines ...
            </div>"#
                )
                .unwrap();
                unchanged_count = 0;
            }
            self.format_html_line(change, &mut output);
        }

        if unchanged_count > 0 {
            writeln!(
                output,
                r#"        <div class="diff-line compact-separator">
            ... {unchanged_count} unchanged lines ...
        </div>"#
            )
            .unwrap();
        }

        writeln!(
            output,
            r"    </div>
</body>
</html>"
        )
        .unwrap();

        output
    }
}

impl HtmlFormatter {
    fn format_html_line(&self, change: &DiffChange, output: &mut String) {
        let class = match change.change_type {
            DiffType::Addition => "addition",
            DiffType::Deletion => "deletion",
            DiffType::Modification => "modification",
            DiffType::Unchanged => "unchanged",
        };

        let old_num = change
            .old_line_number
            .map_or("   ".to_string(), |n| format!("{n:3}"));
        let new_num = change
            .new_line_number
            .map_or("   ".to_string(), |n| format!("{n:3}"));
        let content = change
            .new_line
            .as_ref()
            .or(change.old_line.as_ref())
            .unwrap_or(&String::new())
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");

        writeln!(
            output,
            r#"        <div class="diff-line {class}">
            <span class="line-number">{old_num}:{new_num}</span>{content}
        </div>"#
        )
        .unwrap();
    }
}

/// Main diff display interface
pub struct DiffDisplay {
    colored_formatter: ColoredTerminalFormatter,
    side_by_side_formatter: SideBySideFormatter,
    unified_formatter: UnifiedFormatter,
    html_formatter: HtmlFormatter,
}

impl DiffDisplay {
    /// Create a new diff display instance
    #[must_use]
    pub const fn new() -> Self {
        Self {
            colored_formatter: ColoredTerminalFormatter,
            side_by_side_formatter: SideBySideFormatter,
            unified_formatter: UnifiedFormatter,
            html_formatter: HtmlFormatter,
        }
    }

    /// Format diff with colored terminal output
    #[must_use]
    pub fn format_colored(&self, diff: &DiffResult, options: &DisplayOptions) -> String {
        self.colored_formatter.format(diff, options)
    }

    /// Format diff with side-by-side layout
    #[must_use]
    pub fn format_side_by_side(&self, diff: &DiffResult, options: &DisplayOptions) -> String {
        self.side_by_side_formatter.format(diff, options)
    }

    /// Format diff in unified format (Git-style)
    #[must_use]
    pub fn format_unified(&self, diff: &DiffResult, options: &DisplayOptions) -> String {
        self.unified_formatter.format(diff, options)
    }

    /// Format diff as HTML report
    #[must_use]
    pub fn format_html(&self, diff: &DiffResult, options: &DisplayOptions) -> String {
        self.html_formatter.format(diff, options)
    }
}

impl Default for DiffDisplay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::{ChangeComplexity, DiffSummary, RiskLevel};
    use crate::diff::{DiffOptions, HierarchicalDiff, SemanticDiff, TextDiff};

    fn create_test_diff() -> DiffResult {
        DiffResult {
            text_diff: TextDiff {
                changes: vec![
                    DiffChange {
                        change_type: DiffType::Unchanged,
                        old_line: Some("interface GigabitEthernet0/1".to_string()),
                        new_line: Some("interface GigabitEthernet0/1".to_string()),
                        old_line_number: Some(1),
                        new_line_number: Some(1),
                        context: None,
                    },
                    DiffChange {
                        change_type: DiffType::Deletion,
                        old_line: Some(" ip address 192.168.1.1 255.255.255.0".to_string()),
                        new_line: None,
                        old_line_number: Some(2),
                        new_line_number: None,
                        context: None,
                    },
                    DiffChange {
                        change_type: DiffType::Addition,
                        old_line: None,
                        new_line: Some(" ip address 192.168.1.2 255.255.255.0".to_string()),
                        old_line_number: None,
                        new_line_number: Some(2),
                        context: None,
                    },
                    DiffChange {
                        change_type: DiffType::Unchanged,
                        old_line: Some(" no shutdown".to_string()),
                        new_line: Some(" no shutdown".to_string()),
                        old_line_number: Some(3),
                        new_line_number: Some(3),
                        context: None,
                    },
                ],
                additions: 1,
                deletions: 1,
                modifications: 0,
                context_lines: 3,
            },
            hierarchical_diff: HierarchicalDiff {
                sections: vec![],
                structure_changes: vec![],
                path_changes: std::collections::HashMap::new(),
            },
            semantic_diff: SemanticDiff {
                functional_changes: vec![],
                impact_analysis: vec![],
                change_groups: vec![],
            },
            summary: DiffSummary {
                total_changes: 2,
                additions: 1,
                deletions: 1,
                modifications: 0,
                sections_affected: 1,
                highest_risk: RiskLevel::Medium,
                complexity: ChangeComplexity::Simple,
            },
            options: DiffOptions::default(),
        }
    }

    #[test]
    fn test_colored_terminal_formatter() {
        let diff = create_test_diff();
        let display = DiffDisplay::new();
        let options = DisplayOptions::default();

        let output = display.format_colored(&diff, &options);

        assert!(output.contains("Diff Summary"));
        assert!(output.contains("Total changes: 2"));
        assert!(output.contains("192.168.1.1"));
        assert!(output.contains("192.168.1.2"));
    }

    #[test]
    fn test_side_by_side_formatter() {
        let diff = create_test_diff();
        let display = DiffDisplay::new();
        let options = DisplayOptions::default();

        let output = display.format_side_by_side(&diff, &options);

        assert!(output.contains("Side-by-Side Diff"));
        assert!(output.contains("Old Configuration"));
        assert!(output.contains("New Configuration"));
    }

    #[test]
    fn test_unified_formatter() {
        let diff = create_test_diff();
        let display = DiffDisplay::new();
        let options = DisplayOptions::default();

        let output = display.format_unified(&diff, &options);

        assert!(output.contains("--- old"));
        assert!(output.contains("+++ new"));
        assert!(output.contains("@@"));
        assert!(output.contains('-'));
        assert!(output.contains('+'));
    }

    #[test]
    fn test_html_formatter() {
        let diff = create_test_diff();
        let display = DiffDisplay::new();
        let options = DisplayOptions::default();

        let output = display.format_html(&diff, &options);

        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("Configuration Diff Report"));
        assert!(output.contains("Total Changes"));
        // Test for the summary statistics instead since the HTML formatter compacts unchanged lines
        assert!(output.contains("Total Changes:</strong> 2"));
        assert!(output.contains("Additions:</strong>"));
        assert!(output.contains("Deletions:</strong>"));
    }

    #[test]
    fn test_display_options() {
        let options = DisplayOptions {
            use_colors: false,
            show_line_numbers: false,
            show_context: false,
            ..Default::default()
        };

        assert!(!options.use_colors);
        assert!(!options.show_line_numbers);
        assert!(!options.show_context);
    }
}
