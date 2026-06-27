use std::{collections::HashMap, fmt, io, path::Path};

use rosu_map::{
    section::{general::GameMode as RosuGameMode, hit_objects::HitObjectKind},
    Beatmap,
};

use super::model::*;

/// Parses and extracts reusable analysis features from an osu! beatmap.
pub struct BeatmapAnalyzer {
    map: Beatmap,
}

#[derive(Debug)]
pub enum AnalysisError {
    Parse(io::Error),
    InvalidSectionLength(f64),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "failed to parse beatmap: {err}"),
            Self::InvalidSectionLength(value) => {
                write!(f, "section length must be finite and positive, got {value}")
            }
        }
    }
}

impl std::error::Error for AnalysisError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            Self::InvalidSectionLength(_) => None,
        }
    }
}

impl From<io::Error> for AnalysisError {
    fn from(value: io::Error) -> Self {
        Self::Parse(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Circle,
    Slider,
    Spinner,
    Hold,
}

#[derive(Clone, Copy)]
struct ObjectSample {
    start: f64,
    end: f64,
    kind: Kind,
    slider_duration: f64,
    slider_complexity: f64,
}

impl BeatmapAnalyzer {
    /// Parses a beatmap from a `.osu` file.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, AnalysisError> {
        Ok(Self {
            map: Beatmap::from_path(path)?,
        })
    }

    /// Parses a beatmap from its raw `.osu` contents.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, AnalysisError> {
        Ok(Self {
            map: Beatmap::from_bytes(bytes)?,
        })
    }

    /// Wraps an already parsed beatmap.
    pub fn from_beatmap(map: Beatmap) -> Self {
        Self { map }
    }

    /// Returns the underlying parsed beatmap.
    pub fn beatmap(&self) -> &Beatmap {
        &self.map
    }

    /// Extracts structured metrics, sections, tags, and ML-ready features.
    pub fn analyze(&self, options: AnalysisOptions) -> Result<BeatmapAnalysis, AnalysisError> {
        if !options.section_length.is_finite() || options.section_length <= 0.0 {
            return Err(AnalysisError::InvalidSectionLength(options.section_length));
        }

        let objects = collect_objects(&self.map);
        let starts: Vec<f64> = objects.iter().map(|o| o.start).collect();
        let gaps: Vec<f64> = starts.windows(2).map(|w| w[1] - w[0]).collect();
        let distances = object_distances(&self.map);
        let total_length = objects.iter().map(|o| o.end).fold(0.0, f64::max).max(0.0);
        let first_time = starts.first().copied().unwrap_or(0.0).max(0.0);
        let playable_length = (total_length - first_time).max(0.0);
        let drain_time =
            (playable_length - break_duration(&self.map, first_time, total_length)).max(0.0);
        let duration_seconds = playable_length / 1000.0;

        let metadata = metadata(&self.map);
        let general = GeneralStats {
            approach_rate: self.map.approach_rate,
            overall_difficulty: self.map.overall_difficulty,
            circle_size: self.map.circle_size,
            hp_drain_rate: self.map.hp_drain_rate,
            drain_time,
            total_length,
            object_count: objects.len(),
        };
        let timing = timing_analysis(&self.map, total_length);
        let object_analysis = object_analysis(&objects, &distances, duration_seconds);
        let rhythm = rhythm_analysis(&gaps);
        let (aim, aim_pressures) = aim_analysis(&self.map, &distances, &gaps);
        let speed_pressures: Vec<f64> = gaps
            .iter()
            .map(|gap| (200.0 / gap).clamp(0.0, 4.0))
            .collect();
        let slider_pressures: Vec<f64> = objects
            .iter()
            .map(|o| {
                if o.kind == Kind::Slider {
                    (o.slider_complexity / 3.0).clamp(0.25, 3.0)
                } else {
                    0.0
                }
            })
            .collect();
        let streams = stream_analysis(&self.map, &distances);
        let sections = section_analysis(
            &objects,
            &gaps,
            &distances,
            &aim_pressures,
            &speed_pressures,
            &slider_pressures,
            total_length,
            options.section_length,
        );
        let speed = speed_analysis(&starts, &gaps, &speed_pressures, &sections);
        let sliders = slider_analysis(&self.map, &objects, duration_seconds, &slider_pressures);
        let tags = tags(&object_analysis, &rhythm, &aim, &speed, &streams, &sliders);

        Ok(BeatmapAnalysis {
            metadata,
            general,
            timing,
            objects: object_analysis,
            rhythm,
            aim,
            speed,
            streams,
            sliders,
            sections,
            tags,
        })
    }
}

