# osu! Map Analyzer Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild the osu!standard beatmap analyzer around local timing and normalized geometry, with typed jump, stream, burst, slider, and tech results.

**Architecture:** `Analyzer` borrows a `rosu_map::Beatmap`, validates configuration and map mode, extracts an immutable internal feature set, then runs focused detectors over the same transitions. Detector results are assembled into stable public models and deterministic rankings.

**Tech Stack:** Rust 2021, `rosu-map` 0.2.1, optional `serde` 1.x, Cargo unit and integration tests.

**Spec:** `docs/superpowers/specs/2026-09-08-analyzer-rewrite-design.md`

## Global Constraints

- Change only `osu-map-analyzer-lib`.
- Support osu!standard maps only.
- Use local uninherited timing points for elapsed-beat calculations.
- Normalize spatial distance by circle radius.
- Count pattern lengths in objects, not adjacent edges.
- Do not require network access or external beatmap files in tests.
- Keep all normalized public scores in `0.0..=1.0`.
- Keep `serialize` as an alias for the new `serde` feature.
- Do not mutate the caller's beatmap.

---

### Task 1: Public API, configuration, and errors

**Files:**
- Modify: `Cargo.toml`
- Replace: `src/lib.rs`
- Create: `src/analyzer.rs`
- Create: `src/config.rs`
- Create: `src/error.rs`
- Create: `src/model.rs`
- Create: `tests/support/mod.rs`
- Create: `tests/api.rs`

**Interfaces:**
- Produces: `Analyzer<'map>`, `AnalysisConfig`, `AnalysisError`, `Pattern`, `Peak`, `PatternScore`, `MapAnalysis`, and the five per-pattern metric structs.
- Produces: `support::beatmap(timing_points: &[&str], hit_objects: &[String]) -> rosu_map::Beatmap` for later tests.

- [ ] **Step 1: Write failing API and validation tests**

```rust
#[test]
fn rejects_non_standard_maps() {
    let mut map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[support::circle(256, 192, 0)]);
    map.mode = rosu_map::section::general::GameMode::Mania;

    assert!(matches!(
        Analyzer::new(&map).analyze(),
        Err(AnalysisError::UnsupportedMode(_))
    ));
}

#[test]
fn rejects_an_empty_map() {
    let map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[]);
    assert_eq!(Analyzer::new(&map).analyze(), Err(AnalysisError::EmptyMap));
}

#[test]
fn rejects_non_positive_fast_interval() {
    let map = support::beatmap(&["0,500,4,2,1,50,1,0"], &[support::circle(256, 192, 0)]);
    let config = AnalysisConfig { fast_interval_ms: 0.0, ..AnalysisConfig::default() };

    assert!(matches!(
        Analyzer::with_config(&map, config),
        Err(AnalysisError::InvalidConfig("fast_interval_ms"))
    ));
}
```

- [ ] **Step 2: Run `cargo test --test api` and confirm unresolved imports fail**

Run: `cargo test --test api`

Expected: compilation fails because `Analyzer`, `AnalysisConfig`, and `AnalysisError` do not exist.

- [ ] **Step 3: Implement the public models and validation shell**

```rust
pub struct Analyzer<'map> {
    map: &'map Beatmap,
    config: AnalysisConfig,
}

impl<'map> Analyzer<'map> {
    pub fn new(map: &'map Beatmap) -> Self;
    pub fn with_config(map: &'map Beatmap, config: AnalysisConfig) -> Result<Self, AnalysisError>;
    pub fn analyze(&self) -> Result<MapAnalysis, AnalysisError>;
}
```

Use `AnalysisConfig::validate` for finite positive numeric fields, `stream_min_notes >= 4`, and `primary_score_min` in `0.0..=1.0`. Initially return an all-zero `MapAnalysis` after mode and object validation so the API tests pass.

- [ ] **Step 4: Run the API tests and full library tests**

Run: `cargo test --test api && cargo test --lib`

Expected: API tests pass. Remove the old print-only tests from `src/lib.rs`, so the library test target has zero failures.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src tests
git commit -m "feat: define analyzer API"
```

### Task 2: Local timing and immutable feature extraction

**Files:**
- Create: `src/features.rs`
- Modify: `src/analyzer.rs`
- Modify: `src/features.rs` to include its unit tests
- Modify: `tests/support/mod.rs`

**Interfaces:**
- Produces: `FeatureSet::extract(map: &Beatmap, config: &AnalysisConfig) -> FeatureSet`.
- Produces internally: ordered `ObjectFeature` values and `Transition` values with `delta_ms`, `elapsed_beats`, `normalized_distance`, `angle`, `section`, and sequence boundaries.
- Consumes: `AnalysisConfig` from Task 1.

- [ ] **Step 1: Write failing behavior tests**

```rust
#[cfg(test)]
mod tests {
#[test]
fn integrates_beats_across_a_timing_change() {
    let map = support::beatmap(
        &["0,500,4,2,1,50,1,0", "1000,250,4,2,1,50,1,0"],
        &[support::circle(64, 192, 750), support::circle(128, 192, 1125)],
    );

    let features = FeatureSet::extract(&map, &AnalysisConfig::default());
    assert!((features.transitions[0].elapsed_beats - 1.0).abs() < 1e-9);
}

#[test]
fn normalizes_distance_by_circle_radius() {
    let mut map = support::beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[support::circle(0, 192, 0), support::circle(100, 192, 250)],
    );
    map.circle_size = 4.0;

