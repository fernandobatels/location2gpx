use std::fs;

use geo::Point;
use time::{macros::datetime, OffsetDateTime};

use super::gpx::GpxGenerator;
use super::position::{DevicePosition, RawPosition};
use super::tracker::{SourceToTracks, Tracker};
use crate::PositionsSource;

#[test]
fn simple_track() -> Result<(), String> {
    let p1 = RawPosition::basic(
        Point::new(-48.8702222, -26.31832),
        datetime!(2021-05-24 0:00 UTC),
    );
    let p2 = RawPosition::basic(
        Point::new(-48.8619776, -26.3185919),
        datetime!(2021-05-24 0:05 UTC),
    );
    let p3 = RawPosition::basic(
        Point::new(-48.8619871, -26.3185861),
        datetime!(2021-05-24 0:10 UTC),
    );

    let track = Tracker::new("my dev 1".to_string(), "running in joinville".to_string())
        .source("my app v0.1".to_string())
        .build(vec![&p1, &p2, &p3])?;
    assert_eq!(1, track.segments.len());
    assert_eq!(Some("running in joinville".to_string()), track.name);
    assert_eq!(Some("Tracked by `my dev 1`".to_string()), track.description);
    assert_eq!(Some("my app v0.1".to_string()), track.source);

    let segment = &track.segments[0];
    assert_eq!(3, segment.points.len());
    assert_eq!(p1.coordinates, segment.points[0].point());
    assert_eq!(Some(p1.time.into()), segment.points[0].time);
    assert_eq!(p2.coordinates, segment.points[1].point());
    assert_eq!(Some(p2.time.into()), segment.points[1].time);
    assert_eq!(p3.coordinates, segment.points[2].point());
    assert_eq!(Some(p3.time.into()), segment.points[2].time);

    Ok(())
}

#[test]
fn simple_track_reversed() -> Result<(), String> {
    let p1 = RawPosition::basic(
        Point::new(-48.8702222, -26.31832),
        datetime!(2021-05-24 0:00 UTC),
    );
    let p2 = RawPosition::basic(
        Point::new(-48.8619776, -26.3185919),
        datetime!(2021-05-24 0:05 UTC),
    );
    let p3 = RawPosition::basic(
        Point::new(-48.8619871, -26.3185861),
        datetime!(2021-05-24 0:10 UTC),
    );

    let track = Tracker::new("my dev 1".to_string(), "running in joinville".to_string())
        .build(vec![&p3, &p2, &p1])?;
    assert_eq!(1, track.segments.len());

    let segment = &track.segments[0];
    assert_eq!(3, segment.points.len());
    assert_eq!(p1.coordinates, segment.points[0].point());
    assert_eq!(Some(p1.time.into()), segment.points[0].time);
    assert_eq!(p2.coordinates, segment.points[1].point());
    assert_eq!(Some(p2.time.into()), segment.points[1].time);
    assert_eq!(p3.coordinates, segment.points[2].point());
    assert_eq!(Some(p3.time.into()), segment.points[2].time);

    Ok(())
}

#[test]
fn simple_gpx() -> Result<(), String> {
    let p1 = RawPosition::basic(
        Point::new(-48.8702222, -26.31832),
        datetime!(2021-05-24 0:00 UTC),
    );
    let p2 = RawPosition::basic(
        Point::new(-48.8619776, -26.3185919),
        datetime!(2021-05-24 0:05 UTC),
    );
    let p3 = RawPosition::basic(
        Point::new(-48.8619871, -26.3185861),
        datetime!(2021-05-24 0:10 UTC),
    );

    let track = Tracker::new("my dev 1".to_string(), "running in joinville".to_string())
        .build(vec![&p1, &p2, &p3])?;

    let mut gpx = GpxGenerator::empty();
    gpx.tracks.push(track);

    let doc = gpx.generate()?;

    let mut bdoc: Vec<u8> = Vec::new();
    gpx::write(&doc, &mut bdoc).map_err(|e| e.to_string())?;
    let doc = String::from_utf8(bdoc).map_err(|e| e.to_string())?;

    let edoc = fs::read_to_string("samples/simple.gpx").map_err(|e| e.to_string())?;

    assert_eq!(
        edoc.lines().collect::<String>(),
        doc.lines().collect::<String>()
    );

    Ok(())
}

