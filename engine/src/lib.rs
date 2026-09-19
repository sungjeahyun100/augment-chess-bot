//! Deterministic cardless augment-chess rules with validated replay boundaries.
#![forbid(unsafe_code)]
mod action;
mod bear;
mod board;
mod castling;
mod chance;
mod effects;
mod large;
mod log;
mod merchant;
mod movement;
mod shotgun;
mod state;
mod transition;
mod types;
mod variants;
mod victory;
mod wizard;
pub use bear::PendingBearRetaliation;
pub use wizard::{DelayedSpell, DelayedSpellKind, WizardSpell};

pub use action::*;
pub use board::*;
pub use chance::*;
pub use state::*;
pub use types::*;
pub use victory::star_tiebreak;

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
