//! Track generator API

use gpx::Track;
use super::position::RawPosition;

pub struct Tracker {
}

impl Tracker {
    pub fn build(positions: Vec<RawPosition>) -> Result<Track, String> {
        todo!();
    }
}
