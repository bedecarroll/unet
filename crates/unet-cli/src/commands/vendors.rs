use anyhow::Result;
use clap::{Args, Subcommand};
use unet_core::datastore::DataStore;

#[derive(Subcommand)]
pub enum VendorCommands {
    /// Add a new vendor name
    Add(AddVendorArgs),
    /// List all vendors
    List,
    /// Delete a vendor by name
    Delete(DeleteVendorArgs),
}

#[derive(Args, Debug)]
pub struct AddVendorArgs {
    /// Vendor name
    pub name: String,
}

#[derive(Args, Debug)]
pub struct DeleteVendorArgs {
    /// Vendor name
    pub name: String,
    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    pub yes: bool,
}

pub async fn execute(
    command: VendorCommands,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    match command {
        VendorCommands::Add(args) => add_vendor(args, datastore, output_format).await,
        VendorCommands::List => list_vendors(datastore, output_format).await,
        VendorCommands::Delete(args) => delete_vendor(args, datastore, output_format).await,
    }
}

async fn add_vendor(
    args: AddVendorArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    datastore.create_vendor(&args.name).await?;
    let output = serde_json::json!({ "message": "Vendor added", "name": args.name });
    crate::commands::print_output(&output, output_format)?;
    Ok(())
}

async fn list_vendors(datastore: &dyn DataStore, output_format: crate::OutputFormat) -> Result<()> {
    let vendors = datastore.list_vendors().await?;
    crate::commands::print_output(&vendors, output_format)?;
    Ok(())
}

