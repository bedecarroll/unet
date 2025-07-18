//! Core Location model implementation

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Physical or logical location in a hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    /// Unique identifier for the location
    pub id: Uuid,
    /// Human-readable name for the location
    pub name: String,
    /// Location type (e.g., "country", "city", "building", "floor", "rack")
    pub location_type: String,
    /// Parent location identifier (None for root locations)
    pub parent_id: Option<Uuid>,
    /// Full path from root (e.g., "USA/California/San Francisco/Building A/Floor 3")
    pub path: String,
    /// Description of the location
    pub description: Option<String>,
    /// Address or coordinates
    pub address: Option<String>,
    /// Extended/custom data as JSON
    pub custom_data: Value,
}

impl Location {
    /// Creates a new root location (no parent)
    #[must_use]
    pub fn new_root(name: String, location_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.clone(),
            location_type,
            parent_id: None,
            path: name,
            description: None,
            address: None,
            custom_data: Value::Null,
        }
    }

    /// Creates a new child location with the given parent
    #[must_use]
    pub fn new_child(name: String, location_type: String, parent_path: &str) -> Self {
        let path = if parent_path.is_empty() {
            name.clone()
        } else {
            format!("{parent_path}/{name}")
        };

        Self {
            id: Uuid::new_v4(),
            name,
            location_type,
            parent_id: None, // Will be set by caller
            path,
            description: None,
            address: None,
            custom_data: Value::Null,
        }
    }

    /// Validates the location configuration
    ///
    /// # Errors
    /// Returns an error if the location name or type is empty, path is inconsistent,
    /// or hierarchy constraints are violated.
    pub fn validate(&self) -> Result<(), String> {
        // Validate name
        if self.name.is_empty() {
            return Err("Location name cannot be empty".to_string());
        }

        // Validate location type
        if self.location_type.is_empty() {
            return Err("Location type cannot be empty".to_string());
        }

        // Validate path consistency
        if self.path.is_empty() {
            return Err("Location path cannot be empty".to_string());
        }

        // For root locations, path should equal name
        if self.parent_id.is_none() && self.path != self.name {
            return Err("Root location path must equal name".to_string());
        }

        // For child locations, path should end with name
        if self.parent_id.is_some() && !self.path.ends_with(&self.name) {
            return Err("Location path must end with location name".to_string());
        }

        Ok(())
    }

    /// Updates the path based on parent path and name
    pub fn update_path(&mut self, parent_path: Option<&str>) {
        match parent_path {
            Some(parent) if !parent.is_empty() => {
                self.path = format!("{}/{}", parent, self.name);
            }
            _ => {
                self.path = self.name.clone();
            }
        }
    }

    /// Gets the depth level in the hierarchy (0 for root)
    #[must_use]
    pub fn get_depth(&self) -> usize {
        if self.path.is_empty() {
            0
        } else {
            self.path.matches('/').count()
        }
    }

    /// Gets all path components as a vector
    #[must_use]
    pub fn get_path_components(&self) -> Vec<&str> {
        if self.path.is_empty() {
            Vec::new()
        } else {
            self.path.split('/').collect()
        }
    }

    /// Checks if this location is an ancestor of another location
    #[must_use]
    pub fn is_ancestor_of(&self, other: &Self) -> bool {
        other.path.starts_with(&format!("{}/", self.path))
            || (self.parent_id.is_none()
                && other.parent_id.is_some()
                && other.path.starts_with(&self.path))
    }

    /// Checks if this location is a descendant of another location
    #[must_use]
    pub fn is_descendant_of(&self, other: &Self) -> bool {
        other.is_ancestor_of(self)
    }

    /// Checks if this location is a direct child of another location
    #[must_use]
    pub fn is_child_of(&self, other: &Self) -> bool {
        self.parent_id == Some(other.id)
    }

    /// Checks if this location is the direct parent of another location
    #[must_use]
    pub fn is_parent_of(&self, other: &Self) -> bool {
        other.is_child_of(self)
    }

    /// Gets a value from `custom_data` by path
    #[must_use]
    pub fn get_custom_data(&self, path: &str) -> Option<&Value> {
        // Simple dot-notation path traversal (same as Node and Link)
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
    /// # Errors
    /// Returns an error if the path is empty, cannot navigate through non-object,
    /// or cannot set value on non-object.
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

    /// Detects circular references in a location hierarchy
    #[must_use]
    pub fn detect_circular_reference(
        locations: &[Self],
        potential_parent_id: Uuid,
        child_id: Uuid,
    ) -> bool {
        if potential_parent_id == child_id {
            return true;
        }

        // Find the potential parent
        let potential_parent = locations.iter().find(|l| l.id == potential_parent_id);
        if let Some(parent) = potential_parent {
            // Check if the child is an ancestor of the potential parent
            let child = locations.iter().find(|l| l.id == child_id);
            if let Some(child_loc) = child {
                return child_loc.is_ancestor_of(parent);
            }
        }

        false
    }

    /// Gets all ancestors of a location
    #[must_use]
    pub fn get_ancestors<'a>(&self, all_locations: &'a [Self]) -> Vec<&'a Self> {
        let mut ancestors = Vec::new();
        let mut current = self;

        while let Some(parent_id) = current.parent_id {
            if let Some(parent) = all_locations.iter().find(|l| l.id == parent_id) {
                ancestors.push(parent);
                current = parent;
            } else {
                break; // Parent not found, stop traversal
            }
        }

        ancestors
    }

    /// Gets all descendants of a location
    #[must_use]
    pub fn get_descendants<'a>(&self, all_locations: &'a [Self]) -> Vec<&'a Self> {
        let mut descendants = Vec::new();
        let mut to_check = vec![self.id];

        while let Some(current_id) = to_check.pop() {
            for location in all_locations {
                if location.parent_id == Some(current_id) {
                    descendants.push(location);
                    to_check.push(location.id);
                }
            }
        }

        descendants
    }

    /// Gets direct children of a location
    #[must_use]
    pub fn get_children<'a>(&self, all_locations: &'a [Self]) -> Vec<&'a Self> {
        all_locations
            .iter()
            .filter(|l| l.parent_id == Some(self.id))
            .collect()
    }
}