fn collect_objects(map: &Beatmap) -> Vec<ObjectSample> {
    let mut cloned = map.hit_objects.clone();
    cloned
        .iter_mut()
        .map(|object| {
            let start = object.start_time;
            let end = object.end_time();
            let (kind, slider_duration, slider_complexity) = match &object.kind {
                HitObjectKind::Circle(_) => (Kind::Circle, 0.0, 0.0),
                HitObjectKind::Slider(v) => (
                    Kind::Slider,
                    end - start,
                    v.path.control_points().len() as f64 + v.repeat_count.max(0) as f64,
                ),
                HitObjectKind::Spinner(_) => (Kind::Spinner, 0.0, 0.0),
                HitObjectKind::Hold(_) => (Kind::Hold, 0.0, 0.0),
            };
            ObjectSample {
                start,
                end,
                kind,
                slider_duration,
                slider_complexity,
            }
        })
        .collect()
}

fn metadata(map: &Beatmap) -> BeatmapMetadata {
    BeatmapMetadata {
        mode: match map.mode {
            RosuGameMode::Osu => GameMode::Osu,
            RosuGameMode::Taiko => GameMode::Taiko,
            RosuGameMode::Catch => GameMode::Catch,
            RosuGameMode::Mania => GameMode::Mania,
        },
        title: map.title.clone(),
        artist: map.artist.clone(),
        creator: map.creator.clone(),
        version: map.version.clone(),
        beatmap_id: (map.beatmap_id > 0).then_some(map.beatmap_id),
        beatmap_set_id: (map.beatmap_set_id > 0).then_some(map.beatmap_set_id),
    }
}

fn break_duration(map: &Beatmap, start: f64, end: f64) -> f64 {
    map.breaks
        .iter()
        .map(|b| {
            let break_start = b.start_time.max(start);
            let break_end = b.end_time.min(end);
            (break_end - break_start).max(0.0)
        })
        .sum()
}

fn timing_analysis(map: &Beatmap, total_length: f64) -> TimingAnalysis {
    let points = &map.control_points.timing_points;
    let mut sections = Vec::with_capacity(points.len());
    let mut weighted_bpm = 0.0;
    let mut weighted_duration = 0.0;
    let mut durations: HashMap<u64, f64> = HashMap::new();
    let mut bpms = Vec::with_capacity(points.len());

    for (index, point) in points.iter().enumerate() {
        let bpm = 60_000.0 / point.beat_len;
        let end = points
            .get(index + 1)
            .map_or(total_length.max(point.time), |p| p.time);
        let duration = (end - point.time.max(0.0)).max(0.0);
        sections.push(TimingSection {
            start_time: point.time,
            end_time: end,
            bpm,
        });
        bpms.push(bpm);
        weighted_bpm += bpm * duration;
        weighted_duration += duration;
        *durations
            .entry((point.beat_len * 1000.0).round().to_bits())
            .or_default() += duration;
    }
    let main_bpm = durations
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(bits, _)| 60_000.0 / (f64::from_bits(bits) / 1000.0))
        .unwrap_or(0.0);
    let average_bpm = if weighted_duration > 0.0 {
        weighted_bpm / weighted_duration
    } else {
        main_bpm
    };
    let min_bpm = bpms.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let max_bpm = bpms.iter().copied().reduce(f64::max).unwrap_or(0.0);
    let velocity_changes = map
        .control_points
        .difficulty_points
        .windows(2)
        .filter(|w| (w[1].slider_velocity - w[0].slider_velocity).abs() > 1e-9)
        .count();

    TimingAnalysis {
        main_bpm,
        average_bpm,
        min_bpm,
        max_bpm,
        bpm_changes: points
            .windows(2)
            .filter(|window| (window[0].beat_len - window[1].beat_len).abs() > 1e-9)
            .count(),
        inherited_timing_points: map
            .control_points
            .difficulty_points
            .iter()
            .filter(|difficulty| {
                !points
                    .iter()
                    .any(|timing| (timing.time - difficulty.time).abs() < f64::EPSILON)
            })
            .count(),
        slider_velocity_changes: velocity_changes,
        sections,
    }
}

