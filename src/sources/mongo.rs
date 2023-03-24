//! Mongodb source integration

use bson::{doc, Bson, DateTime, Document};
use geo::geometry::Point;
use mongodb::sync::Collection;
use time::format_description::well_known;
use time::OffsetDateTime;

use super::{FieldsConfiguration, PositionsSource};
use crate::DevicePosition;

/// MongoDB tracks source
pub struct MongoDbSource {
    collection: Collection<Document>,
    fields: FieldsConfiguration,
}

impl MongoDbSource {
    pub fn new(collection: Collection<Document>, fields: Option<FieldsConfiguration>) -> Self {
        Self {
            collection,
            fields: match fields {
                Some(f) => f,
                None => FieldsConfiguration::default(),
            },
        }
    }
}

impl PositionsSource for MongoDbSource {
    fn fetch(
        &mut self,
        start: OffsetDateTime,
        end: OffsetDateTime,
    ) -> Result<Vec<DevicePosition>, String> {
        let mut pos = vec![];

        let filter = doc! {
            self.fields.time.clone(): doc! {
                "$gte": DateTime::from_time_0_3(start),
                "$lte": DateTime::from_time_0_3(end),
            },
            self.fields.coordinates.clone(): doc! {
                "$size": 2,
            }
        };
        let cursor = self
            .collection
            .find(filter, None)
            .map_err(|e| format!("Failed on fetch the docs: {}", e.to_string()))?;

        for rdoc in cursor {
            let doc = rdoc.map_err(|e| format!("Failed on read some doc: {}", e.to_string()))?;

            let id = doc
                .get_object_id("_id")
                .map_err(|e| format!("Failed on access the doc id: {}", e.to_string()))?;

            let dpos = match parse_doc(&self.fields, &doc) {
                Ok(dpos) => Ok(dpos),
                Err(e) => Err(format!("Error with doc {0}: {1}", id, e)),
            }?;

            pos.push(dpos);
        }

        Ok(pos)
    }
}

