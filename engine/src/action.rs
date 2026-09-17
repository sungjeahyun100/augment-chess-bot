use crate::{CardInstanceId, Color, PieceId, PieceKind, Square};
use serde::{Deserialize, Serialize};

/// Ordered targets and route are semantic input, not a sequence of UI clicks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Target {
    Square { square: Square },
    Piece { piece: PieceId },
    Player { color: Color },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Move {
        from: Square,
        to: Square,
        route: Vec<Square>,
    },
    Promote {
        piece: PieceId,
        into: PieceKind,
    },
    ActivateCard {
        card: CardInstanceId,
        targets: Vec<Target>,
    },
    ChooseDraftCard {
        offer: u32,
        card: CardInstanceId,
    },
    ResolveChoice {
        request: u32,
        option: u32,
    },
    FinishOptionalContinuation,
}
