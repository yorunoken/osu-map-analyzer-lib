macro_rules! analysis_type {
    ($item:item) => {
        #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
        #[cfg_attr(feature = "serialize", derive(serde::Serialize))]
        $item
    };
}

/// Gameplay mods retained as analysis and dataset metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub enum GameMod {
    DT,
    NC,
    HT,
    HR,
    EZ,
    HD,
    FL,
}

impl GameMod {
    pub(crate) const fn legacy_bits(self) -> u32 {
        match self {
            Self::EZ => 2,
            Self::HD => 8,
            Self::HR => 16,
            Self::DT => 64,
            Self::HT => 256,
            Self::NC => 64 | 512,
            Self::FL => 1024,
        }
    }
}

impl std::fmt::Display for GameMod {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::str::FromStr for GameMod {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_uppercase().as_str() {
            "DT" => Ok(Self::DT),
            "NC" => Ok(Self::NC),
            "HT" => Ok(Self::HT),
            "HR" => Ok(Self::HR),
            "EZ" => Ok(Self::EZ),
            "HD" => Ok(Self::HD),
            "FL" => Ok(Self::FL),
            value => Err(format!(
                "unknown mod `{value}`; expected DT, NC, HT, HR, EZ, HD, or FL"
            )),
        }
    }
}

analysis_type! {
    pub struct AnalysisOptions {
        /// Width of timeline sections in milliseconds.
        pub section_length: f64,
        /// Explicit playback-rate override. `None` derives rate from mods.
        #[serde(default)]
        pub rate: Option<f64>,
        /// Mods describing the played version. Only rate mods affect heuristic analysis.
        #[serde(default)]
        pub mods: Vec<GameMod>,
    }
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            section_length: 10_000.0,
            rate: None,
            mods: Vec::new(),
        }
    }
}

analysis_type! {
    pub struct BeatmapAnalysis {
        pub metadata: BeatmapMetadata,
        pub general: GeneralStats,
        pub timing: TimingAnalysis,
        pub objects: ObjectAnalysis,
        pub rhythm: RhythmAnalysis,
        pub aim: AimAnalysis,
        pub speed: SpeedAnalysis,
        pub streams: StreamsAnalysis,
        pub sliders: SliderAnalysis,
        pub sections: Vec<SectionAnalysis>,
        pub tags: Vec<MapTag>,
        #[serde(default)]
        pub mods: Vec<GameMod>,
        #[serde(default = "default_rate")]
        pub rate: f64,
        pub star_rating_nomod: Option<f64>,
        pub star_rating_adjusted: Option<f64>,
    }
}

analysis_type! {
    pub struct BeatmapMetadata {
        pub mode: GameMode,
        pub title: String,
        pub artist: String,
        pub creator: String,
        pub version: String,
        pub beatmap_id: Option<i32>,
        pub beatmap_set_id: Option<i32>,
    }
}

analysis_type! {
    pub enum GameMode { Osu, Taiko, Catch, Mania }
}

analysis_type! {
    pub struct GeneralStats {
        pub approach_rate: f32,
        pub overall_difficulty: f32,
        pub circle_size: f32,
        pub hp_drain_rate: f32,
        pub drain_time: f64,
        pub total_length: f64,
        pub object_count: usize,
    }
}

analysis_type! {
    pub struct TimingAnalysis {
        pub main_bpm: f64,
        pub average_bpm: f64,
        pub min_bpm: f64,
        pub max_bpm: f64,
        pub bpm_changes: usize,
        pub inherited_timing_points: usize,
        pub slider_velocity_changes: usize,
        pub sections: Vec<TimingSection>,
    }
}

analysis_type! {
    pub struct TimingSection {
        pub start_time: f64,
        pub end_time: f64,
        pub bpm: f64,
    }
}

analysis_type! {
    pub struct ObjectAnalysis {
        pub circle_count: usize,
        pub slider_count: usize,
        pub spinner_count: usize,
        pub hold_count: usize,
        pub circle_ratio: f64,
        pub slider_ratio: f64,
        pub spinner_ratio: f64,
        pub hold_ratio: f64,
        pub object_density: f64,
        pub notes_per_second: f64,
        pub average_spacing: f64,
        pub spacing_variance: f64,
        pub jump_distances: Distribution,
    }
}

