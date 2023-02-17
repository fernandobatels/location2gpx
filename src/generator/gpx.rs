//! GPX builder API

use gpx::Gpx;
use super::position::RawPosition;

pub struct GpxBuilder {
}

impl GpxBuilder {
    pub fn build(positions: Vec<RawPosition>) -> Result<Gpx, String> {
        todo!();
    }
}
