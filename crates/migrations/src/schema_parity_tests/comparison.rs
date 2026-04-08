use std::collections::HashMap;

use super::types::{DifferenceType, SchemaDifference};

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
