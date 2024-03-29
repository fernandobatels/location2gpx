//! location2gpx - GPX generator from many location sources

mod generator;
pub mod sources;

pub use generator::gpx::GpxGenerator;
pub use generator::position::{DevicePosition, RawPosition};
pub use generator::tracker::{SourceToTracks, TrackSegmentOptions, Tracker};
pub use sources::{FieldsConfiguration, PositionsSource};
