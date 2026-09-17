//! Phase 1: validated state and replay boundaries. Move generation is Phase 2.
#![forbid(unsafe_code)]
mod action;
mod board;
mod chance;
mod state;
mod types;

pub use action::*;
pub use board::*;
pub use chance::*;
pub use state::*;
pub use types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineError {
    InvalidState(String),
    InvalidAction(String),
    Unsupported(&'static str),
    Terminal,
    ChanceExhausted,
    InvalidChance,
    Serialization(String),
}
impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EngineError {}
pub type EngineResult<T> = Result<T, EngineError>;
pub(crate) fn invalid(message: impl Into<String>) -> EngineError {
    EngineError::InvalidState(message.into())
}
