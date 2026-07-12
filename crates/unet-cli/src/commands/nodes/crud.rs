/// CRUD operations for node management
// Re-export functions from individual modules
pub use crate::commands::nodes::add::add_node;
pub use crate::commands::nodes::delete::delete_node;
pub use crate::commands::nodes::list::list_nodes;
pub use crate::commands::nodes::show::show_node;
pub use crate::commands::nodes::update::update_node;

#[cfg(test)]
mod tests {
    //! Tests are organized into focused modules by functionality.

    #[cfg(test)]
    mod args_tests {
        include!("crud/args_tests.rs");
    }

    #[cfg(test)]
    mod data_parsing_tests {
        include!("crud/data_parsing_tests.rs");
    }

    #[cfg(test)]
    mod query_operations_tests {
        include!("crud/query_operations_tests.rs");
    }

    #[cfg(test)]
    mod crud_business_logic_tests {
        include!("crud_business_logic_tests.rs");
    }
}
