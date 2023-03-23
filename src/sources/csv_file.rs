//! CSV file source integration

use std::io::Read;

use csv::{Reader, StringRecord};
use geo::geometry::Point;
use time::format_description::well_known;
use time::OffsetDateTime;

use super::{FieldsBuilder, PositionsSource};
use crate::DevicePosition;

/// MongoDB tracks source
pub struct CsvSource<T>
where
    T: Read,
{
    rdr: Reader<T>,
    fields: FieldsBuilder,
}

impl<T> CsvSource<T>
where
    T: Read,
{
    pub fn new(rdr: Reader<T>, fields: Option<FieldsBuilder>) -> Self {
        Self {
            rdr,
            fields: match fields {
                Some(f) => f,
                None => FieldsBuilder::default(),
            },
        }
    }
}

impl<T> PositionsSource for CsvSource<T>
where
    T: Read,
{
    fn fetch(
        &mut self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<DevicePosition>, String> {
        let mut pos = vec![];

        let mut header = self
            .rdr
            .headers()
            .map_err(|e| format!("Failed on read the header: {}", e.to_string()))?
            .clone();
        let header_idx = parse_header(&self.fields, &mut header)?;

        let mut recs = self.rdr.records();
        while let Some(row) = recs.next() {
            let mut rec = row.map_err(|e| format!("Failed on read some row: {}", e.to_string()))?;

            if rec.len() < 3 {
                continue;
            }

            let row_pos = match parse_row(&header_idx, &self.fields, &mut rec) {
                Ok(dpos) => Ok(dpos),
                Err(e) => Err(format!("Error with row {:?}: {}", rec, e)),
            }?;

            if let Some(dpos) = row_pos {
                if start <= dpos.pos.time && dpos.pos.time <= end {
                    pos.push(dpos);
                }
            }
        }

        Ok(pos)
    }
}

/// Field to index map
#[derive(Debug)]
struct FieldsIndex {
    device: usize,
    coordinates: usize,
    time: usize,
    route: Option<usize>,
    speed: Option<usize>,
    elevation: Option<usize>,
}

fn parse_header(fields: &FieldsBuilder, header: &mut StringRecord) -> Result<FieldsIndex, String> {
    header.trim();

    let device = match header
        .iter()
        .position(|h| h.to_lowercase() == fields.device_id)
    {
        Some(p) => Ok(p),
        None => Err("Device header not found"),
    }?;

    let coordinates = match header
        .iter()
        .position(|h| h.to_lowercase() == fields.coordinates)
    {
        Some(p) => Ok(p),
        None => Err("Coordinates header not found"),
    }?;

    let time = match header.iter().position(|h| h.to_lowercase() == fields.time) {
        Some(p) => Ok(p),
        None => Err("Time header not found"),
    }?;

    let route = header.iter().position(|h| h.to_lowercase() == fields.route);

    let speed = header.iter().position(|h| h.to_lowercase() == fields.speed);

    let elevation = header
        .iter()
        .position(|h| h.to_lowercase() == fields.elevation);

    Ok(FieldsIndex {
        device,
        coordinates,
        time,
        route,
        speed,
        elevation,
    })
}

fn parse_row(
    header: &FieldsIndex,
    fields: &FieldsBuilder,
    row: &mut StringRecord,
) -> Result<Option<DevicePosition>, String> {
    row.trim();

    let device_id = match row.get(header.device) {
        Some(d) => Ok(d.to_string()),
        None => Err("Device field not found"),
    }?;

    let raw_coordinates = match row.get(header.coordinates) {
        Some(d) => Ok(d.to_string()),
        None => Err("Coordinates field not found"),
    }?;
    let separator = match raw_coordinates.clone() {
        s if s.contains(",") => ",",
        s if s.contains(";") => ";",
        _ => " ",
    };
    let scoordinates: Vec<String> = raw_coordinates
        .split(separator)
        .map(|s| s.trim().to_string())
        .collect();
    if scoordinates.len() != 2 {
        return Ok(None);
    }

    let mut ilat = 1;
    let mut ilng = 0;
    if fields.flip_coordinates {
        ilat = 0;
        ilng = 1;
    }

    let lat = scoordinates[ilat]
        .parse::<f64>()
        .map_err(|e| format!("Invalid latitude format: {}", e.to_string()))?;
    let lng = scoordinates[ilng]
        .parse::<f64>()
        .map_err(|e| format!("Invalid longitude format: {}", e.to_string()))?;

    let time = match row.get(header.time) {
        Some(d) => OffsetDateTime::parse(d, &well_known::Rfc3339)
            .map_err(|e| format!("Failed on parse the time: {}", e.to_string())),
        None => Err("Time field not found".to_string()),
    }?;

    let mut dpos = DevicePosition::basic(device_id.clone(), Point::new(lng, lat), time);

    if let Some(iroute) = header.route {
        dpos.route_name = match row.get(iroute) {
            Some(d) => {
                if !d.trim().is_empty() {
                    Some(d.trim().to_string())
                } else {
                    None
                }
            }
            None => None,
        };
    }

    if let Some(ispeed) = header.speed {
        dpos.pos.speed = match row.get(ispeed) {
            Some(d) => d.parse::<f64>().ok(),
            None => None,
        };
    }

    if let Some(ielevation) = header.elevation {
        dpos.pos.altitude = match row.get(ielevation) {
            Some(d) => d.parse::<f64>().ok(),
            None => None,
        };
    }

    Ok(Some(dpos))
}

#[cfg(test)]
pub mod tests {
    use csv::ReaderBuilder;
    use geo::geometry::Point;
    use time::macros::datetime;

    use super::CsvSource;
    use crate::{SourceToTracks, TrackSegmentOptions};

    #[test]
    fn track() -> Result<(), String> {
        let data = "\n
            device,coordinates,time\n
            AA251,\"-48.8702222, -26.31832\",\"2019-10-01T00:01:00.000+00:00\"\n
            AA251,\"-48.8802222 -26.31832\",\"2019-10-01T00:02:00.000+00:00\"\n
            AA251,\"-48.8902222;-26.31832\",\"2019-10-01T00:03:00.000+00:00\"\n
        ";
        let rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        let source = CsvSource::new(rdr, None);
        let op = TrackSegmentOptions::new();

        let tracks = SourceToTracks::build(
            source,
            datetime!(2010-05-24 0:00 UTC),
            datetime!(2023-05-24 0:00 UTC),
            op,
        )?;
        assert_eq!(1, tracks.len());

        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        assert_eq!(Some("2019-10-01".to_string()), track.name);
        assert_eq!(Some("Tracked by `AA251`".to_string()), track.description);
        let segment = &track.segments[0];
        assert_eq!(3, segment.points.len());
        assert_eq!(
            Point::new(-48.8702222, -26.31832),
            segment.points[0].point()
        );

        Ok(())
    }

    #[test]
    fn track_filter() -> Result<(), String> {
        let data = "\n
            device,coordinates,time\n
            AA251,\"-48.8702222,-26.31832\",\"2019-10-01T00:01:00.000+00:00\"\n
            AA251,\"-48.8802222 -26.31832\",\"2019-10-02T00:02:00.000+00:00\"\n
            AA251,\"-48.8902222;-26.31832\",\"2019-10-03T00:03:00.000+00:00\"\n
        ";
        let rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        let source = CsvSource::new(rdr, None);
        let op = TrackSegmentOptions::new();

        let tracks = SourceToTracks::build(
            source,
            datetime!(2019-10-01 0:00 UTC),
            datetime!(2019-10-01 2:00 UTC),
            op,
        )?;
        assert_eq!(1, tracks.len());
        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        let segment = &track.segments[0];
        assert_eq!(1, segment.points.len());

        Ok(())
    }

    #[test]
    fn track_filter_out_failed_positions() -> Result<(), String> {
        let data = "\n
            device,coordinates,time\n
            AA251,\"-48.8702222,-26.31832\",\"2019-10-01T00:01:00.000+00:00\"\n
            AA251,,\"2019-10-02T00:02:00.000+00:00\"\n
            AA251, ,\"2019-10-03T00:03:00.000+00:00\"\n
        ";
        let rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        let source = CsvSource::new(rdr, None);
        let op = TrackSegmentOptions::new();

        let tracks = SourceToTracks::build(
            source,
            datetime!(2010-10-01 0:00 UTC),
            datetime!(2020-10-01 2:00 UTC),
            op,
        )?;
        assert_eq!(1, tracks.len());
        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        let segment = &track.segments[0];
        assert_eq!(1, segment.points.len());

        Ok(())
    }

    #[test]
    fn extra_fields() -> Result<(), String> {
        let data = "\n
            device,coordinates,time,route,speed,elevation\n
            AA251,\"-48.8702222,-26.31832\",\"2019-10-01T00:01:00.000+00:00\",\"JOI123\",0.2,200\n
            AA251,\"-48.8702222,-26.31832\",\"2019-10-01T00:01:10.000+00:00\",\"JOI123\",0.7,198.0\n
        ";
        let rdr = ReaderBuilder::new()
            .flexible(true)
            .from_reader(data.as_bytes());

        let source = CsvSource::new(rdr, None);
        let op = TrackSegmentOptions::new();

        let tracks = SourceToTracks::build(
            source,
            datetime!(2010-10-01 0:00 UTC),
            datetime!(2020-10-01 2:00 UTC),
            op,
        )?;
        assert_eq!(1, tracks.len());
        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        assert_eq!(Some("JOI123".to_string()), track.name);
        let segment = &track.segments[0];
        assert_eq!(2, segment.points.len());
        let point = &segment.points[0];
        assert_eq!(Some(0.2), point.speed);
        assert_eq!(Some(200.0), point.elevation);

        Ok(())
    }
}
