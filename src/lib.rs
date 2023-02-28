//! location2gpx - GPX generator from many location sources

mod generator;
mod sources;

pub use generator::gpx::GpxGenerator;
pub use generator::position::{DevicePosition, RawPosition};
pub use generator::tracker::{SourceToTracks, Tracker};
pub use sources::PositionsSource;
