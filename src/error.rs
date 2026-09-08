use std::{error::Error, fmt};

use rosu_map::section::general::GameMode;

/// A map or configuration that cannot be analyzed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisError {
    UnsupportedMode(GameMode),
    EmptyMap,
    InvalidConfig(&'static str),
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedMode(mode) => {
                write!(formatter, "unsupported game mode: {mode:?}; expected osu!standard")
            }
            Self::EmptyMap => formatter.write_str("beatmap contains no playable objects"),
            Self::InvalidConfig(field) => write!(formatter, "invalid analysis config field: {field}"),
        }
    }
}

impl Error for AnalysisError {}
