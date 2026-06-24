/// Represents a difference between migration and entity schemas
#[derive(Debug, Clone)]
pub(super) struct SchemaDifference {
    pub table_name: String,
    pub difference_type: DifferenceType,
    pub migration_value: String,
    pub entity_value: String,
    pub column_name: Option<String>,
}

/// Types of schema differences that can be detected
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DifferenceType {
    TableMissing,
    ColumnType,
    DefaultValue,
    Constraint,
    ColumnMissing,
    WholeTableDifference,
}
