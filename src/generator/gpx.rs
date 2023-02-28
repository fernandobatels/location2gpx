//! GPX generator API

use gpx::{Gpx, GpxVersion, Track};

pub struct GpxGenerator {
    pub tracks: Vec<Track>,
}

impl GpxGenerator {
    pub fn empty() -> Self {
        Self { tracks: vec![] }
    }

    pub fn generate(self) -> Result<Gpx, String> {
        let mut gpx: Gpx = Default::default();
        gpx.version = GpxVersion::Gpx11;
        gpx.creator = Some("location2gpx".to_string());
        gpx.tracks = self.tracks;

        Ok(gpx)
    }
}
