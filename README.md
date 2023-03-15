# location2gpx

GPX file generator from consolidated sources.

Visualize or manipulate your tracks with a `.gpx` file from raw data in mongodb collections or CSV.

## How to use

Basic utilization:

``` bash
cargo run -- "mongodb://localhost:27017/yourdb" yourcollection "2020-01-01T00:00:00.000+00:00" "2020-12-31T00:00:00.000+00:00" /tmp/my-tracks-2020.gpx
```

Output will be like this [sample](https://github.com/fernandobatels/location2gpx/blob/main/samples/simple.gpx).

Help message:
```
CLI of location2gpx - Convert your raw GPS data into a GPX file

USAGE:
    location2gpx [OPTIONS] <CONNECTION> <COLLECTION> <START> <END> <DESTINATION>

ARGS:
    <CONNECTION>     Mongo connection string source
    <COLLECTION>     Mongo collection name
    <START>          Start time, RFC3339 format
    <END>            End time, RFC3339 format
    <DESTINATION>    GPX path file destination

OPTIONS:
        --field-coordinates <FIELD_COORDINATES>
        --field-device <FIELD_DEVICE>
        --field-elevation <FIELD_ELEVATION>
        --field-route <FIELD_ROUTE>
        --field-speed <FIELD_SPEED>
        --field-time <FIELD_TIME>
        --flip-field-coordinates <FLIP_FIELD_COORDINATES>
    -h, --help
            Print help information
        --max-seg-time <MAX_SEG_TIME>
            Max segment time(in seconds) allowed
        --simplify <SIMPLIFY>
            Enable the simplify Visvalingam-Whyatt algorithm providing the tolerance
```

## Goals

- [x] Generate tracks on a gpx file from a collection
- [x] Apply a simplification algorithm on track segments
- [x] Support devices and routes distinctions
- [x] Support mongodb source
- [ ] Support CSV source
- [ ] Support firebird source