analysis_type! {
    pub struct Distribution {
        pub min: f64,
        pub max: f64,
        pub mean: f64,
        pub standard_deviation: f64,
        pub p50: f64,
        #[serde(default)]
        pub p75: f64,
        pub p90: f64,
        #[serde(default)]
        pub p95: f64,
        pub sample_count: usize,
    }
}

analysis_type! {
    pub struct RhythmAnalysis {
        pub timing_gaps: Distribution,
        pub common_note_lengths: Vec<IntervalFrequency>,
        pub variety: f64,
        /// Shannon entropy in bits over note gaps quantized to 10 ms buckets.
        /// Range: 0 to log2(unique gaps), not normalized. Higher means more
        /// varied rhythm timing.
        pub entropy: f64,
        /// Rescaled coefficient of variation: `(stddev - mean) / (stddev + mean)`,
        /// mapped from [-1, 1] to [0, 1]. Higher means gaps are more burst-like.
        pub burstiness: f64,
        pub repeated_interval_patterns: usize,
    }
}

analysis_type! {
    pub struct IntervalFrequency {
        pub interval: f64,
        pub count: usize,
    }
}

analysis_type! {
    pub struct AimAnalysis {
        pub average_jump_distance: f64,
        pub peak_jump_distance: f64,
        pub spacing_variance: f64,
        pub angle_sharpness: f64,
        pub angle_variance: f64,
        /// Mean of `distance / 100 * sqrt(200 / max(gap_ms, 25))` over object
        /// transitions. Unbounded and not normalized; higher means harder aim.
        pub pressure_score_mean: f64,
        /// Maximum aim pressure score. Unbounded and not normalized.
        pub pressure_score_peak: f64,
    }
}

analysis_type! {
    pub struct SpeedAnalysis {
        pub high_density_sections: usize,
        pub fast_interval_count: usize,
        pub peak_notes_per_second: f64,
        /// Mean `clamp(200 / gap_ms, 0, 4)` over transitions. Range: [0, 4];
        /// higher means faster object timing.
        pub pressure_score_mean: f64,
        /// Maximum speed pressure score. Range: [0, 4].
        pub pressure_score_peak: f64,
        /// Maximum rolling mean of eight speed pressure scores. Range: [0, 4];
        /// higher means longer sustained speed.
        pub stamina_pressure_score: f64,
    }
}

analysis_type! {
    pub struct StreamsAnalysis {
        /// Runs of 3-5 notes whose gaps are within 20% of the active BPM's
        /// quarter-beat interval.
        pub burst_count: usize,
        /// Runs of at least 6 notes whose gaps are within 20% of the active
        /// BPM's quarter-beat interval.
        pub stream_count: usize,
        pub longest_burst: usize,
        pub longest_stream: usize,
        pub average_stream_bpm: f64,
        pub spaced_stream_ratio: f64,
    }
}

analysis_type! {
    pub struct SliderAnalysis {
        pub slider_ratio: f64,
        pub average_duration: f64,
        pub density: f64,
        pub velocity_changes: usize,
        /// Mean path control-point count plus repeat count per slider.
        /// Unbounded and not normalized; higher means more complex sliders.
        pub complexity_score: f64,
        /// Mean per-object slider score. A slider contributes
        /// `clamp(complexity / 3, 0.25, 3)` and other objects contribute zero.
        /// Range: [0, 3].
        pub pressure_score_mean: f64,
        /// Maximum per-object slider pressure score. Range: [0, 3].
        pub pressure_score_peak: f64,
    }
}

analysis_type! {
    pub struct SectionAnalysis {
        pub start_time: f64,
        pub end_time: f64,
        pub object_count: usize,
        pub notes_per_second: f64,
        pub average_spacing: f64,
        pub spacing_variance: f64,
        /// Shannon entropy in bits over the section's 10 ms-quantized gaps.
        /// Not normalized; higher means more varied section rhythm.
        pub rhythm_complexity_score: f64,
        /// Mean unbounded aim pressure score for transitions in this section.
        pub aim_pressure_score: f64,
        /// Mean speed pressure score for transitions in this section; [0, 4].
        pub speed_pressure_score: f64,
        /// Mean slider pressure score for objects in this section; [0, 3].
        pub slider_pressure_score: f64,
    }
}

