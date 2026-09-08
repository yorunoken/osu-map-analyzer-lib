[![Crates.io](https://img.shields.io/crates/v/osu-map-analyzer.svg)](https://crates.io/crates/osu-map-analyzer)
[![Documentation](https://docs.rs/osu-map-analyzer/badge.svg)](https://docs.rs/osu-map-analyzer)

# osu-map-analyzer

A deterministic Rust library for classifying patterns in osu!standard beatmaps.

The analyzer uses timing changes at each object, distances normalized by circle
size, slider travel, rhythm variation, movement angles, and fixed-duration peak
windows. It reports comparable jump, stream, burst, slider, and tech scores plus
the measurements behind each score.

## Installation

```toml
[dependencies]
osu-map-analyzer = "0.3"
```

Enable `serde` when analysis results need to be serialized:

```toml
[dependencies]
osu-map-analyzer = { version = "0.3", features = ["serde"] }
```

The legacy `serialize` feature remains an alias for `serde`.

## Usage

```rust,no_run
use osu_map_analyzer::{rosu_map::Beatmap, Analyzer, Pattern};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let map = Beatmap::from_path("path/to/beatmap.osu")?;
    let analysis = Analyzer::new(&map).analyze()?;

    if analysis.primary == Some(Pattern::Stream) {
        println!("longest stream: {} notes", analysis.stream.longest);
    }

    for entry in &analysis.patterns {
        println!("{:?}: {:.3}", entry.pattern, entry.score);
    }

    Ok(())
}
```

Use `Analyzer::with_config` to tune thresholds while preserving validation:

```rust,no_run
use osu_map_analyzer::{rosu_map::Beatmap, AnalysisConfig, Analyzer};

# fn example(map: &Beatmap) -> Result<(), Box<dyn std::error::Error>> {
let config = AnalysisConfig {
    stream_min_notes: 8,
    jump_min_distance: 3.0,
    ..AnalysisConfig::default()
};

let analysis = Analyzer::with_config(map, config)?.analyze()?;
# Ok(())
# }
```

Default thresholds:

| Setting | Default | Meaning |
| --- | ---: | --- |
| `fast_interval_ms` | 200 ms | Fast-tap wall-clock ceiling |
| `fast_interval_beats` | 0.375 beats | Fast-tap local beat ceiling |
| `rhythm_tolerance` | 0.2 | Relative interval variation inside a tap run |
| `stream_min_notes` | 6 | Notes required for a stream |
| `jump_min_distance` | 2.5 radii | Minimum normalized jump spacing |
| `jump_max_interval_ms` | 600 ms | Jump wall-clock ceiling |
| `jump_max_interval_beats` | 1.25 beats | Jump local beat ceiling |
| `peak_window_ms` | 2000 ms | Fixed peak-section duration |
| `primary_score_min` | 0.15 | Minimum score for a primary pattern |

## Result model

Every pattern score is clamped to `0.0..=1.0` and ranked in descending order.
These values are deterministic heuristic strengths, not statistical confidence
or difficulty ratings. Scores are best used to compare characteristics within a
map or across maps analyzed with the same crate version and configuration.

`Peak` identifies the strongest configured time window for each detected
pattern. Detailed result structs also expose counts, run lengths, normalized
distances, intervals, slider travel, and complexity components.

Only osu!standard is supported. Other game modes return
`AnalysisError::UnsupportedMode`. A map without playable circles or sliders
returns `AnalysisError::EmptyMap`, and invalid custom thresholds return
`AnalysisError::InvalidConfig`.

## Version 0.3 migration

The former mutable `analyze::Stream` and `analyze::Jump` entry points have been
replaced by one borrowed, immutable `Analyzer`. The new result is structured,
fully ranked, optionally serializable, and includes burst, slider, and tech
analysis.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

## Acknowledgements

Beatmap parsing is provided by [rosu-map](https://github.com/MaxOhn/rosu-map).
