//! Track generator API

use std::collections::BTreeMap;

use geo::SimplifyVwIdx;
use gpx::{Track, TrackSegment, Waypoint};
use serde::Deserialize;
use time::{macros::format_description, OffsetDateTime};

use super::position::{DevicePosition, RawPosition};
use crate::PositionsSource;

pub struct Tracker {
    /// Device name, number...
    device: String,
    /// Route/Track/Category name, number...
    name: String,
    /// Data source, eg.: track app
    source: Option<String>,
    segment_confs: TrackSegmentOptions,
}

impl Tracker {
    /// Start a new tracker instance
    pub fn new(device: String, name: String) -> Self {
        Self {
            device,
            name,
            source: None,
            segment_confs: TrackSegmentOptions::default(),
        }
    }

    /// Change the segment confs
    pub fn configure_segments(&mut self, conf: &TrackSegmentOptions) -> &mut Self {
        self.segment_confs = (*conf).clone();

        self
    }

    /// App or other source name of data
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
        let max_time = self.segment_confs.max_duration as f64;
        for poi in positions {
            let key = ((poi.time.unix_timestamp() as f64 / max_time).floor() * max_time) as i64;

            let tseg = segs.entry(key).or_insert_with(|| TrackSegment::new());

            let mut wp = Waypoint::new(poi.coordinates);

            wp.time = Some(poi.time.into());
            wp.elevation = poi.altitude;
            wp.speed = poi.speed;

            tseg.points.push(wp);
        }

        for (_, tseg) in segs {
            if let Some(tol) = self.segment_confs.vw_tolerance {
                let keep = tseg.linestring().simplify_vw_idx(&tol);

                let mut ntseg = TrackSegment::new();

                for ipoint in keep {
                    ntseg.points.push(tseg.points[ipoint].clone());
                }

                track.segments.push(ntseg);
            } else {
                track.segments.push(tseg);
            }
        }

        Ok(track)
    }
}

/// Segments configurations
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(default)]
pub struct TrackSegmentOptions {
    /// Max segment duration in seconds
    pub max_duration: u16,
    /// Tolerance value to simplify with Visvalingam-Whyatt algorithm
    pub vw_tolerance: Option<f64>,
}

impl Default for TrackSegmentOptions {
    fn default() -> Self {
        Self {
            max_duration: 300, // 5 minutes
            vw_tolerance: None,
        }
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
        segment_confs: TrackSegmentOptions,
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

            tracker.configure_segments(&segment_confs);

            let raw = dev_pos.iter().map(|dpos| &dpos.pos).collect();
            let track = tracker.build(raw)?;
            tracks.push(track);
        }

        Ok(tracks)
    }
}

#[test]
fn parse_track_seg_options() -> Result<(), String> {
    let yaml = "\nmax_duration: 300";

    let tso: TrackSegmentOptions = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;

    assert_eq!(
        TrackSegmentOptions {
            max_duration: 300,
            vw_tolerance: None
        },
        tso
    );

    let yaml = "\nmax_duration: 300\nvw_tolerance: 0.001";

    let tso: TrackSegmentOptions = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;

    assert_eq!(
        TrackSegmentOptions {
            max_duration: 300,
            vw_tolerance: Some(0.001)
        },
        tso
    );

    Ok(())
}
