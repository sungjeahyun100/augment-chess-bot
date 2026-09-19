use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WizardSpell {
    Lightning,
    Shield,
    Meteor,
    TimeStop,
}
impl WizardSpell {
    fn cost(self) -> u32 {
        match self {
            Self::Lightning => 1,
            Self::Shield => 2,
            Self::Meteor => 3,
            Self::TimeStop => 5,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelayedSpellKind {
    Lightning,
    Meteor,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DelayedSpell {
    pub spell: DelayedSpellKind,
    pub anchor: Square,
    pub owner: Color,
    /// Historical identity; the caster may have been captured before impact.
    pub caster: Option<PieceId>,
}
pub(crate) fn validate(s: &CanonicalState) -> EngineResult<()> {
    if s.time_stopped.len() > 2
        || s.time_stopped.len() == 2 && s.time_stopped[0] == s.time_stopped[1]
    {
        return Err(invalid("duplicate time stop color"));
    }
    for p in &s.pieces {
        if (p.mana.is_some() || p.max_mana.is_some()) && p.kind != PieceKind::Wizard {
            return Err(invalid("mana on non-wizard"));
        }
        if p.mana.unwrap_or(0) > p.max_mana.unwrap_or(5) {
            return Err(invalid("mana exceeds maximum"));
        }
    }
    for spell in &s.delayed_spells {
        if spell.anchor.row >= 8
            || spell.anchor.col >= 8
            || spell.spell == DelayedSpellKind::Meteor
                && (spell.anchor.row >= 7 || spell.anchor.col >= 7)
            || spell
                .caster
                .is_some_and(|id| id.0 == 0 || id.0 >= s.ids.next_piece)
        {
            return Err(invalid("invalid delayed spell"));
        }
    }
    Ok(())
}
pub(crate) fn actions(s: &CanonicalState) -> Vec<Action> {
    let mut out = vec![];
    for p in s
        .pieces
        .iter()
        .filter(|p| p.kind == PieceKind::Wizard && p.owner == s.turn.side.into())
    {
        for spell in [
            WizardSpell::Lightning,
            WizardSpell::Shield,
            WizardSpell::Meteor,
            WizardSpell::TimeStop,
        ] {
            if p.mana.unwrap_or(0) < spell.cost() {
                continue;
            }
            let targets: Vec<_> = match spell {
                WizardSpell::TimeStop => vec![p.anchor],
                WizardSpell::Shield => s
                    .pieces
                    .iter()
                    .filter(|q| q.owner == p.owner && q.kind != PieceKind::Wall)
                    .map(|q| q.anchor)
                    .collect(),
                WizardSpell::Lightning | WizardSpell::Meteor => {
                    let edge = if spell == WizardSpell::Meteor { 7 } else { 8 };
                    (0..edge)
                        .flat_map(|r| (0..edge).map(move |c| Square::new(r, c)))
                        .collect()
                }
            };
            out.extend(targets.into_iter().map(|target| Action::CastSpell {
                wizard: p.id,
                spell,
                target,
            }));
        }
    }
    out
}
pub(crate) fn cast(
    s: &mut CanonicalState,
    wizard: PieceId,
    spell: WizardSpell,
    target: Square,
) -> EngineResult<()> {
    let p = s.pieces.iter_mut().find(|p| p.id == wizard).unwrap();
    p.mana = Some(p.mana.unwrap_or(0) - spell.cost());
    match spell {
        WizardSpell::TimeStop => {
            let color = s.turn.side.opponent();
            if !s.time_stopped.contains(&color) {
                s.time_stopped.push(color);
            }
            return Ok(());
        }
        WizardSpell::Shield => {
            s.pieces
                .iter_mut()
                .find(|p| p.anchor == target)
                .unwrap()
                .shielded = true;
        }
        WizardSpell::Lightning | WizardSpell::Meteor => s.delayed_spells.push(DelayedSpell {
            spell: if spell == WizardSpell::Meteor {
                DelayedSpellKind::Meteor
            } else {
                DelayedSpellKind::Lightning
            },
            anchor: target,
            owner: s.turn.side,
            caster: Some(wizard),
        }),
    }
    victory::end_turn(s)
}
pub(crate) fn grant_mana(s: &mut CanonicalState, owner: Owner) {
    for p in &mut s.pieces {
        if p.owner == owner && p.kind == PieceKind::Wizard {
            p.mana = Some(
                p.mana
                    .unwrap_or(0)
                    .saturating_add(1)
                    .min(p.max_mana.unwrap_or(5)),
            );
        }
    }
}
/// capturePieceAt grants mana before removal; HP death uses its own ordering.
pub(crate) fn remove(s: &mut CanonicalState, id: PieceId) {
    if let Some(owner) = s.pieces.iter().find(|p| p.id == id).map(|p| p.owner) {
        grant_mana(s, owner);
    }
    s.pieces.retain(|p| p.id != id);
}
pub(crate) fn resolve_delayed(s: &mut CanonicalState) -> EngineResult<()> {
    let due: Vec<_> = s
        .delayed_spells
        .iter()
        .filter(|h| h.owner.opponent() == s.turn.side)
        .cloned()
        .collect();
    s.delayed_spells
        .retain(|h| h.owner.opponent() != s.turn.side);
    for hazard in due {
        let caster = hazard
            .caster
            .and_then(|id| s.pieces.iter().find(|p| p.id == id).cloned());
        let cells = if hazard.spell == DelayedSpellKind::Meteor {
            crate::large::cells(hazard.anchor)
        } else {
            vec![hazard.anchor]
        };
        let mut seen = std::collections::BTreeSet::new();
        for sq in cells {
            let Some(p) = crate::movement::at(s, sq).cloned() else {
                continue;
            };
            if p.kind == PieceKind::Wall {
                continue;
            }
            if crate::movement::campfire_protected(s, &p) {
                continue;
            }
            let repeats = hazard.spell == DelayedSpellKind::Meteor && p.hp.is_some();
            if !repeats && !seen.insert(p.id) {
                continue;
            }
            if p.shielded {
                s.pieces.iter_mut().find(|q| q.id == p.id).unwrap().shielded = false;
                continue;
            }
            if p.hp.is_some() {
                crate::large::damage(s, p.id, hazard.owner)?;
            } else {
                let armed = caster.as_ref().is_some_and(|attacker| {
                    crate::bear::arm(s, &p, p.anchor, attacker, hazard.owner)
                });
                remove(s, p.id);
                victory::mark_progress(s);
                if p.kind.defeat_royal() {
                    let winner = if p.owner == hazard.owner.into() {
                        hazard.owner.opponent()
                    } else {
                        hazard.owner
                    };
                    victory::finish(
                        s,
                        GameResult::Win {
                            winner,
                            reason: EndReason::RoyalCapture,
                        },
                    );
                }
                crate::transition::rebuild(s)?;
                if armed {
                    crate::bear::resolve(s, hazard.owner, hazard.caster)?;
                }
            }
        }
    }
    // The oracle projection cannot retain an EP reference to an absent pawn.
    if s.history
        .en_passant
        .as_ref()
        .is_some_and(|ep| !s.pieces.iter().any(|p| p.id == ep.pawn))
    {
        s.history.en_passant = None;
    }
    Ok(())
}

/// Only the current king square is checked by the source castle rule.
pub(crate) fn imminent(s: &CanonicalState, at: Square, color: Color) -> bool {
    s.delayed_spells.iter().any(|h| {
        h.owner == color.opponent()
            && match h.spell {
                DelayedSpellKind::Lightning => h.anchor == at,
                DelayedSpellKind::Meteor => crate::large::cells(h.anchor).contains(&at),
            }
    })
}