#[test]
fn source2tracks() -> Result<(), String> {
    struct TestSource {}
    impl PositionsSource for TestSource {
        fn fetch(
            &mut self,
            _start: OffsetDateTime,
            _end: OffsetDateTime,
        ) -> Result<Vec<DevicePosition>, String> {
            let mut pos = vec![];

            pos.push(DevicePosition::basic(
                "dev 1".to_string(),
                Point::new(-48.8702222, -26.31832),
                datetime!(2021-05-24 0:00 UTC),
            ));
            pos.push(DevicePosition::basic(
                "dev 1".to_string(),
                Point::new(-48.8619776, -26.3185919),
                datetime!(2021-05-24 0:05 UTC),
            ));
            pos.push(DevicePosition::basic(
                "dev 1".to_string(),
                Point::new(-48.8619871, -26.3185861),
                datetime!(2021-05-24 0:10 UTC),
            ));
            pos.push(DevicePosition::basic(
                "dev 2".to_string(),
                Point::new(-48.8619871, -26.3385861),
                datetime!(2021-05-24 0:10 UTC),
            ));

            Ok(pos)
        }
    }

    let tracks = SourceToTracks::build(
        TestSource {},
        datetime!(2021-05-24 0:00 UTC),
        datetime!(2022-05-24 0:00 UTC),
    )?;
    assert_eq!(2, tracks.len());

    let track = &tracks[0];
    assert_eq!(Some("2021-05-24".to_string()), track.name);
    assert_eq!(Some("Tracked by `dev 1`".to_string()), track.description);
    assert_eq!(None, track.source);
    assert_eq!(1, track.segments.len());
    let segment = &track.segments[0];
    assert_eq!(3, segment.points.len());

    let track = &tracks[1];
    assert_eq!(Some("2021-05-24".to_string()), track.name);
    assert_eq!(Some("Tracked by `dev 2`".to_string()), track.description);
    assert_eq!(None, track.source);
    assert_eq!(1, track.segments.len());
    let segment = &track.segments[0];
    assert_eq!(1, segment.points.len());

    Ok(())
}

#[test]
fn source2tracks_with_rotes() -> Result<(), String> {
    struct TestSource {}
    impl PositionsSource for TestSource {
        fn fetch(
            &mut self,
            _start: OffsetDateTime,
            _end: OffsetDateTime,
        ) -> Result<Vec<DevicePosition>, String> {
            let mut pos = vec![];

            pos.push({
                let mut p = DevicePosition::basic(
                    "dev 1".to_string(),
                    Point::new(-48.8702222, -26.31832),
                    datetime!(2021-05-24 0:00 UTC),
                );
                p.route_name = Some("125".to_string());
                p.tracker = Some("my app".to_string());
                p
            });
            pos.push({
                let mut p = DevicePosition::basic(
                    "dev 1".to_string(),
                    Point::new(-48.8702222, -23.31832),
                    datetime!(2021-05-24 0:00 UTC),
                );
                p.route_name = Some("125".to_string());
                p.tracker = Some("my app".to_string());
                p
            });
            pos.push({
                let mut p = DevicePosition::basic(
                    "dev 1".to_string(),
                    Point::new(-48.8702222, -22.31832),
                    datetime!(2021-05-24 0:00 UTC),
                );
                p.route_name = Some("123".to_string());
                p.tracker = Some("my app".to_string());
                p
            });
            pos.push({
                let mut p = DevicePosition::basic(
                    "dev 2".to_string(),
                    Point::new(-48.3702222, -26.31832),
                    datetime!(2021-05-24 0:00 UTC),
                );
                p.route_name = Some("125".to_string());
                p.tracker = Some("my app".to_string());
                p
            });

            Ok(pos)
        }
    }

    let tracks = SourceToTracks::build(
        TestSource {},
        datetime!(2021-05-24 0:00 UTC),
        datetime!(2022-05-24 0:00 UTC),
    )?;
    assert_eq!(3, tracks.len());

    let track = &tracks[0];
    assert_eq!(Some("123".to_string()), track.name);
    assert_eq!(Some("Tracked by `dev 1`".to_string()), track.description);
    assert_eq!(Some("my app".to_string()), track.source);
    assert_eq!(1, track.segments.len());
    let segment = &track.segments[0];
    assert_eq!(1, segment.points.len());

    let track = &tracks[1];
    assert_eq!(Some("125".to_string()), track.name);
    assert_eq!(Some("Tracked by `dev 1`".to_string()), track.description);
    assert_eq!(Some("my app".to_string()), track.source);
    assert_eq!(1, track.segments.len());
    let segment = &track.segments[0];
    assert_eq!(2, segment.points.len());

    let track = &tracks[2];
    assert_eq!(Some("125".to_string()), track.name);
    assert_eq!(Some("Tracked by `dev 2`".to_string()), track.description);
    assert_eq!(Some("my app".to_string()), track.source);
    assert_eq!(1, track.segments.len());
    let segment = &track.segments[0];
    assert_eq!(1, segment.points.len());

    Ok(())
}
