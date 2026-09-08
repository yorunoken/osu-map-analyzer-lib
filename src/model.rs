/// A map characteristic measured by the analyzer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Pattern {
    Jump,
    Stream,
    Burst,
    Slider,
    Tech,
}

/// The strongest fixed-duration section for a pattern.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Peak {
    pub start_time_ms: f64,
    pub score: f64,
}

/// Comparable summary used to rank patterns.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternScore {
    pub pattern: Pattern,
    pub score: f64,
    pub objects: usize,
    pub segments: usize,
    pub peak: Option<Peak>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamAnalysis {
    pub score: f64,
    pub segments: usize,
    pub notes: usize,
    pub longest: usize,
    pub average_note_interval_ms: f64,
    pub peak: Option<Peak>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BurstAnalysis {
    pub score: f64,
    pub segments: usize,
    pub notes: usize,
    pub longest: usize,
    pub average_note_interval_ms: f64,
    pub peak: Option<Peak>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpAnalysis {
    pub score: f64,
    pub segments: usize,
    pub notes: usize,
    pub longest: usize,
    pub average_distance: f64,
    pub max_distance: f64,
    pub peak: Option<Peak>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SliderAnalysis {
    pub score: f64,
    pub count: usize,
    pub repeats: usize,
    pub travel_distance: f64,
    pub duration_ratio: f64,
    pub peak: Option<Peak>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TechAnalysis {
    pub score: f64,
    pub rhythm_complexity: f64,
    pub angle_complexity: f64,
    pub slider_velocity_changes: usize,
    pub peak: Option<Peak>,
}

/// Complete analysis for one osu!standard beatmap.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapAnalysis {
    pub primary: Option<Pattern>,
    pub patterns: Vec<PatternScore>,
    pub jump: JumpAnalysis,
    pub stream: StreamAnalysis,
    pub burst: BurstAnalysis,
    pub slider: SliderAnalysis,
    pub tech: TechAnalysis,
    pub object_count: usize,
    pub tap_object_count: usize,
    pub duration_ms: f64,
}
