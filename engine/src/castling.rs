use crate::movement::at;
use crate::*;
fn big_anchor(king_to: Square, owner: Owner) -> Square {
    Square::new(
        if owner == Owner::White {
            king_to.row - 1
        } else {
            king_to.row
        },
        if king_to.col < 4 {
            king_to.col + 1
        } else {
            king_to.col - 2
        },
    )
}
pub(crate) fn rook_ready(
    s: &CanonicalState,
    king: &Piece,
    rook: &Piece,
    to: Square,
    corner: u16,
) -> bool {
    if rook.owner != king.owner
        || rook.moved
        || !matches!(rook.kind, PieceKind::Rook | PieceKind::BigRook)
    {
        return false;
    }
    let clear = if corner == 7 { 5..7 } else { 1..4 };
    if clear
        .into_iter()
        .any(|c| at(s, Square::new(to.row, c)).is_some_and(|p| p.id != rook.id))
    {
        return false;
    }
    if rook.kind == PieceKind::BigRook {
        let cells = crate::large::cells(big_anchor(to, king.owner));
        if cells.len() != 4
            || cells
                .iter()
                .any(|sq| at(s, *sq).is_some_and(|p| p.owner != king.owner))
        {
            return false;
        }
    }
    true
}
pub(crate) fn apply(s: &mut CanonicalState, action: &Action) -> EngineResult<bool> {
    let Action::Move { from, to, .. } = *action else {
        return Ok(false);
    };
    let Some(king) = at(s, from)
        .filter(|p| p.kind == PieceKind::King && from.col.abs_diff(to.col) == 2)
        .cloned()
    else {
        return Ok(false);
    };
    let corner = if to.col > from.col { 7 } else { 0 };
    let rook = at(s, Square::new(from.row, corner))
        .expect("validated castle rook")
        .clone();
    let anchor = if rook.kind == PieceKind::BigRook {
        big_anchor(to, king.owner)
    } else {
        Square::new(to.row, if corner == 7 { 5 } else { 3 })
    };
    let footprint = if rook.kind == PieceKind::BigRook {
        crate::large::cells(anchor)
    } else {
        vec![anchor]
    };
    // Big-rook castling erases friendly occupants, without capture or royal-loss hooks.
    let erased: Vec<_> = footprint
        .iter()
        .filter_map(|sq| at(s, *sq))
        .filter(|p| p.id != king.id && p.id != rook.id)
        .map(|p| p.id)
        .collect();
    s.pieces.retain(|p| !erased.contains(&p.id));
    let p = s.pieces.iter_mut().find(|p| p.id == king.id).unwrap();
    p.anchor = to;
    p.footprint = vec![to];
    p.moved = true;
    let p = s.pieces.iter_mut().find(|p| p.id == rook.id).unwrap();
    p.anchor = anchor;
    p.footprint = footprint;
    p.moved = true;
    *s.history.castled.get_mut(s.turn.side) = true;
    s.history.en_passant = None;
    crate::transition::rebuild(s)?;
    crate::victory::end_turn(s)?;
    Ok(true)
}
