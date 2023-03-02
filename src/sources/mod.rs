//! Positions sources API

use crate::DevicePosition;
use time::OffsetDateTime;

/// Position source
pub trait PositionsSource {
    /// Fetch the raw positing during the period
    fn fetch(
        &mut self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<DevicePosition>, String>;
}

/// Fields of source customization
pub struct FieldsBuilder {
    device_id: String,
    time: String,
    coordinates: String
}

impl Default for FieldsBuilder {
    fn default() -> Self {
        Self {
            device_id: "device".to_string(),
            time: "time".to_string(),
            coordinates: "coordinates".to_string()
        }
    }
}

impl FieldsBuilder {
    /// Change the device id field name
    pub fn device<S: Into<String>>(mut self, name: S) -> Self {
        self.device_id = name.into();
        self
    }

    /// Change the time field name
    pub fn time<S: Into<String>>(mut self, name: S) -> Self {
        self.time = name.into();
        self
    }

    /// Change the coordinates field name
    pub fn coordinates<S: Into<String>>(mut self, name: S) -> Self {
        self.coordinates = name.into();
        self
    }
}

#[cfg(feature = "mongo")]
mod mongo;

#[cfg(feature = "mongo")]
pub use mongo::MongoDbSource;
