use rosu_map::{section::general::GameMode, Beatmap};

use crate::{
    features::FeatureSet,
    AnalysisConfig, AnalysisError, BurstAnalysis, JumpAnalysis, MapAnalysis, SliderAnalysis,
    StreamAnalysis, TechAnalysis,
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

        Ok(MapAnalysis {
            primary: None,
            patterns: Vec::new(),
            jump: JumpAnalysis::default(),
            stream: StreamAnalysis::default(),
            burst: BurstAnalysis::default(),
            slider: SliderAnalysis::default(),
            tech: TechAnalysis::default(),
            object_count: features.objects.len(),
            tap_object_count: features.circle_count,
            duration_ms: features.duration_ms,
        })
    }
}
