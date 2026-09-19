//! Only primitives required by the currently ported piece transitions.
use crate::*;

/// JS markTransformedOrigin updates both the origin and the fresh-capture clock.
pub(crate) fn transform(p: &mut Piece, into: PieceKind, completed: u32) -> EngineResult<()> {
    let owner = match p.owner {
        Owner::White => Color::White,
        Owner::Black => Color::Black,
        Owner::Neutral => return Err(invalid("cannot transform neutral piece")),
    };
    let completed_turn = completed
        .checked_add(1)
        .ok_or_else(|| invalid("transformation counter overflow"))?;
    p.kind = into;
    p.origin = Some(p.anchor);
    p.windmill_mode = None;
    p.gold = None;
    p.mana = None;
    p.max_mana = None;
    p.ammo = None;
    p.max_ammo = None;
    p.facing = None;
    p.log_direction = None;
    p.log_roll_after_turn = None;
    p.herald_jump_lock_turn = (into == PieceKind::Herald).then_some(completed);
    p.herald_jump_locked = into == PieceKind::Herald;
    p.statuses.clear();
    p.statuses.push(Status::CannotCaptureUntilOwnerTurn {
        owner,
        completed_turn,
    });
    Ok(())
}

/// Apply the specified common spawn lock, omitted by JS leaveRecruiterPawn.
pub(crate) fn recruiter_pawn(s: &mut CanonicalState, owner: Owner, at: Square) -> EngineResult<()> {
    let id = PieceId(s.ids.next_piece);
    s.ids.next_piece = s
        .ids
        .next_piece
        .checked_add(1)
        .ok_or_else(|| invalid("piece allocator overflow"))?;
    let mut pawn = Piece::new(id, owner, PieceKind::Pawn, at);
    transform(
        &mut pawn,
        PieceKind::Pawn,
        *s.turn.completed.get(s.turn.side),
    )?;
    s.pieces.push(pawn);
    Ok(())
}

pub(crate) fn slime_clone(s: &mut CanonicalState, owner: Owner, at: Square) -> EngineResult<()> {
    let id = PieceId(s.ids.next_piece);
    s.ids.next_piece = s
        .ids
        .next_piece
        .checked_add(1)
        .ok_or_else(|| invalid("piece allocator overflow"))?;
    let mut slime = Piece::new(id, owner, PieceKind::Slime, at);
    transform(
        &mut slime,
        PieceKind::Slime,
        *s.turn.completed.get(s.turn.side),
    )?;
    slime.moved = true;
    s.pieces.push(slime);
    Ok(())
}
