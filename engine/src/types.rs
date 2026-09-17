use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    White,
    Black,
}
impl Color {
    pub const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Owner {
    White,
    Black,
    Neutral,
}
impl From<Color> for Owner {
    fn from(color: Color) -> Self {
        match color {
            Color::White => Self::White,
            Color::Black => Self::Black,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sides<T> {
    pub white: T,
    pub black: T,
}
impl<T> Sides<T> {
    pub fn get_mut(&mut self, color: Color) -> &mut T {
        match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        }
    }
    pub fn get(&self, color: Color) -> &T {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PieceId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CardInstanceId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Square {
    pub row: u16,
    pub col: u16,
}
impl Square {
    pub const fn new(row: u16, col: u16) -> Self {
        Self { row, col }
    }
}
/// Stable reference piece identifiers; support is checked before execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    Herald,
    Recruiter,
    Merchant,
    Wizard,
    ShotgunKing,
    Log,
    Berserker,
    Princess,
    Clockwork,
    Squire,
    StandardBearer,
    Checker,
    CheckerKing,
    Man,
    Ferz,
    Alfil,
    Camel,
    Eagle,
    Pegasus,
    Fanatic,
    PrimeMinister,
    RoyalKnight,
    Amazon,
    Knightmaster,
    Windmill,
    Assassin,
    Guard,
    Cannon,
    Grasshopper,
    Hook,
    Cardinal,
    Protestant,
    Colossus,
    BigRook,
    BigBishop,
    Wall,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Status {
    /// JS freshNoCaptureUntil compares against the owner's completed turns.
    CannotCaptureUntilOwnerTurn { owner: Color, completed_turn: u32 },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Piece {
    pub id: PieceId,
    pub owner: Owner,
    pub kind: PieceKind,
    pub anchor: Square,
    /// Absolute occupied squares, a set normalized into row-major order.
    pub footprint: Vec<Square>,
    pub origin: Option<Square>,
    pub moved: bool,
    pub shielded: bool,
    pub hp: Option<u16>,
    pub max_hp: Option<u16>,
    pub statuses: Vec<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub windmill_mode: Option<WindmillMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub herald_jump_lock_turn: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub herald_jump_locked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gold: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mana: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_mana: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ammo: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_ammo: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facing: Option<Facing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_direction: Option<LogDirection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_roll_after_turn: Option<u32>,
}
impl Piece {
    pub fn new(id: PieceId, owner: Owner, kind: PieceKind, anchor: Square) -> Self {
        Self {
            id,
            owner,
            kind,
            anchor,
            footprint: vec![anchor],
            origin: Some(anchor),
            moved: false,
            shielded: false,
            hp: None,
            max_hp: None,
            statuses: vec![],
            windmill_mode: None,
            herald_jump_lock_turn: None,
            herald_jump_locked: false,
            gold: None,
            mana: None,
            max_mana: None,
            ammo: None,
            max_ammo: None,
            facing: None,
            log_direction: None,
            log_roll_after_turn: None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardCategory {
    Opening,
    Middle,
    End,
    Piece,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationType {
    Passive,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindmillMode {
    Bishop,
    Rook,
}

impl PieceKind {
    pub(crate) fn large(self) -> bool {
        matches!(self, Self::Colossus | Self::BigRook | Self::BigBishop)
    }
    pub(crate) fn pawn_mover(self) -> bool {
        matches!(self, Self::Pawn | Self::Squire | Self::StandardBearer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogDirection {
    pub dr: i8,
    pub dc: i8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShotgunDirection {
    pub dr: i8,
    pub dc: i8,
}
impl PieceKind {
    pub(crate) fn royal(self) -> bool {
        matches!(self, Self::King | Self::ShotgunKing | Self::RoyalKnight)
    }
    pub(crate) fn defeat_royal(self) -> bool {
        self.royal() || self == Self::Merchant
    }
}
