[![Crates.io](https://img.shields.io/crates/v/osu-map-analyzer.svg)](https://crates.io/crates/osu-map-analyzer)
[![Documentation](https://docs.rs/osu-map-analyzer/badge.svg)](https://docs.rs/osu-map-analyzer)

# osu-map-analyzer

A Rust library for analyzing osu! beatmaps, and determines what class it is (stream, jump, tech, etc).

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
osu-map-analyzer = "0.2.9"
```

or simply run `cargo add osu-map-analyzer` for the latest version

## Usage

Here's a basic example of how to use the library:

```rust
use std::path::Path;
use osu_map_analyzer::{analyze, rosu_map};

fn main() {
    let path = Path::new("path/to/your/beatmap.osu");
    let map = rosu_map::from_path::<rosu_map::Beatmap>(path).unwrap();

    let mut stream_analyzer = analyze::Stream::new(map.clone());
    let stream_analysis = stream_analyzer.analyze();
    println!("Stream analysis: {:#?}", stream_analysis);

    let mut jump_analyzer = analyze::Jump::new(map);
    let jump_analysis = jump_analyzer.analyze();
    println!("Jump analysis: {:#?}", jump_analysis);
}
```

## Analysis Metrics

### Stream Analysis

- `overall_confidence`: How confident the library is that it's a stream map
- `short_streams`: Number of short streams (6-9 notes)
- `medium_streams`: Number of medium streams (10-19 notes)
- `long_streams`: Number of long streams (20+ notes)
- `max_stream_length`: Length of the longest stream
- `stream_density`: Density of streams in the map
- `bpm_consistency`: Consistency of the BPM in streams (this doesn't really mean much)

### Jump Analysis

- `overall_confidence`: How confident the library is that it's a jump map
- `total_jump_count`: Total number of jumps
- `max_jump_length`: Length of the longest jump sequence
- `long_jumps`: Number of long jumps (12+ notes)
- `medium_jumps`: Number of medium jumps (7-11 notes)
- `short_jumps`: Number of short jumps (4-6 notes)
- `jump_density`: Density of jumps in the map
- `bpm_consistency`: Consistency of the BPM in jumps (this doesn't really mean much)

## License

This project is licensed under the Apache License, Version 2.0 - see [LICENSE](LICENSE) for details.

## Acknowledgements

This library uses [rosu-map](https://github.com/MaxOhn/rosu-map) for parsing osu! beatmaps.
