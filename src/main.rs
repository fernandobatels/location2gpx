//! location2gpx cli - GPX generator from many location sources

use std::fs::{self, File};
use std::io::BufWriter;

use argopt::{cmd_group, subcmd};
use bson::{doc, Document};
use csv::Reader;
use mongodb::sync::Client;
use serde::Deserialize;
use time::format_description::well_known;
use time::OffsetDateTime;

use location2gpx::sources::{CsvSource, MongoDbSource};
use location2gpx::{FieldsConfiguration, GpxGenerator, SourceToTracks, TrackSegmentOptions};

/// CLI of location2gpx - Convert your raw GPS data into a GPX file
#[cmd_group(commands = [mongo,csv])]
fn main() -> Result<(), String> {}

/// Generate a GPX from a CSV file source
#[subcmd]
fn csv(
    /// CSV file source
    csv_path: String,
    /// Start time, RFC3339 format
    start: String,
    /// End time, RFC3339 format
    end: String,
    /// GPX path file destination
    destination: String,
    /// Fields and segments configuration. Default: .loc2gpx.yaml, ~/.loc2gpx.yaml
    #[opt(long)]
    config: Option<String>,
) -> Result<(), String> {
    let start = OffsetDateTime::parse(&start, &well_known::Rfc3339)
        .map_err(|e| format!("Failed on parse the start time: {}", e.to_string()))?;
    let end = OffsetDateTime::parse(&end, &well_known::Rfc3339)
        .map_err(|e| format!("Failed on parse the end time: {}", e.to_string()))?;

    let destination = File::create(destination)
        .map_err(|e| format!("Failed on create the destination file: {}", e.to_string()))?;

    let csv = File::open(csv_path)
        .map_err(|e| format!("Failed on open the CSV file: {}", e.to_string()))?;
    let rcsv = Reader::from_reader(csv);

    let (fields, op) = load_configs(config);

    let source = CsvSource::new(rcsv, Some(fields));

    let tracks = SourceToTracks::build(source, start, end, op)?;

    let mut gpx = GpxGenerator::empty();
    gpx.tracks = tracks;

    let doc = gpx.generate()?;

    let mut writer = BufWriter::new(destination);
    gpx::write(&doc, &mut writer).map_err(|e| e.to_string())?;

    Ok(())
}

/// Generate a GPX from a mongodb collection source
#[subcmd]
fn mongo(
    /// Mongo connection string source
    connection: String,
    /// Mongo collection name
    collection: String,
    /// Start time, RFC3339 format
    start: String,
    /// End time, RFC3339 format
    end: String,
    /// GPX path file destination
    destination: String,
    /// Fields and segments configuration. Default: .loc2gpx.yaml, ~/.loc2gpx.yaml
    #[opt(long)]
    config: Option<String>,
) -> Result<(), String> {
    let start = OffsetDateTime::parse(&start, &well_known::Rfc3339)
        .map_err(|e| format!("Failed on parse the start time: {}", e.to_string()))?;
    let end = OffsetDateTime::parse(&end, &well_known::Rfc3339)
        .map_err(|e| format!("Failed on parse the end time: {}", e.to_string()))?;

    let destination = File::create(destination)
        .map_err(|e| format!("Failed on create the destination file: {}", e.to_string()))?;

    let client = Client::with_uri_str(connection)
        .map_err(|e| format!("Failed on connect: {0}", e.to_string()))?;
    let db = client
        .default_database()
        .ok_or("Default database not provided")?;
    let collection = db.collection::<Document>(&collection);

    let (fields, op) = load_configs(config);

    let source = MongoDbSource::new(collection, Some(fields));

    let tracks = SourceToTracks::build(source, start, end, op)?;

    let mut gpx = GpxGenerator::empty();
    gpx.tracks = tracks;

    let doc = gpx.generate()?;

    let mut writer = BufWriter::new(destination);
    gpx::write(&doc, &mut writer).map_err(|e| e.to_string())?;

    Ok(())
}

/// Load the current config
fn load_configs(provided: Option<String>) -> (FieldsConfiguration, TrackSegmentOptions) {
    let mut options = vec![];

    if let Some(sprovided) = provided {
        options.push(sprovided);
    }

    options.push(".loc2gpx.yaml".to_string());

    if let Some(home) = dirs::home_dir() {
        if let Some(shome) = home.to_str() {
            options.push(format!("{}/.loc2gpx.yaml", shome));
        }
    }

    let mut yaml: Option<String> = None;
    for fi in options {
        if let Ok(s) = fs::read_to_string(fi) {
            yaml = Some(s);
            break;
        }
    }

    if let Some(s) = yaml {
        if let Ok(conf) = serde_yaml::from_str::<Configs>(&s) {
            return (conf.fields, conf.segments);
        }
    }

    (
        FieldsConfiguration::default(),
        TrackSegmentOptions::default(),
    )
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Configs {
    pub fields: FieldsConfiguration,
    pub segments: TrackSegmentOptions,
}

#[test]
fn parse_configs() -> Result<(), String> {
    let yaml = "\nfields:\nsegments:";

    let tso: Configs = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;

    assert_eq!(
        Configs {
            fields: FieldsConfiguration {
                device_id: "device".to_string(),
                time: "time".to_string(),
                route: "route".to_string(),
                coordinates: "coordinates".to_string(),
                speed: "speed".to_string(),
                elevation: "elevation".to_string(),
                flip_coordinates: false,
            },
            segments: TrackSegmentOptions {
                max_duration: 300,
                vw_tolerance: None
            }
        },
        tso
    );

    let yaml = "\nfields:\n  device_id: dev_id\nsegments:\n  max_duration: 600";

    let tso: Configs = serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;

    assert_eq!(
        Configs {
            fields: FieldsConfiguration {
                device_id: "dev_id".to_string(),
                time: "time".to_string(),
                route: "route".to_string(),
                coordinates: "coordinates".to_string(),
                speed: "speed".to_string(),
                elevation: "elevation".to_string(),
                flip_coordinates: false,
            },
            segments: TrackSegmentOptions {
                max_duration: 600,
                vw_tolerance: None
            }
        },
        tso
    );

    Ok(())
}