analysis_type! {
    pub enum MapTag {
        Aim, Stream, Burst, Tech, Reading, Stamina, Slider,
        RhythmComplex, AimControl, Speed,
    }
}

analysis_type! {
    pub struct FeatureVector {
        pub names: Vec<String>,
        pub values: Vec<f32>,
    }
}

impl BeatmapAnalysis {
    /// Exports a stable, ordered set of scalar features for ML pipelines.
    pub fn to_feature_vector(&self) -> FeatureVector {
        use super::stats::{max, mean, percentile, ratio, standard_deviation};

        let section_density: Vec<_> = self
            .sections
            .iter()
            .map(|section| section.notes_per_second)
            .collect();
        let section_aim: Vec<_> = self
            .sections
            .iter()
            .map(|section| section.aim_pressure_score)
            .collect();
        let section_speed: Vec<_> = self
            .sections
            .iter()
            .map(|section| section.speed_pressure_score)
            .collect();
        let section_slider: Vec<_> = self
            .sections
            .iter()
            .map(|section| section.slider_pressure_score)
            .collect();
        let empty_sections = self
            .sections
            .iter()
            .filter(|section| section.object_count == 0)
            .count();
        let duration_minutes = self.general.drain_time / 60_000.0;
        let transition_count = self.general.object_count.saturating_sub(1);
        let has_mod = |game_mod| self.mods.contains(&game_mod) as u8 as f64;

        let features = vec![
            // Difficulty context
            (
                "star_rating_available",
                (self.star_rating_nomod.is_some() && self.star_rating_adjusted.is_some()) as u8
                    as f64,
            ),
            ("star_rating_nomod", self.star_rating_nomod.unwrap_or(0.0)),
            (
                "star_rating_adjusted",
                self.star_rating_adjusted.unwrap_or(0.0),
            ),
            ("rate", self.rate),
            ("is_dt", has_mod(GameMod::DT)),
            ("is_nc", has_mod(GameMod::NC)),
            ("is_ht", has_mod(GameMod::HT)),
            ("is_hr", has_mod(GameMod::HR)),
            ("is_ez", has_mod(GameMod::EZ)),
            // Whole-map features
            ("object_count", self.general.object_count as f64),
            ("circle_ratio", self.objects.circle_ratio),
            ("slider_ratio", self.objects.slider_ratio),
            ("min_bpm", self.timing.min_bpm),
            ("max_bpm", self.timing.max_bpm),
            ("notes_per_second_mean", self.objects.notes_per_second),
            ("peak_nps", self.speed.peak_notes_per_second),
            ("spacing_stddev", self.objects.spacing_variance.sqrt()),
            ("rhythm_entropy", self.rhythm.entropy),
            ("burst_count", self.streams.burst_count as f64),
            ("stream_count", self.streams.stream_count as f64),
            ("longest_stream", self.streams.longest_stream as f64),
            ("aim_pressure_score_mean", self.aim.pressure_score_mean),
            ("aim_pressure_score_peak", self.aim.pressure_score_peak),
            ("speed_pressure_score_mean", self.speed.pressure_score_mean),
            ("speed_pressure_score_peak", self.speed.pressure_score_peak),
            (
                "slider_pressure_score_mean",
                self.sliders.pressure_score_mean,
            ),
            (
                "slider_pressure_score_peak",
                self.sliders.pressure_score_peak,
            ),
            // Density
            ("section_density_mean", mean(&section_density)),
            (
                "section_density_stddev",
                standard_deviation(&section_density),
            ),
            ("section_density_p50", percentile(&section_density, 0.50)),
            ("section_density_p90", percentile(&section_density, 0.90)),
            ("section_density_p95", percentile(&section_density, 0.95)),
            ("section_density_peak", max(&section_density)),
            ("empty_section_count", empty_sections as f64),
            (
                "empty_section_ratio",
                ratio(empty_sections, self.sections.len()),
            ),
            // Aim
            ("section_aim_pressure_score_mean", mean(&section_aim)),
            (
                "section_aim_pressure_score_stddev",
                standard_deviation(&section_aim),
            ),
            (
                "section_aim_pressure_score_p90",
                percentile(&section_aim, 0.90),
            ),
            (
                "section_aim_pressure_score_p95",
                percentile(&section_aim, 0.95),
            ),
            ("section_aim_pressure_score_peak", max(&section_aim)),
            ("jump_distance_mean", self.aim.average_jump_distance),
            ("jump_distance_p90", self.objects.jump_distances.p90),
            ("jump_distance_p95", self.objects.jump_distances.p95),
            ("peak_jump_distance", self.aim.peak_jump_distance),
            ("angle_variance", self.aim.angle_variance),
            ("angle_sharpness", self.aim.angle_sharpness),
            // Speed
            ("section_speed_pressure_score_mean", mean(&section_speed)),
            (
                "section_speed_pressure_score_stddev",
                standard_deviation(&section_speed),
            ),
            (
                "section_speed_pressure_score_p90",
                percentile(&section_speed, 0.90),
            ),
            (
                "section_speed_pressure_score_p95",
                percentile(&section_speed, 0.95),
            ),
            ("section_speed_pressure_score_peak", max(&section_speed)),
            ("fast_interval_count", self.speed.fast_interval_count as f64),
            (
                "fast_interval_ratio",
                ratio(self.speed.fast_interval_count, transition_count),
            ),
            ("stamina_pressure_score", self.speed.stamina_pressure_score),
            // Rhythm
            ("rhythm_variety_ratio", self.rhythm.variety),
            ("rhythm_burstiness_score", self.rhythm.burstiness),
            (
                "repeated_interval_pattern_count",
                self.rhythm.repeated_interval_patterns as f64,
            ),
            ("timing_gap_mean_ms", self.rhythm.timing_gaps.mean),
            (
                "timing_gap_stddev_ms",
                self.rhythm.timing_gaps.standard_deviation,
            ),
            ("timing_gap_p50_ms", self.rhythm.timing_gaps.p50),
            ("timing_gap_p90_ms", self.rhythm.timing_gaps.p90),
            ("timing_gap_p95_ms", self.rhythm.timing_gaps.p95),
            // Streams and bursts
            ("longest_burst", self.streams.longest_burst as f64),
            ("stream_bpm_mean", self.streams.average_stream_bpm),
            (
                "stream_count_per_minute",
                super::stats::safe_div(self.streams.stream_count as f64, duration_minutes),
            ),
            (
                "burst_count_per_minute",
                super::stats::safe_div(self.streams.burst_count as f64, duration_minutes),
            ),
            ("spaced_stream_ratio", self.streams.spaced_stream_ratio),
            // Sliders
            ("slider_density", self.sliders.density),
            ("slider_duration_mean_ms", self.sliders.average_duration),
            ("slider_complexity_score", self.sliders.complexity_score),
            (
                "slider_pressure_score_p90",
                percentile(&section_slider, 0.90),
            ),
            // Timing
            ("main_bpm", self.timing.main_bpm),
            ("average_bpm", self.timing.average_bpm),
            (
                "bpm_range",
                (self.timing.max_bpm - self.timing.min_bpm).max(0.0),
            ),
            ("bpm_change_count", self.timing.bpm_changes as f64),
            (
                "inherited_timing_point_count",
                self.timing.inherited_timing_points as f64,
            ),
            (
                "slider_velocity_change_count",
                self.timing.slider_velocity_changes as f64,
            ),
            // General
            ("spinner_ratio", self.objects.spinner_ratio),
            ("drain_time_seconds", self.general.drain_time / 1000.0),
            ("total_length_seconds", self.general.total_length / 1000.0),
            ("approach_rate", self.general.approach_rate as f64),
            ("overall_difficulty", self.general.overall_difficulty as f64),
            ("circle_size", self.general.circle_size as f64),
            ("hp_drain_rate", self.general.hp_drain_rate as f64),
        ];

        FeatureVector {
            names: features
                .iter()
                .map(|(name, _)| (*name).to_owned())
                .collect(),
            values: features
                .iter()
                .map(|(_, value)| finite_f32(*value))
                .collect(),
        }
    }
}

fn finite_f32(value: f64) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }

    value.clamp(f32::MIN as f64, f32::MAX as f64) as f32
}

fn default_rate() -> f64 {
    1.0
}
