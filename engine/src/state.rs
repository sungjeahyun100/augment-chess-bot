use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SCHEMA_VERSION: u32 = 3;
pub const SOURCE_SHA256: &str = "0dbbad680c6e8e2abcdb6e49817ff486aa2f8ef799a8035bd356baef8a2731ea";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulesProfile {
    #[serde(rename = "local_0dbbad68")]
    LocalReference,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    #[serde(rename = "phase1_state_only")]
    StateOnly,
    #[serde(rename = "phase3_cardless")]
    Cardless,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameConfig {
    pub rules_profile: RulesProfile,
    pub draft_enabled: bool,
    pub opening_rules_enabled: bool,
    pub star_win_limit: u32,
    pub deathmatch_enabled: bool,
    pub deathmatch_limit_turns: u32,
}
impl GameConfig {
    /// Explicit cardless setup; does not pretend to implement the website defaults.
    pub fn cardless() -> Self {
        Self {
            rules_profile: RulesProfile::LocalReference,
            draft_enabled: false,
            opening_rules_enabled: false,
            star_win_limit: 45,
            deathmatch_enabled: true,
            deathmatch_limit_turns: 10,
        }
    }
    fn validate(&self) -> EngineResult<()> {
        if self.star_win_limit == 0
            || self.deathmatch_limit_turns == 0
            || self.deathmatch_limit_turns > u32::MAX / 2
        {
            return Err(invalid("invalid endgame limits"));
        }
        if self.draft_enabled {
            return Err(EngineError::Unsupported("draft generation (Phase 6)"));
        }
        if self.opening_rules_enabled {
            return Err(EngineError::Unsupported("match RULE (Phase 7)"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub color: Color,
    /// Slot order and empty slots are meaningful, never sorted or compacted.
    pub card_slots: Vec<Option<CardInstanceId>>,
    pub cards_used_this_turn: u32,
    pub first_move_cards_forced: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnState {
    pub side: Color,
    pub completed: Sides<u32>,
    pub full_move: u32,
    pub move_count: u32,
    pub actions_remaining: u32,
    pub continuation: Option<Continuation>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Continuation {
    ExtraMove { piece: PieceId, optional: bool },
    CheckerCapture { piece: PieceId },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase {
    Play,
    Terminal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decision {
    Player { color: Color },
    Terminal,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GameResult {
    Win { winner: Color, reason: EndReason },
    Draw { reason: EndReason },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndReason {
    RoyalCapture,
    HeraldAgreement,
    RoyalPurchase,
    NoActions,
    RepetitionStars,
    TurnLimitStars,
    Resignation,
    Agreement,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnPassant {
    pub pawn: PieceId,
    pub target: Square,
    pub available_to: Color,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepetitionEntry {
    pub key: String,
    pub count: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleHistory {
    pub en_passant: Option<EnPassant>,
    pub castling_canceled: Sides<bool>,
    pub castled: Sides<bool>,
    pub repetition_salt: u32,
    /// Set keyed by JS repetition key, not a full-state transposition key.
    pub position_counts: Vec<RepetitionEntry>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdAllocators {
    pub next_piece: u32,
    pub next_card: u32,
    pub next_effect: u32,
}

/// Public interchange DTO. Deserialize is structural only; use GameState::from_snapshot
/// for semantic validation. Unknown fields are rejected, including nested records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalState {
    pub schema_version: u32,
    pub capability: Capability,
    pub config: GameConfig,
    pub board: Board,
    pub pieces: Vec<Piece>,
    pub players: Sides<Player>,
    pub turn: TurnState,
    pub phase: Phase,
    pub history: RuleHistory,
    pub chance: ChanceState,
    pub ids: IdAllocators,
    pub result: Option<GameResult>,
    pub pending_promotion: Option<PieceId>,
    pub deathmatch: Option<Deathmatch>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delayed_spells: Vec<DelayedSpell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub time_stopped: Vec<Color>,
}

/// Validated core. No writable references, global state, clocks, or I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameState {
    pub(crate) snapshot: CanonicalState,
}
impl GameState {
    pub fn new(config: GameConfig, seed: u64) -> EngineResult<Self> {
        config.validate()?;
        let back_rank = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];
        let mut pieces = Vec::with_capacity(32);
        let mut add = |owner, kind, row, col| {
            pieces.push(Piece::new(
                PieceId(pieces.len() as u32 + 1),
                owner,
                kind,
                Square::new(row, col),
            ));
        };
        // Match createInitialBoard's allocation order; ID remap is not based on current square.
        for (col, kind) in back_rank.into_iter().enumerate() {
            add(Owner::Black, kind, 0, col as u16);
            add(Owner::White, kind, 7, col as u16);
        }
        for col in 0..8 {
            add(Owner::Black, PieceKind::Pawn, 1, col);
            add(Owner::White, PieceKind::Pawn, 6, col);
        }
        let player = |color| Player {
            color,
            card_slots: vec![None; 3],
            cards_used_this_turn: 0,
            first_move_cards_forced: false,
        };
        let mut game = Self::from_snapshot(CanonicalState {
            schema_version: SCHEMA_VERSION,
            capability: Capability::Cardless,
            config,
            board: Board::from_pieces(8, 8, &pieces)?,
            pieces,
            players: Sides {
                white: player(Color::White),
                black: player(Color::Black),
            },
            turn: TurnState {
                side: Color::White,
                completed: Sides { white: 0, black: 0 },
                full_move: 1,
                move_count: 0,
                actions_remaining: 1,
                continuation: None,
            },
            phase: Phase::Play,
            history: RuleHistory {
                en_passant: None,
                castling_canceled: Sides {
                    white: false,
                    black: false,
                },
                castled: Sides {
                    white: false,
                    black: false,
                },
                repetition_salt: 0,
                position_counts: vec![],
            },
            chance: ChanceState::seeded(seed),
            ids: IdAllocators {
                next_piece: 33,
                next_card: 1,
                next_effect: 1,
            },
            result: None,
            pending_promotion: None,
            deathmatch: None,
            delayed_spells: vec![],
            time_stopped: vec![],
        })?;
        crate::victory::record_position(&mut game.snapshot)?;
        Ok(game)
    }
    pub fn from_snapshot(mut snapshot: CanonicalState) -> EngineResult<Self> {
        Self::validate_snapshot(&snapshot)?;
        snapshot.time_stopped.sort();
        snapshot.pieces.sort_by_key(|p| p.id);
        for piece in &mut snapshot.pieces {
            if piece.windmill_mode == Some(WindmillMode::Bishop) {
                piece.windmill_mode = None;
            }
            piece.footprint.sort();
            piece.statuses.sort();
        }
        snapshot
            .history
            .position_counts
            .sort_by(|a, b| a.key.cmp(&b.key));
        Ok(Self { snapshot })
    }
    fn validate_snapshot(s: &CanonicalState) -> EngineResult<()> {
        if s.schema_version != SCHEMA_VERSION {
            return Err(EngineError::Unsupported("canonical schema version"));
        }
        s.config.validate()?;
        s.board.validate(&s.pieces)?;
        s.chance.validate()?;
        crate::wizard::validate(s)?;
        crate::log::validate(s)?;
        crate::shotgun::validate(s)?;
        if s.ids.next_piece == 0
            || s.ids.next_card == 0
            || s.ids.next_effect == 0
            || s.pieces.iter().any(|p| p.id.0 >= s.ids.next_piece)
        {
            return Err(invalid("allocator must exceed all allocated IDs"));
        }
        for p in &s.pieces {
            if p.gold.is_some() && p.kind != PieceKind::Merchant {
                return Err(invalid("gold on a non-merchant"));
            }
            if (p.herald_jump_lock_turn.is_some() || p.herald_jump_locked)
                && p.kind != PieceKind::Herald
            {
                return Err(invalid("herald jump state on a different piece"));
            }
            if p.windmill_mode.is_some() && p.kind != PieceKind::Windmill {
                return Err(invalid("windmill mode on a different piece"));
            }
            match (p.hp, p.max_hp) {
                (None, None) => {}
                (Some(hp), Some(max)) if hp > 0 && hp <= max => {}
                _ => return Err(invalid("invalid HP pair")),
            }
            if p.statuses.iter().collect::<BTreeSet<_>>().len() != p.statuses.len() {
                return Err(invalid("duplicate status"));
            }
            for status in &p.statuses {
                match status {
                    Status::CannotCaptureUntilOwnerTurn { owner, .. }
                        if p.owner != Owner::from(*owner) =>
                    {
                        return Err(invalid("capture deadline owner mismatch"));
                    }
                    _ => {}
                }
            }
        }
        if s.players.white.color != Color::White || s.players.black.color != Color::Black {
            return Err(invalid("player color mismatch"));
        }
        if s.players.white.card_slots.len() != 3 || s.players.black.card_slots.len() != 3 {
            return Err(invalid(
                "normal profile requires three card slots per player",
            ));
        }
        if s.players
            .white
            .card_slots
            .iter()
            .chain(&s.players.black.card_slots)
            .any(Option::is_some)
        {
            return Err(EngineError::Unsupported("card instances (Phase 5)"));
        }
        if s.turn.full_move == 0 {
            return Err(invalid("full_move starts at 1"));
        }
        if matches!(s.phase, Phase::Terminal) != s.result.is_some() {
            return Err(invalid("terminal phase/result mismatch"));
        }
        if let Some(
            Continuation::ExtraMove { piece, .. } | Continuation::CheckerCapture { piece },
        ) = s.turn.continuation
        {
            let p = s
                .pieces
                .iter()
                .find(|p| p.id == piece)
                .ok_or_else(|| invalid("unknown continuation piece"))?;
            if matches!(
                s.turn.continuation,
                Some(Continuation::CheckerCapture { .. })
            ) && !matches!(p.kind, PieceKind::Checker | PieceKind::CheckerKing)
            {
                return Err(invalid("checker continuation requires a checker"));
            }
            if p.owner != Owner::from(s.turn.side) || s.result.is_some() {
                return Err(invalid("invalid continuation owner/phase"));
            }
        }
        if let Some(ep) = &s.history.en_passant {
            let p = s
                .pieces
                .iter()
                .find(|p| p.id == ep.pawn)
                .ok_or_else(|| invalid("unknown en passant pawn"))?;
            if p.kind != PieceKind::Pawn
                || p.owner == Owner::Neutral
                || !s.board.contains(ep.target)
            {
                return Err(invalid("invalid en passant reference"));
            }
            if s.capability == Capability::Cardless {
                // Purchase changes ownership, but the recorded double-step color remains.
                let advanced_owner = Owner::from(ep.available_to.opponent());
                let expected_row = match advanced_owner {
                    Owner::White => p.anchor.row.checked_add(1),
                    Owner::Black => p.anchor.row.checked_sub(1),
                    Owner::Neutral => None,
                };
                if expected_row != Some(ep.target.row)
                    || ep.target.col != p.anchor.col
                    || !p.moved
                    || !match advanced_owner {
                        Owner::White => [4, 5].contains(&p.anchor.row),
                        Owner::Black => [2, 3].contains(&p.anchor.row),
                        Owner::Neutral => false,
                    }
                {
                    return Err(invalid("invalid cardless en passant geometry"));
                }
            }
        }
        if let Some(id) = s.pending_promotion {
            let p = s
                .pieces
                .iter()
                .find(|p| p.id == id)
                .ok_or_else(|| invalid("unknown promotion piece"))?;
            if s.result.is_some()
                || !p.kind.pawn_mover()
                || p.owner != Owner::from(s.turn.side)
                || p.anchor.row
                    != if s.turn.side == Color::White {
                        0
                    } else {
                        s.board.rows() - 1
                    }
            {
                return Err(invalid("invalid pending promotion"));
            }
        }
        if let Some(dm) = &s.deathmatch {
            if !s.config.deathmatch_enabled
                || dm.interval_half_turns == 0
                || dm.half_turns_since_progress > dm.interval_half_turns
            {
                return Err(invalid("invalid deathmatch state"));
            }
        }
        let mut keys = BTreeSet::new();
        if s.history
            .position_counts
            .iter()
            .any(|e| e.count == 0 || !keys.insert(&e.key))
        {
            return Err(invalid("invalid repetition counts"));
        }
        Ok(())
    }
    pub fn snapshot(&self) -> CanonicalState {
        self.snapshot.clone()
    }
    pub fn board(&self) -> &Board {
        &self.snapshot.board
    }
    pub fn pieces(&self) -> &[Piece] {
        &self.snapshot.pieces
    }
    pub fn piece(&self, id: PieceId) -> Option<&Piece> {
        self.snapshot
            .pieces
            .binary_search_by_key(&id, |p| p.id)
            .ok()
            .map(|i| &self.snapshot.pieces[i])
    }
    pub fn player(&self, color: Color) -> &Player {
        self.snapshot.players.get(color)
    }
    pub fn turn(&self) -> &TurnState {
        &self.snapshot.turn
    }
    pub fn side_to_move(&self) -> Color {
        self.snapshot.turn.side
    }
    pub fn decision(&self) -> Decision {
        if self.is_terminal() {
            Decision::Terminal
        } else {
            Decision::Player {
                color: self.side_to_move(),
            }
        }
    }
    pub fn is_terminal(&self) -> bool {
        self.snapshot.result.is_some()
    }
    pub fn result(&self) -> Option<&GameResult> {
        self.snapshot.result.as_ref()
    }
    pub fn legal_actions(&self) -> EngineResult<Vec<Action>> {
        crate::movement::legal_actions(&self.snapshot)
    }
    /// Validate first, resolve on a clone, and commit only a valid complete result.
    pub fn apply_action(&mut self, action: Action) -> EngineResult<()> {
        if self.is_terminal() {
            return Err(EngineError::Terminal);
        }
        if !self.legal_actions()?.contains(&action) {
            return Err(EngineError::InvalidAction(
                "action is not legal in this state".into(),
            ));
        }
        let mut next = self.snapshot.clone();
        crate::transition::apply(&mut next, action)?;
        *self = Self::from_snapshot(next)?;
        Ok(())
    }
    /// Threat information is advisory; it does not filter ordinary moves.
    pub fn is_in_check(&self, color: Color) -> EngineResult<bool> {
        crate::movement::ensure_supported(&self.snapshot)?;
        Ok(self.pieces().iter().any(|p| {
            p.owner == Owner::from(color)
                && p.kind.royal()
                && crate::movement::attacked(&self.snapshot, p.anchor, color.opponent())
        }))
    }
    /// Sorted object keys, compact UTF-8 JSON, no floats; ordered arrays preserved.
    pub fn to_canonical_json(&self) -> EngineResult<String> {
        // serde_json without preserve_order stores every object in a BTreeMap.
        let value = serde_json::to_value(&self.snapshot)
            .map_err(|e| EngineError::Serialization(e.to_string()))?;
        serde_json::to_string(&value).map_err(|e| EngineError::Serialization(e.to_string()))
    }
    pub fn from_canonical_json(json: &str) -> EngineResult<Self> {
        // Deserialize directly to the DTO: duplicate struct keys cannot be lost
        // in an intermediate JSON Value map.
        let snapshot =
            serde_json::from_str(json).map_err(|e| EngineError::Serialization(e.to_string()))?;
        Self::from_snapshot(snapshot)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deathmatch {
    pub started_at_turn: u32,
    pub half_turns_since_progress: u32,
    pub interval_half_turns: u32,
    pub progress_this_turn: bool,
}