fn object_distances(map: &Beatmap) -> Vec<Option<f64>> {
    map.hit_objects
        .windows(2)
        .map(|window| {
            let (x1, y1) = position(&window[0])?;
            let (x2, y2) = position(&window[1])?;

            Some((x2 - x1).hypot(y2 - y1))
        })
        .collect()
}

fn object_analysis(
    objects: &[ObjectSample],
    distances: &[Option<f64>],
    duration_seconds: f64,
) -> ObjectAnalysis {
    let circles = objects.iter().filter(|o| o.kind == Kind::Circle).count();
    let sliders = objects.iter().filter(|o| o.kind == Kind::Slider).count();
    let spinners = objects.iter().filter(|o| o.kind == Kind::Spinner).count();
    let holds = objects.iter().filter(|o| o.kind == Kind::Hold).count();
    let total = objects.len() as f64;
    let positional_distances: Vec<f64> = distances.iter().flatten().copied().collect();
    let stats = distribution(&positional_distances);
    ObjectAnalysis {
        circle_count: circles,
        slider_count: sliders,
        spinner_count: spinners,
        hold_count: holds,
        circle_ratio: ratio(circles, total),
        slider_ratio: ratio(sliders, total),
        spinner_ratio: ratio(spinners, total),
        hold_ratio: ratio(holds, total),
        object_density: safe_div(total, duration_seconds),
        notes_per_second: safe_div(total, duration_seconds),
        average_spacing: stats.mean,
        spacing_variance: stats.standard_deviation.powi(2),
        jump_distances: stats,
    }
}

fn rhythm_analysis(gaps: &[f64]) -> RhythmAnalysis {
    let gaps: Vec<f64> = gaps
        .iter()
        .copied()
        .filter(|gap| gap.is_finite() && *gap > 0.0)
        .collect();
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for gap in &gaps {
        *counts.entry((gap / 10.0).round() as i64 * 10).or_default() += 1;
    }
    let mut common: Vec<_> = counts
        .iter()
        .map(|(interval, count)| IntervalFrequency {
            interval: *interval as f64,
            count: *count,
        })
        .collect();
    common.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| a.interval.total_cmp(&b.interval))
    });
    common.truncate(8);
    let total = gaps.len() as f64;
    let entropy = counts
        .values()
        .map(|count| {
            let p = *count as f64 / total.max(1.0);
            if p > 0.0 {
                -p * p.log2()
            } else {
                0.0
            }
        })
        .sum();
    let stats = distribution(&gaps);
    let burstiness = if stats.mean + stats.standard_deviation > 0.0 {
        ((stats.standard_deviation - stats.mean) / (stats.standard_deviation + stats.mean) + 1.0)
            / 2.0
    } else {
        0.0
    };
    let repeated = gaps
        .windows(4)
        .filter(|w| (w[0] - w[2]).abs() <= 10.0 && (w[1] - w[3]).abs() <= 10.0)
        .count();
    RhythmAnalysis {
        timing_gaps: stats,
        common_note_lengths: common,
        variety: safe_div(counts.len() as f64, total),
        entropy,
        burstiness,
        repeated_interval_patterns: repeated,
    }
}

