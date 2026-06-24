//! Configuration slicing.

use crate::parser::MatchSpec;
use clap::ValueEnum;
use tracing::info;

/// Supported input formats for configuration slicing.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum Vendor {
    /// Detect the input format from the config text.
    Autodetect,
    /// Brace-delimited Junos-style configuration.
    Junos,
    /// Flat, line-oriented configuration such as `set` commands.
    Flat,
}

/// Slice a configuration string using a parsed match expression.
#[must_use]
pub fn slice_text(text: &str, spec: &MatchSpec, vendor: Vendor) -> Vec<String> {
    match resolve_vendor(text, vendor) {
        Vendor::Junos => slice_junos(text, spec),
        Vendor::Flat | Vendor::Autodetect => slice_flat(text, spec),
    }
}

fn resolve_vendor(text: &str, vendor: Vendor) -> Vendor {
    if vendor != Vendor::Autodetect {
        return vendor;
    }

    let resolved = if text
        .lines()
        .take(20)
        .any(|line| line.contains('{') || line.trim_start().starts_with('}'))
    {
        Vendor::Junos
    } else {
        Vendor::Flat
    };

    info!("auto-detected vendor as {resolved:?}");
    resolved
}

fn slice_flat(text: &str, spec: &MatchSpec) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let segments = flat_segments(line);
            if !segments.is_empty() && spec.matches_path_prefix(&segments) {
                Some(line.to_string())
            } else {
                None
            }
        })
        .collect()
}

fn flat_segments(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with('!')
        || trimmed.starts_with("//")
    {
        return Vec::new();
    }

    let normalized = ["set ", "delete ", "activate ", "deactivate "]
        .iter()
        .find_map(|prefix| trimmed.strip_prefix(prefix))
        .unwrap_or(trimmed);

    normalized
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>()
}

fn slice_junos(text: &str, spec: &MatchSpec) -> Vec<String> {
    let mut current_path = Vec::new();
    let mut active_depth = None;
    let mut output = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with('}') {
            if active_depth.is_some_and(|depth| current_path.len() >= depth) {
                output.push(line.to_string());
            }

            if !current_path.is_empty() {
                current_path.pop();
            }

            if active_depth.is_some_and(|depth| current_path.len() < depth) {
                active_depth = None;
            }
            continue;
        }

        let opens_block = trimmed.ends_with('{');
        let Some(segment) = junos_segment(trimmed) else {
            continue;
        };

        let mut candidate_path = current_path.clone();
        candidate_path.push(segment);

        let line_matches = spec.matches_path_prefix(&candidate_path);
        let activates_match = line_matches && candidate_path.len() == spec.depth();
        let inside_match = active_depth
            .is_some_and(|depth| current_path.len() >= depth && candidate_path.len() > depth);

        if activates_match {
            active_depth = Some(candidate_path.len());
            output.push(line.to_string());
        } else if line_matches || inside_match {
            output.push(line.to_string());
        }

        if opens_block {
            current_path.push(candidate_path.pop().unwrap());
        } else if activates_match {
            active_depth = None;
        }
    }

    output
}

fn junos_segment(line: &str) -> Option<String> {
    let segment = line.trim_end_matches('{').trim_end_matches(';').trim();

    if segment.is_empty() {
        None
    } else {
        Some(segment.to_string())
    }
}
