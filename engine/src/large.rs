//! Shared entity footprint, landing capture and HP behavior for large pieces.
use crate::{movement::*, victory::*, *};
use std::collections::BTreeSet;

pub(crate) fn cells(anchor: Square) -> Vec<Square> {
    [(0, 0), (0, 1), (1, 0), (1, 1)]
        .into_iter()
        .filter_map(|(r, c)| offset(anchor, r, c))
        .collect()
}
fn landing(s: &CanonicalState, p: &Piece, to: Square) -> Option<Vec<PieceId>> {
    let footprint = cells(to);
    if footprint.len() != 4 {
        return None;
    }
    let mut seen = BTreeSet::new();
    let mut victims = vec![];
    for sq in footprint {
        if let Some(q) = at(s, sq).filter(|q| q.id != p.id) {
            // Big rook/bishop explicitly permit trampling friendly entities.
            let mut attacker = p.clone();
            if p.kind != PieceKind::Colossus && p.owner == q.owner {
                attacker.owner = if p.owner == Owner::White {
                    Owner::Black
                } else {
                    Owner::White
                };
            }
            if !can_capture(s, &attacker, q) {
                return None;
            }
            if seen.insert(q.id) {
                victims.push(q.id);
            }
        }
    }
    let limit = match p.kind {
        PieceKind::BigRook => 2,
        PieceKind::BigBishop => 3,
        _ => usize::MAX,
    };
    (victims.len() <= limit).then_some(victims)
}
pub(crate) fn sectors(p: &Piece) -> [Vec<Square>; 2] {
    let r = (p.anchor.row as i16 + if p.owner == Owner::White { -3 } else { 3 }).clamp(0, 6) as u16;
    let rows = if p.owner == Owner::White {
        [r + 1, r]
    } else {
        [r, r + 1]
    };
    [3, -3].map(|delta| {
        let c = (p.anchor.col as i16 + delta).clamp(0, 6) as u16;
        rows.into_iter()
            .flat_map(|row| [Square::new(row, c), Square::new(row, c + 1)])
            .collect()
    })
}
fn sector_target(s: &CanonicalState, p: &Piece, q: &Piece) -> bool {
    can_capture(s, p, q) && !q.shielded && q.owner != Owner::Neutral
}
pub(crate) fn actions(s: &CanonicalState, p: &Piece) -> Vec<Action> {
    let mut out = vec![];
    let dirs = if p.kind == PieceKind::BigBishop {
        DIAGONAL
    } else {
        STRAIGHT
    };
    for (dr, dc) in dirs {
        let mut next = offset(p.anchor, dr, dc);
        while let Some(to) = next {
            let Some(captures) = landing(s, p, to) else {
                break;
            };
            out.push(Action::Move {
                from: p.anchor,
                to,
                route: vec![],
            });
            if p.kind == PieceKind::Colossus || !captures.is_empty() {
                break;
            }
            next = offset(to, dr, dc);
        }
    }
    if p.kind == PieceKind::Colossus {
        for (index, sector) in sectors(p).into_iter().enumerate() {
            if sector
                .iter()
                .any(|sq| at(s, *sq).is_some_and(|q| sector_target(s, p, q)))
            {
                out.push(Action::AttackSector {
                    piece: p.id,
                    sector: index as u8,
                });
            }
        }
    }
    out
}
pub(crate) fn attacks(s: &CanonicalState, p: &Piece, to: Square) -> bool {
    if p.kind == PieceKind::Colossus {
        return sectors(p).iter().any(|sector| sector.contains(&to));
    }
    // JS visits every footprint cell and does not normalize big-rook threat origins.
    p.footprint.iter().any(|anchor| {
        let mut probe = p.clone();
        probe.anchor = *anchor;
        actions(s, &probe)
            .iter()
            .any(|a| matches!(a,Action::Move {to:anchor,..} if cells(*anchor).contains(&to)))
    })
}
pub(crate) fn damage(s: &mut CanonicalState, id: PieceId, by: Color) -> EngineResult<()> {
    let Some(p) = s.pieces.iter_mut().find(|p| p.id == id) else {
        return Ok(());
    };
    let hp = p.hp.expect("HP target");
    if hp > 1 {
        p.hp = Some(hp - 1);
    } else {
        let owner = p.owner;
        let royal = p.kind.defeat_royal();
        s.pieces.retain(|p| p.id != id);
        crate::wizard::grant_mana(s, owner);
        if royal {
            finish(
                s,
                GameResult::Win {
                    winner: if owner == by.into() {
                        by.opponent()
                    } else {
                        by
                    },
                    reason: EndReason::RoyalCapture,
                },
            );
        }
        mark_progress(s);
    }
    crate::transition::rebuild(s)
}
pub(crate) fn apply(s: &mut CanonicalState, action: &Action) -> EngineResult<bool> {
    match *action {
        Action::Move { from, to, .. } if at(s, from).is_some_and(|p| p.kind.large()) => {
            let moving = at(s, from).unwrap().clone();
            let victims = landing(s, &moving, to).expect("validated large landing");
            // JS landing forEach continues after terminal capture; last royal wins.
            let winner = victims
                .iter()
                .filter_map(|id| {
                    s.pieces
                        .iter()
                        .find(|p| p.id == *id && p.kind.defeat_royal())
                })
                .map(|p| {
                    if p.owner == moving.owner {
                        s.turn.side.opponent()
                    } else {
                        s.turn.side
                    }
                })
                .next_back();
            if !victims.is_empty() {
                mark_progress(s);
            }
            for id in &victims {
                crate::wizard::remove(s, *id);
            }
            let p = s.pieces.iter_mut().find(|p| p.id == moving.id).unwrap();
            p.anchor = to;
            p.footprint = cells(to);
            p.moved = true;
            s.history.en_passant = None;
            crate::transition::rebuild(s)?;
            if let Some(winner) = winner {
                finish(
                    s,
                    GameResult::Win {
                        winner,
                        reason: EndReason::RoyalCapture,
                    },
                );
            } else {
                end_turn(s)?;
            }
            Ok(true)
        }
        Action::AttackSector { piece, sector } => {
            let moving = s.pieces.iter().find(|p| p.id == piece).unwrap().clone();
            for sq in &sectors(&moving)[usize::from(sector)] {
                let Some(target) = at(s, *sq).filter(|q| sector_target(s, &moving, q)).cloned()
                else {
                    continue;
                };
                if target.hp.is_some() {
                    damage(s, target.id, s.turn.side)?;
                } else {
                    let armed = crate::bear::arm(s, &target, target.anchor, &moving, s.turn.side);
                    crate::wizard::remove(s, target.id);
                    mark_progress(s);
                    crate::transition::rebuild(s)?;
                    if target.kind.defeat_royal() {
                        finish(
                            s,
                            GameResult::Win {
                                winner: s.turn.side,
                                reason: EndReason::RoyalCapture,
                            },
                        );
                        break;
                    }
                    if armed {
                        crate::bear::resolve(s, s.turn.side, Some(moving.id))?;
                        if !s.pieces.iter().any(|p| p.id == moving.id) {
                            break;
                        }
                    }
                }
                if s.result.is_some() {
                    break;
                }
            }
            s.history.en_passant = None;
            if s.result.is_none() {
                end_turn(s)?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
