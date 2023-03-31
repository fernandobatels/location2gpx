# location2gpx

[![Crate](https://img.shields.io/crates/v/location2gpx.svg)](https://crates.io/crates/location2gpx)
[![API](https://docs.rs/location2gpx/badge.svg)](https://docs.rs/location2gpx)
[![github sponsors](https://img.shields.io/github/sponsors/fernandobatels)](https://github.com/sponsors/fernandobatels)

GPX file generator from consolidated sources.

[Visualize](https://www.gpxsee.org/) or [manipulate](https://qgis.org) your tracks with a `.gpx` file from raw data in mongodb collections or CSV.

## How location2gpx works

1. Connect to your source and fetch all the records of provided range
2. Groups all your positions by a device and route
  - Each device + route will be a GPX track
3. Track positions will be split by a maximum time value into GPX track segments
  - In segment you have all points with elevation, speed and time
  - If desired, each track segment can be simplified with the Visvalingam-Whyatt algorithm
4. Your GPX file are ready

## How to use

With mongodb:
``` bash
cargo run -- mongo "mongodb://localhost:27017/yourdb" yourcollection "2020-01-01T00:00:00.000+00:00" "2020-12-31T00:00:00.000+00:00" /tmp/my-tracks-2020.gpx
```

With CSV file:
``` bash
cargo run -- csv yourfile.csv "2020-01-01T00:00:00.000+00:00" "2020-12-31T00:00:00.000+00:00" /tmp/my-tracks-2020.gpx
```

Output will be like this [sample](https://github.com/fernandobatels/location2gpx/blob/main/samples/simple.gpx).

## How configure fields

To configurate, you need to setup a yaml file and use the `--config` parameter. You can also leave your config on your $HOME with `~/.loc2gpx.yaml`.

Configuration example:
``` yaml
fields:
  device_id: dev_id
  time: dev_time
  coordinates: coords
  # route:
  # elevation:
  # speed:
segments:
  vw_simplify: 0.000001 # Tolerance for Visvalingam-Whyatt simplification algorithm
  max_segment_time: 300 # Max segment time(in seconds) allowed
```

## Help messages

General:
```
CLI of location2gpx - Convert your raw GPS data into a GPX file

USAGE:
    location2gpx <SUBCOMMAND>

OPTIONS:
    -h, --help    Print help information

SUBCOMMANDS:
    csv      Generate a GPX from a CSV file source
    help     Print this message or the help of the given subcommand(s)
    mongo    Generate a GPX from a mongodb collection source
```

Mongodb command:
```
Generate a GPX from a mongodb collection source

USAGE:
    location2gpx mongo [OPTIONS] <CONNECTION> <COLLECTION> <START> <END> <DESTINATION>

ARGS:
    <CONNECTION>     Mongo connection string source
    <COLLECTION>     Mongo collection name
    <START>          Start time, RFC3339 format
    <END>            End time, RFC3339 format
    <DESTINATION>    GPX path file destination

OPTIONS:
        --config <CONFIG>    Fields and segments configuration. Default: .loc2gpx.yaml, ~/.loc2gpx.yaml
    -h, --help               Print help information
```

CSV command:
```
Generate a GPX from a CSV file source

USAGE:
    location2gpx csv [OPTIONS] <CSV_PATH> <START> <END> <DESTINATION>

ARGS:
    <CSV_PATH>       CSV file source
    <START>          Start time, RFC3339 format
    <END>            End time, RFC3339 format
    <DESTINATION>    GPX path file destination

OPTIONS:
        --config <CONFIG>    Fields and segments configuration. Default: .loc2gpx.yaml, ~/.loc2gpx.yaml
    -h, --help               Print help information
```

## Goals

- [x] Generate tracks on a gpx file from a collection
- [x] Apply a simplification algorithm on track segments
- [x] Support devices and routes distinctions
- [x] Support mongodb source
- [x] Support CSV source
- [ ] Support firebird source
