//! Tests for `vendors` entity

#[cfg(test)]
mod tests {
    use super::super::super::vendors::*;

    #[test]
    fn test_vendor_model_creation() {
        let vendor = Model {
            name: "Cisco".to_string(),
        };
        assert_eq!(vendor.name, "Cisco");
    }
}
