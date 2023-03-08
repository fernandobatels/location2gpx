//! Track generator API

use std::collections::BTreeMap;

use gpx::{Track, TrackSegment, Waypoint};
use time::{macros::format_description, OffsetDateTime};

use super::position::{DevicePosition, RawPosition};
use crate::PositionsSource;

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
            max_segment_duration: 5, // 5 minutes
        }
    }

    pub fn max_segment(&mut self, max: u8) -> &mut Self {
        self.max_segment_duration = if max < 1 { 1 } else { max };

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

        let mut positions = positions.clone();
        positions.sort_by_key(|p| p.time);

        let mut segs: BTreeMap<i64, TrackSegment> = BTreeMap::new();

        // We make small segments of tracks rounding
        // the times to the closest 5min sloot
        for poi in positions {
            let key = ((poi.time.unix_timestamp() as f64 / 300f64).floor() * 300f64) as i64;

            let tseg = segs.entry(key).or_insert_with(|| TrackSegment::new());

            let mut wp = Waypoint::new(poi.coordinates);

            wp.time = Some(poi.time.into());
            wp.elevation = poi.altitude;
            wp.speed = poi.speed;

            tseg.points.push(wp);
        }

        for (_,tseg) in segs {
            track.segments.push(tseg);
        }

        Ok(track)
    }
}

/// Default tracks generator from source
pub struct SourceToTracks {}

impl SourceToTracks {
    /// Run the source and build the tracks
    pub fn build<SU>(
        mut source: SU,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<Track>, String>
    where
        SU: PositionsSource,
    {
        let mut devices: BTreeMap<(String, String), Vec<DevicePosition>> = BTreeMap::new();
        let mut tracks = vec![];
        let route_day_format = format_description!("[year]-[month]-[day]");

        let positions = source.fetch(start, end)?;

        for pos in positions {
            let route = match pos.route_name.clone() {
                Some(ro) => ro,
                None => {
                    let day = pos
                        .pos
                        .time
                        .format(route_day_format)
                        .map_err(|e| e.to_string())?;
                    day
                }
            };
            let key = (pos.device_id.clone(), route);

            let dev = devices.entry(key).or_insert(vec![]);
            dev.push(pos);
        }

        for ((device_id, route_name), dev_pos) in devices {
            let mut tracker = Tracker::new(device_id.clone(), route_name.clone());

            if let Some(trk) = &dev_pos[0].tracker {
                tracker.source(trk.to_string());
            }

            let raw = dev_pos.iter().map(|dpos| &dpos.pos).collect();
            let track = tracker.build(raw)?;
            tracks.push(track);
        }

        Ok(tracks)
    }
}
