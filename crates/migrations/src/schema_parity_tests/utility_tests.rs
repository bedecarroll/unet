use sea_orm::Database;
use std::collections::HashMap;

use super::comparison::compare_schemas;
use super::database::extract_create_table_statements;
use super::formatting::format_schema_differences;

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
