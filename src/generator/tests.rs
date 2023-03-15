use std::fs;

use geo::Point;
use time::{macros::datetime, OffsetDateTime};

use super::gpx::GpxGenerator;
use super::position::{DevicePosition, RawPosition};
use super::tracker::{SourceToTracks, Tracker, TrackSegmentOptions};
use crate::PositionsSource;

#[test]
fn simple_track() -> Result<(), String> {
    let p1 = RawPosition::basic(
        Point::new(-48.8702222, -26.31832),
        datetime!(2021-05-24 0:00 UTC),
    );
    let p2 = RawPosition::basic(
        Point::new(-48.8619776, -26.3185919),
        datetime!(2021-05-24 0:02 UTC),
    );
    let p3 = RawPosition::basic(
        Point::new(-48.8619871, -26.3185861),
        datetime!(2021-05-24 0:04 UTC),
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
        datetime!(2021-05-24 0:01 UTC),
    );
    let p3 = RawPosition::basic(
        Point::new(-48.8619871, -26.3185861),
        datetime!(2021-05-24 0:02 UTC),
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
        datetime!(2021-05-24 0:02 UTC),
    );
    let p3 = RawPosition::basic(
        Point::new(-48.8619871, -26.3185861),
        datetime!(2021-05-24 0:04 UTC),
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
                datetime!(2021-05-24 0:01 UTC),
            ));
            pos.push(DevicePosition::basic(
                "dev 1".to_string(),
                Point::new(-48.8619871, -26.3185861),
                datetime!(2021-05-24 0:02 UTC),
            ));
            pos.push(DevicePosition::basic(
                "dev 2".to_string(),
                Point::new(-48.8619871, -26.3385861),
                datetime!(2021-05-24 0:02 UTC),
            ));

            Ok(pos)
        }
    }

    let op = TrackSegmentOptions::new();
    let tracks = SourceToTracks::build(
        TestSource {},
        datetime!(2021-05-24 0:00 UTC),
        datetime!(2022-05-24 0:00 UTC),
        op,
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

    let op = TrackSegmentOptions::new();
    let tracks = SourceToTracks::build(
        TestSource {},
        datetime!(2021-05-24 0:00 UTC),
        datetime!(2022-05-24 0:00 UTC),
        op,
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

#[test]
fn speed_and_elevation_info() -> Result<(), String> {
    let mut p1 = RawPosition::basic(
        Point::new(-48.8702222, -26.31832),
        datetime!(2021-05-24 0:00 UTC),
    );
    p1.altitude = Some(50.0);
    p1.speed = Some(7.0);

    let track = Tracker::new("my dev 1".to_string(), "running in joinville".to_string())
        .build(vec![&p1])?;
    assert_eq!(1, track.segments.len());

    let segment = &track.segments[0];
    assert_eq!(1, segment.points.len());
    assert_eq!(p1.coordinates, segment.points[0].point());
    assert_eq!(Some(p1.time.into()), segment.points[0].time);
    assert_eq!(Some(7.0), segment.points[0].speed);
    assert_eq!(Some(50.0), segment.points[0].elevation);

    Ok(())
}

#[test]
fn time_segmented_track() -> Result<(), String> {
    let times = vec![
        datetime!(2021-05-24 0:00 UTC),
        datetime!(2021-05-24 0:01 UTC),
        datetime!(2021-05-24 0:02 UTC),
        datetime!(2021-05-24 0:03 UTC),
        datetime!(2021-05-24 0:04 UTC),

        datetime!(2021-05-24 0:05 UTC),
        datetime!(2021-05-24 0:06 UTC),
        datetime!(2021-05-24 0:07 UTC),
        datetime!(2021-05-24 0:08 UTC),
        datetime!(2021-05-24 0:09 UTC),

        datetime!(2021-05-24 0:17 UTC),
        datetime!(2021-05-24 0:18 UTC),
        datetime!(2021-05-24 0:19 UTC),

        datetime!(2021-05-24 0:21 UTC),
        datetime!(2021-05-24 0:22 UTC),
        datetime!(2021-05-24 0:24 UTC),

        datetime!(2021-05-24 1:21 UTC),
    ];

    let raw: Vec<RawPosition> = times.iter()
        .map(|tm| RawPosition::basic(
            Point::new(-48.8702222, -26.31832),
            *tm
        ))
        .collect();
    let pos = raw.iter().map(|p| p).collect();
    let track = Tracker::new("my dev 1".to_string(), "running in joinville".to_string())
        .build(pos)?;
    assert_eq!(5, track.segments.len());

    let segment = &track.segments[0];
    assert_eq!(5, segment.points.len());
    assert_eq!(Some(datetime!(2021-05-24 0:00 UTC).into()), segment.points[0].time);
    assert_eq!(Some(datetime!(2021-05-24 0:04 UTC).into()), segment.points[4].time);

    let segment = &track.segments[1];
    assert_eq!(5, segment.points.len());
    assert_eq!(Some(datetime!(2021-05-24 0:05 UTC).into()), segment.points[0].time);
    assert_eq!(Some(datetime!(2021-05-24 0:09 UTC).into()), segment.points[4].time);

    let segment = &track.segments[2];
    assert_eq!(3, segment.points.len());
    assert_eq!(Some(datetime!(2021-05-24 0:17 UTC).into()), segment.points[0].time);
    assert_eq!(Some(datetime!(2021-05-24 0:19 UTC).into()), segment.points[2].time);

    let segment = &track.segments[3];
    assert_eq!(3, segment.points.len());
    assert_eq!(Some(datetime!(2021-05-24 0:21 UTC).into()), segment.points[0].time);
    assert_eq!(Some(datetime!(2021-05-24 0:24 UTC).into()), segment.points[2].time);

    let segment = &track.segments[4];
    assert_eq!(1, segment.points.len());
    assert_eq!(Some(datetime!(2021-05-24 1:21 UTC).into()), segment.points[0].time);
    assert_eq!(Some(datetime!(2021-05-24 1:21 UTC).into()), segment.points[0].time);

    Ok(())
}

#[test]
fn simplify_track() -> Result<(), String> {
    let locs = vec![
        Point::new(5.0, 2.0),
        Point::new(3.0, 8.0),
        Point::new(6.0, 20.0),
        Point::new(7.0, 25.0),
        Point::new(10.0, 10.0),
    ];

    let raw: Vec<RawPosition> = locs.iter()
        .map(|loc| RawPosition::basic(
            *loc,
            datetime!(2021-05-24 0:00 UTC),
        ))
        .collect();
    let pos = raw.iter().map(|p| p).collect();
    let mut op = TrackSegmentOptions::new();
    op.simplify_with_vw(30.0);
    let track = Tracker::new("my dev 1".to_string(), "running in joinville".to_string())
        .configure_segments(&op)
        .build(pos)?;
    assert_eq!(1, track.segments.len());

    let segment = &track.segments[0];
    assert_eq!(3, segment.points.len());
    assert_eq!(Point::new(5.0, 2.0), segment.points[0].point());
    assert_eq!(Point::new(7.0, 25.0), segment.points[1].point());
    assert_eq!(Point::new(10.0, 10.0), segment.points[2].point());

    Ok(())
}
