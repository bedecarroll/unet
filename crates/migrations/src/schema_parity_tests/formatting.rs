use std::collections::HashMap;
use std::fmt::Write;

use super::types::{DifferenceType, SchemaDifference};

/// Normalize CREATE TABLE SQL for consistent comparison
/// Removes formatting differences that don't affect schema functionality
fn normalize_create_table_sql(sql: &str) -> String {
    sql
        // Remove extra whitespace and newlines
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        // Convert to lowercase for case-insensitive comparison
        .to_lowercase()
        // Remove quotes around table and column names for consistent comparison
        .replace(['"', '`'], "")
}

/// Format schema differences for human-readable error messages
fn format_schema_differences(differences: &[SchemaDifference]) -> String {
    if differences.is_empty() {
        return "No differences found".to_string();
    }

    let mut output = String::new();
    output.push_str("\n========== SCHEMA PARITY FAILURES ==========\n");

    // Group differences by table
    let mut by_table: HashMap<String, Vec<&SchemaDifference>> = HashMap::new();
    for diff in differences {
        by_table
            .entry(diff.table_name.clone())
            .or_default()
            .push(diff);
    }

    for (table_name, table_diffs) in by_table {
        writeln!(output, "\n📋 TABLE: {table_name}").unwrap();
        output.push_str("─".repeat(50).as_str());
        output.push('\n');

        for diff in table_diffs {
            match &diff.difference_type {
                DifferenceType::TableMissing => {
                    output.push_str("❌ TABLE MISSING\n");
                    if diff.migration_value == "TABLE NOT FOUND" {
                        output.push_str("   ➤ Table exists in entities but not in migrations\n");
                        writeln!(output, "   📄 Entity Schema: {}", diff.entity_value).unwrap();
                    } else {
                        output.push_str("   ➤ Table exists in migrations but not in entities\n");
                        writeln!(output, "   📄 Migration Schema: {}", diff.migration_value)
                            .unwrap();
                    }
                }
                DifferenceType::WholeTableDifference => {
                    output.push_str("⚠️  COMPLETE TABLE SCHEMA DIFFERENCE\n");
                    writeln!(output, "   📄 Migration: {}", diff.migration_value).unwrap();
                    writeln!(output, "   📄 Entity:    {}", diff.entity_value).unwrap();
                }
                DifferenceType::ColumnType => {
                    let unknown = "unknown".to_string();
                    let col_name = diff.column_name.as_ref().unwrap_or(&unknown);
                    writeln!(output, "🔄 COLUMN TYPE MISMATCH: {col_name}").unwrap();
                    writeln!(output, "   📄 Migration: {}", diff.migration_value).unwrap();
                    writeln!(output, "   📄 Entity:    {}", diff.entity_value).unwrap();
                }
                DifferenceType::DefaultValue => {
                    let unknown = "unknown".to_string();
                    let col_name = diff.column_name.as_ref().unwrap_or(&unknown);
                    writeln!(output, "⚙️  DEFAULT VALUE DIFFERENCE: {col_name}").unwrap();
                    writeln!(output, "   📄 Migration: {}", diff.migration_value).unwrap();
                    writeln!(output, "   📄 Entity:    {}", diff.entity_value).unwrap();
                }
                DifferenceType::Constraint => {
                    let unknown = "unknown".to_string();
                    let col_name = diff.column_name.as_ref().unwrap_or(&unknown);
                    writeln!(output, "🔒 CONSTRAINT DIFFERENCE: {col_name}").unwrap();
                    writeln!(output, "   📄 Migration: {}", diff.migration_value).unwrap();
                    writeln!(output, "   📄 Entity:    {}", diff.entity_value).unwrap();
                }
                DifferenceType::ColumnMissing => {
                    let unknown = "unknown".to_string();
                    let col_name = diff.column_name.as_ref().unwrap_or(&unknown);
                    writeln!(output, "❌ MISSING COLUMN: {col_name}").unwrap();
                    if diff.migration_value == "COLUMN NOT FOUND" {
                        output.push_str("   ➤ Column exists in entities but not in migrations\n");
                        writeln!(output, "   📄 Entity Definition: {}", diff.entity_value).unwrap();
                    } else {
                        output.push_str("   ➤ Column exists in migrations but not in entities\n");
                        writeln!(
                            output,
                            "   📄 Migration Definition: {}",
                            diff.migration_value
                        )
                        .unwrap();
                    }
                }
            }
            output.push('\n');
        }
    }

    output.push_str("========== END SCHEMA FAILURES ==========\n");
    output.push_str("🚨 CRITICAL: These differences can cause runtime 'no such column' errors!\n");
    output.push_str("🔧 ACTION REQUIRED: Fix migrations to match entity definitions exactly.\n");
    output.push_str("📖 See migration_problem.md for detailed analysis and solution steps.\n");

    output
}
