use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub(super) struct ComparisonEntry {
    pub(crate) scope: String,
    pub(crate) field: String,
    pub(crate) node_a: Value,
    pub(crate) node_b: Value,
    pub(crate) different: bool,
}

#[derive(Serialize)]
pub(super) struct ComparisonSection {
    pub(crate) matches: bool,
    pub(crate) compared_field_count: usize,
    pub(crate) difference_count: usize,
    pub(crate) entries: Vec<ComparisonEntry>,
}

pub(super) struct SectionBuilder {
    diff_only: bool,
    compared_field_count: usize,
    difference_count: usize,
    entries: Vec<ComparisonEntry>,
}

impl SectionBuilder {
    pub(super) const fn new(diff_only: bool) -> Self {
        Self {
            diff_only,
            compared_field_count: 0,
            difference_count: 0,
            entries: Vec::new(),
        }
    }

    pub(super) fn record<T>(&mut self, scope: &str, field: &str, node_a: T, node_b: T)
    where
        T: Serialize,
    {
        let node_a = serde_json::to_value(node_a).unwrap_or(Value::Null);
        let node_b = serde_json::to_value(node_b).unwrap_or(Value::Null);
        let different = node_a != node_b;

        self.compared_field_count += 1;
        if different {
            self.difference_count += 1;
        }

        if !self.diff_only || different {
            self.entries.push(ComparisonEntry {
                scope: scope.to_string(),
                field: field.to_string(),
                node_a,
                node_b,
                different,
            });
        }
    }

    pub(super) fn finish(self) -> ComparisonSection {
        ComparisonSection {
            matches: self.difference_count == 0,
            compared_field_count: self.compared_field_count,
            difference_count: self.difference_count,
            entries: self.entries,
        }
    }
}
