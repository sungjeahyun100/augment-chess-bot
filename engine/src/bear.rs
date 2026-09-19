use crate::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingBearRetaliation {
    pub color: Color,
    pub square: Square,
    pub attacker: PieceId,
    pub attacker_color: Color,
    pub captured_by: Color,
    pub counter_destination: Option<Square>,
    pub bear: Piece,
    pub remaining: u8,
}

fn piece_color(piece: &Piece) -> Option<Color> {
    match piece.owner {
        Owner::White => Some(Color::White),
        Owner::Black => Some(Color::Black),
        Owner::Neutral => None,
    }
}

pub(crate) fn validate(s: &CanonicalState) -> EngineResult<()> {
    for pending in &s.pending_bear_retaliations {
        if pending.square.row >= 8
            || pending.square.col >= 8
            || pending
                .counter_destination
                .is_some_and(|sq| sq.row >= 8 || sq.col >= 8)
            || pending.attacker.0 == 0
            || pending.attacker.0 >= s.ids.next_piece
            || pending.bear.id.0 == 0
            || pending.bear.id.0 >= s.ids.next_piece
            || !matches!(pending.bear.kind, PieceKind::Bear | PieceKind::Hedgehog)
            || piece_color(&pending.bear) != Some(pending.color)
            || pending.attacker_color == pending.color
            || pending.captured_by != pending.attacker_color
            || pending.remaining > 1
            || s.pieces.iter().any(|p| p.id == pending.bear.id)
        {
            return Err(invalid("invalid pending bear retaliation"));
        }
    }
    Ok(())
}

pub(crate) fn arm(
    s: &mut CanonicalState,
    captured: &Piece,
    square: Square,
    attacker: &Piece,
    captured_by: Color,
) -> bool {
    let Some(color) = piece_color(captured) else {
        return false;
    };
    let Some(attacker_color) = piece_color(attacker) else {
        return false;
    };
    let remaining = captured.bear_retaliations_remaining.unwrap_or(0);
    if !matches!(captured.kind, PieceKind::Bear | PieceKind::Hedgehog)
        || remaining == 0
        || color == attacker_color
    {
        return false;
    }
    let counter_destination = s
        .pieces
        .iter()
        .find(|p| p.id == attacker.id)
        .map(|p| p.anchor);
    s.pending_bear_retaliations.push(PendingBearRetaliation {
        color,
        square,
        attacker: attacker.id,
        attacker_color,
        captured_by,
        counter_destination,
        bear: captured.clone(),
        remaining: remaining - 1,
    });
    true
}

pub(crate) fn resolve(
    s: &mut CanonicalState,
    attacker_color: Color,
    attacker: Option<PieceId>,
) -> EngineResult<usize> {
    let mut due = vec![];
    let mut waiting = vec![];
    for pending in std::mem::take(&mut s.pending_bear_retaliations) {
        if pending.attacker_color == attacker_color
            && attacker.is_none_or(|id| id == pending.attacker)
        {
            due.push(pending);
        } else {
            waiting.push(pending);
        }
    }
    s.pending_bear_retaliations = waiting;
    let mut removed = BTreeSet::new();
    let mut restored = 0;
    for pending in due {
        let attacker_square = s
            .pieces
            .iter()
            .find(|p| p.id == pending.attacker)
            .filter(|p| piece_color(p) == Some(pending.attacker_color))
            .map(|p| (p.anchor, p.kind));
        let mut destination = pending.counter_destination.or(attacker_square.map(|v| v.0));
        if removed.insert(pending.attacker) {
            if let Some((_, kind)) = attacker_square {
                s.pieces.retain(|p| p.id != pending.attacker);
                if s.history
                    .en_passant
                    .as_ref()
                    .is_some_and(|ep| ep.pawn == pending.attacker)
                {
                    s.history.en_passant = None;
                }
                victory::mark_progress(s);
                if kind.defeat_royal() {
                    victory::finish(
                        s,
                        GameResult::Win {
                            winner: pending.color,
                            reason: EndReason::RoyalCapture,
                        },
                    );
                }
            }
        }
        if destination.is_none_or(|sq| movement::at(s, sq).is_some())
            && movement::at(s, pending.square).is_none()
        {
            destination = Some(pending.square);
        }
        let Some(destination) = destination.filter(|sq| movement::at(s, *sq).is_none()) else {
            continue;
        };
        let mut bear = pending.bear;
        if bear.kind != PieceKind::Hedgehog {
            bear.kind = PieceKind::Bear;
        }
        bear.anchor = destination;
        bear.footprint = vec![destination];
        bear.moved = true;
        bear.bear_retaliations_remaining = Some(pending.remaining);
        bear.bear_move_locked_until_turn = Some(
            s.turn
                .completed
                .get(pending.color)
                .checked_add(1)
                .ok_or_else(|| invalid("bear lock deadline overflow"))?,
        );
        s.pieces.push(bear);
        restored += 1;
    }
    crate::transition::rebuild(s)?;
    Ok(restored)
}
