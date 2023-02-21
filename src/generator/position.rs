//! Position definition

use time::OffsetDateTime;
use geo::geometry::Point;

/// Raw version of a recorded position
pub struct RawPosition {
    pub coordinates: Point,
    pub time: OffsetDateTime,
    pub speed: Option<f64>,
    pub precision: Option<f32>,
    pub altitude: Option<f32>,
}

impl RawPosition {

    pub fn basic(coordinates: Point, time: OffsetDateTime) -> Self {
        Self {
            coordinates,
            time,
            speed: None,
            precision: None,
            altitude: None
        }
    }
}