    let features = FeatureSet::extract(&map, &AnalysisConfig::default());
    assert!((features.transitions[0].normalized_distance - 2.739_726).abs() < 1e-5);
}

#[test]
fn spinner_breaks_a_transition_sequence() {
    let map = support::beatmap(
        &["0,500,4,2,1,50,1,0"],
        &[
            support::circle(64, 192, 0),
            support::spinner(125, 375),
            support::circle(128, 192, 500),
        ],
    );
    assert!(FeatureSet::extract(&map, &AnalysisConfig::default()).transitions.is_empty());
}
}
```

- [ ] **Step 2: Run `cargo test features::tests` and confirm the module is missing**

- [ ] **Step 3: Implement extraction**

Sort object references by finite `start_time`, increment a sequence id on spinners, holds, or invalid values, calculate slider path distance through `borrowed_curve`, and integrate beats piecewise:

```rust
fn elapsed_beats(points: &[TimingPoint], start: f64, end: f64) -> f64 {
    let mut cursor = start;
    let mut beat_len = active_beat_len(points, start);
    let mut beats = 0.0;

    for point in points.iter().filter(|point| point.time > start && point.time < end) {
        beats += (point.time - cursor) / beat_len;
        cursor = point.time;
        beat_len = point.beat_len;
    }

    beats + (end - cursor) / beat_len
}
```

- [ ] **Step 4: Run `cargo test features::tests` and `cargo test --test api`**

- [ ] **Step 5: Commit**

```bash
git add src tests
git commit -m "feat: extract local timing and geometry"
```

### Task 3: Stream and burst detection

**Files:**
- Create: `src/patterns/mod.rs`
- Create: `src/patterns/tap.rs`
- Modify: `src/analyzer.rs`
- Create: `tests/tap_patterns.rs`

**Interfaces:**
- Produces: `tap::detect(features: &FeatureSet, config: &AnalysisConfig) -> TapDetection`.
- `TapDetection` provides public `StreamAnalysis` and `BurstAnalysis` plus a boolean vector identifying transitions that belong to streams.

- [ ] **Step 1: Write failing stream and burst tests**

```rust
#[test]
fn counts_notes_instead_of_edges() {
    let map = support::circle_run(7, 125.0, 40.0);
    let result = Analyzer::new(&map).analyze().unwrap();
    assert_eq!(result.stream.longest, 7);
    assert_eq!(result.stream.notes, 7);
    assert_eq!(result.stream.segments, 1);
}

#[test]
fn reports_a_240_note_stream_without_capping_it() {
    let map = support::circle_run(240, 100.0, 24.0);
    assert_eq!(Analyzer::new(&map).analyze().unwrap().stream.longest, 240);
}

#[test]
fn classifies_three_to_five_notes_as_bursts() {
    let map = support::separated_circle_runs(&[3, 4, 5], 125.0, 1000.0);
    let result = Analyzer::new(&map).analyze().unwrap();
    assert_eq!(result.burst.segments, 3);
    assert_eq!(result.burst.notes, 12);
    assert_eq!(result.stream.segments, 0);
}
```

- [ ] **Step 2: Run `cargo test --test tap_patterns` and confirm zero-valued placeholder results fail**

- [ ] **Step 3: Implement fast-edge grouping and scores**

An edge is fast when it joins circles in one sequence, `delta_ms <= fast_interval_ms`, `elapsed_beats <= fast_interval_beats`, and its beat duration differs from the prior fast edge by no more than `rhythm_tolerance`. Convert an edge run to `edge_count + 1` notes. Build disjoint stream and burst runs and record participating objects for peak calculations.

- [ ] **Step 4: Run `cargo test --test tap_patterns && cargo test`**

- [ ] **Step 5: Commit**

```bash
git add src tests
git commit -m "feat: detect streams and bursts"
```

### Task 4: Jump detection without deathstream false positives

**Files:**
- Create: `src/patterns/jump.rs`
- Modify: `src/patterns/mod.rs`
- Modify: `src/analyzer.rs`
- Create: `tests/jumps.rs`

**Interfaces:**
- Produces: `jump::detect(features: &FeatureSet, stream_edges: &[bool], config: &AnalysisConfig) -> JumpAnalysis`.

- [ ] **Step 1: Write failing jump tests**

```rust
#[test]
fn spaced_deathstream_is_not_a_jump_sequence() {
    let map = support::circle_run(240, 100.0, 120.0);
    let result = Analyzer::new(&map).analyze().unwrap();
    assert_eq!(result.stream.longest, 240);
    assert_eq!(result.jump.segments, 0);
}

