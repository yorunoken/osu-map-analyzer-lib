macro_rules! analysis_type {
    ($item:item) => {
        #[derive(Debug, Clone, PartialEq, serde::Deserialize)]
        #[cfg_attr(feature = "serialize", derive(serde::Serialize))]
        $item
    };
}

analysis_type! {
    pub struct AnalysisOptions {
        /// Width of timeline sections in milliseconds.
        pub section_length: f64,
    }
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            section_length: 10_000.0,
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
        pub p90: f64,
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
        Aim, Stream, Burst, Tech, Reading, Stamina, SliderHeavy,
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
        let features = [
            ("object_count", self.general.object_count as f64),
            ("circle_ratio", self.objects.circle_ratio),
            ("slider_ratio", self.objects.slider_ratio),
            ("avg_bpm", self.timing.average_bpm),
            ("min_bpm", self.timing.min_bpm),
            ("max_bpm", self.timing.max_bpm),
            ("avg_nps", self.objects.notes_per_second),
            ("peak_nps", self.speed.peak_notes_per_second),
            ("avg_spacing", self.objects.average_spacing),
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
        ];

        FeatureVector {
            names: features
                .iter()
                .map(|(name, _)| (*name).to_owned())
                .collect(),
            values: features
                .iter()
                .map(|(_, value)| finite(*value) as f32)
                .collect(),
        }
    }
}

fn finite(value: f64) -> f64 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