fn parse_doc(fields: &FieldsConfiguration, doc: &Document) -> Result<DevicePosition, String> {
    let device_id = match doc.get(fields.device_id.clone()) {
        Some(Bson::String(di)) => Ok(di.clone()),
        Some(Bson::Int32(di)) => Ok(di.to_string()),
        Some(Bson::Int64(di)) => Ok(di.to_string()),
        Some(Bson::Double(di)) => Ok(di.to_string()),
        Some(_) => Err("Device field type not supported"),
        None => Err("Device field not found"),
    }?;

    let coordinates = doc
        .get_array(fields.coordinates.clone())
        .map_err(|e| format!("Failed on access the `coordinates`: {}", e.to_string()))?;
    if coordinates.len() != 2 {
        return Err("Coordinates size invalid".to_string());
    }

    let mut ilat = 1;
    let mut ilng = 0;
    if fields.flip_coordinates {
        ilat = 0;
        ilng = 1;
    }

    let lat = match coordinates[ilat] {
        Bson::Double(l) => Ok(l),
        _ => Err("Invalid type of latitude".to_string()),
    }?;
    let lng = match coordinates[ilng] {
        Bson::Double(l) => Ok(l),
        _ => Err("Invalid type of longitude".to_string()),
    }?;

    let time = match doc.get(fields.time.clone()) {
        Some(Bson::String(tm)) => OffsetDateTime::parse(tm, &well_known::Rfc3339)
            .map_err(|e| format!("Failed on parse the time: {}", e.to_string())),
        Some(Bson::DateTime(tm)) => Ok(tm.to_time_0_3()),
        Some(Bson::Timestamp(tm)) => OffsetDateTime::from_unix_timestamp(tm.time.into())
            .map_err(|e| format!("Failed on parse the time tiemstamp: {}", e.to_string())),
        Some(_) => Err("Time field type not supported".to_string()),
        None => Err("Time field not found".to_string()),
    }?;

    let mut dpos = DevicePosition::basic(device_id.clone(), Point::new(lng, lat), time);

    dpos.route_name = match doc.get(fields.route.clone()) {
        Some(Bson::String(ro)) => Some(ro.clone()),
        Some(Bson::Int32(ro)) => Some(ro.to_string()),
        Some(Bson::Array(ro)) => {
            if ro.len() > 0 {
                match &ro[0] {
                    Bson::String(di) => Some(di.clone()),
                    Bson::Int32(di) => Some(di.to_string()),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    };

    dpos.pos.speed = match doc.get(fields.speed.clone()) {
        Some(Bson::Int32(sp)) => Some((*sp).into()),
        Some(Bson::Double(sp)) => Some((*sp).into()),
        _ => None,
    };

    dpos.pos.altitude = match doc.get(fields.elevation.clone()) {
        Some(Bson::Int32(sp)) => Some((*sp).into()),
        Some(Bson::Double(sp)) => Some((*sp).into()),
        _ => None,
    };

    Ok(dpos)
}

#[cfg(test)]
pub mod tests {
    use bson::{doc, Bson, Document};
    use geo::geometry::Point;
    use mongodb::sync::Client;
    use time::macros::datetime;

    use super::MongoDbSource;
    use crate::{FieldsConfiguration, SourceToTracks, TrackSegmentOptions};

    #[test]
    fn track() -> Result<(), String> {
        let client =
            Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "device": "AA251", "coordinates": [-48.8702222, -26.31832], "time": datetime!(2022-02-07 0:01 UTC) },
            doc! { "device": "AA251", "coordinates": [-48.8802222, -26.31832], "time": datetime!(2022-02-07 0:02 UTC) },
            doc! { "device": "AA251", "coordinates": [-48.8902222, -26.31832], "time": datetime!(2022-02-07 0:03 UTC) },
        ];
        collection
            .insert_many(docs, None)
            .map_err(|e| e.to_string())?;

        let source = MongoDbSource::new(collection, None);
        let op = TrackSegmentOptions::default();

        let tracks = SourceToTracks::build(
            source,
            datetime!(2021-05-24 0:00 UTC),
            datetime!(2023-05-24 0:00 UTC),
            op,
        )?;
        assert_eq!(1, tracks.len());

        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        assert_eq!(Some("2022-02-07".to_string()), track.name);
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
    fn track_flip_coordinates() -> Result<(), String> {
        let client =
            Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "device": "AA251", "coordinates": [-26.31832, -48.8702222], "time": datetime!(2022-02-07 0:01 UTC) },
            doc! { "device": "AA251", "coordinates": [-26.31832, -48.8802222], "time": datetime!(2022-02-07 0:02 UTC) },
            doc! { "device": "AA251", "coordinates": [-26.31832, -48.8902222], "time": datetime!(2022-02-07 0:03 UTC) },
        ];
        collection
            .insert_many(docs, None)
            .map_err(|e| e.to_string())?;

        let op = TrackSegmentOptions::default();
        let mut fields = FieldsConfiguration::default();
        fields.flip_coordinates = true;

        let source = MongoDbSource::new(collection, Some(fields));

        let tracks = SourceToTracks::build(
            source,
            datetime!(2021-05-24 0:00 UTC),
            datetime!(2023-05-24 0:00 UTC),
            op,
        )?;
        assert_eq!(1, tracks.len());
        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        let segment = &track.segments[0];
        assert_eq!(3, segment.points.len());
        assert_eq!(
            Point::new(-48.8702222, -26.31832),
            segment.points[0].point()
        );

        Ok(())
    }

    #[test]
    fn track_others_fields_types() -> Result<(), String> {
        let client =
            Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "device": 251, "coordinates": [-48.8702222, -26.31832], "time": datetime!(2023-05-24 0:00 UTC) },
            doc! { "device": 251, "coordinates": [-48.8802222, -26.31832], "time": datetime!(2023-05-24 0:00 UTC) },
            doc! { "device": 251, "coordinates": [-48.8902222, -26.31832], "time": datetime!(2023-05-24 0:00 UTC) },
        ];
        collection
            .insert_many(docs, None)
            .map_err(|e| e.to_string())?;

        let op = TrackSegmentOptions::default();
        let source = MongoDbSource::new(collection, None);

        let tracks = SourceToTracks::build(
            source,
            datetime!(2021-05-24 0:00 UTC),
            datetime!(2023-05-24 0:00 UTC),
            op,
        )?;
        assert_eq!(1, tracks.len());
        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        let segment = &track.segments[0];
        assert_eq!(3, segment.points.len());

        Ok(())
    }

    #[test]
    fn track_filter() -> Result<(), String> {
        let client =
            Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "device": "AA251", "coordinates": [-48.8702222, -26.31832], "time": datetime!(2022-02-07 0:01 UTC) },
            doc! { "device": "AA251", "coordinates": [-48.8802222, -26.31832], "time": datetime!(2022-02-06 0:01 UTC) },
            doc! { "device": "AA251", "coordinates": [-48.8902222, -26.31832], "time": datetime!(2022-02-03 0:01 UTC) },
        ];
        collection
            .insert_many(docs, None)
            .map_err(|e| e.to_string())?;

        let op = TrackSegmentOptions::default();
        let source = MongoDbSource::new(collection, None);

        let tracks = SourceToTracks::build(
            source,
            datetime!(2022-02-06 0:00 UTC),
            datetime!(2022-02-06 5:00 UTC),
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
    fn track_custom_fields() -> Result<(), String> {
        let client =
            Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "dev": "AA251", "coords": [-48.8702222, -26.31832], "dev_time": datetime!(2022-02-07 0:01 UTC) },
            doc! { "dev": "AA251", "coords": [-48.8802222, -26.31832], "dev_time": datetime!(2022-02-06 0:01 UTC) },
            doc! { "dev": "AA251", "coords": [-48.8902222, -26.31832], "dev_time": datetime!(2022-02-03 0:01 UTC) },
        ];
        collection
            .insert_many(docs, None)
            .map_err(|e| e.to_string())?;

        let mut fields = FieldsConfiguration::default();
        fields.device_id = "dev".to_string();
        fields.coordinates = "coords".to_string();
        fields.time = "dev_time".to_string();

        let op = TrackSegmentOptions::default();
        let source = MongoDbSource::new(collection, Some(fields));

        let tracks = SourceToTracks::build(
            source,
            datetime!(2022-02-06 0:00 UTC),
            datetime!(2022-02-06 5:00 UTC),
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
        let client =
            Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "device": "AA251", "coordinates": [-48.8702222, -26.31832], "time": datetime!(2022-02-06 0:01 UTC) },
            doc! { "device": "AA251", "coordinates": Bson::Null, "time": datetime!(2022-02-07 0:01 UTC) },
            doc! { "device": "AA251", "coordinates": [], "time": datetime!(2022-02-06 0:01 UTC) },
            doc! { "device": "AA251", "time": datetime!(2022-02-03 0:01 UTC) },
        ];
        collection
            .insert_many(docs, None)
            .map_err(|e| e.to_string())?;

        let op = TrackSegmentOptions::default();
        let source = MongoDbSource::new(collection, None);

        let tracks = SourceToTracks::build(
            source,
            datetime!(2022-01-06 0:00 UTC),
            datetime!(2022-03-06 5:00 UTC),
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
    fn track_route_field() -> Result<(), String> {
        let client =
            Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "device": "AA251", "coordinates": [-48.8702222, -26.31832], "time": datetime!(2022-02-07 0:01 UTC), "route": ["01"] },
            doc! { "device": "AA251", "coordinates": [-48.8802222, -26.31832], "time": datetime!(2022-02-07 0:02 UTC), "route": ["01", "02"] },
            doc! { "device": "AA251", "coordinates": [-48.8902222, -26.31832], "time": datetime!(2022-02-07 0:03 UTC), "route": ["01"] },
            doc! { "device": "AA251", "coordinates": [-48.8902222, -26.31832], "time": datetime!(2022-02-07 0:03 UTC), "route": Bson::Null },
            doc! { "device": "AA251", "coordinates": [-48.8902222, -26.31832], "time": datetime!(2022-02-07 0:03 UTC), "route": "04" },
            doc! { "device": "AA251", "coordinates": [-48.8702222, -26.31832], "time": datetime!(2022-02-07 0:01 UTC), "route": [12] },
            doc! { "device": "AA251", "coordinates": [-48.8702222, -26.31832], "time": datetime!(2022-02-07 0:01 UTC), "route": 12 },
        ];
        collection
            .insert_many(docs, None)
            .map_err(|e| e.to_string())?;

        let op = TrackSegmentOptions::default();
        let source = MongoDbSource::new(collection, None);

        let tracks = SourceToTracks::build(
            source,
            datetime!(2021-05-24 0:00 UTC),
            datetime!(2023-05-24 0:00 UTC),
            op,
        )?;
        assert_eq!(4, tracks.len());

        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        assert_eq!(Some("01".to_string()), track.name);
        let segment = &track.segments[0];
        assert_eq!(3, segment.points.len());

        let track = &tracks[1];
        assert_eq!(1, track.segments.len());
        assert_eq!(Some("04".to_string()), track.name);
        let segment = &track.segments[0];
        assert_eq!(1, segment.points.len());

        let track = &tracks[2];
        assert_eq!(1, track.segments.len());
        assert_eq!(Some("12".to_string()), track.name);
        let segment = &track.segments[0];
        assert_eq!(2, segment.points.len());

        let track = &tracks[3];
        assert_eq!(1, track.segments.len());
        assert_eq!(Some("2022-02-07".to_string()), track.name);
        let segment = &track.segments[0];
        assert_eq!(1, segment.points.len());

        Ok(())
    }
}
