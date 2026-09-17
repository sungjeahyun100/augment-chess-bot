use crate::*;

/// Source merchantPurchasePrice uses encyclopedia values, not piecePower.
fn cost(kind: PieceKind) -> Option<u32> {
    use PieceKind::*;
    Some(match kind {
        Ferz | Wall => return None,
        King | ShotgunKing | RoyalKnight | Merchant => 20,
        Queen | Recruiter | Wizard | PrimeMinister => 9,
        Rook | Herald | Grasshopper | Clockwork | Pegasus => 5,
        Knight | Bishop | Protestant | Knightmaster => 3,
        Cannon | Man | Assassin | CheckerKing | Windmill => 4,
        Amazon => 13,
        Cardinal | Berserker => 7,
        Hook => 15,
        Princess => 6,
        Colossus => 12,
        BigRook | BigBishop => 8,
        Fanatic | Log | Pawn | Squire | StandardBearer | Eagle | Camel | Alfil | Guard
        | Checker => 2,
    })
}
pub(crate) fn actions(s: &CanonicalState, merchant: &Piece) -> Vec<Action> {
    if merchant.statuses.iter().any(|status| match status {
        Status::CannotCaptureUntilOwnerTurn {
            owner,
            completed_turn,
        } => s.turn.completed.get(*owner) < completed_turn,
    }) {
        return vec![];
    }
    s.pieces
        .iter()
        .filter(|p| {
            p.owner == Owner::from(s.turn.side.opponent())
                && !matches!(p.kind, PieceKind::Merchant | PieceKind::Wall)
                && !p.shielded
                && cost(p.kind).is_some_and(|price| price <= merchant.gold.unwrap_or(0))
        })
        .map(|p| Action::Purchase {
            merchant: merchant.id,
            target: p.id,
        })
        .collect()
}
pub(crate) fn purchase(
    s: &mut CanonicalState,
    merchant: PieceId,
    target: PieceId,
) -> EngineResult<()> {
    let target_kind = s.pieces.iter().find(|p| p.id == target).unwrap().kind;
    let buyer = s.pieces.iter_mut().find(|p| p.id == merchant).unwrap();
    buyer.gold =
        Some(buyer.gold.unwrap_or(0) - cost(target_kind).expect("validated purchase price"));
    if target_kind.royal() {
        victory::finish(
            s,
            GameResult::Win {
                winner: s.turn.side,
                reason: EndReason::RoyalPurchase,
            },
        );
        return Ok(());
    }
    let purchased = s.pieces.iter_mut().find(|p| p.id == target).unwrap();
    purchased.owner = s.turn.side.into();
    purchased.moved = true;
    for status in &mut purchased.statuses {
        match status {
            Status::CannotCaptureUntilOwnerTurn { owner, .. } => *owner = s.turn.side,
        }
    }
    victory::end_turn(s)
}
pub(crate) fn grant_gold(s: &mut CanonicalState) -> EngineResult<()> {
    for p in &mut s.pieces {
        if p.kind == PieceKind::Merchant && p.owner == s.turn.side.into() {
            let mut gold = p.gold.unwrap_or(0);
            victory::increment(&mut gold)?;
            p.gold = Some(gold);
        }
    }
    Ok(())
}
