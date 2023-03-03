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
#[derive(Debug, Clone)]
pub struct FieldsBuilder {
    device_id: String,
    time: String,
    route: String,
    coordinates: String,
    flip_coordinates: bool
}

impl Default for FieldsBuilder {
    fn default() -> Self {
        Self {
            device_id: "device".to_string(),
            time: "time".to_string(),
            route: "route".to_string(),
            coordinates: "coordinates".to_string(),
            flip_coordinates: false
        }
    }
}

impl FieldsBuilder {
    /// Change the device id field name
    pub fn device<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.device_id = name.into();
        self
    }

    /// Change the time field name
    pub fn time<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.time = name.into();
        self
    }

    /// Change the route field name
    pub fn route<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.route = name.into();
        self
    }

    /// Change the coordinates field name
    pub fn coordinates<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.coordinates = name.into();
        self
    }

    /// Flip the lat,lng coordinates order
    pub fn flip_coordinates(&mut self, flip: bool) -> &mut Self {
        self.flip_coordinates = flip;
        self
    }

    pub fn done(&mut self) -> Self {
        self.clone()
    }
}

#[cfg(feature = "mongo")]
mod mongo;

#[cfg(feature = "mongo")]
pub use mongo::MongoDbSource;
