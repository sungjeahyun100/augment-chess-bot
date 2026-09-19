use crate::*;

pub(crate) fn ensure_supported(s: &CanonicalState) -> EngineResult<()> {
    if s.capability != Capability::Cardless
        || s.board.rows() != 8
        || s.board.cols() != 8
        || matches!(s.turn.continuation, Some(Continuation::ExtraMove { .. }))
        || s.turn.actions_remaining != 1
        || s.pieces.iter().any(|p| {
            (p.owner == Owner::Neutral && p.kind != PieceKind::Wall)
                || (if p.kind.large() {
                    p.footprint.len() != 4
                        || p.footprint != crate::large::cells(p.anchor)
                        || p.hp.is_none()
                } else {
                    p.footprint != [p.anchor]
                        || (p.hp.is_some() && p.kind != PieceKind::ShotgunKing)
                        || (p.kind == PieceKind::ShotgunKing && p.hp.is_none())
                })
        })
    {
        return Err(EngineError::Unsupported(
            "only normal 8x8 cardless play is implemented",
        ));
    }
    Ok(())
}
pub(crate) fn at(s: &CanonicalState, sq: Square) -> Option<&Piece> {
    let id = s.board.at(sq).ok().flatten()?;
    s.pieces.iter().find(|p| p.id == id)
}
pub(crate) fn offset(sq: Square, dr: i16, dc: i16) -> Option<Square> {
    let (r, c) = (sq.row as i16 + dr, sq.col as i16 + dc);
    ((0..8).contains(&r) && (0..8).contains(&c)).then_some(Square::new(r as u16, c as u16))
}
pub(crate) fn can_capture(s: &CanonicalState, p: &Piece, q: &Piece) -> bool {
    p.owner != q.owner
        && !matches!(p.kind, PieceKind::Campfire | PieceKind::Paladin)
        && !campfire_protected(s, q)
        && (p.kind != PieceKind::Jester || q.kind.jester_target())
        && (q.kind != PieceKind::Jester || p.kind.royal())
        && !matches!(p.kind, PieceKind::Guard | PieceKind::Recruiter)
        && !matches!(q.kind, PieceKind::Guard | PieceKind::Wall)
        && !p.statuses.iter().any(|status| match status {
            Status::CannotCaptureUntilOwnerTurn {
                owner,
                completed_turn,
            } => s.turn.completed.get(*owner) < completed_turn,
        })
}
pub(crate) fn campfire_protected(s: &CanonicalState, target: &Piece) -> bool {
    !target.kind.royal()
        && s.pieces.iter().any(|fire| {
            fire.id != target.id
                && fire.owner == target.owner
                && fire.kind == PieceKind::Campfire
                && fire.footprint.iter().any(|fire_square| {
                    target.footprint.iter().any(|target_square| {
                        fire_square.row.abs_diff(target_square.row)
                            + fire_square.col.abs_diff(target_square.col)
                            == 1
                    })
                })
        })
}
pub(crate) const DIAGONAL: [(i16, i16); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
pub(crate) const STRAIGHT: [(i16, i16); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
pub(crate) const KNIGHT: [(i16, i16); 8] = [
    (2, 1),
    (2, -1),
    (-2, 1),
    (-2, -1),
    (1, 2),
    (1, -2),
    (-1, 2),
    (-1, -2),
];
fn paladin_radiance(s: &CanonicalState, mover: &Piece) -> std::collections::BTreeSet<Square> {
    s.pieces
        .iter()
        .filter(|p| {
            p.kind == PieceKind::Paladin
                && p.owner != mover.owner
                && (p.anchor.row + p.anchor.col) % 2 == 0
        })
        .flat_map(|p| {
            DIAGONAL
                .into_iter()
                .chain(STRAIGHT)
                .filter_map(|(dr, dc)| offset(p.anchor, dr, dc))
        })
        .collect()
}
fn radiance_line_clear(
    from: Square,
    to: Square,
    radiance: &std::collections::BTreeSet<Square>,
) -> bool {
    let dr = to.row as i16 - from.row as i16;
    let dc = to.col as i16 - from.col as i16;
    if dr != 0 && dc != 0 && dr.abs() != dc.abs() {
        return true;
    }
    let distance = dr.abs().max(dc.abs());
    (1..distance).all(|step| {
        let row = (from.row as i16 + dr.signum() * step) as u16;
        let col = (from.col as i16 + dc.signum() * step) as u16;
        !radiance.contains(&Square::new(row, col))
    })
}
fn radiance_variant_path(
    s: &CanonicalState,
    mover: &Piece,
    to: Square,
    radiance: &std::collections::BTreeSet<Square>,
) -> bool {
    let mut blocked = s.clone();
    let mut id = u32::MAX;
    for square in radiance {
        if at(&blocked, *square).is_some() {
            continue;
        }
        while blocked.pieces.iter().any(|p| p.id == PieceId(id)) {
            id -= 1;
        }
        blocked
            .pieces
            .push(Piece::new(PieceId(id), Owner::Neutral, PieceKind::Wall, *square));
        id -= 1;
    }
    blocked.board = Board::from_pieces(8, 8, &blocked.pieces).expect("radiance blockers are unique");
    let mover = blocked
        .pieces
        .iter()
        .find(|p| p.id == mover.id)
        .expect("moving piece remains present");
    crate::variants::destinations(&blocked, mover).contains(&to)
}
fn radiance_allows_move(s: &CanonicalState, from: Square, to: Square) -> bool {
    let Some(mover) = at(s, from) else {
        return false;
    };
    if (from.row + from.col) % 2 == 0 {
        return true;
    }
    let radiance = paladin_radiance(s, mover);
    if radiance.is_empty() {
        return true;
    }
    let landing = if mover.kind.large() {
        crate::large::cells(to)
    } else {
        vec![to]
    };
    if landing.iter().any(|sq| radiance.contains(sq)) {
        return false;
    }
    let dr = to.row as i16 - from.row as i16;
    let dc = to.col as i16 - from.col as i16;
    let leaps = matches!(
        mover.kind,
        PieceKind::Knight
            | PieceKind::Paladin
            | PieceKind::RoyalKnight
            | PieceKind::Camel
            | PieceKind::Alfil
            | PieceKind::Eagle
            | PieceKind::Assassin
            | PieceKind::Herald
            | PieceKind::Grasshopper
            | PieceKind::Slime
            | PieceKind::Pegasus
            | PieceKind::Checker
            | PieceKind::CheckerKing
    ) || mover.kind == PieceKind::Amazon && KNIGHT.contains(&(dr, dc));
    if leaps {
        return true;
    }
    if matches!(mover.kind, PieceKind::Hook | PieceKind::Cardinal | PieceKind::Protestant | PieceKind::PrimeMinister) {
        return radiance_variant_path(s, mover, to, &radiance);
    }
    if mover.kind == PieceKind::King && from.col.abs_diff(to.col) == 2 {
        let rook_col = if to.col > from.col { 7 } else { 0 };
        let rook_to = Square::new(from.row, if to.col > from.col { 5 } else { 3 });
        if let Some(rook) = at(s, Square::new(from.row, rook_col)) {
            if (rook.anchor.row + rook.anchor.col) % 2 == 1
                && (!radiance_line_clear(rook.anchor, rook_to, &radiance)
                    || radiance.contains(&rook_to))
            {
                return false;
            }
        }
    }
    radiance_line_clear(from, to, &radiance)
}
pub(crate) fn ray_clear(s: &CanonicalState, from: Square, to: Square) -> bool {
    let dr = (to.row as i16 - from.row as i16).signum();
    let dc = (to.col as i16 - from.col as i16).signum();
    let mut current = offset(from, dr, dc);
    while let Some(sq) = current {
        if sq == to {
            return true;
        }
        if at(s, sq).is_some() {
            return false;
        }
        current = offset(sq, dr, dc);
    }
    false
}
/// JS pieceAttacksSquare uses geometry, including pawn diagonals on empty cells.
/// It does not remove pinned attackers or simulate moving the castling king.
pub(crate) fn attacked(s: &CanonicalState, to: Square, by: Color) -> bool {
    s.pieces
        .iter()
        .filter(|p| p.owner == Owner::from(by))
        .any(|p| {
            if p.kind == PieceKind::Campfire
                || p.statuses.iter().any(|status| match status {
                    Status::CannotCaptureUntilOwnerTurn {
                        owner,
                        completed_turn,
                    } => s.turn.completed.get(*owner) < completed_turn,
                })
                || at(s, to).is_some_and(|q| q.kind == PieceKind::Guard)
            {
                return false;
            }
            let dr = to.row as i16 - p.anchor.row as i16;
            let dc = to.col as i16 - p.anchor.col as i16;
            if dr == 0 && dc == 0 {
                return false;
            }
            match p.kind {
                PieceKind::Pawn | PieceKind::Squire | PieceKind::StandardBearer => {
                    if let Some(ready) = (p.kind == PieceKind::Pawn)
                        .then(|| crate::variants::knightmaster_aura(s, p))
                        .flatten()
                    {
                        ready && KNIGHT.contains(&(dr, dc))
                    } else {
                        (dr == if by == Color::White { -1 } else { 1 } && dc.abs() == 1)
                            || (dr == 0
                                && dc.abs() == 1
                                && crate::variants::standard_bearer_aura(s, p) == Some(true))
                    }
                }
                PieceKind::Knight => KNIGHT.contains(&(dr, dc)),
                PieceKind::King => dr.abs().max(dc.abs()) == 1,
                PieceKind::Bishop => dr.abs() == dc.abs() && ray_clear(s, p.anchor, to),
                PieceKind::Rook => (dr == 0 || dc == 0) && ray_clear(s, p.anchor, to),
                PieceKind::Queen => {
                    (dr == 0 || dc == 0 || dr.abs() == dc.abs()) && ray_clear(s, p.anchor, to)
                }
                _ => crate::variants::attacks(s, p, to),
            }
        })
}
pub(crate) fn legal_actions(s: &CanonicalState) -> EngineResult<Vec<Action>> {
    if s.result.is_some() {
        return Ok(vec![]);
    }
    ensure_supported(s)?;
    if let Some(piece) = s.pending_promotion {
        return Ok([
            PieceKind::Queen,
            PieceKind::Rook,
            PieceKind::Bishop,
            PieceKind::Knight,
        ]
        .into_iter()
        .map(|into| Action::Promote { piece, into })
        .collect());
    }
    let forced_checker = s.pieces.iter().any(|p| {
        p.owner == Owner::from(s.turn.side) && !crate::variants::checker_captures(s, p).is_empty()
    });
    let mut actions = crate::wizard::actions(s);
    actions.extend(crate::shotgun::reload_actions(s));
    for p in s
        .pieces
        .iter()
        .filter(|p| p.owner == Owner::from(s.turn.side))
    {
        if let Some(Continuation::CheckerCapture { piece }) = s.turn.continuation {
            if p.id != piece {
                continue;
            }
        }
        if forced_checker && crate::variants::checker_captures(s, p).is_empty() {
            continue;
        }
        if p.kind == PieceKind::ShotgunKing {
            actions.extend(crate::shotgun::actions(s, p));
            continue;
        }
        if p.kind == PieceKind::Log {
            actions.extend(crate::log::actions(p));
            continue;
        }
        if p.kind == PieceKind::Merchant {
            actions.extend(crate::merchant::actions(s, p));
            continue;
        }
        if p.kind.large() {
            actions.extend(crate::large::actions(s, p));
            continue;
        }
        let mut destinations = vec![];
        let mut jump = |dr, dc| {
            if let Some(sq) = offset(p.anchor, dr, dc) {
                if at(s, sq).is_none_or(|q| can_capture(s, p, q)) {
                    destinations.push(sq);
                }
            }
        };
        match p.kind {
            PieceKind::Pawn if crate::variants::knightmaster_aura(s, p).is_some() => {
                let ready = crate::variants::knightmaster_aura(s, p) == Some(true);
                for (dr, dc) in KNIGHT {
                    if let Some(sq) = offset(p.anchor, dr, dc) {
                        if at(s, sq).is_none_or(|q| ready && can_capture(s, p, q)) {
                            destinations.push(sq);
                        }
                    }
                }
            }
            PieceKind::Pawn | PieceKind::Squire | PieceKind::StandardBearer => {
                let dir = if s.turn.side == Color::White { -1 } else { 1 };
                if let Some(one) = offset(p.anchor, dir, 0) {
                    if at(s, one).is_none() {
                        destinations.push(one);
                        let home = if s.turn.side == Color::White {
                            [6, 7]
                        } else {
                            [0, 1]
                        };
                        if !p.moved && home.contains(&p.anchor.row) {
                            if let Some(two) = offset(p.anchor, dir * 2, 0) {
                                if at(s, two).is_none() {
                                    destinations.push(two);
                                }
                            }
                        }
                    }
                }
                for dc in [-1, 1] {
                    if let Some(sq) = offset(p.anchor, dir, dc) {
                        if at(s, sq).is_some_and(|q| can_capture(s, p, q)) {
                            destinations.push(sq);
                        }
                        if let Some(ep) = &s.history.en_passant {
                            if ep.available_to == s.turn.side
                                && ep.target == sq
                                && s.pieces
                                    .iter()
                                    .find(|q| q.id == ep.pawn)
                                    .is_some_and(|q| can_capture(s, p, q))
                                && at(s, sq).is_none_or(|q| can_capture(s, p, q))
                            {
                                destinations.push(sq);
                            }
                        }
                    }
                }
            }
            PieceKind::Knight => {
                for (dr, dc) in KNIGHT {
                    jump(dr, dc);
                }
            }
            PieceKind::King => {
                for (dr, dc) in DIAGONAL.into_iter().chain(STRAIGHT) {
                    jump(dr, dc);
                }
                let row = if s.turn.side == Color::White { 7 } else { 0 };
                if !p.moved
                    && p.anchor == Square::new(row, 4)
                    && !s.history.castling_canceled.get(s.turn.side)
                    && !attacked(s, p.anchor, s.turn.side.opponent())
                    && !crate::wizard::imminent(s, p.anchor, s.turn.side)
                {
                    for (rook_col, pass, end) in [(7, 5, 6), (0, 3, 2)] {
                        let rook = at(s, Square::new(row, rook_col));
                        if rook.is_some_and(|q| {
                            crate::castling::rook_ready(s, p, q, Square::new(row, end), rook_col)
                        }) && [pass, end]
                            .into_iter()
                            .all(|c| !attacked(s, Square::new(row, c), s.turn.side.opponent()))
                        {
                            destinations.push(Square::new(row, end));
                        }
                    }
                }
            }
            PieceKind::Rook | PieceKind::Bishop | PieceKind::Queen => {
                for (dr, dc) in DIAGONAL
                    .into_iter()
                    .filter(|_| p.kind != PieceKind::Rook)
                    .chain(STRAIGHT.into_iter().filter(|_| p.kind != PieceKind::Bishop))
                {
                    let mut next = offset(p.anchor, dr, dc);
                    while let Some(sq) = next {
                        if let Some(q) = at(s, sq) {
                            if can_capture(s, p, q) {
                                destinations.push(sq);
                            }
                            break;
                        }
                        destinations.push(sq);
                        next = offset(sq, dr, dc);
                    }
                }
            }
            _ => destinations.extend(crate::variants::destinations(s, p)),
        }
        if matches!(p.kind, PieceKind::Pawn | PieceKind::StandardBearer) {
            if let Some(ready) = crate::variants::standard_bearer_aura(s, p) {
                if p.kind != PieceKind::Pawn || crate::variants::knightmaster_aura(s, p).is_none() {
                    for dc in [-1, 1] {
                        if let Some(sq) = offset(p.anchor, 0, dc) {
                            if at(s, sq).is_none_or(|q| ready && can_capture(s, p, q)) {
                                destinations.push(sq);
                            }
                        }
                    }
                }
            }
        }
        destinations.sort();
        destinations.dedup();
        actions.extend(destinations.into_iter().map(|to| Action::Move {
            from: p.anchor,
            to,
            route: vec![],
        }));
    }
    actions.retain(|action| match action {
        Action::Move { from, to, .. } => radiance_allows_move(s, *from, *to),
        _ => true,
    });
    Ok(actions)
}
