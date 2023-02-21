
use time::OffsetDateTime;
use time::macros::{datetime};
use geo::Point;

use super::position::RawPosition;
use super::tracker::Tracker;

#[test]
fn simple_track() -> Result<(), String> {

    let p1 = RawPosition::basic(Point::new(-48.8702222, -26.31832), datetime!(2021-05-24 0:00 UTC));
    let p2 = RawPosition::basic(Point::new(-48.8619776, -26.3185919), datetime!(2021-05-24 0:05 UTC));
    let p3 = RawPosition::basic(Point::new(-48.8619871, -26.3185861), datetime!(2021-05-24 0:10 UTC));

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

    let p1 = RawPosition::basic(Point::new(-48.8702222, -26.31832), datetime!(2021-05-24 0:00 UTC));
    let p2 = RawPosition::basic(Point::new(-48.8619776, -26.3185919), datetime!(2021-05-24 0:05 UTC));
    let p3 = RawPosition::basic(Point::new(-48.8619871, -26.3185861), datetime!(2021-05-24 0:10 UTC));

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
