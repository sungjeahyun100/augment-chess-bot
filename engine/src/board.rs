use crate::{EngineResult, Piece, PieceId, Square, invalid};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Occupancy contains IDs, never duplicate entity records or shared pointers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Board {
    rows: u16,
    cols: u16,
    cells: Vec<Option<PieceId>>,
}
impl Board {
    pub fn empty(rows: u16, cols: u16) -> EngineResult<Self> {
        if rows == 0 || cols == 0 || rows > 64 || cols > 64 {
            return Err(invalid("board dimensions must be 1..=64"));
        }
        Ok(Self {
            rows,
            cols,
            cells: vec![None; usize::from(rows) * usize::from(cols)],
        })
    }
    pub fn from_pieces(rows: u16, cols: u16, pieces: &[Piece]) -> EngineResult<Self> {
        let mut board = Self::empty(rows, cols)?;
        let mut ids = BTreeSet::new();
        for piece in pieces {
            if piece.id.0 == 0 || !ids.insert(piece.id) {
                return Err(invalid("zero or duplicate piece ID"));
            }
            let footprint: BTreeSet<_> = piece.footprint.iter().copied().collect();
            if footprint.len() != piece.footprint.len() || !footprint.contains(&piece.anchor) {
                return Err(invalid(
                    "footprint must be a nonempty set containing anchor",
                ));
            }
            if piece.origin.is_some_and(|s| !board.contains(s)) {
                return Err(invalid("origin out of bounds"));
            }
            for square in footprint {
                let index = board
                    .index(square)
                    .ok_or_else(|| invalid("footprint out of bounds"))?;
                if board.cells[index].replace(piece.id).is_some() {
                    return Err(invalid("overlapping entities"));
                }
            }
        }
        Ok(board)
    }
    pub const fn rows(&self) -> u16 {
        self.rows
    }
    pub const fn cols(&self) -> u16 {
        self.cols
    }
    pub fn contains(&self, square: Square) -> bool {
        square.row < self.rows && square.col < self.cols
    }
    fn index(&self, square: Square) -> Option<usize> {
        self.contains(square)
            .then(|| usize::from(square.row) * usize::from(self.cols) + usize::from(square.col))
    }
    pub fn at(&self, square: Square) -> EngineResult<Option<PieceId>> {
        let index = self
            .index(square)
            .ok_or_else(|| invalid("square out of bounds"))?;
        self.cells
            .get(index)
            .copied()
            .ok_or_else(|| invalid("invalid cell count"))
    }
    pub fn cells(&self) -> &[Option<PieceId>] {
        &self.cells
    }
    pub fn validate(&self, pieces: &[Piece]) -> EngineResult<()> {
        if *self != Self::from_pieces(self.rows, self.cols, pieces)? {
            return Err(invalid("board/entity mismatch"));
        }
        Ok(())
    }
}