fn aim_analysis(map: &Beatmap, distances: &[Option<f64>], gaps: &[f64]) -> (AimAnalysis, Vec<f64>) {
    let pressures: Vec<f64> = distances
        .iter()
        .enumerate()
        .map(|(i, distance)| {
            let gap = gaps.get(i).copied().unwrap_or(1000.0).max(25.0);
            (distance.unwrap_or(0.0) / 100.0) * (200.0 / gap).sqrt()
        })
        .collect();
    let mut angles = Vec::new();
    for window in map.hit_objects.windows(3) {
        let a = position(&window[0]);
        let b = position(&window[1]);
        let c = position(&window[2]);
        if let (Some(a), Some(b), Some(c)) = (a, b, c) {
            let v1 = (a.0 - b.0, a.1 - b.1);
            let v2 = (c.0 - b.0, c.1 - b.1);
            let denom = (v1.0.hypot(v1.1) * v2.0.hypot(v2.1)).max(1e-9);
            angles.push(
                ((v1.0 * v2.0 + v1.1 * v2.1) / denom)
                    .clamp(-1.0, 1.0)
                    .acos(),
            );
        }
    }
    let positional_distances: Vec<f64> = distances.iter().flatten().copied().collect();
    let distance_stats = distribution(&positional_distances);
    let angle_stats = distribution(&angles);
    let sharpness = if angles.is_empty() {
        0.0
    } else {
        angles
            .iter()
            .map(|a| 1.0 - a / std::f64::consts::PI)
            .sum::<f64>()
            / angles.len() as f64
    };
    (
        AimAnalysis {
            average_jump_distance: distance_stats.mean,
            peak_jump_distance: distance_stats.max,
            spacing_variance: distance_stats.standard_deviation.powi(2),
            angle_sharpness: sharpness,
            angle_variance: angle_stats.standard_deviation.powi(2),
            pressure_score_mean: mean(&pressures),
            pressure_score_peak: max(&pressures),
        },
        pressures,
    )
}

fn stream_analysis(map: &Beatmap, distances: &[Option<f64>]) -> StreamsAnalysis {
    let mut runs: Vec<(usize, f64, f64)> = Vec::new();
    let mut run_notes = 0;
    let mut interval_sum = 0.0;
    let mut distance_sum = 0.0;
    for (i, pair) in map.hit_objects.windows(2).enumerate() {
        let gap = pair[1].start_time - pair[0].start_time;
        let beat_len = map
            .control_points
            .timing_point_at(pair[0].start_time)
            .map(|p| p.beat_len)
            .unwrap_or(500.0);
        let expected = beat_len / 4.0;
        if gap > 0.0 && ((gap - expected).abs() / expected.max(1.0)) <= 0.2 {
            if run_notes == 0 {
                run_notes = 1;
            }
            run_notes += 1;
            interval_sum += gap;
            distance_sum += distances.get(i).copied().flatten().unwrap_or(0.0);
        } else if run_notes > 0 {
            runs.push((run_notes, interval_sum, distance_sum));
            run_notes = 0;
            interval_sum = 0.0;
            distance_sum = 0.0;
        }
    }
    if run_notes > 0 {
        runs.push((run_notes, interval_sum, distance_sum));
    }
    let bursts: Vec<_> = runs.iter().filter(|r| (3..=5).contains(&r.0)).collect();
    let streams: Vec<_> = runs.iter().filter(|r| r.0 >= 6).collect();
    let total_stream_intervals: f64 = streams.iter().map(|r| r.1).sum();
    let total_stream_edges: usize = streams.iter().map(|r| r.0.saturating_sub(1)).sum();
    let avg_gap = safe_div(total_stream_intervals, total_stream_edges as f64);
    let spaced = streams
        .iter()
        .filter(|r| safe_div(r.2, r.0.saturating_sub(1) as f64) >= 80.0)
        .count();
    StreamsAnalysis {
        burst_count: bursts.len(),
        stream_count: streams.len(),
        longest_burst: bursts.iter().map(|r| r.0).max().unwrap_or(0),
        longest_stream: streams.iter().map(|r| r.0).max().unwrap_or(0),
        average_stream_bpm: if avg_gap > 0.0 {
            15_000.0 / avg_gap
        } else {
            0.0
        },
        spaced_stream_ratio: safe_div(spaced as f64, streams.len() as f64),
    }
}

