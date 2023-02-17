//! Position definition

use chrono::{DateTime, Utc};
use geo::geometry::Point;

pub struct RawPosition {
    coordinates: Point,
    time: DateTime<Utc>,
    speed: Option<f64>,
    precision: Option<f64>,
    altitude: Option<f64>,
}