#[test]
fn detects_a_quarter_beat_jump_sequence() {
    let map = support::circle_run(12, 250.0, 180.0);
    let result = Analyzer::new(&map).analyze().unwrap();
    assert_eq!(result.jump.longest, 12);
    assert!(result.jump.average_distance > 4.0);
}

#[test]
fn the_same_pixel_distance_changes_with_circle_size() {
    let small_circles = support::jump_pair_with_cs(5.0, 100.0);
    let large_circles = support::jump_pair_with_cs(2.0, 100.0);
    assert!(Analyzer::new(&small_circles).analyze().unwrap().jump.score
        > Analyzer::new(&large_circles).analyze().unwrap().jump.score);
}
```

- [ ] **Step 2: Run `cargo test --test jumps` and confirm jump metrics fail**

- [ ] **Step 3: Implement jump edge grouping**

Require normalized distance, maximum milliseconds, and maximum beats. Exclude every stream-owned edge before grouping. Slider heads are allowed, while feature sequence ids preserve spinner and hold boundaries. Scores combine participating-object density, average normalized distance, and longest sequence.

- [ ] **Step 4: Run `cargo test --test jumps && cargo test`**

- [ ] **Step 5: Commit**

```bash
git add src tests
git commit -m "feat: detect normalized jump sequences"
```

### Task 5: Slider, tech, peaks, and deterministic ranking

**Files:**
- Create: `src/patterns/slider.rs`
- Create: `src/patterns/tech.rs`
- Modify: `src/patterns/mod.rs`
- Modify: `src/analyzer.rs`
- Create: `tests/classification.rs`

**Interfaces:**
- Produces: slider and tech detectors returning their public metric structs and participating object times.
- Produces: `peak_for(times: &[f64], features: &FeatureSet, window_ms: f64) -> Option<Peak>`.
- Produces: sorted `Vec<PatternScore>` and `MapAnalysis::primary`.

- [ ] **Step 1: Write failing classification tests**

```rust
#[test]
fn slider_metrics_include_repeats_travel_and_duration() {
    let map = support::slider_map();
    let result = Analyzer::new(&map).analyze().unwrap();
    assert_eq!(result.slider.count, 3);
    assert_eq!(result.slider.repeats, 2);
    assert!(result.slider.travel_distance > 0.0);
    assert!(result.slider.duration_ratio > 0.0);
}

#[test]
fn tech_requires_combined_rhythm_and_geometry_evidence() {
    let simple = support::circle_run(16, 250.0, 180.0);
    let technical = support::technical_map();
    assert!(Analyzer::new(&technical).analyze().unwrap().tech.score
        > Analyzer::new(&simple).analyze().unwrap().tech.score);
}

#[test]
fn rankings_are_descending_and_ties_are_stable() {
    let result = Analyzer::new(&support::mixed_map()).analyze().unwrap();
    assert!(result.patterns.windows(2).all(|pair| pair[0].score >= pair[1].score));
    assert_eq!(result.primary, Some(result.patterns[0].pattern));
    assert!(result.patterns.iter().all(|entry| (0.0..=1.0).contains(&entry.score)));
}
```

- [ ] **Step 2: Run `cargo test --test classification` and confirm missing detector behavior fails**

- [ ] **Step 3: Implement remaining detectors and assembly**

Slider score combines object share, duration share, normalized travel, and repeats. Tech score combines quantized rhythm changes, non-trivial movement angles, slider share, and meaningful slider-velocity changes. Build peak section occupancy from participating times, assemble all five `PatternScore` entries, sort by `score.total_cmp` descending and `Pattern` ascending on ties, and apply `primary_score_min`.

- [ ] **Step 4: Run `cargo test --test classification && cargo test --all-features`**

- [ ] **Step 5: Commit**

```bash
git add src tests
git commit -m "feat: classify slider and tech patterns"
```

### Task 6: Serde, rustdoc, README, and quality gates

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Modify: all public model modules requiring rustdoc
- Replace: `README.md`
- Create: `tests/serde.rs`

**Interfaces:**
- Consumes: all public types from Tasks 1 through 5.
- Produces: feature-gated JSON serialization and user-facing crate documentation.

- [ ] **Step 1: Write the failing serde round-trip test**

```rust
#[cfg(feature = "serde")]
#[test]
fn map_analysis_round_trips_through_json() {
    let map = support::circle_run(12, 125.0, 40.0);
    let analysis = Analyzer::new(&map).analyze().unwrap();
    let json = serde_json::to_string(&analysis).unwrap();
    assert_eq!(serde_json::from_str::<MapAnalysis>(&json).unwrap(), analysis);
}
```

- [ ] **Step 2: Run `cargo test --all-features --test serde` and confirm derives are missing**

- [ ] **Step 3: Add conditional derives and rewrite documentation**

Use `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` on every public result and enum. Document score semantics, supported mode, defaults, errors, a complete `Analyzer` example, and migration from the old `Stream` and `Jump` entry points.

- [ ] **Step 4: Run all final quality gates**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
```

Expected: every command exits 0 with no warnings.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock README.md src tests
git commit -m "docs: finish analyzer rewrite"
```
