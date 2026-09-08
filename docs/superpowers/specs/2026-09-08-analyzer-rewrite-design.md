# osu! Map Analyzer Rewrite Design

## Goal

Replace the existing global-BPM stream and jump heuristics with a deterministic, local-timing-aware analyzer for osu!standard beatmaps. The new library must classify jump, stream, burst, slider, and tech characteristics, expose useful supporting metrics and peak locations, and ship with tests that contain real assertions and no external beatmap files.

## Scope

This rewrite changes only `osu-map-analyzer-lib`. It does not modify the website or publish a crate release. The crate will target osu!standard maps and return a typed error for other game modes or maps without playable objects.

The existing pre-1.0 API may break. The crate version will become `0.3.0`, and the new API will be the supported interface.

## Considered Approaches

### Selected: deterministic feature extraction

Parse the map with `rosu-map`, turn adjacent playable objects into timing and geometry transitions, group those transitions into patterns, and derive normalized scores from observable participation and intensity. This keeps the crate small, fast, explainable, and compatible with the beatmap type it already exposes.

### Rejected: use `rosu-pp` strain output

`rosu-pp` has strong aim and speed calculations, but its current public beatmap representation differs from `rosu-map::Beatmap`. Using it would require callers to change parser types or the analyzer to encode and parse every map twice. Its section strains also do not directly identify stream, burst, slider, or tech segments.

### Rejected: trained classifier

A trained model could eventually outperform fixed heuristics, but the project has no versioned, labeled dataset or evaluation harness. Adding a model now would make results opaque and difficult to reproduce without evidence that they are more accurate.

## Public API

The crate will expose:

- `Analyzer::new(&Beatmap)` and `Analyzer::with_config(&Beatmap, AnalysisConfig)`.
- `Analyzer::analyze() -> Result<MapAnalysis, AnalysisError>`.
- `AnalysisConfig` with validated defaults for fast-note timing, stream length, jump spacing, jump timing, and peak section length.
- `Pattern` with `Jump`, `Stream`, `Burst`, `Slider`, and `Tech` variants.
- `MapAnalysis` containing the optional primary pattern, a ranked list of all pattern scores, per-pattern metric structs, object totals, and map duration.
- `PatternScore` containing a pattern, normalized score in `0.0..=1.0`, participating object count, segment count, and strongest section.
- `Peak` containing a section start time and normalized score.
- `AnalysisError` for unsupported game modes, empty maps, and invalid configuration.
- A `serde` feature that enables both serialization and deserialization. The old `serialize` feature remains as an alias for compatibility.
- A re-export of `rosu_map` so callers can use the exact accepted beatmap type.

Every public result derives `Debug`, `Clone`, and `PartialEq`. Small value types also derive `Copy`. Pattern ordering is deterministic: descending score, then the declared `Pattern` order for ties.

## Feature Extraction

Analysis orders hit objects once and builds a cumulative timing lookup once.
Each transition then resolves its local beat position with binary search instead
of rescanning every timing point.

Playable objects are circles and slider heads. Spinners and mania holds break consecutive patterns and do not contribute to tap counts. Each playable object records start time, position, kind, slider duration, repeat count, path distance, and active slider velocity.

Each adjacent playable pair becomes a transition only when no spinner or hold occurs between them. A transition records elapsed milliseconds, elapsed beats, normalized distance, and turning angle when a previous transition exists. Playable objects retain sequence membership, and fixed peak-section counts are collected during extraction.

Elapsed beats are integrated across every uninherited timing point crossed by the interval. This is the central correction over the current implementation, which assumes one dominant BPM for the whole map.

Distances are divided by the map's circle radius. This makes jump thresholds behave consistently across circle sizes.

## Pattern Detection

### Streams and bursts

A fast edge must join two circles, fit both the configured millisecond and beat-duration ceilings, and be rhythmically close to the previous fast edge. Consecutive fast edges form one tap run. A run containing at least the configured stream note count is a stream. Runs of three up to one fewer than the stream minimum are bursts.

Run lengths count objects, not edges. Two adjacent transitions therefore represent three notes. This fixes the existing longest-stream off-by-one error.

### Jumps

A jump edge must meet the circle-radius-normalized distance threshold and the configured maximum elapsed beats and milliseconds. Edges already belonging to a stream are excluded from jump sequences so a spaced deathstream is not simultaneously classified as a full jump map. Slider heads may participate in jumps, while spinners and holds split sequences.

### Sliders

Slider evidence combines slider share, time spent holding sliders, repeats, and total normalized path travel. The output includes each of those raw measurements.

### Tech

Tech evidence combines rhythm changes, changes in movement angle, slider share, and slider-velocity changes. No one component can produce a maximum score. The output exposes every component so callers can explain the result.

## Scores and Primary Pattern

Scores are bounded in `0.0..=1.0` and represent strength within the analyzed beatmap, not statistical certainty. Each detector combines object participation with pattern-specific intensity. A pattern below the configured primary-score floor cannot become the primary pattern. It still appears in the ranked results with its measured score.

Peak scores use fixed-duration sections. Each participating transition or slider contributes evidence to the section containing its start time. The peak identifies the strongest section for future visualizations without returning a large time series.

## Errors and Edge Cases

- Reject non-osu!standard modes before feature extraction.
- Reject maps without circles or sliders.
- Validate all configurable thresholds for finiteness, positivity, and valid ordering.
- Sort a copied list of object indices by start time if input objects are not ordered; never mutate the caller's map.
- Clamp all public normalized values and prevent division by zero.
- Treat missing timing points as the `rosu-map` default 60 BPM timing point.
- Treat non-finite object data as a pattern break rather than allowing NaN into public results.

## Testing

Tests build compact `.osu` documents in memory and parse them with the real `rosu-map` parser. They cover:

- stream and burst run lengths, including the edge-to-note off-by-one regression;
- long streams beyond 199 notes;
- BPM changes crossed within a run;
- spaced deathstreams not becoming jump sequences;
- circle-size-normalized jump detection;
- spinner and slider boundaries;
- slider count, duration, repeats, travel, and peak section;
- tech rhythm, angle, and slider-velocity components;
- deterministic ranking and score bounds;
- unsupported modes, empty maps, invalid configuration, and unsorted objects;
- serialization when the `serde` feature is enabled.

Quality gates are `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-features`, and `cargo doc --all-features --no-deps` with warnings denied.

## Documentation

The README will explain what each score means, show the new API, document supported modes and features, and explicitly state that classification is heuristic. Public types and methods will have rustdoc examples or focused field documentation.
