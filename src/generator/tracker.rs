//! Track generator API

use gpx::{Track, TrackSegment, Waypoint};
use super::position::RawPosition;

pub struct Tracker {
    /// Device name, number...
    pub device: String,
    /// Route/Track/Category name, number...
    pub name: String,
    /// Max segment duration in minutes
    pub max_segment_duration: u8,
    /// Data source, eg.: track app
    pub source: Option<String>,
}

impl Tracker {

    /// Start a new tracker instance
    pub fn new(device: String, name: String) -> Self {
        Self {
            device,
            name,
            source: None,
            max_segment_duration: 5 // 5 minutes
        }
    }

    pub fn max_segment(&mut self, max: u8) -> &mut Self {
        self.max_segment_duration = if max < 1 {
            1
        } else {
            max
        };

        self
    }

    pub fn source(&mut self, source: String) -> &mut Self {
        self.source = Some(source);

        self
    }

    /// Build the track with the tracker params
    pub fn build(&self, positions: Vec<&RawPosition>) -> Result<Track, String> {
        let mut track = Track::new();
        track.name = Some(self.name.clone());
        track.description = Some(format!("Tracked by `{}`", self.device.clone()));
        track.source = self.source.clone();

        let mut tseg = TrackSegment::new();

        let mut positions = positions.clone();
        positions.sort_by_key(|p| p.time);
        for poi in positions {

            let mut wp = Waypoint::new(poi.coordinates);
            wp.time = Some(poi.time.into());

            tseg.points.push(wp);
        }

        track.segments.push(tseg);

        Ok(track)
    }
}
