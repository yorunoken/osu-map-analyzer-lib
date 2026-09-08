/// A map characteristic measured by the analyzer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Pattern {
    /// Spatially separated notes with playable timing gaps.
    Jump,
    /// A fast tap run containing at least the configured stream length.
    Stream,
    /// A short fast tap run below the configured stream length.
    Burst,
    /// Slider-focused movement and duration.
    Slider,
    /// Rhythm, angle, slider, and velocity-change complexity.
    Tech,
}

/// The strongest fixed-duration section for a pattern.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Peak {
    /// Start of the strongest window in milliseconds.
    pub start_time_ms: f64,
    /// Normalized strength of the pattern inside the window.
    pub score: f64,
}

/// Comparable summary used to rank patterns.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternScore {
    /// Pattern represented by this ranking entry.
    pub pattern: Pattern,
    /// Normalized pattern strength in the range `0.0..=1.0`.
    pub score: f64,
    /// Number of objects attributed to the pattern.
    pub objects: usize,
    /// Number of contiguous pattern segments.
    pub segments: usize,
    /// Strongest fixed-duration section, when the pattern is present.
    pub peak: Option<Peak>,
}

/// Stream-specific measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StreamAnalysis {
    /// Normalized stream strength in the range `0.0..=1.0`.
    pub score: f64,
    /// Number of detected stream segments.
    pub segments: usize,
    /// Total notes contained in stream segments.
    pub notes: usize,
    /// Notes in the longest stream segment.
    pub longest: usize,
    /// Mean interval between stream notes in milliseconds.
    pub average_note_interval_ms: f64,
    /// Strongest fixed-duration stream section.
    pub peak: Option<Peak>,
}

/// Burst-specific measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BurstAnalysis {
    /// Normalized burst strength in the range `0.0..=1.0`.
    pub score: f64,
    /// Number of detected burst segments.
    pub segments: usize,
    /// Total notes contained in burst segments.
    pub notes: usize,
    /// Notes in the longest burst segment.
    pub longest: usize,
    /// Mean interval between burst notes in milliseconds.
    pub average_note_interval_ms: f64,
    /// Strongest fixed-duration burst section.
    pub peak: Option<Peak>,
}

/// Jump-specific measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpAnalysis {
    /// Normalized jump strength in the range `0.0..=1.0`.
    pub score: f64,
    /// Number of detected jump sequences.
    pub segments: usize,
    /// Total notes contained in jump sequences.
    pub notes: usize,
    /// Notes in the longest jump sequence.
    pub longest: usize,
    /// Mean edge distance measured in circle radii.
    pub average_distance: f64,
    /// Largest edge distance measured in circle radii.
    pub max_distance: f64,
    /// Strongest fixed-duration jump section.
    pub peak: Option<Peak>,
}

/// Slider-specific measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SliderAnalysis {
    /// Normalized slider strength in the range `0.0..=1.0`.
    pub score: f64,
    /// Number of sliders in the beatmap.
    pub count: usize,
    /// Total extra slider spans after each slider's first span.
    pub repeats: usize,
    /// Total slider travel measured in circle radii.
    pub travel_distance: f64,
    /// Fraction of playable map duration occupied by sliders.
    pub duration_ratio: f64,
    /// Strongest fixed-duration slider section.
    pub peak: Option<Peak>,
}

/// Technical-complexity measurements.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TechAnalysis {
    /// Normalized technical strength in the range `0.0..=1.0`.
    pub score: f64,
    /// Normalized timing-pattern complexity.
    pub rhythm_complexity: f64,
    /// Normalized movement-angle complexity.
    pub angle_complexity: f64,
    /// Number of inherited slider-velocity changes encountered.
    pub slider_velocity_changes: usize,
    /// Strongest fixed-duration technical section.
    pub peak: Option<Peak>,
}

/// Complete analysis for one osu!standard beatmap.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MapAnalysis {
    /// Highest-ranked pattern, provided only when it clears the configured floor.
    pub primary: Option<Pattern>,
    /// All supported patterns in descending score order.
    pub patterns: Vec<PatternScore>,
    /// Detailed jump measurements.
    pub jump: JumpAnalysis,
    /// Detailed stream measurements.
    pub stream: StreamAnalysis,
    /// Detailed burst measurements.
    pub burst: BurstAnalysis,
    /// Detailed slider measurements.
    pub slider: SliderAnalysis,
    /// Detailed technical-complexity measurements.
    pub tech: TechAnalysis,
    /// Number of playable circles and sliders.
    pub object_count: usize,
    /// Number of circles, excluding sliders and sequence-breaking objects.
    pub tap_object_count: usize,
    /// Time between the first playable object and the last playable end time.
    pub duration_ms: f64,
}
