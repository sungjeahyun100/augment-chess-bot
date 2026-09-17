use crate::{EngineError, EngineResult, invalid};
use serde::{Deserialize, Serialize};

/// Versioned explicit randomness. Never used for entity IDs or presentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "algorithm", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChanceState {
    /// SplitMix64, stored as two u32 words to preserve JSON integer precision.
    Splitmix64V1 { high: u32, low: u32 },
    /// Each entry records the candidate count as well as the chosen index.
    TapeV1 {
        outcomes: Vec<ChanceOutcome>,
        cursor: usize,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChanceOutcome {
    pub candidates: u32,
    pub index: u32,
}
impl ChanceState {
    pub fn seeded(seed: u64) -> Self {
        Self::Splitmix64V1 {
            high: (seed >> 32) as u32,
            low: seed as u32,
        }
    }
    pub fn tape(outcomes: Vec<ChanceOutcome>) -> EngineResult<Self> {
        let result = Self::TapeV1 {
            outcomes,
            cursor: 0,
        };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> EngineResult<()> {
        if let Self::TapeV1 { outcomes, cursor } = self {
            if *cursor > outcomes.len()
                || outcomes
                    .iter()
                    .any(|o| o.candidates == 0 || o.index >= o.candidates)
            {
                return Err(invalid("invalid chance tape"));
            }
        }
        Ok(())
    }
    /// Failure leaves the tape/RNG unchanged. Candidate order belongs to caller.
    pub fn choose(&mut self, candidates: u32) -> EngineResult<u32> {
        if candidates == 0 {
            return Err(EngineError::InvalidChance);
        }
        self.validate()?;
        match self {
            Self::TapeV1 { outcomes, cursor } => {
                let outcome = outcomes.get(*cursor).ok_or(EngineError::ChanceExhausted)?;
                if outcome.candidates != candidates {
                    return Err(EngineError::InvalidChance);
                }
                *cursor += 1;
                Ok(outcome.index)
            }
            Self::Splitmix64V1 { high, low } => {
                let mut state = (u64::from(*high) << 32) | u64::from(*low);
                let bound = u64::from(candidates);
                let threshold = bound.wrapping_neg() % bound;
                loop {
                    state = state.wrapping_add(0x9e3779b97f4a7c15);
                    let mut z = state;
                    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
                    z ^= z >> 31;
                    if z >= threshold {
                        *high = (state >> 32) as u32;
                        *low = state as u32;
                        return Ok((z % bound) as u32);
                    }
                }
            }
        }
    }
}
