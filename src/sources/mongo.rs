//! Mongodb source integration

use mongodb::bson::{doc, Document, Bson};
use mongodb::sync::Collection;
use time::format_description::well_known;
use time::OffsetDateTime;
use geo::geometry::Point;

use super::{PositionsSource, FieldsBuilder};
use crate::DevicePosition;

/// MongoDB tracks source
pub struct MongoDbSource {
    collection: Collection<Document>,
    fields: FieldsBuilder
}

impl MongoDbSource {
    pub fn new(collection: Collection<Document>, fields: Option<FieldsBuilder>) -> Self {
        Self {
            collection,
            fields: match fields {
                Some(f) => f,
                None => FieldsBuilder::default()
            }
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
                "$gte": start.format(&well_known::Rfc3339).map_err(|e| e.to_string())?,
                "$lte": end.format(&well_known::Rfc3339).map_err(|e| e.to_string())?
            }
        };
        let cursor = self.collection.find(filter, None)
            .map_err(|e| format!("Failed on fetch the docs: {}", e.to_string()))?;

        for rdoc in cursor {
            let doc = rdoc.map_err(|e| format!("Failed on read some doc: {}", e.to_string()))?;

            let id = doc.get_object_id("_id")
                .map_err(|e| format!("Failed on access the doc id: {}", e.to_string()))?;

            let dpos = match parse_doc(&self.fields, &doc) {
                Ok(dpos) => Ok(dpos),
                Err(e) => Err(format!("Error with doc {0}: {1}", id, e))
            }?;

            pos.push(dpos);
        }

        Ok(pos)
    }
}

fn parse_doc(fields: &FieldsBuilder, doc: &Document) -> Result<DevicePosition, String> {

    let device_id = doc.get_str(fields.device_id.clone())
        .map_err(|e| format!("Failed on access the `device`: {}", e.to_string()))?;

    let coordinates = doc.get_array(fields.coordinates.clone())
        .map_err(|e| format!("Failed on access the `coordinates`: {}", e.to_string()))?;
    if coordinates.len() != 2 {
        return Err("Coordinates size invalid".to_string());
    }
    let lat = match coordinates[1] {
        Bson::Double(l) => Ok(l),
        _ => Err("Invalid type of latitude".to_string())
    }?;
    let lng = match coordinates[0] {
        Bson::Double(l) => Ok(l),
        _ => Err("Invalid type of longitude".to_string())
    }?;

    let stime = doc.get_str(fields.time.clone())
        .map_err(|e| format!("Failed on access the `time`: {}", e.to_string()))?;
    let time = OffsetDateTime::parse(stime, &well_known::Rfc3339)
        .map_err(|e| format!("Failed on parse the time: {}", e.to_string()))?;

    let dpos = DevicePosition::basic(device_id.to_string(), Point::new(lng, lat), time);

    Ok(dpos)
}

#[cfg(test)]
pub mod tests {
    use mongodb::sync::Client;
    use mongodb::bson::{doc, Document};
    use time::macros::datetime;

    use crate::{SourceToTracks, FieldsBuilder};
    use super::MongoDbSource;

    #[test]
    fn mongo_track() -> Result<(), String> {

        let client = Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "device": "AA251", "coordinates": [-48.8702222, -26.31832], "time": "2022-02-07T02:13:51Z" },
            doc! { "device": "AA251", "coordinates": [-48.8802222, -26.31832], "time": "2022-02-07T02:13:55Z" },
            doc! { "device": "AA251", "coordinates": [-48.8902222, -26.31832], "time": "2022-02-07T02:13:57Z" },
        ];
        collection.insert_many(docs, None).map_err(|e| e.to_string())?;

        let source = MongoDbSource::new(collection, None);

        let tracks = SourceToTracks::build(source, datetime!(2021-05-24 0:00 UTC), datetime!(2023-05-24 0:00 UTC))?;
        assert_eq!(1, tracks.len());

        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        assert_eq!(Some("2022-02-07".to_string()), track.name);
        assert_eq!(Some("Tracked by `AA251`".to_string()), track.description);
        let segment = &track.segments[0];
        assert_eq!(3, segment.points.len());

        Ok(())
    }

    #[test]
    fn mongo_track_filter() -> Result<(), String> {

        let client = Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "device": "AA251", "coordinates": [-48.8702222, -26.31832], "time": "2022-02-07T02:13:51Z" },
            doc! { "device": "AA251", "coordinates": [-48.8802222, -26.31832], "time": "2022-02-06T02:13:55Z" },
            doc! { "device": "AA251", "coordinates": [-48.8902222, -26.31832], "time": "2022-02-03T02:13:57Z" },
        ];
        collection.insert_many(docs, None).map_err(|e| e.to_string())?;

        let source = MongoDbSource::new(collection, None);

        let tracks = SourceToTracks::build(source, datetime!(2022-02-06 0:00 UTC), datetime!(2022-02-06 5:00 UTC))?;
        assert_eq!(1, tracks.len());
        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        let segment = &track.segments[0];
        assert_eq!(1, segment.points.len());

        Ok(())
    }

    #[test]
    fn mongo_track_custom_fields() -> Result<(), String> {

        let client = Client::with_uri_str("mongodb://localhost:27017").map_err(|e| e.to_string())?;
        let db = client.database("location2gpx_tests");
        let collection = db.collection::<Document>("tracks");
        collection.drop(None).map_err(|e| e.to_string())?;

        let docs = vec![
            doc! { "dev": "AA251", "coords": [-48.8702222, -26.31832], "dev_time": "2022-02-07T02:13:51Z" },
            doc! { "dev": "AA251", "coords": [-48.8802222, -26.31832], "dev_time": "2022-02-06T02:13:55Z" },
            doc! { "dev": "AA251", "coords": [-48.8902222, -26.31832], "dev_time": "2022-02-03T02:13:57Z" },
        ];
        collection.insert_many(docs, None).map_err(|e| e.to_string())?;

        let fields = FieldsBuilder::default()
            .device("dev")
            .coordinates("coords")
            .time("dev_time");
        let source = MongoDbSource::new(collection, Some(fields));

        let tracks = SourceToTracks::build(source, datetime!(2022-02-06 0:00 UTC), datetime!(2022-02-06 5:00 UTC))?;
        assert_eq!(1, tracks.len());
        let track = &tracks[0];
        assert_eq!(1, track.segments.len());
        let segment = &track.segments[0];
        assert_eq!(1, segment.points.len());

        Ok(())
    }
}
