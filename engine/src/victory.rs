use crate::*;

pub(crate) fn increment(value: &mut u32) -> EngineResult<()> {
    *value = value
        .checked_add(1)
        .ok_or_else(|| invalid("counter overflow"))?;
    Ok(())
}
pub(crate) fn finish(s: &mut CanonicalState, result: GameResult) {
    s.result = Some(result);
    s.phase = Phase::Terminal;
}
/// Reference repetition identity deliberately omits IDs, moved and en-passant.
pub(crate) fn position_key(s: &CanonicalState) -> String {
    let mut pieces: Vec<_> = s
        .pieces
        .iter()
        .filter(|p| p.kind != PieceKind::Wall)
        .map(|p| {
            let owner = match p.owner {
                Owner::White => "white",
                Owner::Black => "black",
                Owner::Neutral => "neutral",
            };
            let kind = serde_json::to_value(p.kind).expect("piece identifier");
            let kind = kind.as_str().expect("piece string identifier");
            format!(
                "{},{}:{owner}:{kind}:{}:{}:0",
                p.anchor.row,
                p.anchor.col,
                p.hp.map(|hp| hp.to_string()).unwrap_or_default(),
                p.ammo.map(|ammo| ammo.to_string()).unwrap_or_default()
            )
        })
        .collect();
    pieces.sort();
    format!(
        "{}|{}|{}",
        if s.turn.side == Color::White {
            "white"
        } else {
            "black"
        },
        s.history.repetition_salt,
        pieces.join(";")
    )
}
pub(crate) fn record_position(s: &mut CanonicalState) -> EngineResult<u32> {
    let key = position_key(s);
    if let Some(entry) = s.history.position_counts.iter_mut().find(|e| e.key == key) {
        increment(&mut entry.count)?;
        return Ok(entry.count);
    }
    s.history
        .position_counts
        .push(RepetitionEntry { key, count: 1 });
    s.history.position_counts.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(1)
}
/// Less total card stars wins; half-star integer units avoid rounding.
pub fn star_tiebreak(stars: Sides<u32>, reason: EndReason) -> GameResult {
    use std::cmp::Ordering;
    match stars.white.cmp(&stars.black) {
        Ordering::Equal => GameResult::Draw { reason },
        Ordering::Less => GameResult::Win {
            winner: Color::White,
            reason,
        },
        Ordering::Greater => GameResult::Win {
            winner: Color::Black,
            reason,
        },
    }
}
fn stars(s: &mut CanonicalState, reason: EndReason) {
    if let Some(color) = [Color::White, Color::Black].into_iter().find(|c| {
        s.pieces
            .iter()
            .any(|p| p.owner == (*c).into() && p.kind == PieceKind::ShotgunKing)
    }) {
        finish(
            s,
            GameResult::Win {
                winner: color.opponent(),
                reason,
            },
        );
        return;
    }
    // Phase 2 has empty decks. Card instances will provide totals in Phase 5.
    finish(s, star_tiebreak(Sides { white: 0, black: 0 }, reason));
}
pub(crate) fn mark_progress(s: &mut CanonicalState) {
    if s.result.is_none() {
        if let Some(dm) = &mut s.deathmatch {
            dm.half_turns_since_progress = 0;
            dm.progress_this_turn = true;
        }
    }
}
pub(crate) fn end_turn(s: &mut CanonicalState) -> EngineResult<()> {
    if herald_agreement(s) {
        return Ok(());
    }
    let moving = s.turn.side;
    crate::log::advance(s)?;
    if herald_agreement(s) {
        return Ok(());
    }
    if let Some(index) = s
        .time_stopped
        .iter()
        .position(|color| *color == moving.opponent())
    {
        s.time_stopped.remove(index);
        s.history.en_passant = None;
        return Ok(());
    }
    crate::wizard::resolve_delayed(s)?;
    if herald_agreement(s) {
        return Ok(());
    }
    increment(&mut s.turn.move_count)?;
    if *s.turn.completed.get(moving) == 0 {
        s.players.get_mut(moving).first_move_cards_forced = true;
    }
    increment(s.turn.completed.get_mut(moving))?;
    if moving == Color::Black {
        if let Some(dm) = &mut s.deathmatch {
            if dm.progress_this_turn {
                dm.half_turns_since_progress = 0;
                dm.progress_this_turn = false;
            } else {
                dm.half_turns_since_progress = dm
                    .half_turns_since_progress
                    .saturating_add(2)
                    .min(dm.interval_half_turns);
            }
            if dm.half_turns_since_progress >= dm.interval_half_turns {
                stars(s, EndReason::TurnLimitStars);
                return Ok(());
            }
        }
        increment(&mut s.turn.full_move)?;
    }
    s.players.get_mut(moving).cards_used_this_turn = 0;
    s.turn.side = moving.opponent();
    crate::merchant::grant_gold(s)?;
    if !s.pieces.iter().any(|p| p.kind == PieceKind::ShotgunKing) && record_position(s)? >= 3 {
        stars(s, EndReason::RepetitionStars);
        return Ok(());
    }
    let shared = s.turn.completed.white.min(s.turn.completed.black);
    if s.deathmatch.is_none() && shared >= s.config.star_win_limit {
        if s.config.deathmatch_enabled {
            s.deathmatch = Some(Deathmatch {
                started_at_turn: shared,
                half_turns_since_progress: 0,
                interval_half_turns: s.config.deathmatch_limit_turns * 2,
                progress_this_turn: false,
            });
        } else {
            stars(s, EndReason::TurnLimitStars);
            return Ok(());
        }
    }
    if crate::movement::legal_actions(s)?.is_empty() {
        finish(
            s,
            GameResult::Win {
                winner: moving,
                reason: EndReason::NoActions,
            },
        );
    }
    Ok(())
}

/// Reference checks the actor first, then the opponent, even for fresh heralds.
pub(crate) fn herald_agreement(s: &mut CanonicalState) -> bool {
    if s.result.is_some() {
        return true;
    }
    for color in [s.turn.side, s.turn.side.opponent()] {
        let won = s
            .pieces
            .iter()
            .filter(|p| p.owner == Owner::from(color) && p.kind == PieceKind::Herald)
            .any(|p| {
                s.pieces.iter().any(|q| {
                    q.owner == Owner::from(color.opponent())
                        && q.kind.defeat_royal()
                        && p.anchor
                            .row
                            .abs_diff(q.anchor.row)
                            .max(p.anchor.col.abs_diff(q.anchor.col))
                            == 1
                })
            });
        if won {
            finish(
                s,
                GameResult::Win {
                    winner: color,
                    reason: EndReason::HeraldAgreement,
                },
            );
            return true;
        }
    }
    false
}
