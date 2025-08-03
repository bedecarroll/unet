//! Node utility methods and custom data operations
//!
//! Contains methods for manipulating custom data and other utility functions
//! for the Node struct.

use super::core::Node;
use serde_json::Value;

impl Node {
    /// Gets a value from `custom_data` by dot-notation path
    ///
    /// # Examples
    /// ```
    /// # use unet_core::models::{Node, DeviceRole, Vendor};
    /// # use serde_json::json;
    /// let mut node = Node::new("test".to_string(), "example.com".to_string(), Vendor::Cisco, DeviceRole::Router);
    /// node.custom_data = json!({"config": {"vlan": 100}});
    ///
    /// let vlan = node.get_custom_data("config.vlan");
    /// assert_eq!(vlan, Some(&json!(100)));
    /// ```
    #[must_use]
    pub fn get_custom_data(&self, path: &str) -> Option<&Value> {
        // Simple dot-notation path traversal
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.custom_data;

        for part in parts {
            if let Value::Object(obj) = current {
                current = obj.get(part)?;
            } else {
                return None;
            }
        }

        Some(current)
    }

    /// Sets a value in `custom_data` by path
    ///
    /// Creates nested objects as needed and converts null `custom_data` to an object.
    ///
    /// # Errors
    /// Returns an error if the path is empty, cannot navigate through non-object,
    /// or cannot set value on non-object.
    ///
    /// # Examples
    /// ```
    /// # use unet_core::models::{Node, DeviceRole, Vendor};
    /// # use serde_json::json;
    /// let mut node = Node::new("test".to_string(), "example.com".to_string(), Vendor::Cisco, DeviceRole::Router);
    ///
    /// node.set_custom_data("config.vlan", json!(100)).unwrap();
    /// assert_eq!(node.get_custom_data("config.vlan"), Some(&json!(100)));
    /// ```
    pub fn set_custom_data(&mut self, path: &str, value: Value) -> Result<(), String> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err("Path cannot be empty".to_string());
        }

        // Initialize custom_data as empty object if null
        if self.custom_data.is_null() {
            self.custom_data = Value::Object(serde_json::Map::new());
        }

        // Navigate to the parent and set the final key
        let mut current = &mut self.custom_data;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - set the value
                if let Value::Object(obj) = current {
                    obj.insert((*part).to_string(), value);
                    return Ok(());
                }
                return Err("Cannot set value on non-object".to_string());
            }
            // Navigate deeper, creating objects as needed
            if let Value::Object(obj) = current {
                let entry = obj
                    .entry((*part).to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));
                current = entry;
            } else {
                return Err("Cannot navigate through non-object".to_string());
            }
        }

        Ok(())
    }
}
