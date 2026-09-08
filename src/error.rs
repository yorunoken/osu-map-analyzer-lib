use std::{error::Error, fmt};

use rosu_map::section::general::GameMode;

/// A map or configuration that cannot be analyzed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisError {
    /// The beatmap uses a mode other than osu!standard.
    UnsupportedMode(GameMode),
    /// The beatmap contains no circles or sliders to analyze.
    EmptyMap,
    /// A named [`crate::AnalysisConfig`] field is outside its valid range.
    InvalidConfig(&'static str),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMode(mode) => {
                write!(
                    formatter,
                    "unsupported game mode: {mode:?}; expected osu!standard"
                )
            }
            Self::EmptyMap => formatter.write_str("beatmap contains no playable objects"),
            Self::InvalidConfig(field) => {
                write!(formatter, "invalid analysis config field: {field}")
            }
        }
    }
}

impl Error for AnalysisError {}
