//! Tests to validate schema parity between migrations and entities
//!
//! This test ensures that the schema created by running migrations matches
//! exactly the schema created by using `SeaORM` entity definitions. This prevents
//! runtime "no such column" errors caused by subtle schema differences like:
//! - Column type differences (TEXT vs varchar)
//! - Default value differences
//! - Constraint differences
//!
//! Following TDD: This test is written FIRST and should FAIL initially,
//! demonstrating the schema mismatch problem described in `migration_problem.md`

use sea_orm::{ConnectionTrait, Database, DatabaseBackend, Schema};
use sea_orm_migration::MigratorTrait;
use std::collections::HashMap;
use std::fmt::Write;

use crate::Migrator;

/// Test that migration-created schema exactly matches entity-created schema
///
/// This test validates that database schemas created by running migrations
/// are identical to schemas created from SeaORM entity definitions.
///
/// EXPECTED RESULT: This test currently FAILS due to known schema drift issues:
/// - Migrations use DEFAULT values that entities don't have
/// - Entities include foreign key constraints that migrations don't have
/// - See migration_problem.md for detailed analysis and fix procedures
#[tokio::test]
async fn test_migration_entity_schema_parity_detects_drift() {
    let differences = compare_migration_vs_entity_schemas()
        .await
        .expect("Failed to compare schemas");

    // Filter out expected differences (seaql_migrations table is expected)
    let unexpected_differences: Vec<_> = differences
        .iter()
        .filter(|diff| {
            // Skip the expected seaql_migrations table difference
            !(diff.difference_type == DifferenceType::TableMissing
                && diff.table_name == "seaql_migrations")
        })
        .collect();

    // Test fails if ANY unexpected differences are found
    assert!(
        unexpected_differences.is_empty(),
        "Schema parity validation FAILED! Found {} unexpected differences between migration and entity schemas:\n{}",
        unexpected_differences.len(),
        format_schema_differences(&differences)
    );
}

/// Compare schemas created by migrations vs entities and return differences
async fn compare_migration_vs_entity_schemas()
-> Result<Vec<SchemaDifference>, Box<dyn std::error::Error>> {
    // Create two temporary in-memory SQLite databases
    let migration_db = Database::connect("sqlite::memory:").await?;
    let entity_db = Database::connect("sqlite::memory:").await?;

    // Apply migrations to first database
    Migrator::up(&migration_db, None).await?;

    // Apply entity-based schema to second database
    create_schema_from_entities(&entity_db).await?;

    // Extract CREATE TABLE statements from both databases
    let migration_schema = extract_create_table_statements(&migration_db).await?;
    let entity_schema = extract_create_table_statements(&entity_db).await?;

    // Compare schemas and return differences
    compare_schemas(&migration_schema, &entity_schema)
}

