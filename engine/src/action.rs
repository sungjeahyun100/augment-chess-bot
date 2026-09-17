use crate::{
    CardInstanceId, Color, LogDirection, PieceId, PieceKind, ShotgunDirection, Square, WizardSpell,
};
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
    SetLogDirection {
        piece: PieceId,
        direction: LogDirection,
    },
    Reload {
        piece: PieceId,
    },
    ShotgunBlast {
        piece: PieceId,
        direction: ShotgunDirection,
    },
    ShotgunSnipe {
        piece: PieceId,
        target: Square,
    },
    CastSpell {
        wizard: PieceId,
        spell: WizardSpell,
        target: Square,
    },
    Purchase {
        merchant: PieceId,
        target: PieceId,
    },
    AttackSector {
        piece: PieceId,
        sector: u8,
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