#[allow(clippy::too_many_arguments)]
fn section_analysis(
    objects: &[ObjectSample],
    gaps: &[f64],
    distances: &[Option<f64>],
    aim: &[f64],
    speed: &[f64],
    slider: &[f64],
    total_length: f64,
    width: f64,
) -> Vec<SectionAnalysis> {
    if objects.is_empty() {
        return Vec::new();
    }
    let count = (total_length / width).ceil().max(1.0) as usize;
    (0..count)
        .map(|index| {
            let start = index as f64 * width;
            let end = if total_length > 0.0 {
                ((index + 1) as f64 * width).min(total_length)
            } else {
                width
            };
            let indices: Vec<usize> = objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    (o.start >= start || index == 0)
                        && (o.start < end || (index + 1 == count && o.start <= end))
                })
                .map(|(i, _)| i)
                .collect();
            let section_distances: Vec<f64> = indices
                .iter()
                .filter_map(|i| {
                    i.checked_sub(1)
                        .and_then(|v| distances.get(v))
                        .copied()
                        .flatten()
                })
                .collect();
            let section_gaps: Vec<f64> = indices
                .iter()
                .filter_map(|i| i.checked_sub(1).and_then(|v| gaps.get(v)))
                .copied()
                .collect();
            let spacing = distribution(&section_distances);
            let rhythm = rhythm_analysis(&section_gaps);
            SectionAnalysis {
                start_time: start,
                end_time: end,
                object_count: indices.len(),
                notes_per_second: safe_div(indices.len() as f64, (end - start) / 1000.0),
                average_spacing: spacing.mean,
                spacing_variance: spacing.standard_deviation.powi(2),
                rhythm_complexity_score: rhythm.entropy,
                aim_pressure_score: mean_selected(aim, &indices, true),
                speed_pressure_score: mean_selected(speed, &indices, true),
                slider_pressure_score: mean_selected(slider, &indices, false),
            }
        })
        .collect()
}

fn speed_analysis(
    starts: &[f64],
    gaps: &[f64],
    pressures: &[f64],
    sections: &[SectionAnalysis],
) -> SpeedAnalysis {
    let peak_nps = peak_notes_per_second(starts);
    let threshold = sections.iter().map(|s| s.notes_per_second).sum::<f64>()
        / sections.len().max(1) as f64
        * 1.5;
    let stamina = if pressures.is_empty() {
        0.0
    } else {
        pressures
            .windows(8)
            .map(mean)
            .reduce(f64::max)
            .unwrap_or_else(|| mean(pressures))
    };
    SpeedAnalysis {
        high_density_sections: sections
            .iter()
            .filter(|s| s.object_count >= 4 && s.notes_per_second > threshold.max(2.0))
            .count(),
        fast_interval_count: gaps
            .iter()
            .filter(|gap| **gap > 0.0 && **gap <= 125.0)
            .count(),
        peak_notes_per_second: peak_nps,
        pressure_score_mean: mean(pressures),
        pressure_score_peak: max(pressures),
        stamina_pressure_score: stamina,
    }
}

fn peak_notes_per_second(starts: &[f64]) -> f64 {
    let mut left = 0;
    let mut peak = 0;

    for right in 0..starts.len() {
        while starts[right] - starts[left] > 1000.0 {
            left += 1;
        }

        peak = peak.max(right - left + 1);
    }

    peak as f64
}

