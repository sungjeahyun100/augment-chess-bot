use crate::movement::{DIAGONAL, STRAIGHT, at, can_capture, offset};
use crate::*;

pub(crate) fn validate(s: &CanonicalState) -> EngineResult<()> {
    for p in &s.pieces {
        if (p.log_direction.is_some() || p.log_roll_after_turn.is_some())
            && p.kind != PieceKind::Log
        {
            return Err(invalid("log state on non-log"));
        }
        if p.log_direction.is_some_and(|d| {
            !(-1..=1).contains(&d.dr) || !(-1..=1).contains(&d.dc) || (d.dr == 0 && d.dc == 0)
        }) {
            return Err(invalid("invalid log direction"));
        }
    }
    Ok(())
}
pub(crate) fn actions(p: &Piece) -> Vec<Action> {
    DIAGONAL
        .into_iter()
        .chain(STRAIGHT)
        .filter(|(r, c)| offset(p.anchor, *r, *c).is_some())
        .map(|(r, c)| Action::SetLogDirection {
            piece: p.id,
            direction: LogDirection {
                dr: r as i8,
                dc: c as i8,
            },
        })
        .collect()
}
fn stop(s: &mut CanonicalState, id: PieceId) {
    if let Some(p) = s.pieces.iter_mut().find(|p| p.id == id) {
        p.log_direction = None;
        p.log_roll_after_turn = None;
    }
}
pub(crate) fn advance(s: &mut CanonicalState) -> EngineResult<()> {
    // Reference snapshots board traversal order, not allocation order.
    let mut logs: Vec<_> = s
        .pieces
        .iter()
        .filter(|p| {
            p.kind == PieceKind::Log && p.owner == s.turn.side.into() && p.log_direction.is_some()
        })
        .cloned()
        .collect();
    logs.sort_by_key(|p| p.anchor);
    let completed = *s.turn.completed.get(s.turn.side);
    for p in logs {
        if s.result.is_some() {
            break;
        }
        if !s
            .pieces
            .iter()
            .any(|q| q.id == p.id && q.anchor == p.anchor)
        {
            continue;
        }
        if p.log_roll_after_turn
            .is_some_and(|deadline| completed < deadline)
        {
            continue;
        }
        s.pieces
            .iter_mut()
            .find(|q| q.id == p.id)
            .unwrap()
            .log_roll_after_turn = Some(
            completed
                .checked_add(1)
                .ok_or_else(|| invalid("log deadline overflow"))?,
        );
        let dir = p.log_direction.unwrap();
        let Some(to) = offset(p.anchor, dir.dr.into(), dir.dc.into()) else {
            stop(s, p.id);
            continue;
        };
        if let Some(target) = at(s, to).cloned() {
            if !can_capture(s, &p, &target) || target.shielded {
                stop(s, p.id);
                continue;
            }
            if target.hp.is_some() {
                crate::large::damage(s, target.id, s.turn.side)?;
                stop(s, p.id);
                continue;
            }
            crate::bear::arm(s, &target, target.anchor, &p, s.turn.side);
            crate::wizard::remove(s, target.id);
            victory::mark_progress(s);
            if target.kind.defeat_royal() {
                victory::finish(
                    s,
                    GameResult::Win {
                        winner: s.turn.side,
                        reason: EndReason::RoyalCapture,
                    },
                );
            }
        }
        let log = s.pieces.iter_mut().find(|q| q.id == p.id).unwrap();
        log.anchor = to;
        log.footprint = vec![to];
        log.moved = true;
        crate::transition::rebuild(s)?;
    }
    if s.history
        .en_passant
        .as_ref()
        .is_some_and(|ep| !s.pieces.iter().any(|p| p.id == ep.pawn))
    {
        s.history.en_passant = None;
    }
    Ok(())
}
