/// Node listing operations
use anyhow::Result;
use unet_core::datastore::DataStore;
use unet_core::prelude::*;

use super::types::ListNodeArgs;

pub async fn list_nodes(
    args: ListNodeArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    let mut filters = Vec::new();

    if let Some(lifecycle) = args.lifecycle {
        filters.push(Filter {
            field: "lifecycle".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(lifecycle),
        });
    }

    if let Some(role) = args.role {
        filters.push(Filter {
            field: "role".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(role),
        });
    }

    if let Some(vendor) = args.vendor {
        filters.push(Filter {
            field: "vendor".to_owned(),
            operation: FilterOperation::Equals,
            value: FilterValue::String(vendor),
        });
    }

    let options = QueryOptions {
        filters,
        sort: vec![Sort {
            field: "name".to_owned(),
            direction: SortDirection::Ascending,
        }],
        pagination: Some(Pagination {
            offset: usize::try_from((args.page - 1) * args.per_page)?,
            limit: usize::try_from(args.per_page)?,
        }),
    };

    let result = datastore.list_nodes(&options).await?;

    crate::commands::print_output(&result, output_format)?;

    Ok(())
}
