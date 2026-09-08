use crate::AnalysisError;

/// Thresholds used by [`crate::Analyzer`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalysisConfig {
    /// Maximum wall-clock interval between notes in a fast tap run.
    pub fast_interval_ms: f64,
    /// Maximum local beat interval between notes in a fast tap run.
    pub fast_interval_beats: f64,
    /// Maximum relative interval deviation allowed inside one tap run.
    pub rhythm_tolerance: f64,
    /// Minimum number of notes that turns a fast tap run into a stream.
    pub stream_min_notes: usize,
    /// Minimum jump distance measured in circle radii.
    pub jump_min_distance: f64,
    /// Maximum wall-clock interval between notes in a jump sequence.
    pub jump_max_interval_ms: f64,
    /// Maximum local beat interval between notes in a jump sequence.
    pub jump_max_interval_beats: f64,
    /// Duration of the sliding window used to calculate pattern peaks.
    pub peak_window_ms: f64,
    /// Minimum top score required to choose a primary pattern.
    pub primary_score_min: f64,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            fast_interval_ms: 200.0,
            fast_interval_beats: 0.375,
            rhythm_tolerance: 0.2,
            stream_min_notes: 6,
            jump_min_distance: 2.5,
            jump_max_interval_ms: 600.0,
            jump_max_interval_beats: 1.25,
            peak_window_ms: 2_000.0,
            primary_score_min: 0.15,
        }
    }
}

impl AnalysisConfig {
    pub(crate) fn validate(self) -> Result<Self, AnalysisError> {
        validate_positive("fast_interval_ms", self.fast_interval_ms)?;
        validate_positive("fast_interval_beats", self.fast_interval_beats)?;
        validate_fraction("rhythm_tolerance", self.rhythm_tolerance)?;

        if self.stream_min_notes < 4 {
            return Err(AnalysisError::InvalidConfig("stream_min_notes"));
        }

        validate_positive("jump_min_distance", self.jump_min_distance)?;
        validate_positive("jump_max_interval_ms", self.jump_max_interval_ms)?;
        validate_positive("jump_max_interval_beats", self.jump_max_interval_beats)?;
        validate_positive("peak_window_ms", self.peak_window_ms)?;
        validate_fraction("primary_score_min", self.primary_score_min)?;

        Ok(self)
    }
}

fn validate_positive(name: &'static str, value: f64) -> Result<(), AnalysisError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(AnalysisError::InvalidConfig(name))
    }
}

fn validate_fraction(name: &'static str, value: f64) -> Result<(), AnalysisError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(AnalysisError::InvalidConfig(name))
    }
}
