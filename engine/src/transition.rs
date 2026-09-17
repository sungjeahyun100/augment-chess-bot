use crate::*;
use crate::{movement::at, victory::*};

fn relocate(p: &mut Piece, to: Square) {
    p.anchor = to;
    p.footprint = vec![to];
}
pub(crate) fn rebuild(s: &mut CanonicalState) -> EngineResult<()> {
    s.board = Board::from_pieces(8, 8, &s.pieces)?;
    Ok(())
}
pub(crate) fn apply(s: &mut CanonicalState, action: Action) -> EngineResult<()> {
    if crate::castling::apply(s, &action)? {
        return Ok(());
    }
    if crate::shotgun::apply(s, &action)? {
        return Ok(());
    }
    if let Action::SetLogDirection { piece, direction } = action {
        let after = s
            .turn
            .completed
            .get(s.turn.side)
            .checked_add(1)
            .ok_or_else(|| invalid("log deadline overflow"))?;
        let p = s.pieces.iter_mut().find(|p| p.id == piece).unwrap();
        p.log_direction = Some(direction);
        p.log_roll_after_turn = Some(after);
        p.moved = true;
        s.history.en_passant = None;
        return end_turn(s);
    }
    if let Action::CastSpell {
        wizard,
        spell,
        target,
    } = action
    {
        return crate::wizard::cast(s, wizard, spell, target);
    }
    if let Action::Purchase { merchant, target } = action {
        return crate::merchant::purchase(s, merchant, target);
    }
    if crate::large::apply(s, &action)? {
        return Ok(());
    }
    match action {
        Action::Move { from, to, .. } => {
            let moving = at(s, from).expect("validated move").clone();
            let ep_victim = s
                .history
                .en_passant
                .as_ref()
                .filter(|ep| {
                    moving.kind.pawn_mover()
                        && (moving.kind == PieceKind::Pawn || at(s, to).is_none())
                        && ep.available_to == s.turn.side
                        && s.pieces
                            .iter()
                            .find(|p| p.id == ep.pawn)
                            .is_some_and(|p| crate::movement::can_capture(s, &moving, p))
                        && ep.target == to
                        && from.col.abs_diff(to.col) == 1
                        && to.row as i16 - from.row as i16
                            == if moving.owner == Owner::White { -1 } else { 1 }
                })
                .map(|ep| ep.pawn);
            let protected = at(s, to).filter(|p| p.shielded).map(|p| p.id).or_else(|| {
                ep_victim.filter(|id| s.pieces.iter().any(|p| p.id == *id && p.shielded))
            });
            if let Some(id) = protected {
                s.pieces.iter_mut().find(|p| p.id == id).unwrap().shielded = false;
                s.turn.continuation = None;
                s.history.en_passant = None;
                return end_turn(s);
            }
            if let Some(target) = at(s, to).filter(|p| p.hp.is_some()) {
                let id = target.id;
                crate::large::damage(s, id, s.turn.side)?;
                s.history.en_passant = None;
                return end_turn(s);
            }
            let checker_jump = matches!(moving.kind, PieceKind::Checker | PieceKind::CheckerKing)
                && from.row.abs_diff(to.row) == 2;
            let mut captures = vec![];
            if checker_jump {
                let mid = Square::new((from.row + to.row) / 2, (from.col + to.col) / 2);
                let target = at(s, mid).expect("validated checker jump");
                if target.shielded {
                    let id = target.id;
                    s.pieces.iter_mut().find(|p| p.id == id).unwrap().shielded = false;
                } else if target.hp.is_some() {
                    let id = target.id;
                    crate::large::damage(s, id, s.turn.side)?;
                } else {
                    captures.push(target.id);
                }
            }
            s.turn.continuation = None;
            if let Some(p) = at(s, to) {
                captures.push(p.id);
            }
            captures.extend(ep_victim);
            let royal_capture = s
                .pieces
                .iter()
                .any(|p| captures.contains(&p.id) && p.kind.defeat_royal());
            if !captures.is_empty() {
                mark_progress(s);
            }
            for id in &captures {
                crate::wizard::remove(s, *id);
            }
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
            if moving.kind == PieceKind::ShotgunKing {
                p.facing = Some(crate::shotgun::facing(
                    to.row as i16 - from.row as i16,
                    to.col as i16 - from.col as i16,
                ));
            }
            if moving.kind == PieceKind::Herald {
                p.herald_jump_locked = false;
            }
            if moving.kind == PieceKind::Checker
                && to.row == if moving.owner == Owner::White { 0 } else { 7 }
            {
                crate::effects::transform(
                    p,
                    PieceKind::CheckerKing,
                    *s.turn.completed.get(s.turn.side),
                )?;
            }
            if moving.kind == PieceKind::Squire
                && !captures.is_empty()
                && to.row != if moving.owner == Owner::White { 0 } else { 7 }
            {
                crate::effects::transform(
                    p,
                    PieceKind::Knight,
                    *s.turn.completed.get(s.turn.side),
                )?;
            }
            if moving.kind == PieceKind::Windmill {
                p.windmill_mode = Some(if moving.windmill_mode == Some(WindmillMode::Rook) {
                    WindmillMode::Bishop
                } else {
                    WindmillMode::Rook
                });
            }
            s.history.en_passant = if moving.kind == PieceKind::Pawn
                && from.col == to.col
                && from.row.abs_diff(to.row) == 2
            {
                Some(EnPassant {
                    pawn: moving.id,
                    target: Square::new((from.row + to.row) / 2, to.col),
                    available_to: s.turn.side.opponent(),
                })
            } else if moving.kind == PieceKind::King && from.col.abs_diff(to.col) == 2 {
                None
            } else {
                s.history
                    .en_passant
                    .clone()
                    .filter(|ep| ep.available_to != s.turn.side)
            };
            if moving.kind == PieceKind::Recruiter {
                crate::effects::recruiter_pawn(s, moving.owner, from)?;
            }
            rebuild(s)?;
            if moving.kind == PieceKind::Pawn {
                mark_progress(s);
            }
            if moving.kind.pawn_mover() && to.row == if s.turn.side == Color::White { 0 } else { 7 }
            {
                s.pending_promotion = Some(moving.id);
                return Ok(());
            }
            if herald_agreement(s) {
                return Ok(());
            }
            if checker_jump {
                let p = s.pieces.iter().find(|p| p.id == moving.id).unwrap();
                if !crate::variants::checker_captures(s, p).is_empty() {
                    s.turn.continuation = Some(Continuation::CheckerCapture { piece: moving.id });
                    increment(&mut s.turn.move_count)?;
                    return Ok(());
                }
            }
        }
        Action::Promote { piece, into } => {
            let p = s.pieces.iter_mut().find(|p| p.id == piece).unwrap();
            crate::effects::transform(p, into, *s.turn.completed.get(s.turn.side))?;
            p.moved = true;
            s.pending_promotion = None;
        }
        _ => unreachable!("only generated actions are resolved"),
    }
    end_turn(s)
}
