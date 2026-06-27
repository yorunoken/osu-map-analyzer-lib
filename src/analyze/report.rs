use std::{cmp::Ordering, fmt::Write};

use super::{BeatmapAnalysis, FeatureVector, MapTag, SectionAnalysis};

/// Results of structural and numerical sanity checks over an analysis.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct ValidationReport {
    pub passed: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ValidationReport {
    /// Returns whether validation found no errors. Warnings do not fail validation.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

impl BeatmapAnalysis {
    /// Returns a compact report intended for terminals, bot replies, and logs.
    pub fn compact_report(&self) -> String {
        let mut output = String::new();
        let tags = format_tags(&self.tags);

        writeln!(output, "{} — {}", self.metadata.artist, self.metadata.title).unwrap();
        writeln!(
            output,
            "Mapped by {} [{}]",
            self.metadata.creator, self.metadata.version
        )
        .unwrap();
        writeln!(output, "Objects: {}", self.general.object_count).unwrap();
        writeln!(
            output,
            "Length: {}  Drain: {}",
            format_duration(self.general.total_length),
            format_duration(self.general.drain_time)
        )
        .unwrap();
        writeln!(
            output,
            "BPM: {}  Range: {}–{}",
            format_number(self.timing.main_bpm, 2),
            format_number(self.timing.min_bpm, 2),
            format_number(self.timing.max_bpm, 2)
        )
        .unwrap();
        writeln!(
            output,
            "Density: {} avg / {} peak NPS",
            format_number(self.objects.notes_per_second, 2),
            format_number(self.speed.peak_notes_per_second, 2)
        )
        .unwrap();
        writeln!(
            output,
            "Objects: {} circles / {} sliders / {} spinners",
            format_percent(self.objects.circle_ratio),
            format_percent(self.objects.slider_ratio),
            format_percent(self.objects.spinner_ratio)
        )
        .unwrap();
        writeln!(output, "Tags: {tags}").unwrap();
        writeln!(output, "Highlights:").unwrap();
        writeln!(
            output,
            "  Streams: {} (longest {})  Bursts: {}",
            self.streams.stream_count, self.streams.longest_stream, self.streams.burst_count
        )
        .unwrap();
        writeln!(
            output,
            "  Spacing: {} avg / {} peak",
            format_number(self.objects.average_spacing, 2),
            format_number(self.aim.peak_jump_distance, 2)
        )
        .unwrap();
        write!(
            output,
            "  Pressure scores: {} speed / {} aim peak",
            format_number(self.speed.pressure_score_peak, 2),
            format_number(self.aim.pressure_score_peak, 2)
        )
        .unwrap();

        output
    }

    /// Returns grouped detail without dumping raw timing or analysis section arrays.
    pub fn detailed_report(&self) -> String {
        let mut output = String::new();

        writeln!(output, "{} — {}", self.metadata.artist, self.metadata.title).unwrap();
        writeln!(
            output,
            "Mapped by {} [{}]",
            self.metadata.creator, self.metadata.version
        )
        .unwrap();

        writeln!(output, "\nGeneral").unwrap();
        writeln!(output, "  Objects: {}", self.general.object_count).unwrap();
        writeln!(
            output,
            "  Length: {}",
            format_duration(self.general.total_length)
        )
        .unwrap();
        writeln!(
            output,
            "  Drain time: {}",
            format_duration(self.general.drain_time)
        )
        .unwrap();
        writeln!(
            output,
            "  AR {}  OD {}  CS {}  HP {}",
            format_number(self.general.approach_rate as f64, 2),
            format_number(self.general.overall_difficulty as f64, 2),
            format_number(self.general.circle_size as f64, 2),
            format_number(self.general.hp_drain_rate as f64, 2)
        )
        .unwrap();
        writeln!(output, "  Tags: {}", format_tags(&self.tags)).unwrap();

        writeln!(output, "\nTiming").unwrap();
        writeln!(
            output,
            "  Main / average BPM: {} / {}",
            format_number(self.timing.main_bpm, 2),
            format_number(self.timing.average_bpm, 2)
        )
        .unwrap();
        writeln!(
            output,
            "  BPM range: {}–{}  Changes: {}",
            format_number(self.timing.min_bpm, 2),
            format_number(self.timing.max_bpm, 2),
            self.timing.bpm_changes
        )
        .unwrap();
        writeln!(
            output,
            "  Inherited points: {}  SV changes: {}",
            self.timing.inherited_timing_points, self.timing.slider_velocity_changes
        )
        .unwrap();

        writeln!(output, "\nObjects").unwrap();
        writeln!(
            output,
            "  Circles: {} ({})  Sliders: {} ({})  Spinners: {} ({})  Holds: {} ({})",
            self.objects.circle_count,
            format_percent(self.objects.circle_ratio),
            self.objects.slider_count,
            format_percent(self.objects.slider_ratio),
            self.objects.spinner_count,
            format_percent(self.objects.spinner_ratio),
            self.objects.hold_count,
            format_percent(self.objects.hold_ratio)
        )
        .unwrap();
        writeln!(
            output,
            "  Average NPS: {}",
            format_number(self.objects.notes_per_second, 2)
        )
        .unwrap();
        writeln!(
            output,
            "  Spacing mean / stddev: {} / {}",
            format_number(self.objects.average_spacing, 2),
            format_number(self.objects.spacing_variance.sqrt(), 2)
        )
        .unwrap();

        writeln!(output, "\nRhythm").unwrap();
        writeln!(
            output,
            "  Entropy: {} bits  Variety: {}  Burstiness: {}",
            format_number(self.rhythm.entropy, 3),
            format_number(self.rhythm.variety, 3),
            format_number(self.rhythm.burstiness, 3)
        )
        .unwrap();
        writeln!(
            output,
            "  Repeated interval patterns: {}",
            self.rhythm.repeated_interval_patterns
        )
        .unwrap();

        writeln!(output, "\nStreams/Bursts").unwrap();
        writeln!(
            output,
            "  Streams: {}  Longest: {}",
            self.streams.stream_count, self.streams.longest_stream
        )
        .unwrap();
        writeln!(
            output,
            "  Bursts: {}  Longest: {}",
            self.streams.burst_count, self.streams.longest_burst
        )
        .unwrap();
        writeln!(
            output,
            "  Average stream BPM: {}",
            format_number(self.streams.average_stream_bpm, 2)
        )
        .unwrap();

        writeln!(output, "\nAim").unwrap();
        writeln!(
            output,
            "  Jump distance mean / peak: {} / {}",
            format_number(self.aim.average_jump_distance, 2),
            format_number(self.aim.peak_jump_distance, 2)
        )
        .unwrap();
        writeln!(
            output,
            "  Pressure score mean / peak: {} / {} (unbounded)",
            format_number(self.aim.pressure_score_mean, 3),
            format_number(self.aim.pressure_score_peak, 3)
        )
        .unwrap();

        writeln!(output, "\nSpeed").unwrap();
        writeln!(
            output,
            "  Peak NPS: {}  Fast intervals: {}",
            format_number(self.speed.peak_notes_per_second, 2),
            self.speed.fast_interval_count
        )
        .unwrap();
        writeln!(
            output,
            "  Pressure score mean / peak: {} / {} [0, 4]",
            format_number(self.speed.pressure_score_mean, 3),
            format_number(self.speed.pressure_score_peak, 3)
        )
        .unwrap();
        writeln!(
            output,
            "  Stamina pressure score: {} [0, 4]",
            format_number(self.speed.stamina_pressure_score, 3)
        )
        .unwrap();

        writeln!(output, "\nSliders").unwrap();
        writeln!(
            output,
            "  Ratio: {}  Density: {} per second",
            format_percent(self.sliders.slider_ratio),
            format_number(self.sliders.density, 2)
        )
        .unwrap();
        writeln!(
            output,
            "  Average duration: {} ms  SV changes: {}",
            format_number(self.sliders.average_duration, 2),
            self.sliders.velocity_changes
        )
        .unwrap();
        writeln!(
            output,
            "  Complexity score: {} (unbounded)",
            format_number(self.sliders.complexity_score, 3)
        )
        .unwrap();
        writeln!(
            output,
            "  Pressure score mean / peak: {} / {} [0, 3]",
            format_number(self.sliders.pressure_score_mean, 3),
            format_number(self.sliders.pressure_score_peak, 3)
        )
        .unwrap();

        writeln!(output, "\nSections summary").unwrap();
        write_section_rankings(&mut output, "Peak speed", &self.sections, |s| {
            s.speed_pressure_score
        });
        write_section_rankings(&mut output, "Peak aim", &self.sections, |s| {
            s.aim_pressure_score
        });
        write_section_rankings(&mut output, "Highest density", &self.sections, |s| {
            s.notes_per_second
        });
        let empty = self
            .sections
            .iter()
            .filter(|section| section.object_count == 0)
            .count();
        writeln!(output, "  Empty/break sections: {empty}").unwrap();

        output
    }

    /// Validates this analysis and its generated feature vector.
    pub fn validate(&self) -> ValidationReport {
        self.validate_with_feature_vector(&self.to_feature_vector())
    }

    /// Validates this analysis against a caller-provided feature vector.
    pub fn validate_with_feature_vector(&self, features: &FeatureVector) -> ValidationReport {
        let mut report = ValidationReport {
            passed: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        let object_sum = self.objects.circle_count
            + self.objects.slider_count
            + self.objects.spinner_count
            + self.objects.hold_count;
        check_equal(
            &mut report,
            "object counts sum to total",
            object_sum,
            self.general.object_count,
        );

        let ratio_sum = self.objects.circle_ratio
            + self.objects.slider_ratio
            + self.objects.spinner_ratio
            + self.objects.hold_ratio;
        if self.general.object_count == 0 || (ratio_sum - 1.0).abs() <= 1e-6 {
            report.passed.push("object ratios sum to one".into());
        } else {
            report.errors.push(format!(
                "object ratios sum to {}, expected 1",
                format_number(ratio_sum, 6)
            ));
        }

        let section_sum = self
            .sections
            .iter()
            .map(|section| section.object_count)
            .sum();
        check_equal(
            &mut report,
            "section object counts sum to total",
            section_sum,
            self.general.object_count,
        );
        check_equal(
            &mut report,
            "feature names and values have equal length",
            features.names.len(),
            features.values.len(),
        );

        validate_finite_values(self, features, &mut report);
        validate_sorted(self, &mut report);
        validate_last_section(self, &mut report);
        validate_pressure_ranges(self, &mut report);

        let equal_bpm = self
            .timing
            .sections
            .windows(2)
            .filter(|window| (window[0].bpm - window[1].bpm).abs() <= 0.01)
            .count();
        if equal_bpm > 0 {
            report.warnings.push(format!("{equal_bpm} adjacent timing section pair(s) have effectively equal BPM; human display treats them as redundant"));
        } else {
            report
                .passed
                .push("adjacent timing sections have distinct BPM".into());
        }

        if contains_negative_zero(&self.compact_report())
            || contains_negative_zero(&self.detailed_report())
        {
            report
                .errors
                .push("human output contains negative zero".into());
        } else {
            report
                .passed
                .push("human output contains no negative zero".into());
        }

        report
    }
}

fn write_section_rankings(
    output: &mut String,
    label: &str,
    sections: &[SectionAnalysis],
    value: impl Fn(&SectionAnalysis) -> f64,
) {
    let mut ranked: Vec<_> = sections.iter().collect();
    ranked.sort_by(|a, b| value(b).partial_cmp(&value(a)).unwrap_or(Ordering::Equal));
    writeln!(output, "  {label}:").unwrap();
    for section in ranked.into_iter().take(5) {
        writeln!(
            output,
            "    {}–{}  {}",
            format_duration(section.start_time),
            format_duration(section.end_time),
            format_number(value(section), 3)
        )
        .unwrap();
    }
}

fn validate_finite_values(
    analysis: &BeatmapAnalysis,
    features: &FeatureVector,
    report: &mut ValidationReport,
) {
    let mut invalid = Vec::new();
    let mut check = |name: &str, value: f64| {
        if !value.is_finite() {
            invalid.push(name.to_owned());
        }
    };
    let fixed_values = [
        (
            "general.approach_rate",
            analysis.general.approach_rate as f64,
        ),
        (
            "general.overall_difficulty",
            analysis.general.overall_difficulty as f64,
        ),
        ("general.circle_size", analysis.general.circle_size as f64),
        (
            "general.hp_drain_rate",
            analysis.general.hp_drain_rate as f64,
        ),
        ("general.total_length", analysis.general.total_length),
        ("general.drain_time", analysis.general.drain_time),
        ("timing.main_bpm", analysis.timing.main_bpm),
        ("timing.average_bpm", analysis.timing.average_bpm),
        ("timing.min_bpm", analysis.timing.min_bpm),
        ("timing.max_bpm", analysis.timing.max_bpm),
        ("objects.circle_ratio", analysis.objects.circle_ratio),
        ("objects.slider_ratio", analysis.objects.slider_ratio),
        ("objects.spinner_ratio", analysis.objects.spinner_ratio),
        ("objects.hold_ratio", analysis.objects.hold_ratio),
        ("objects.object_density", analysis.objects.object_density),
        (
            "objects.notes_per_second",
            analysis.objects.notes_per_second,
        ),
        ("objects.average_spacing", analysis.objects.average_spacing),
        (
            "objects.spacing_variance",
            analysis.objects.spacing_variance,
        ),
        (
            "objects.jump_distances.min",
            analysis.objects.jump_distances.min,
        ),
        (
            "objects.jump_distances.max",
            analysis.objects.jump_distances.max,
        ),
        (
            "objects.jump_distances.mean",
            analysis.objects.jump_distances.mean,
        ),
        (
            "objects.jump_distances.standard_deviation",
            analysis.objects.jump_distances.standard_deviation,
        ),
        (
            "objects.jump_distances.p50",
            analysis.objects.jump_distances.p50,
        ),
        (
            "objects.jump_distances.p90",
            analysis.objects.jump_distances.p90,
        ),
        ("rhythm.timing_gaps.min", analysis.rhythm.timing_gaps.min),
        ("rhythm.timing_gaps.max", analysis.rhythm.timing_gaps.max),
        ("rhythm.timing_gaps.mean", analysis.rhythm.timing_gaps.mean),
        (
            "rhythm.timing_gaps.standard_deviation",
            analysis.rhythm.timing_gaps.standard_deviation,
        ),
        ("rhythm.timing_gaps.p50", analysis.rhythm.timing_gaps.p50),
        ("rhythm.timing_gaps.p90", analysis.rhythm.timing_gaps.p90),
        ("rhythm.variety", analysis.rhythm.variety),
        ("rhythm.entropy", analysis.rhythm.entropy),
        ("rhythm.burstiness", analysis.rhythm.burstiness),
        (
            "aim.average_jump_distance",
            analysis.aim.average_jump_distance,
        ),
        ("aim.peak_jump_distance", analysis.aim.peak_jump_distance),
        ("aim.spacing_variance", analysis.aim.spacing_variance),
        ("aim.angle_sharpness", analysis.aim.angle_sharpness),
        ("aim.angle_variance", analysis.aim.angle_variance),
        ("aim.pressure_score_mean", analysis.aim.pressure_score_mean),
        ("aim.pressure_score_peak", analysis.aim.pressure_score_peak),
        (
            "speed.peak_notes_per_second",
            analysis.speed.peak_notes_per_second,
        ),
        (
            "speed.pressure_score_mean",
            analysis.speed.pressure_score_mean,
        ),
        (
            "speed.pressure_score_peak",
            analysis.speed.pressure_score_peak,
        ),
        (
            "speed.stamina_pressure_score",
            analysis.speed.stamina_pressure_score,
        ),
        (
            "streams.average_stream_bpm",
            analysis.streams.average_stream_bpm,
        ),
        (
            "streams.spaced_stream_ratio",
            analysis.streams.spaced_stream_ratio,
        ),
        ("sliders.slider_ratio", analysis.sliders.slider_ratio),
        (
            "sliders.average_duration",
            analysis.sliders.average_duration,
        ),
        ("sliders.density", analysis.sliders.density),
        (
            "sliders.complexity_score",
            analysis.sliders.complexity_score,
        ),
        (
            "sliders.pressure_score_mean",
            analysis.sliders.pressure_score_mean,
        ),
        (
            "sliders.pressure_score_peak",
            analysis.sliders.pressure_score_peak,
        ),
    ];
    for (name, value) in fixed_values {
        check(name, value);
    }
    for (index, interval) in analysis.rhythm.common_note_lengths.iter().enumerate() {
        check(
            &format!("rhythm.common_note_lengths[{index}].interval"),
            interval.interval,
        );
    }
    for (index, section) in analysis.sections.iter().enumerate() {
        check(&format!("sections[{index}].start_time"), section.start_time);
        check(&format!("sections[{index}].end_time"), section.end_time);
        check(
            &format!("sections[{index}].notes_per_second"),
            section.notes_per_second,
        );
        check(
            &format!("sections[{index}].average_spacing"),
            section.average_spacing,
        );
        check(
            &format!("sections[{index}].spacing_variance"),
            section.spacing_variance,
        );
        check(
            &format!("sections[{index}].rhythm_complexity_score"),
            section.rhythm_complexity_score,
        );
        check(
            &format!("sections[{index}].aim_pressure_score"),
            section.aim_pressure_score,
        );
        check(
            &format!("sections[{index}].speed_pressure_score"),
            section.speed_pressure_score,
        );
        check(
            &format!("sections[{index}].slider_pressure_score"),
            section.slider_pressure_score,
        );
    }
    for (index, section) in analysis.timing.sections.iter().enumerate() {
        check(
            &format!("timing.sections[{index}].start_time"),
            section.start_time,
        );
        check(
            &format!("timing.sections[{index}].end_time"),
            section.end_time,
        );
        check(&format!("timing.sections[{index}].bpm"), section.bpm);
    }
    for (index, value) in features.values.iter().enumerate() {
        check(&format!("features.values[{index}]"), *value as f64);
    }
    if invalid.is_empty() {
        report
            .passed
            .push("all checked numeric values are finite".into());
    } else {
        report
            .errors
            .push(format!("non-finite values: {}", invalid.join(", ")));
    }
}

fn validate_sorted(analysis: &BeatmapAnalysis, report: &mut ValidationReport) {
    if analysis
        .timing
        .sections
        .windows(2)
        .all(|w| w[0].start_time <= w[1].start_time)
    {
        report.passed.push("timing sections are sorted".into());
    } else {
        report
            .errors
            .push("timing sections are not sorted by start_time".into());
    }
    if analysis
        .sections
        .windows(2)
        .all(|w| w[0].start_time <= w[1].start_time)
    {
        report.passed.push("analysis sections are sorted".into());
    } else {
        report
            .errors
            .push("analysis sections are not sorted by start_time".into());
    }
}

fn validate_last_section(analysis: &BeatmapAnalysis, report: &mut ValidationReport) {
    match analysis.sections.last() {
        Some(last) if (last.end_time - analysis.general.total_length).abs() <= 1.0 => report
            .passed
            .push("last section ends at total length".into()),
        Some(last) => report.errors.push(format!(
            "last section ends at {} ms, total length is {} ms",
            format_number(last.end_time, 3),
            format_number(analysis.general.total_length, 3)
        )),
        None if analysis.general.object_count == 0 => report
            .passed
            .push("empty map has no analysis sections".into()),
        None => report
            .errors
            .push("non-empty map has no analysis sections".into()),
    }
}

fn validate_pressure_ranges(analysis: &BeatmapAnalysis, report: &mut ValidationReport) {
    if in_range(analysis.speed.pressure_score_mean, 0.0, 4.0)
        && in_range(analysis.speed.pressure_score_peak, 0.0, 4.0)
        && in_range(analysis.speed.stamina_pressure_score, 0.0, 4.0)
        && in_range(analysis.sliders.pressure_score_mean, 0.0, 3.0)
        && in_range(analysis.sliders.pressure_score_peak, 0.0, 3.0)
        && analysis.sections.iter().all(|s| {
            in_range(s.speed_pressure_score, 0.0, 4.0)
                && in_range(s.slider_pressure_score, 0.0, 3.0)
        })
    {
        report
            .passed
            .push("bounded pressure scores are within documented ranges".into());
    } else {
        report
            .errors
            .push("a bounded pressure score is outside its documented range".into());
    }
    if analysis.aim.pressure_score_mean >= 0.0 && analysis.aim.pressure_score_peak >= 0.0 {
        report
            .passed
            .push("unbounded aim pressure scores are non-negative".into());
    } else {
        report
            .errors
            .push("an aim pressure score is negative".into());
    }
}

fn check_equal(report: &mut ValidationReport, label: &str, actual: usize, expected: usize) {
    if actual == expected {
        report.passed.push(label.into());
    } else {
        report
            .errors
            .push(format!("{label}: got {actual}, expected {expected}"));
    }
}

fn in_range(value: f64, min: f64, max: f64) -> bool {
    value.is_finite() && value >= min && value <= max
}

fn format_tags(tags: &[MapTag]) -> String {
    if tags.is_empty() {
        "None".into()
    } else {
        tags.iter()
            .map(|tag| format!("{tag:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn format_duration(milliseconds: f64) -> String {
    let seconds = if milliseconds.is_finite() {
        (milliseconds.max(0.0) / 1000.0).round() as u64
    } else {
        0
    };
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn format_percent(ratio: f64) -> String {
    format!("{}%", format_number(ratio * 100.0, 1))
}

fn format_number(value: f64, decimals: usize) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let threshold = 0.5 * 10_f64.powi(-(decimals as i32));
    let value = if value.abs() < threshold { 0.0 } else { value };
    let mut rendered = format!("{value:.decimals$}");
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
    }
    rendered
}

fn contains_negative_zero(output: &str) -> bool {
    output
        .split(|character: char| {
            !character.is_ascii_digit() && character != '-' && character != '.'
        })
        .any(|token| token.starts_with('-') && token.parse::<f64>().is_ok_and(|value| value == 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::{AnalysisOptions, BeatmapAnalyzer, TimingSection};

    const MAP: &[u8] = br#"osu file format v14

[General]
Mode:0

[Metadata]
Title:Readable Test
Artist:Test Artist
Creator:Test Creator
Version:Normal

[Difficulty]
HPDrainRate:5
CircleSize:4
OverallDifficulty:7
ApproachRate:8
SliderMultiplier:1.4
SliderTickRate:1

[TimingPoints]
0,500,4,2,1,100,1,0

[HitObjects]
64,64,1000,1,0,0:0:0:0:
192,64,2000,1,0,0:0:0:0:
256,192,11000,2,0,B|320:192|384:256,1,180
"#;

    fn analysis() -> BeatmapAnalysis {
        BeatmapAnalyzer::from_bytes(MAP)
            .unwrap()
            .analyze(AnalysisOptions::default())
            .unwrap()
    }

    #[test]
    fn validation_passes_on_normal_map() {
        let report = analysis().validate();

        assert!(report.is_valid(), "{:?}", report.errors);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    }

    #[test]
    fn validation_catches_mismatched_feature_lengths() {
        let analysis = analysis();
        let mut features = analysis.to_feature_vector();
        features.values.pop();

        let report = analysis.validate_with_feature_vector(&features);

        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("feature names")));
    }

    #[test]
    fn validation_catches_non_finite_values() {
        let mut analysis = analysis();
        analysis.aim.pressure_score_peak = f64::NAN;
        analysis.speed.pressure_score_mean = f64::INFINITY;

        let report = analysis.validate();

        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("non-finite")));
    }

    #[test]
    fn validation_catches_section_object_mismatch() {
        let mut analysis = analysis();
        analysis.sections[0].object_count += 1;

        let report = analysis.validate();

        assert!(report
            .errors
            .iter()
            .any(|error| error.contains("section object")));
    }

    #[test]
    fn compact_report_does_not_dump_section_arrays() {
        let report = analysis().compact_report();

        assert!(!report.contains("TimingSection"));
        assert!(!report.contains("SectionAnalysis"));
        assert!(!report.contains("start_time"));
    }

    #[test]
    fn float_formatting_removes_negative_zero() {
        let mut analysis = analysis();
        analysis.objects.average_spacing = -0.0;
        analysis.aim.pressure_score_peak = -0.0;

        assert!(!contains_negative_zero(&analysis.compact_report()));
        assert!(!contains_negative_zero(&analysis.detailed_report()));
    }

    #[test]
    fn adjacent_same_bpm_sections_produce_warning() {
        let mut analysis = analysis();
        analysis.timing.sections.push(TimingSection {
            start_time: 5_000.0,
            end_time: 11_000.0,
            bpm: analysis.timing.sections[0].bpm,
        });

        let report = analysis.validate();

        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("equal BPM")));
    }
}
