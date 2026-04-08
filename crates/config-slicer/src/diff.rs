//! Configuration diffing.

use crate::{MatchSpec, Vendor, slice_text};
use similar::TextDiff;

/// Diff two configurations after slicing them with the same match expression.
#[must_use]
pub fn diff_text(source: &str, target: &str, spec: &MatchSpec, vendor: Vendor) -> String {
    let source_lines = slice_text(source, spec, vendor);
    let target_lines = slice_text(target, spec, vendor);
    let source = normalize_for_diff(&source_lines);
    let target = normalize_for_diff(&target_lines);

    if source == target {
        return String::new();
    }

    TextDiff::from_lines(&source, &target)
        .unified_diff()
        .header("source", "target")
        .to_string()
}

fn normalize_for_diff(lines: &[String]) -> String {
    let mut rendered = lines.join("\n");
    if !rendered.is_empty() {
        rendered.push('\n');
    }
    rendered
}
