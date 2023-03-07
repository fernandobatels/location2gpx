//! Position definition

use geo::geometry::Point;
use time::OffsetDateTime;

/// Raw version of a recorded position
pub struct RawPosition {
    pub coordinates: Point,
    pub time: OffsetDateTime,
    /// in m/s
    pub speed: Option<f64>,
    /// in m
    pub precision: Option<f64>,
    /// in m
    pub altitude: Option<f64>,
}

impl RawPosition {
    pub fn basic(coordinates: Point, time: OffsetDateTime) -> Self {
        Self {
            coordinates,
            time,
            speed: None,
            precision: None,
            altitude: None,
        }
    }
}

/// Position with device and other context datas
pub struct DevicePosition {
    /// Device unique ID
    pub device_id: String,
    pub pos: RawPosition,
    /// Route numer or name
    pub route_name: Option<String>,
    /// Tracker app or software
    pub tracker: Option<String>,
}

impl DevicePosition {
    pub fn basic(device_id: String, coordinates: Point, time: OffsetDateTime) -> Self {
        Self {
            device_id,
            pos: RawPosition::basic(coordinates, time),
            route_name: None,
            tracker: None,
        }
    }
}
