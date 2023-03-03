//! location2gpx cli - GPX generator from many location sources

use std::io::BufWriter;
use std::fs::File;

use mongodb::sync::Client;
use bson::{doc, Document};
use time::format_description::well_known;
use time::OffsetDateTime;

use location2gpx::{SourceToTracks, FieldsBuilder, GpxGenerator};
use location2gpx::sources::MongoDbSource;

/// CLI of location2gpx - Convert your raw GPS data into a GPX file
#[argopt::cmd]
fn main(
    /// Mongo connection string source
    connection: String,
    /// Mongo collection name
    collection: String,
    /// Start time, RFC3339 format
    start: String,
    /// End time, RFC3339 format
    end: String,
    /// OFX path file destination
    destination: String,
    #[opt(long)]
    field_device: Option<String>,
    #[opt(long)]
    field_coordinates: Option<String>,
    #[opt(long)]
    field_time: Option<String>,
) -> Result<(), String> {

    let start = OffsetDateTime::parse(&start, &well_known::Rfc3339)
        .map_err(|e| format!("Failed on parse the start time: {}", e.to_string()))?;
    let end = OffsetDateTime::parse(&end, &well_known::Rfc3339)
        .map_err(|e| format!("Failed on parse the end time: {}", e.to_string()))?;

    let destination = File::create(destination)
        .map_err(|e| format!("Failed on create the destination file: {}", e.to_string()))?;

    let client = Client::with_uri_str(connection)
        .map_err(|e| format!("Failed on connect: {0}", e.to_string()))?;
    let db = client.default_database()
        .ok_or("Default database not provided")?;
    let collection = db.collection::<Document>(&collection);

    let mut fields = FieldsBuilder::default();
    if let Some(f) = field_device {
        fields.device(f);
    }
    if let Some(f) = field_coordinates {
        fields.coordinates(f);
    }
    if let Some(f) = field_time {
        fields.time(f);
    }

    let source = MongoDbSource::new(collection, Some(fields));

    let tracks = SourceToTracks::build(source, start, end)?;

    let mut gpx = GpxGenerator::empty();
    gpx.tracks = tracks;

    let doc = gpx.generate()?;

    let mut writer = BufWriter::new(destination);
    gpx::write(&doc, &mut writer)
        .map_err(|e| e.to_string())?;

    Ok(())
}
