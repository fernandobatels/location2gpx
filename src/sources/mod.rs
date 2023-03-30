//! Positions sources API

use serde::Deserialize;
use time::OffsetDateTime;

use crate::DevicePosition;

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
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct FieldsConfiguration {
    /// Device name or ID
    pub device_id: String,
    pub time: String,
    /// Route name or ID
    pub route: String,
    pub coordinates: String,
    pub speed: String,
    pub elevation: String,
    /// Flip the lat,lng coordinates order
    pub flip_coordinates: bool,
}

impl Default for FieldsConfiguration {
    fn default() -> Self {
        Self {
            device_id: "device".to_string(),
            time: "time".to_string(),
            route: "route".to_string(),
            coordinates: "coordinates".to_string(),
            speed: "speed".to_string(),
            elevation: "elevation".to_string(),
            flip_coordinates: false,
        }
    }
}

#[cfg(feature = "mongo")]
mod mongo;
#[cfg(feature = "mongo")]
pub use mongo::MongoDbSource;

#[cfg(feature = "csv")]
mod csv_file;
#[cfg(feature = "csv")]
pub use csv_file::CsvSource;

#[test]
fn parse_fields() -> Result<(), String> {
    let yaml = "";

    let fb: FieldsConfiguration = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;

    assert_eq!(
        FieldsConfiguration {
            device_id: "device".to_string(),
            time: "time".to_string(),
            route: "route".to_string(),
            coordinates: "coordinates".to_string(),
            speed: "speed".to_string(),
            elevation: "elevation".to_string(),
            flip_coordinates: false,
        },
        fb
    );

    let yaml = "\ndevice_id: dev\ntime: time\ncoordinates: coords";

    let fb: FieldsConfiguration = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;

    assert_eq!(
        FieldsConfiguration {
            device_id: "dev".to_string(),
            time: "time".to_string(),
            route: "route".to_string(),
            coordinates: "coords".to_string(),
            speed: "speed".to_string(),
            elevation: "elevation".to_string(),
            flip_coordinates: false,
        },
        fb
    );

    Ok(())
}
