use crate::*;
use crate::{movement::at, victory::*};

fn relocate(p: &mut Piece, to: Square) {
    p.anchor = to;
    p.footprint = vec![to];
}
fn rebuild(s: &mut CanonicalState) -> EngineResult<()> {
    s.board = Board::from_pieces(8, 8, &s.pieces)?;
    Ok(())
}
pub(crate) fn apply(s: &mut CanonicalState, action: Action) -> EngineResult<()> {
    match action {
        Action::Move { from, to, .. } => {
            let moving = at(s, from).expect("validated move").clone();
            let mut captures = vec![];
            if let Some(p) = at(s, to) {
                captures.push(p.id);
            }
            if moving.kind == PieceKind::Pawn {
                if let Some(ep) = &s.history.en_passant {
                    if ep.available_to == s.turn.side && ep.target == to && from.col != to.col {
                        captures.push(ep.pawn);
                    }
                }
            }
            let royal_capture = s
                .pieces
                .iter()
                .any(|p| captures.contains(&p.id) && p.kind == PieceKind::King);
            if !captures.is_empty() {
                mark_progress(s);
            }
            s.pieces.retain(|p| !captures.contains(&p.id));
            let p = s.pieces.iter_mut().find(|p| p.id == moving.id).unwrap();
            relocate(p, to);
            // Reference returns after landing on a captured king, before moved/EP/turn bookkeeping.
            if royal_capture {
                finish(
                    s,
                    GameResult::Win {
                        winner: s.turn.side,
                        reason: EndReason::RoyalCapture,
                    },
                );
                rebuild(s)?;
                // A captured EP pawn cannot remain a dangling entity reference.
                if s.history
                    .en_passant
                    .as_ref()
                    .is_some_and(|ep| captures.contains(&ep.pawn))
                {
                    s.history.en_passant = None;
                }
                return Ok(());
            }
            p.moved = true;
            if moving.kind == PieceKind::King && from.col.abs_diff(to.col) == 2 {
                let (rook_from, rook_to) = if to.col > from.col { (7, 5) } else { (0, 3) };
                let rook = s
                    .pieces
                    .iter_mut()
                    .find(|p| p.anchor == Square::new(from.row, rook_from))
                    .unwrap();
                relocate(rook, Square::new(from.row, rook_to));
                rook.moved = true;
                *s.history.castled.get_mut(s.turn.side) = true;
            }
            s.history.en_passant =
                if moving.kind == PieceKind::Pawn && from.row.abs_diff(to.row) == 2 {
                    Some(EnPassant {
                        pawn: moving.id,
                        target: Square::new((from.row + to.row) / 2, to.col),
                        available_to: s.turn.side.opponent(),
                    })
                } else {
                    None
                };
            rebuild(s)?;
            if moving.kind == PieceKind::Pawn {
                mark_progress(s);
                if to.row == if s.turn.side == Color::White { 0 } else { 7 } {
                    s.pending_promotion = Some(moving.id);
                    return Ok(());
                }
            }
        }
        Action::Promote { piece, into } => {
            let p = s.pieces.iter_mut().find(|p| p.id == piece).unwrap();
            p.kind = into;
            p.origin = Some(p.anchor);
            p.moved = true;
            let deadline = s
                .turn
                .completed
                .get(s.turn.side)
                .checked_add(1)
                .ok_or_else(|| invalid("promotion counter overflow"))?;
            p.statuses.clear();
            p.statuses.push(Status::CannotCaptureUntilOwnerTurn {
                owner: s.turn.side,
                completed_turn: deadline,
            });
            s.pending_promotion = None;
        }
        _ => unreachable!("only generated actions are resolved"),
    }
    end_turn(s)
}
