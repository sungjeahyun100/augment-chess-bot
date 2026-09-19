//! Native piece movement, independent of card acquisition and UI.
use crate::{movement::*, *};

pub(crate) fn knightmaster_aura(s: &CanonicalState, p: &Piece) -> Option<bool> {
    let neighbors: Vec<_> = s
        .pieces
        .iter()
        .filter(|q| {
            q.owner == p.owner
                && q.kind == PieceKind::Knightmaster
                && q.anchor != p.anchor
                && q.anchor
                    .row
                    .abs_diff(p.anchor.row)
                    .max(q.anchor.col.abs_diff(p.anchor.col))
                    == 1
        })
        .collect();
    (!neighbors.is_empty()).then(|| {
        neighbors.iter().any(|q| {
            !q.statuses.iter().any(|status| match status {
                Status::CannotCaptureUntilOwnerTurn {
                    owner,
                    completed_turn,
                } => s.turn.completed.get(*owner) < completed_turn,
            })
        })
    })
}
fn jumps(s: &CanonicalState, p: &Piece, deltas: impl Iterator<Item = (i16, i16)>) -> Vec<Square> {
    deltas
        .filter_map(|(dr, dc)| offset(p.anchor, dr, dc))
        .filter(|sq| at(s, *sq).is_none_or(|q| can_capture(s, p, q)))
        .collect()
}
fn ray(s: &CanonicalState, p: &Piece, start: Square, dr: i16, dc: i16) -> Vec<Square> {
    let mut out = vec![];
    let mut next = offset(start, dr, dc);
    while let Some(sq) = next {
        if let Some(q) = at(s, sq) {
            if can_capture(s, p, q) {
                out.push(sq);
            }
            break;
        }
        out.push(sq);
        next = offset(sq, dr, dc);
    }
    out
}
pub(crate) fn destinations(s: &CanonicalState, p: &Piece) -> Vec<Square> {
    let queen = || DIAGONAL.into_iter().chain(STRAIGHT);
    match p.kind {
        PieceKind::Missionary => {
            let capture_locked = p.statuses.iter().any(|status| match status {
                Status::CannotCaptureUntilOwnerTurn {
                    owner,
                    completed_turn,
                } => s.turn.completed.get(*owner) < completed_turn,
            });
            DIAGONAL
                .into_iter()
                .filter_map(|(r, c)| offset(p.anchor, r, c))
                .filter(|to| match at(s, *to) {
                    None => true,
                    Some(q) => {
                        !capture_locked
                            && q.owner != p.owner
                            && q.owner != Owner::Neutral
                            && q.kind != PieceKind::Wall
                    }
                })
                .collect()
        }
        PieceKind::Jester => queen()
            .flat_map(|(r, c)| ray(s, p, p.anchor, r, c))
            .collect(),
        PieceKind::Bat => STRAIGHT
            .into_iter()
            .flat_map(|(r, c)| {
                ray(s, p, p.anchor, r, c)
                    .into_iter()
                    .filter(|to| to.row.abs_diff(p.anchor.row) + to.col.abs_diff(p.anchor.col) <= 2)
            })
            .collect(),
        PieceKind::Bear => {
            let color = match p.owner {
                Owner::White => Color::White,
                Owner::Black => Color::Black,
                Owner::Neutral => return vec![],
            };
            if p.bear_move_locked_until_turn
                .is_some_and(|deadline| *s.turn.completed.get(color) < deadline)
            {
                vec![]
            } else {
                queen()
                    .flat_map(|(r, c)| ray(s, p, p.anchor, r, c))
                    .collect()
            }
        }
        PieceKind::Hedgehog => {
            let color = match p.owner {
                Owner::White => Color::White,
                Owner::Black => Color::Black,
                Owner::Neutral => return vec![],
            };
            if p.bear_move_locked_until_turn
                .is_some_and(|deadline| *s.turn.completed.get(color) < deadline)
            {
                vec![]
            } else {
                jumps(s, p, queen())
            }
        }
        PieceKind::Campfire => STRAIGHT
            .into_iter()
            .filter_map(|(dr, dc)| offset(p.anchor, dr, dc))
            .filter(|to| at(s, *to).is_none())
            .collect(),
        PieceKind::Lobster => {
            let dr = match p.owner {
                Owner::White => -1,
                Owner::Black => 1,
                Owner::Neutral => return vec![],
            };
            jumps(s, p, [-1, 0, 1].into_iter().map(|dc| (dr, dc)))
        }
        PieceKind::Slime => jumps(s, p, STRAIGHT.into_iter().map(|(dr, dc)| (dr * 3, dc * 3))),
        PieceKind::Paladin => jumps(s, p, KNIGHT.into_iter()),
        PieceKind::RoyalKnight => jumps(s, p, KNIGHT.into_iter()),
        PieceKind::PrimeMinister => {
            // Two king steps, with an empty intermediate square. Different
            // paths to the same destination are the same cardless action.
            let mut out = jumps(s, p, queen());
            for mid in queen()
                .filter_map(|(r, c)| offset(p.anchor, r, c))
                .filter(|sq| at(s, *sq).is_none())
            {
                out.extend(queen().filter_map(|(r, c)| offset(mid, r, c)).filter(|to| {
                    *to != p.anchor && at(s, *to).is_none_or(|q| can_capture(s, p, q))
                }));
            }
            out.sort_unstable();
            out.dedup();
            out
        }
        PieceKind::Pegasus => (0..8)
            .flat_map(|r| (0..8).map(move |c| Square::new(r, c)))
            .filter(|to| match at(s, *to) {
                None => true,
                Some(q) => {
                    KNIGHT.contains(&(
                        to.row as i16 - p.anchor.row as i16,
                        to.col as i16 - p.anchor.col as i16,
                    )) && can_capture(s, p, q)
                }
            })
            .collect(),
        PieceKind::Fanatic => {
            let dir = if p.owner == Owner::White { -1 } else { 1 };
            ray(s, p, p.anchor, dir, 0)
                .into_iter()
                .filter(|to| to.row.abs_diff(p.anchor.row) <= 2)
                .collect()
        }
        PieceKind::Wizard => queen()
            .filter_map(|(r, c)| offset(p.anchor, r, c))
            .filter(|sq| at(s, *sq).is_none())
            .collect(),
        PieceKind::Herald => {
            let locked = p
                .herald_jump_lock_turn
                .map_or(p.herald_jump_locked, |turn| {
                    p.owner == Owner::from(s.turn.side)
                        && *s.turn.completed.get(s.turn.side) == turn
                });
            let mut out = vec![];
            for (dr, dc) in STRAIGHT {
                for distance in 1..=3 {
                    let Some(to) = offset(p.anchor, dr * distance, dc * distance) else {
                        break;
                    };
                    if at(s, to).is_none() {
                        out.push(to);
                    } else if locked {
                        break;
                    }
                }
            }
            out
        }
        PieceKind::Princess => {
            if s.pieces
                .iter()
                .any(|q| q.owner == p.owner && q.kind == PieceKind::Queen)
            {
                jumps(s, p, DIAGONAL.into_iter())
            } else {
                queen()
                    .flat_map(|(r, c)| ray(s, p, p.anchor, r, c))
                    .collect()
            }
        }
        PieceKind::Clockwork => {
            if queen()
                .filter_map(|(r, c)| offset(p.anchor, r, c))
                .any(|sq| at(s, sq).is_some_and(|q| q.owner == p.owner && q.id != p.id))
            {
                queen()
                    .flat_map(|(r, c)| ray(s, p, p.anchor, r, c))
                    .collect()
            } else {
                vec![]
            }
        }
        PieceKind::Berserker => {
            let count = s.pieces.iter().filter(|q| q.owner == p.owner).count();
            if count <= 5 {
                queen()
                    .flat_map(|(r, c)| ray(s, p, p.anchor, r, c))
                    .chain(jumps(s, p, KNIGHT.into_iter()))
                    .collect()
            } else if count <= 9 {
                STRAIGHT
                    .into_iter()
                    .flat_map(|(r, c)| ray(s, p, p.anchor, r, c))
                    .chain(jumps(s, p, queen()))
                    .collect()
            } else {
                jumps(s, p, queen())
            }
        }
        PieceKind::Checker | PieceKind::CheckerKing => {
            let captures = checker_captures(s, p);
            if !captures.is_empty()
                || matches!(
                    s.turn.continuation,
                    Some(Continuation::CheckerCapture { .. })
                )
            {
                return captures;
            }
            checker_directions(p)
                .into_iter()
                .filter_map(|(r, c)| offset(p.anchor, r, c))
                .filter(|sq| at(s, *sq).is_none())
                .collect()
        }
        PieceKind::Man | PieceKind::Vip | PieceKind::Guard | PieceKind::Recruiter => {
            jumps(s, p, queen())
        }
        PieceKind::Ferz | PieceKind::Knightmaster => jumps(s, p, DIAGONAL.into_iter()),
        PieceKind::Alfil => jumps(s, p, DIAGONAL.into_iter().map(|(r, c)| (r * 2, c * 2))),
        PieceKind::Eagle => jumps(s, p, queen().map(|(r, c)| (r * 2, c * 2))),
        PieceKind::Camel => jumps(
            s,
            p,
            KNIGHT.into_iter().map(|(r, c)| {
                (
                    if r.abs() == 2 { r.signum() * 3 } else { r },
                    if c.abs() == 2 { c.signum() * 3 } else { c },
                )
            }),
        ),
        PieceKind::Amazon => queen()
            .flat_map(|(r, c)| ray(s, p, p.anchor, r, c))
            .chain(jumps(s, p, KNIGHT.into_iter()))
            .collect(),
        PieceKind::Windmill => (if p.windmill_mode == Some(WindmillMode::Rook) {
            STRAIGHT
        } else {
            DIAGONAL
        })
        .into_iter()
        .flat_map(|(r, c)| ray(s, p, p.anchor, r, c))
        .collect(),
        PieceKind::Assassin => {
            let mut out = jumps(s, p, KNIGHT.into_iter());
            for (dr, dc) in queen() {
                out.extend(
                    ray(s, p, p.anchor, dr, dc)
                        .into_iter()
                        .filter(|sq| at(s, *sq).is_some_and(|q| q.kind.royal())),
                );
            }
            out
        }
        PieceKind::Protestant => jumps(
            s,
            p,
            DIAGONAL
                .into_iter()
                .flat_map(|(r, c)| (1..=3).map(move |d| (r * d, c * d))),
        ),
        PieceKind::Cannon | PieceKind::Grasshopper => {
            let mut out = vec![];
            for (dr, dc) in
                queen().filter(|(r, c)| p.kind == PieceKind::Grasshopper || *r == 0 || *c == 0)
            {
                let mut next = offset(p.anchor, dr, dc);
                while let Some(sq) = next {
                    if let Some(q) = at(s, sq) {
                        if p.kind == PieceKind::Grasshopper {
                            if let Some(landing) = offset(sq, dr, dc) {
                                if at(s, landing).is_none_or(|target| can_capture(s, p, target)) {
                                    out.push(landing);
                                }
                            }
                        } else if q.kind != PieceKind::Cannon {
                            out.extend(ray(s, p, sq, dr, dc).into_iter().filter(|sq| {
                                at(s, *sq).is_none_or(|target| target.kind != PieceKind::Cannon)
                            }));
                        }
                        break;
                    }
                    next = offset(sq, dr, dc);
                }
            }
            out
        }
        PieceKind::Hook => {
            let mut out = vec![];
            for (dr, dc) in STRAIGHT {
                for sq in ray(s, p, p.anchor, dr, dc) {
                    out.push(sq);
                    if at(s, sq).is_none() {
                        for (tr, tc) in STRAIGHT {
                            if tr * dr + tc * dc == 0 {
                                out.extend(ray(s, p, sq, tr, tc));
                            }
                        }
                    }
                }
            }
            out
        }
        PieceKind::Cardinal => {
            let mut out = vec![];
            for (mut dr, mut dc) in DIAGONAL {
                let mut current = p.anchor;
                let mut seen = std::collections::BTreeSet::new();
                loop {
                    if !(0..8).contains(&(current.row as i16 + dr)) {
                        dr = -dr;
                    }
                    if !(0..8).contains(&(current.col as i16 + dc)) {
                        dc = -dc;
                    }
                    let Some(sq) = offset(current, dr, dc) else {
                        break;
                    };
                    if !seen.insert((sq, dr, dc)) {
                        break;
                    }
                    if let Some(q) = at(s, sq) {
                        if q.kind == PieceKind::Wall {
                            dr = -dr;
                            dc = -dc;
                            continue;
                        }
                        if can_capture(s, p, q) {
                            out.push(sq);
                        }
                        break;
                    }
                    out.push(sq);
                    current = sq;
                }
            }
            out
        }
        _ => vec![],
    }
}
pub(crate) fn attacks(s: &CanonicalState, p: &Piece, to: Square) -> bool {
    let dr = to.row as i16 - p.anchor.row as i16;
    let dc = to.col as i16 - p.anchor.col as i16;
    match p.kind {
        PieceKind::Pegasus | PieceKind::RoyalKnight => KNIGHT.contains(&(dr, dc)),
        PieceKind::Bat => {
            (dr == 0 || dc == 0) && dr.abs() + dc.abs() <= 2 && ray_clear(s, p.anchor, to)
        }
        PieceKind::Bear => {
            (dr == 0 || dc == 0 || dr.abs() == dc.abs()) && ray_clear(s, p.anchor, to)
        }
        PieceKind::Hedgehog => {
            let color = match p.owner {
                Owner::White => Color::White,
                Owner::Black => Color::Black,
                Owner::Neutral => return false,
            };
            p.bear_move_locked_until_turn
                .is_none_or(|deadline| *s.turn.completed.get(color) >= deadline)
                && dr.abs().max(dc.abs()) == 1
        }
        PieceKind::Lobster => dr == if p.owner == Owner::White { -1 } else { 1 } && dc.abs() <= 1,
        PieceKind::Slime => (dr == 0 || dc == 0) && dr.abs() + dc.abs() == 3,
        PieceKind::ShotgunKing => crate::shotgun::attacks(s, p, to),
        PieceKind::Herald | PieceKind::Wizard | PieceKind::Campfire | PieceKind::Paladin => false,
        // The source's advisory threat switch omits clockwork entirely.
        PieceKind::Clockwork => false,
        PieceKind::Princess => {
            if s.pieces
                .iter()
                .any(|q| q.owner == p.owner && q.kind == PieceKind::Queen)
            {
                dr.abs() == 1 && dc.abs() == 1
            } else {
                (dr == 0 || dc == 0 || dr.abs() == dc.abs()) && ray_clear(s, p.anchor, to)
            }
        }
        PieceKind::Berserker => {
            let count = s.pieces.iter().filter(|q| q.owner == p.owner).count();
            if count <= 5 {
                KNIGHT.contains(&(dr, dc))
                    || (dr == 0 || dc == 0 || dr.abs() == dc.abs()) && ray_clear(s, p.anchor, to)
            } else {
                dr.abs().max(dc.abs()) == 1
                    || count <= 9 && (dr == 0 || dc == 0) && ray_clear(s, p.anchor, to)
            }
        }
        PieceKind::Colossus | PieceKind::BigRook | PieceKind::BigBishop => {
            crate::large::attacks(s, p, to)
        }
        PieceKind::Checker | PieceKind::CheckerKing => {
            checker_directions(p).contains(&(dr, dc))
                && offset(to, dr, dc).is_some_and(|sq| at(s, sq).is_none())
                && at(s, to).is_none_or(|q| can_capture(s, p, q))
        }
        PieceKind::Man | PieceKind::Vip => dr.abs().max(dc.abs()) == 1,
        PieceKind::Guard | PieceKind::Recruiter => false,
        PieceKind::Ferz | PieceKind::Knightmaster => dr.abs() == 1 && dc.abs() == 1,
        PieceKind::Alfil => dr.abs() == 2 && dc.abs() == 2,
        PieceKind::Camel => (dr.abs() == 3 && dc.abs() == 1) || (dr.abs() == 1 && dc.abs() == 3),
        PieceKind::Eagle => {
            [0, 2].contains(&dr.abs()) && [0, 2].contains(&dc.abs()) && dr.abs().max(dc.abs()) == 2
        }
        PieceKind::Amazon => {
            KNIGHT.contains(&(dr, dc))
                || (dr == 0 || dc == 0 || dr.abs() == dc.abs()) && ray_clear(s, p.anchor, to)
        }
        PieceKind::Windmill => {
            (if p.windmill_mode == Some(WindmillMode::Rook) {
                dr == 0 || dc == 0
            } else {
                dr.abs() == dc.abs()
            }) && ray_clear(s, p.anchor, to)
        }
        PieceKind::Assassin => {
            KNIGHT.contains(&(dr, dc))
                || at(s, to).is_some_and(|q| q.kind.royal())
                    && (dr == 0 || dc == 0 || dr.abs() == dc.abs())
                    && ray_clear(s, p.anchor, to)
        }
        _ => destinations(s, p).contains(&to),
    }
}

