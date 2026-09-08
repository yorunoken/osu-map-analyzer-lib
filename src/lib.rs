//! Deterministic pattern analysis for osu!standard beatmaps.

mod analyzer;
mod config;
mod error;
mod features;
mod model;

pub use analyzer::Analyzer;
pub use config::AnalysisConfig;
pub use error::AnalysisError;
pub use model::{
    BurstAnalysis, JumpAnalysis, MapAnalysis, Pattern, PatternScore, Peak, SliderAnalysis,
    StreamAnalysis, TechAnalysis,
};
pub use rosu_map;
