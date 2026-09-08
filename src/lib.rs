//! Deterministic pattern analysis for osu!standard beatmaps.
//!
//! The crate extracts local timing, normalized geometry, tap runs, slider
//! movement, and rhythm changes from a borrowed [`rosu_map::Beatmap`]. Results
//! are normalized to `0.0..=1.0` so pattern strengths can be compared within a
//! map.
//!
//! # Example
//!
//! ```no_run
//! use osu_map_analyzer::{rosu_map::Beatmap, Analyzer};
//!
//! let map = Beatmap::from_path("example.osu")?;
//! let analysis = Analyzer::new(&map).analyze()?;
//!
//! println!("primary pattern: {:?}", analysis.primary);
//! println!("stream score: {:.3}", analysis.stream.score);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![warn(missing_docs)]

mod analyzer;
mod config;
mod error;
mod features;
mod model;
mod patterns;

pub use analyzer::Analyzer;
pub use config::AnalysisConfig;
pub use error::AnalysisError;
pub use model::{
    BurstAnalysis, JumpAnalysis, MapAnalysis, Pattern, PatternScore, Peak, SliderAnalysis,
    StreamAnalysis, TechAnalysis,
};
pub use rosu_map;
