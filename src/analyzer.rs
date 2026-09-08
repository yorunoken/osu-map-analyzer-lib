use rosu_map::{section::general::GameMode, Beatmap};

use crate::{
    features::FeatureSet,
    patterns::{jump, tap},
    AnalysisConfig, AnalysisError, MapAnalysis, SliderAnalysis, TechAnalysis,
};

/// Analyzer for one borrowed osu!standard beatmap.
pub struct Analyzer<'map> {
    map: &'map Beatmap,
    config: AnalysisConfig,
}

impl<'map> Analyzer<'map> {
    /// Create an analyzer with [`AnalysisConfig::default`].
    pub fn new(map: &'map Beatmap) -> Self {
        Self {
            map,
            config: AnalysisConfig::default(),
        }
    }

    /// Create an analyzer with validated custom thresholds.
    pub fn with_config(
        map: &'map Beatmap,
        config: AnalysisConfig,
    ) -> Result<Self, AnalysisError> {
        Ok(Self {
            map,
            config: config.validate()?,
        })
    }

    /// Analyze the beatmap.
    pub fn analyze(&self) -> Result<MapAnalysis, AnalysisError> {
        if self.map.mode != GameMode::Osu {
            return Err(AnalysisError::UnsupportedMode(self.map.mode));
        }

        self.config.validate()?;

        let features = FeatureSet::extract(self.map, &self.config);

        if features.objects.is_empty() {
            return Err(AnalysisError::EmptyMap);
        }

        let tap = tap::detect(&features, &self.config);
        let jump = jump::detect(&features, &tap.stream_edges, &self.config);

        Ok(MapAnalysis {
            primary: None,
            patterns: Vec::new(),
            jump,
            stream: tap.stream,
            burst: tap.burst,
            slider: SliderAnalysis::default(),
            tech: TechAnalysis::default(),
            object_count: features.objects.len(),
            tap_object_count: features.circle_count,
            duration_ms: features.duration_ms,
        })
    }
}
