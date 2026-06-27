[![Crates.io](https://img.shields.io/crates/v/osu-map-analyzer.svg)](https://crates.io/crates/osu-map-analyzer)
[![Documentation](https://docs.rs/osu-map-analyzer/badge.svg)](https://docs.rs/osu-map-analyzer)

# osu-map-analyzer

A reusable feature extraction library for osu! beatmaps. It parses `.osu` files through
[`rosu-map`](https://github.com/MaxOhn/rosu-map) and produces human-readable metrics,
timeline sections, deterministic map tags, and a stable flat feature vector.

## Installation

```toml
[dependencies]
osu-map-analyzer = "0.2.9"
```

JSON serialization is enabled by default for all analysis output types. It can be
disabled with `default-features = false` if a consumer does not need it.

```toml
osu-map-analyzer = { version = "0.2.9", default-features = false }
```

## Structured analysis

```rust,no_run
use osu_map_analyzer::{AnalysisOptions, BeatmapAnalyzer};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let analysis = BeatmapAnalyzer::from_path("map.osu")?
    .analyze(AnalysisOptions::default())?;

println!("main BPM: {}", analysis.timing.main_bpm);
println!("tags: {:?}", analysis.tags);

let features = analysis.to_feature_vector();
assert_eq!(features.names.len(), features.values.len());
# Ok(())
# }
```

`AnalysisOptions::default()` uses 10-second timeline sections. Times and durations in
the result are milliseconds; rates such as `notes_per_second` are per second.

The result contains:

- metadata, difficulty settings, drain time, total length, and object counts
- weighted/main/min/max BPM and timing/SV changes
- object ratios, density, spacing, and jump-distance distributions
- rhythm entropy, interval frequencies, burstiness, and repeated patterns
- aim, speed, stamina, stream/burst, and slider metrics
- configurable chart-ready timeline sections
- deterministic tags and a named, ordered feature vector

Pressure, complexity, and tag values are intentionally heuristic. They are stable
feature signals, not replacements for osu! difficulty or performance calculations.

Analyze a map from the command line:

```bash
cargo run -- path/to/map.osu
cargo run -- --details path/to/map.osu
cargo run -- --json path/to/map.osu
```

Install the standalone CLI from crates.io with `cargo install osu-map-analyzer`, then
run the same commands through `osu-map-analyzer` directly.

Compact human output is the default. `--details` prints grouped metrics and only the
top five speed, aim, and density sections. Raw arrays are emitted only with `--json`.

Analysis results also provide `compact_report()`, `detailed_report()`, and `validate()`.
Validation checks structural consistency, finite values, section ordering and coverage,
documented pressure-score ranges, and redundant adjacent BPM sections.

## Existing analyzers

The original `analyze::Jump` and `analyze::Stream` APIs remain available for callers
that use their classification-oriented output.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).