fn checker_directions(p: &Piece) -> Vec<(i16, i16)> {
    if p.kind == PieceKind::CheckerKing {
        DIAGONAL.to_vec()
    } else {
        let dir = if p.owner == Owner::White { -1 } else { 1 };
        vec![(dir, -1), (dir, 1)]
    }
}
pub(crate) fn checker_captures(s: &CanonicalState, p: &Piece) -> Vec<Square> {
    if !matches!(p.kind, PieceKind::Checker | PieceKind::CheckerKing) {
        return vec![];
    }
    checker_directions(p)
        .into_iter()
        .filter_map(|(r, c)| {
            let mid = offset(p.anchor, r, c)?;
            let end = offset(mid, r, c)?;
            (at(s, end).is_none() && at(s, mid).is_some_and(|q| can_capture(s, p, q)))
                .then_some(end)
        })
        .collect()
}

pub(crate) fn standard_bearer_aura(s: &CanonicalState, p: &Piece) -> Option<bool> {
    if !matches!(p.kind, PieceKind::Pawn | PieceKind::StandardBearer) {
        return None;
    }
    let neighbors: Vec<_> = s
        .pieces
        .iter()
        .filter(|q| {
            q.kind == PieceKind::StandardBearer
                && q.owner == p.owner
                && q.anchor.row == p.anchor.row
        })
        .collect();
    (!neighbors.is_empty()).then(|| {
        neighbors.iter().any(|q| {
            !q.statuses.iter().any(|status| match status {
                Status::CannotCaptureUntilOwnerTurn {
                    owner,
                    completed_turn,
                } => s.turn.completed.get(*owner) < completed_turn,
            })
        })
    })
}