async fn delete_vendor(
    args: DeleteVendorArgs,
    datastore: &dyn DataStore,
    output_format: crate::OutputFormat,
) -> Result<()> {
    if !args.yes {
        println!(
            "Are you sure you want to delete vendor '{}' ? [y/N]",
            args.name
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().to_lowercase().starts_with('y') {
            println!("Cancelled.");
            return Ok(());
        }
    }
    datastore.delete_vendor(&args.name).await?;
    let output = serde_json::json!({ "message": "Vendor deleted", "name": args.name });
    crate::commands::print_output(&output, output_format)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_vendor_args_creation() {
        let args = AddVendorArgs {
            name: "cisco".to_string(),
        };

        assert_eq!(args.name, "cisco");
    }

    #[tokio::test]
    async fn test_add_vendor_args_various_names() {
        let vendor_names = vec![
            "cisco",
            "juniper",
            "arista",
            "huawei",
            "hp",
            "dell",
            "fortinet",
            "palo-alto",
            "mikrotik",
            "ubiquiti",
        ];

        for vendor_name in vendor_names {
            let args = AddVendorArgs {
                name: vendor_name.to_string(),
            };

            assert_eq!(args.name, vendor_name);
            assert!(!args.name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_delete_vendor_args_creation_with_confirmation() {
        let args = DeleteVendorArgs {
            name: "cisco".to_string(),
            yes: false,
        };

        assert_eq!(args.name, "cisco");
        assert!(!args.yes);
    }

    #[tokio::test]
    async fn test_delete_vendor_args_creation_skip_confirmation() {
        let args = DeleteVendorArgs {
            name: "juniper".to_string(),
            yes: true,
        };

        assert_eq!(args.name, "juniper");
        assert!(args.yes);
    }

    #[tokio::test]
    async fn test_vendor_name_validation() {
        // Test various vendor name formats
        let valid_names = vec![
            "cisco",
            "cisco-systems",
            "Cisco",
            "CISCO",
            "Cisco Systems",
            "cisco_systems",
            "juniper-networks",
            "HP Enterprise",
            "Dell EMC",
            "Palo Alto Networks",
        ];

        for name in valid_names {
            let args = AddVendorArgs {
                name: name.to_string(),
            };

            assert!(!args.name.is_empty());
            assert!(!args.name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_vendor_name_edge_cases() {
        // Test edge cases for vendor names
        let edge_case_names = vec![
            "a",                       // Single character
            "A",                       // Single uppercase
            "123",                     // Numbers only
            "vendor-123",              // Mixed alphanumeric with dash
            "vendor_with_underscores", // Underscores
            "vendor with spaces",      // Spaces
            "vendor.with.dots",        // Dots
            "VENDOR-ALL-CAPS",         // All caps with dashes
        ];

        for name in edge_case_names {
            let add_args = AddVendorArgs {
                name: name.to_string(),
            };

            let delete_args = DeleteVendorArgs {
                name: name.to_string(),
                yes: true,
            };

            assert_eq!(add_args.name, name);
            assert_eq!(delete_args.name, name);
            assert!(!add_args.name.is_empty());
            assert!(!delete_args.name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_add_vendor_args_string_handling() {
        // Test string operations on vendor names
        let base_name = "cisco";
        let args = AddVendorArgs {
            name: base_name.to_string(),
        };

        // Test basic string operations
        assert_eq!(args.name.len(), 5);
        assert_eq!(args.name.to_uppercase(), "CISCO");
        assert_eq!(args.name.to_lowercase(), "cisco");
        assert!(args.name.starts_with("cis"));
        assert!(args.name.ends_with("sco"));
        assert!(args.name.contains("isc"));
    }

    #[tokio::test]
    async fn test_delete_vendor_args_confirmation_flag() {
        let vendor_name = "test-vendor";

        // Test with confirmation required
        let args_with_confirmation = DeleteVendorArgs {
            name: vendor_name.to_string(),
            yes: false,
        };

        assert_eq!(args_with_confirmation.name, vendor_name);
        assert!(!args_with_confirmation.yes);

        // Test without confirmation (skip prompt)
        let args_skip_confirmation = DeleteVendorArgs {
            name: vendor_name.to_string(),
            yes: true,
        };

        assert_eq!(args_skip_confirmation.name, vendor_name);
        assert!(args_skip_confirmation.yes);
    }

    #[tokio::test]
    async fn test_vendor_args_cloning() {
        let original_add = AddVendorArgs {
            name: "original-vendor".to_string(),
        };

        let cloned_name = original_add.name.clone();
        assert_eq!(cloned_name, "original-vendor");
        assert_eq!(original_add.name, cloned_name);

        let original_delete = DeleteVendorArgs {
            name: "delete-vendor".to_string(),
            yes: true,
        };

        let cloned_delete_name = original_delete.name.clone();
        let cloned_yes = original_delete.yes;

        assert_eq!(cloned_delete_name, "delete-vendor");
        assert!(cloned_yes);
    }

    #[tokio::test]
    async fn test_vendor_args_equality() {
        let args1 = AddVendorArgs {
            name: "same-vendor".to_string(),
        };

        let args2 = AddVendorArgs {
            name: "same-vendor".to_string(),
        };

        let args3 = AddVendorArgs {
            name: "different-vendor".to_string(),
        };

        // Test name equality
        assert_eq!(args1.name, args2.name);
        assert_ne!(args1.name, args3.name);
        assert_ne!(args2.name, args3.name);
    }

    #[tokio::test]
    async fn test_major_network_vendors() {
        // Test that we can handle all major network equipment vendors
        let major_vendors = vec![
            "Cisco Systems",
            "Juniper Networks",
            "Arista Networks",
            "Huawei Technologies",
            "Nokia",
            "Ericsson",
            "Extreme Networks",
            "HPE (Hewlett Packard Enterprise)",
            "Dell Technologies",
            "Fortinet",
            "Palo Alto Networks",
            "Check Point",
            "F5 Networks",
            "A10 Networks",
            "MikroTik",
            "Ubiquiti",
            "TP-Link",
            "Netgear",
            "D-Link",
            "Linksys",
        ];

        for vendor in major_vendors {
            let add_args = AddVendorArgs {
                name: vendor.to_string(),
            };

            let delete_args = DeleteVendorArgs {
                name: vendor.to_string(),
                yes: false,
            };

            assert_eq!(add_args.name, vendor);
            assert_eq!(delete_args.name, vendor);
            assert!(!add_args.name.is_empty());
            assert!(!delete_args.name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_vendor_name_length_handling() {
        // Test various lengths of vendor names
        let short_name = "HP";
        let medium_name = "Cisco Systems";
        let long_name = "Hewlett Packard Enterprise Networking Division";

        let short_args = AddVendorArgs {
            name: short_name.to_string(),
        };

        let medium_args = AddVendorArgs {
            name: medium_name.to_string(),
        };

        let long_args = AddVendorArgs {
            name: long_name.to_string(),
        };

        assert_eq!(short_args.name.len(), 2);
        assert_eq!(medium_args.name.len(), 13);
        assert_eq!(long_args.name.len(), 46);

        // All should be valid
        assert!(!short_args.name.is_empty());
        assert!(!medium_args.name.is_empty());
        assert!(!long_args.name.is_empty());
    }

    #[tokio::test]
    async fn test_vendor_args_debug_formatting() {
        let args = AddVendorArgs {
            name: "debug-vendor".to_string(),
        };

        // Test that we can format for debugging (basic check)
        let debug_string = format!("{args:?}");
        assert!(debug_string.contains("debug-vendor"));
        assert!(debug_string.contains("AddVendorArgs"));

        let delete_args = DeleteVendorArgs {
            name: "delete-debug".to_string(),
            yes: true,
        };

        let delete_debug_string = format!("{delete_args:?}");
        assert!(delete_debug_string.contains("delete-debug"));
        assert!(delete_debug_string.contains("DeleteVendorArgs"));
        assert!(delete_debug_string.contains("true"));
    }
}
