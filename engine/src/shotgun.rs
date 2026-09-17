use crate::movement::{DIAGONAL, STRAIGHT, at, can_capture, offset, ray_clear};
use crate::*;
pub(crate) fn validate(s: &CanonicalState) -> EngineResult<()> {
    for p in &s.pieces {
        if (p.ammo.is_some() || p.max_ammo.is_some() || p.facing.is_some())
            && p.kind != PieceKind::ShotgunKing
        {
            return Err(invalid("shotgun state on different piece"));
        }
        if p.ammo.unwrap_or(0) > p.max_ammo.unwrap_or(3) {
            return Err(invalid("ammo exceeds capacity"));
        }
    }
    Ok(())
}
pub(crate) fn facing(dr: i16, dc: i16) -> Facing {
    if dr.abs() >= dc.abs() {
        if dr < 0 { Facing::Up } else { Facing::Down }
    } else if dc < 0 {
        Facing::Left
    } else {
        Facing::Right
    }
}
fn cells(from: Square, dr: i16, dc: i16) -> Vec<Square> {
    let sides = if dr == 0 {
        vec![(-1, 0), (0, 0), (1, 0)]
    } else if dc == 0 {
        vec![(0, -1), (0, 0), (0, 1)]
    } else {
        vec![(0, 0), (-dr, 0), (0, -dc)]
    };
    let mut out = vec![];
    let straight = dr == 0 || dc == 0;
    for distance in 1..=if straight { 3 } else { 2 } {
        let offsets = if straight && distance == 3 {
            vec![(0, 0)]
        } else {
            sides.clone()
        };
        for (r, c) in offsets {
            if let Some(sq) = offset(from, dr * distance + r, dc * distance + c) {
                if !out.contains(&sq) {
                    out.push(sq);
                }
            }
        }
    }
    out
}
fn locked(s: &CanonicalState, p: &Piece) -> bool {
    p.statuses.iter().any(|status| match status {
        Status::CannotCaptureUntilOwnerTurn {
            owner,
            completed_turn,
        } => s.turn.completed.get(*owner) < completed_turn,
    })
}
pub(crate) fn reload_actions(s: &CanonicalState) -> Vec<Action> {
    s.pieces
        .iter()
        .filter(|p| {
            p.owner == s.turn.side.into()
                && p.kind == PieceKind::ShotgunKing
                && p.ammo.unwrap_or(0) < p.max_ammo.unwrap_or(3)
        })
        .map(|p| Action::Reload { piece: p.id })
        .collect()
}
pub(crate) fn actions(s: &CanonicalState, p: &Piece) -> Vec<Action> {
    let mut out = vec![];
    for (dr, dc) in DIAGONAL.into_iter().chain(STRAIGHT) {
        if let Some(to) = offset(p.anchor, dr, dc) {
            if at(s, to).is_none() {
                out.push(Action::Move {
                    from: p.anchor,
                    to,
                    route: vec![],
                });
            }
            if p.ammo.unwrap_or(0) >= 2 && !locked(s, p) {
                out.push(Action::ShotgunBlast {
                    piece: p.id,
                    direction: ShotgunDirection {
                        dr: dr as i8,
                        dc: dc as i8,
                    },
                });
            }
        }
        if p.ammo.unwrap_or(0) >= 3 && !locked(s, p) {
            let mut next = offset(p.anchor, dr, dc);
            while let Some(to) = next {
                if let Some(q) = at(s, to) {
                    if can_capture(s, p, q) {
                        out.push(Action::ShotgunSnipe {
                            piece: p.id,
                            target: to,
                        });
                    }
                    break;
                }
                next = offset(to, dr, dc);
            }
        }
    }
    out
}
pub(crate) fn attacks(s: &CanonicalState, p: &Piece, to: Square) -> bool {
    let ammo = p.ammo.unwrap_or(0);
    if ammo >= 2
        && DIAGONAL
            .into_iter()
            .chain(STRAIGHT)
            .any(|(r, c)| cells(p.anchor, r, c).contains(&to))
    {
        return true;
    }
    let dr = to.row as i16 - p.anchor.row as i16;
    let dc = to.col as i16 - p.anchor.col as i16;
    ammo >= 3 && (dr == 0 || dc == 0 || dr.abs() == dc.abs()) && ray_clear(s, p.anchor, to)
}
fn hit(s: &mut CanonicalState, id: PieceId, by: Color) -> EngineResult<()> {
    let Some(p) = s.pieces.iter().find(|p| p.id == id).cloned() else {
        return Ok(());
    };
    if p.shielded {
        s.pieces.iter_mut().find(|p| p.id == id).unwrap().shielded = false;
    } else if p.hp.is_some() {
        crate::large::damage(s, id, by)?;
    } else {
        crate::wizard::remove(s, id);
        victory::mark_progress(s);
        if p.kind.defeat_royal() {
            victory::finish(
                s,
                GameResult::Win {
                    winner: if p.owner == by.into() {
                        by.opponent()
                    } else {
                        by
                    },
                    reason: EndReason::RoyalCapture,
                },
            );
        }
        crate::transition::rebuild(s)?;
    }
    Ok(())
}
pub(crate) fn apply(s: &mut CanonicalState, a: &Action) -> EngineResult<bool> {
    let (id, cost) = match a {
        Action::Reload { piece } => (*piece, 0),
        Action::ShotgunBlast { piece, .. } => (*piece, 2),
        Action::ShotgunSnipe { piece, .. } => (*piece, 3),
        _ => return Ok(false),
    };
    let p = s.pieces.iter().find(|p| p.id == id).unwrap().clone();
    let by = s.turn.side;
    match *a {
        Action::ShotgunBlast { direction, .. } => {
            let mut seen = std::collections::BTreeSet::new();
            for sq in cells(p.anchor, direction.dr.into(), direction.dc.into()) {
                let Some(q) = at(s, sq).cloned() else {
                    continue;
                };
                if q.id == id
                    || matches!(q.kind, PieceKind::Wall | PieceKind::Guard)
                    || !seen.insert(q.id)
                {
                    continue;
                }
                hit(s, q.id, by)?;
            }
        }
        Action::ShotgunSnipe { target, .. } => {
            let id = at(s, target).unwrap().id;
            hit(s, id, by)?;
        }
        _ => {}
    }
    let shooter = s.pieces.iter_mut().find(|p| p.id == id).unwrap();
    shooter.moved = true;
    shooter.ammo = Some(if cost == 0 {
        p.ammo.unwrap_or(0) + 1
    } else {
        p.ammo.unwrap_or(0) - cost
    });
    if let Action::ShotgunBlast { direction, .. } = *a {
        shooter.facing = Some(facing(direction.dr.into(), direction.dc.into()));
    }
    s.history.en_passant = None;
    victory::end_turn(s)?;
    Ok(true)
}
