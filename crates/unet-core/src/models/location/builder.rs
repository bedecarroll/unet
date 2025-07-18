//! `LocationBuilder` implementation for creating locations with validation

use super::model::Location;
use serde_json::Value;
use uuid::Uuid;

/// Builder pattern for Location creation with validation
#[derive(Debug, Default)]
pub struct LocationBuilder {
    /// Location ID (optional, will generate UUID if not provided)
    id: Option<Uuid>,
    /// Location name (required)
    name: Option<String>,
    /// Location type (required)
    location_type: Option<String>,
    /// Parent location ID (optional for root locations)
    parent_id: Option<Uuid>,
    /// Parent path for building the full path (optional for root locations)
    parent_path: Option<String>,
    /// Description (optional)
    description: Option<String>,
    /// Address (optional)
    address: Option<String>,
    /// Custom data (optional)
    custom_data: Option<Value>,
}

impl LocationBuilder {
    /// Creates a new location builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the location ID (optional, will generate UUID if not provided)
    #[must_use]
    pub const fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the location name (required)
    #[must_use]
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the location type (required)
    #[must_use]
    pub fn location_type<S: Into<String>>(mut self, location_type: S) -> Self {
        self.location_type = Some(location_type.into());
        self
    }

    /// Sets the parent location ID (optional for root locations)
    #[must_use]
    pub const fn parent_id(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Sets the parent path for building the full path (optional for root locations)
    #[must_use]
    pub fn parent_path<S: Into<String>>(mut self, parent_path: S) -> Self {
        self.parent_path = Some(parent_path.into());
        self
    }

    /// Sets the description (optional)
    #[must_use]
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the address (optional)
    #[must_use]
    pub fn address<S: Into<String>>(mut self, address: S) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Sets custom data (optional)
    #[must_use]
    pub fn custom_data(mut self, custom_data: Value) -> Self {
        self.custom_data = Some(custom_data);
        self
    }

    /// Builds the location with validation
    ///
    /// # Errors
    /// Returns an error if required fields (name, `location_type`) are missing,
    /// or if the created location fails validation.
    pub fn build(self) -> Result<Location, String> {
        let name = self.name.ok_or("Name is required")?;
        let location_type = self.location_type.ok_or("Location type is required")?;

        let path = match self.parent_path {
            Some(parent_path) if !parent_path.is_empty() => {
                format!("{parent_path}/{name}")
            }
            _ => name.clone(),
        };

        let location = Location {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            name,
            location_type,
            parent_id: self.parent_id,
            path,
            description: self.description,
            address: self.address,
            custom_data: self.custom_data.unwrap_or(Value::Null),
        };

        location.validate()?;
        Ok(location)
    }
}
