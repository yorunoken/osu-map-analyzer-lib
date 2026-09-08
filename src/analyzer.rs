use rosu_map::{
    section::{general::GameMode, hit_objects::HitObjectKind},
    Beatmap,
};

use crate::{
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

        let mut playable_times = self.map.hit_objects.iter().filter_map(|object| {
            matches!(
                object.kind,
                HitObjectKind::Circle(_) | HitObjectKind::Slider(_)
            )
            .then_some(object.start_time)
            .filter(|time| time.is_finite())
        });
        let Some(first_time) = playable_times.next() else {
            return Err(AnalysisError::EmptyMap);
        };
        let (object_count, last_time) = playable_times.fold(
            (1, first_time),
            |(count, last), time| (count + 1, last.max(time)),
        );

        Ok(MapAnalysis {
            primary: None,
            patterns: Vec::new(),
            jump: JumpAnalysis::default(),
            stream: StreamAnalysis::default(),
            burst: BurstAnalysis::default(),
            slider: SliderAnalysis::default(),
            tech: TechAnalysis::default(),
            object_count,
            tap_object_count: object_count,
            duration_ms: (last_time - first_time).max(0.0),
        })
    }
}