/// Create database schema using entity definitions (not migrations)
async fn create_schema_from_entities(
    connection: &impl ConnectionTrait,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema = Schema::new(DatabaseBackend::Sqlite);

    // Create tables for all entities that have corresponding migrations
    // TODO: This will fail initially because we haven't imported entity modules yet

    // Create locations table
    let stmt = schema.create_table_from_entity(unet_core::entities::locations::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    // Create nodes table
    let stmt = schema.create_table_from_entity(unet_core::entities::nodes::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    // Create links table
    let stmt = schema.create_table_from_entity(unet_core::entities::links::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    // Create derived state tables (interface_status, node_status, polling_tasks)
    let stmt = schema.create_table_from_entity(unet_core::entities::interface_status::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    let stmt = schema.create_table_from_entity(unet_core::entities::node_status::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    let stmt = schema.create_table_from_entity(unet_core::entities::polling_tasks::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    // Create vendors table
    let stmt = schema.create_table_from_entity(unet_core::entities::vendors::Entity);
    connection
        .execute(connection.get_database_backend().build(&stmt))
        .await?;

    Ok(())
}

/// Extract CREATE TABLE statements from a `SQLite` database
async fn extract_create_table_statements(
    connection: &impl ConnectionTrait,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    use sea_orm::Statement;

    // Query SQLite's sqlite_master table to get CREATE TABLE statements
    let query = Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name".to_string()
    );

    let raw_result = connection.query_all(query).await?;
    let mut statements = HashMap::new();

    for row in raw_result {
        let table_name: String = row.try_get("", "name")?;
        let create_sql: String = row.try_get("", "sql")?;

        statements.insert(table_name, normalize_create_table_sql(&create_sql));
    }

    Ok(statements)
}

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

/// Compare two schema maps and return differences
fn compare_schemas(
    migration_schema: &HashMap<String, String>,
    entity_schema: &HashMap<String, String>,
) -> Result<Vec<SchemaDifference>, Box<dyn std::error::Error>> {
    let mut differences = Vec::new();

    // Get all table names from both schemas
    let mut all_tables = std::collections::HashSet::new();
    all_tables.extend(migration_schema.keys());
    all_tables.extend(entity_schema.keys());

    for table_name in all_tables {
        match (
            migration_schema.get(table_name),
            entity_schema.get(table_name),
        ) {
            (Some(migration_sql), Some(entity_sql)) => {
                // Both schemas have the table - compare SQL
                if migration_sql != entity_sql {
                    differences.push(SchemaDifference {
                        table_name: table_name.clone(),
                        difference_type: DifferenceType::WholeTableDifference,
                        migration_value: migration_sql.clone(),
                        entity_value: entity_sql.clone(),
                        column_name: None,
                    });

                    // Try to identify specific column differences
                    let column_diffs =
                        compare_table_columns(table_name, migration_sql, entity_sql)?;
                    differences.extend(column_diffs);
                }
            }
            (Some(migration_sql), None) => {
                // Table only exists in migration schema
                differences.push(SchemaDifference {
                    table_name: table_name.clone(),
                    difference_type: DifferenceType::TableMissing,
                    migration_value: migration_sql.clone(),
                    entity_value: "TABLE NOT FOUND".to_string(),
                    column_name: None,
                });
            }
            (None, Some(entity_sql)) => {
                // Table only exists in entity schema
                differences.push(SchemaDifference {
                    table_name: table_name.clone(),
                    difference_type: DifferenceType::TableMissing,
                    migration_value: "TABLE NOT FOUND".to_string(),
                    entity_value: entity_sql.clone(),
                    column_name: None,
                });
            }
            (None, None) => {
                // This should never happen given our logic above
                unreachable!("Table name appeared in set but not in either schema");
            }
        }
    }

    Ok(differences)
}

/// Compare column definitions between two CREATE TABLE statements
fn compare_table_columns(
    table_name: &str,
    migration_sql: &str,
    entity_sql: &str,
) -> Result<Vec<SchemaDifference>, Box<dyn std::error::Error>> {
    let mut differences = Vec::new();

    // Extract column definitions from CREATE TABLE statements
    let migration_columns = extract_column_definitions(migration_sql)?;
    let entity_columns = extract_column_definitions(entity_sql)?;

    // Compare each column
    for (column_name, migration_def) in &migration_columns {
        if let Some(entity_def) = entity_columns.get(column_name) {
            if migration_def != entity_def {
                // Identify specific type of difference
                let diff_type = if migration_def.contains("text") && entity_def.contains("varchar")
                {
                    DifferenceType::ColumnType
                } else if migration_def.contains("default") != entity_def.contains("default") {
                    DifferenceType::DefaultValue
                } else {
                    DifferenceType::Constraint
                };

                differences.push(SchemaDifference {
                    table_name: table_name.to_string(),
                    difference_type: diff_type,
                    migration_value: migration_def.clone(),
                    entity_value: entity_def.clone(),
                    column_name: Some(column_name.clone()),
                });
            }
        } else {
            differences.push(SchemaDifference {
                table_name: table_name.to_string(),
                difference_type: DifferenceType::ColumnMissing,
                migration_value: migration_def.clone(),
                entity_value: "COLUMN NOT FOUND".to_string(),
                column_name: Some(column_name.clone()),
            });
        }
    }

    // Check for columns that exist in entity but not migration
    for (column_name, entity_def) in &entity_columns {
        if !migration_columns.contains_key(column_name) {
            differences.push(SchemaDifference {
                table_name: table_name.to_string(),
                difference_type: DifferenceType::ColumnMissing,
                migration_value: "COLUMN NOT FOUND".to_string(),
                entity_value: entity_def.clone(),
                column_name: Some(column_name.clone()),
            });
        }
    }

    Ok(differences)
}

/// Extract column definitions from a CREATE TABLE statement
/// Returns a map of `column_name` -> `column_definition`
fn extract_column_definitions(
    create_sql: &str,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut columns = HashMap::new();

    // Find the content between the parentheses
    let start = create_sql
        .find('(')
        .ok_or("No opening parenthesis found in CREATE TABLE")?;
    let end = create_sql
        .rfind(')')
        .ok_or("No closing parenthesis found in CREATE TABLE")?;

    let columns_section = &create_sql[start + 1..end];

    // Split by commas, but be careful about commas inside parentheses or quotes
    let mut column_parts = Vec::new();
    let mut current_part = String::new();
    let mut paren_depth = 0;
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for ch in columns_section.chars() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
                current_part.push(ch);
            }
            ch if in_quotes && ch == quote_char => {
                in_quotes = false;
                current_part.push(ch);
            }
            '(' if !in_quotes => {
                paren_depth += 1;
                current_part.push(ch);
            }
            ')' if !in_quotes => {
                paren_depth -= 1;
                current_part.push(ch);
            }
            ',' if !in_quotes && paren_depth == 0 => {
                column_parts.push(current_part.trim().to_string());
                current_part.clear();
            }
            _ => {
                current_part.push(ch);
            }
        }
    }

    if !current_part.trim().is_empty() {
        column_parts.push(current_part.trim().to_string());
    }

    // Extract column names and definitions
    for part in column_parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Skip constraint definitions (they don't start with column names)
        if trimmed.starts_with("primary key")
            || trimmed.starts_with("foreign key")
            || trimmed.starts_with("unique")
            || trimmed.starts_with("check")
        {
            continue;
        }

        // Extract column name (first word)
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if let Some(column_name) = parts.first() {
            columns.insert((*column_name).to_string(), trimmed.to_string());
        }
    }

    Ok(columns)
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

/// Represents a difference between migration and entity schemas
#[derive(Debug, Clone)]
pub struct SchemaDifference {
    pub table_name: String,
    pub difference_type: DifferenceType,
    pub migration_value: String,
    pub entity_value: String,
    pub column_name: Option<String>,
}

/// Types of schema differences that can be detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DifferenceType {
    TableMissing,
    ColumnType,
    DefaultValue,
    Constraint,
    ColumnMissing,
    WholeTableDifference,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test extract_create_table_statements function with empty database
    #[tokio::test]
    async fn test_extract_create_table_statements_empty_database() {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        let result = extract_create_table_statements(&db).await.unwrap();
        assert!(result.is_empty(), "Empty database should return no tables");
    }

    /// Test `compare_schemas` function with identical schemas
    #[test]
    fn test_compare_schemas_identical() {
        let mut schema1 = HashMap::new();
        let mut schema2 = HashMap::new();

        schema1.insert(
            "test_table".to_string(),
            "create table test_table (id varchar)".to_string(),
        );
        schema2.insert(
            "test_table".to_string(),
            "create table test_table (id varchar)".to_string(),
        );

        let differences = compare_schemas(&schema1, &schema2).unwrap();
        assert!(
            differences.is_empty(),
            "Identical schemas should have no differences"
        );
    }

    /// Test `format_schema_differences` function with no differences
    #[test]
    fn test_format_schema_differences_empty() {
        let differences = vec![];
        let result = format_schema_differences(&differences);
        assert_eq!(result, "No differences found");
    }
}