fn slider_analysis(
    map: &Beatmap,
    objects: &[ObjectSample],
    seconds: f64,
    pressures: &[f64],
) -> SliderAnalysis {
    let sliders: Vec<_> = objects.iter().filter(|o| o.kind == Kind::Slider).collect();
    let changes = map
        .control_points
        .difficulty_points
        .windows(2)
        .filter(|w| (w[0].slider_velocity - w[1].slider_velocity).abs() > 1e-9)
        .count();
    SliderAnalysis {
        slider_ratio: safe_div(sliders.len() as f64, objects.len() as f64),
        average_duration: safe_div(
            sliders.iter().map(|o| o.slider_duration).sum(),
            sliders.len() as f64,
        ),
        density: safe_div(sliders.len() as f64, seconds),
        velocity_changes: changes,
        complexity_score: safe_div(
            sliders.iter().map(|o| o.slider_complexity).sum(),
            sliders.len() as f64,
        ),
        pressure_score_mean: mean(pressures),
        pressure_score_peak: max(pressures),
    }
}

fn tags(
    objects: &ObjectAnalysis,
    rhythm: &RhythmAnalysis,
    aim: &AimAnalysis,
    speed: &SpeedAnalysis,
    streams: &StreamsAnalysis,
    sliders: &SliderAnalysis,
) -> Vec<MapTag> {
    let mut tags = Vec::new();
    if aim.pressure_score_mean >= 1.0 || aim.peak_jump_distance >= 180.0 {
        tags.push(MapTag::Aim);
    }
    if streams.stream_count >= 2 || streams.longest_stream >= 12 {
        tags.push(MapTag::Stream);
    }
    if streams.burst_count >= 3 {
        tags.push(MapTag::Burst);
    }
    if sliders.complexity_score >= 3.5 && rhythm.entropy >= 1.5 {
        tags.push(MapTag::Tech);
    }
    if rhythm.entropy >= 2.5 {
        tags.push(MapTag::Reading);
        tags.push(MapTag::RhythmComplex);
    }
    if speed.stamina_pressure_score >= 1.2 || streams.longest_stream >= 20 {
        tags.push(MapTag::Stamina);
    }
    if objects.slider_ratio >= 0.45 {
        tags.push(MapTag::SliderHeavy);
    }
    if aim.angle_sharpness >= 0.55 && aim.pressure_score_mean >= 0.7 {
        tags.push(MapTag::AimControl);
    }
    if speed.fast_interval_count >= 20 || speed.pressure_score_peak >= 1.6 {
        tags.push(MapTag::Speed);
    }
    tags
}

fn position(object: &rosu_map::section::hit_objects::HitObject) -> Option<(f64, f64)> {
    match &object.kind {
        HitObjectKind::Circle(v) => Some((v.pos.x as f64, v.pos.y as f64)),
        HitObjectKind::Slider(v) => Some((v.pos.x as f64, v.pos.y as f64)),
        _ => None,
    }
}

fn distribution(values: &[f64]) -> Distribution {
    if values.is_empty() {
        return Distribution {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
            standard_deviation: 0.0,
            p50: 0.0,
            p90: 0.0,
            sample_count: 0,
        };
    }
    let avg = mean(values);
    let variance = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / values.len() as f64;
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Distribution {
        min: sorted[0],
        max: sorted[sorted.len() - 1],
        mean: avg,
        standard_deviation: variance.sqrt(),
        p50: percentile(&sorted, 0.5),
        p90: percentile(&sorted, 0.9),
        sample_count: values.len(),
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}
fn mean(values: &[f64]) -> f64 {
    safe_div(values.iter().sum(), values.len() as f64)
}
fn max(values: &[f64]) -> f64 {
    values.iter().copied().reduce(f64::max).unwrap_or(0.0)
}
fn safe_div(value: f64, divisor: f64) -> f64 {
    if divisor > 0.0 {
        value / divisor
    } else {
        0.0
    }
}
fn ratio(value: usize, total: f64) -> f64 {
    safe_div(value as f64, total)
}
fn mean_selected(values: &[f64], indices: &[usize], transition: bool) -> f64 {
    let selected: Vec<f64> = indices
        .iter()
        .filter_map(|i| {
            let index = if transition { i.checked_sub(1)? } else { *i };
            values.get(index).copied()
        })
        .collect();
    mean(&selected)
}
