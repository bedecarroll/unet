use super::database::compare_migration_vs_entity_schemas;
use super::formatting::format_schema_differences;
use super::types::DifferenceType;

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
