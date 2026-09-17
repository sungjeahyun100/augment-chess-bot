use PieceKind::*;
use augment_chess_engine::*;
fn game(entries: &[(Owner, PieceKind, u16, u16)]) -> GameState {
    let mut s = GameState::new(GameConfig::cardless(), 17)
        .unwrap()
        .snapshot();
    s.pieces = entries
        .iter()
        .enumerate()
        .map(|(i, &(owner, kind, r, c))| {
            Piece::new(PieceId(i as u32 + 1), owner, kind, Square::new(r, c))
        })
        .collect();
    s.ids.next_piece = s.pieces.len() as u32 + 1;
    s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
    s.history.position_counts.clear();
    GameState::from_snapshot(s).unwrap()
}
fn mv(r: u16, c: u16, tr: u16, tc: u16) -> Action {
    Action::Move {
        from: Square::new(r, c),
        to: Square::new(tr, tc),
        route: vec![],
    }
}
fn targets(g: &GameState, id: PieceId) -> Vec<Square> {
    let from = g.piece(id).unwrap().anchor;
    g.legal_actions()
        .unwrap()
        .into_iter()
        .filter_map(|a| match a {
            Action::Move { from: f, to, .. } if f == from => Some(to),
            _ => None,
        })
        .collect()
}
use Owner::{Black, White};
macro_rules! leaper_case {
    ($name:ident,$kind:ident,$count:expr,$to:expr) => {
        #[test]
        fn $name() {
            let g = game(&[(White, $kind, 4, 4)]);
            assert_eq!(targets(&g, PieceId(1)).len(), $count);
            let to = $to;
            let mut g = game(&[
                (White, $kind, 4, 4),
                (Black, Pawn, to.0, to.1),
                (Black, King, 0, 7),
            ]);
            assert!(g.legal_actions().unwrap().contains(&mv(4, 4, to.0, to.1)));
            let mut locked = g.snapshot();
            locked.pieces[0]
                .statuses
                .push(Status::CannotCaptureUntilOwnerTurn {
                    owner: Color::White,
                    completed_turn: 1,
                });
            let locked = GameState::from_snapshot(locked).unwrap();
            assert!(
                !locked
                    .legal_actions()
                    .unwrap()
                    .contains(&mv(4, 4, to.0, to.1))
            );
            g.apply_action(mv(4, 4, to.0, to.1)).unwrap();
            assert!(g.piece(PieceId(2)).is_none());
            assert_eq!(
                GameState::from_canonical_json(&g.to_canonical_json().unwrap()).unwrap(),
                g
            );
        }
    };
}
leaper_case!(man, Man, 8, (3, 3));
leaper_case!(ferz, Ferz, 4, (3, 3));
leaper_case!(alfil, Alfil, 4, (2, 2));
leaper_case!(camel, Camel, 8, (1, 3));
leaper_case!(eagle, Eagle, 8, (2, 4));
leaper_case!(amazon, Amazon, 35, (2, 3));
leaper_case!(prime_minister, PrimeMinister, 24, (2, 3));
leaper_case!(royal_knight, RoyalKnight, 8, (2, 3));

#[test]
fn royal_knight_is_royal_for_threats_capture_and_herald_agreement() {
    let mut g = game(&[(White, Assassin, 4, 0), (Black, RoyalKnight, 4, 4)]);
    assert!(g.is_in_check(Color::Black).unwrap());
    g.apply_action(mv(4, 0, 4, 4)).unwrap();
    assert_eq!(
        g.snapshot().result,
        Some(GameResult::Win {
            winner: Color::White,
            reason: EndReason::RoyalCapture,
        })
    );
    let mut g = game(&[(White, Herald, 4, 0), (Black, RoyalKnight, 3, 3)]);
    g.apply_action(mv(4, 0, 4, 3)).unwrap();
    assert_eq!(
        g.snapshot().result,
        Some(GameResult::Win {
            winner: Color::White,
            reason: EndReason::HeraldAgreement,
        })
    );
}

#[test]
fn prime_minister_needs_one_empty_intermediate_and_cannot_capture_twice() {
    let entries = [
        (White, PrimeMinister, 4, 4),
        (White, Pawn, 3, 3),
        (Black, Pawn, 3, 4),
        (White, Pawn, 3, 5),
        (Black, Rook, 2, 4),
        (Black, King, 0, 7),
    ];
    let blocked = game(&entries);
    assert!(!targets(&blocked, PieceId(1)).contains(&Square::new(2, 4)));
    assert!(targets(&blocked, PieceId(1)).contains(&Square::new(3, 4)));
    // The straight path stays blocked; an empty diagonal path suffices.
    let mut open = game(&[entries[0], entries[1], entries[2], entries[4], entries[5]]);
    assert!(targets(&open, PieceId(1)).contains(&Square::new(2, 4)));
    open.apply_action(mv(4, 4, 2, 4)).unwrap();
    assert!(open.piece(PieceId(4)).is_none());
    assert!(open.piece(PieceId(3)).is_some());
}
leaper_case!(knightmaster, Knightmaster, 4, (3, 3));
leaper_case!(protestant, Protestant, 12, (1, 1));
#[test]
fn windmill_changes_mode_after_move_and_capture_but_not_royal_capture() {
    let mut g = game(&[
        (White, Windmill, 4, 4),
        (Black, Pawn, 3, 3),
        (Black, King, 0, 7),
    ]);
    g.apply_action(mv(4, 4, 3, 3)).unwrap();
    assert_eq!(
        g.piece(PieceId(1)).unwrap().windmill_mode,
        Some(WindmillMode::Rook)
    );
    g.apply_action(mv(0, 7, 0, 6)).unwrap();
    g.apply_action(mv(3, 3, 3, 4)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().windmill_mode, None);
    let mut g = game(&[(White, Windmill, 4, 4), (Black, King, 3, 3)]);
    g.apply_action(mv(4, 4, 3, 3)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().windmill_mode, None);
}
#[test]
fn assassin_rays_only_capture_royalty() {
    let g = game(&[
        (White, Assassin, 4, 4),
        (Black, Pawn, 4, 5),
        (Black, King, 4, 7),
    ]);
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(4, 7)));
    let mut g = game(&[(White, Assassin, 4, 4), (Black, King, 4, 7)]);
    assert!(targets(&g, PieceId(1)).contains(&Square::new(4, 7)));
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(4, 5)));
    g.apply_action(mv(4, 4, 4, 7)).unwrap();
    assert!(g.is_terminal());
}
#[test]
fn guard_cannot_capture_or_be_captured() {
    let g = game(&[(White, Guard, 4, 4), (Black, Pawn, 3, 3)]);
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(3, 3)));
    let g = game(&[(White, Rook, 4, 4), (Black, Guard, 4, 6)]);
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(4, 6)));
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(4, 7)));
}
#[test]
fn cannon_requires_screen_for_quiet_moves_and_cannot_jump_or_capture_cannon() {
    let g = game(&[
        (White, Cannon, 4, 0),
        (White, Pawn, 4, 2),
        (Black, Pawn, 4, 5),
    ]);
    assert_eq!(
        targets(&g, PieceId(1)),
        vec![Square::new(4, 3), Square::new(4, 4), Square::new(4, 5)]
    );
    let g = game(&[
        (White, Cannon, 4, 0),
        (White, Cannon, 4, 2),
        (Black, Pawn, 4, 5),
    ]);
    assert!(targets(&g, PieceId(1)).is_empty());
    let g = game(&[
        (White, Cannon, 4, 0),
        (White, Pawn, 4, 2),
        (Black, Cannon, 4, 5),
    ]);
    assert_eq!(
        targets(&g, PieceId(1)),
        vec![Square::new(4, 3), Square::new(4, 4)]
    );
}
#[test]
fn grasshopper_lands_immediately_past_first_obstacle() {
    let mut g = game(&[
        (White, Grasshopper, 4, 0),
        (White, Pawn, 4, 2),
        (Black, Pawn, 4, 3),
        (Black, King, 0, 7),
    ]);
    assert_eq!(targets(&g, PieceId(1)), vec![Square::new(4, 3)]);
    g.apply_action(mv(4, 0, 4, 3)).unwrap();
    assert!(g.piece(PieceId(2)).is_some());
    assert!(g.piece(PieceId(3)).is_none());
}
#[test]
fn hook_can_bend_once_after_an_empty_square() {
    let g = game(&[
        (White, Hook, 4, 4),
        (White, Pawn, 4, 5),
        (White, Pawn, 3, 4),
        (White, Pawn, 5, 4),
        (White, Pawn, 4, 3),
    ]);
    assert!(targets(&g, PieceId(1)).is_empty());
    let g = game(&[(White, Hook, 4, 4)]);
    assert_eq!(targets(&g, PieceId(1)).len(), 63);
}
#[test]
fn cardinal_reflects_at_board_edges() {
    let g = game(&[(White, Cardinal, 4, 4)]);
    let out = targets(&g, PieceId(1));
    assert!(out.contains(&Square::new(6, 6)));
    assert!(out.contains(&Square::new(6, 0)));
    assert!(!out.contains(&Square::new(4, 4)));
}
#[test]
fn knightmaster_aura_replaces_pawn_moves_and_expires_capture_lock() {
    let g = game(&[
        (White, Pawn, 4, 4),
        (White, Knightmaster, 4, 3),
        (Black, Pawn, 2, 5),
        (Black, King, 0, 7),
    ]);
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(3, 4)));
    assert!(targets(&g, PieceId(1)).contains(&Square::new(2, 5)));
    let mut s = g.snapshot();
    s.pieces[1]
        .statuses
        .push(Status::CannotCaptureUntilOwnerTurn {
            owner: Color::White,
            completed_turn: 1,
        });
    let locked = GameState::from_snapshot(s.clone()).unwrap();
    assert!(!targets(&locked, PieceId(1)).contains(&Square::new(2, 5)));
    s.turn.completed.white = 1;
    let mut ready = GameState::from_snapshot(s).unwrap();
    ready.apply_action(mv(4, 4, 2, 5)).unwrap();
    assert!(ready.snapshot().history.en_passant.is_none());
}
#[test]
fn fresh_piece_does_not_threaten_king_or_prevent_castling() {
    let g = game(&[
        (White, King, 7, 4),
        (White, Rook, 7, 7),
        (Black, Amazon, 0, 5),
    ]);
    assert!(!g.legal_actions().unwrap().contains(&mv(7, 4, 7, 6)));
    let mut s = g.snapshot();
    s.pieces[2]
        .statuses
        .push(Status::CannotCaptureUntilOwnerTurn {
            owner: Color::Black,
            completed_turn: 1,
        });
    let g = GameState::from_snapshot(s).unwrap();
    assert!(g.legal_actions().unwrap().contains(&mv(7, 4, 7, 6)));
}
#[test]
fn checker_forces_capture_for_all_pieces_and_continues_same_turn() {
    let mut g = game(&[
        (White, Checker, 6, 0),
        (Black, Pawn, 5, 1),
        (Black, Pawn, 3, 3),
        (White, Rook, 7, 7),
        (Black, King, 0, 7),
    ]);
    assert_eq!(g.legal_actions().unwrap(), vec![mv(6, 0, 4, 2)]);
    g.apply_action(mv(6, 0, 4, 2)).unwrap();
    assert_eq!(g.side_to_move(), Color::White);
    assert_eq!(g.turn().completed.white, 0);
    assert_eq!(g.turn().move_count, 1);
    assert_eq!(
        g.turn().continuation,
        Some(Continuation::CheckerCapture { piece: PieceId(1) })
    );
    assert_eq!(g.legal_actions().unwrap(), vec![mv(4, 2, 2, 4)]);
    let mut replay = GameState::from_canonical_json(&g.to_canonical_json().unwrap()).unwrap();
    g.apply_action(mv(4, 2, 2, 4)).unwrap();
    replay.apply_action(mv(4, 2, 2, 4)).unwrap();
    assert_eq!(g, replay);
    assert_eq!(g.turn().completed.white, 1);
    assert_eq!(g.turn().move_count, 2);
    assert_eq!(g.side_to_move(), Color::Black);
    assert_eq!(g.turn().continuation, None);
}
#[test]
fn checker_king_can_capture_backwards_but_promotion_ends_chain_due_to_fresh_lock() {
    let mut g = game(&[
        (White, Checker, 2, 0),
        (Black, Pawn, 1, 1),
        (Black, Pawn, 1, 3),
        (Black, King, 0, 7),
    ]);
    g.apply_action(mv(2, 0, 0, 2)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().kind, CheckerKing);
    assert_eq!(g.piece(PieceId(1)).unwrap().origin, Some(Square::new(0, 2)));
    assert_eq!(g.side_to_move(), Color::Black);
    assert_eq!(g.turn().continuation, None);
    g.apply_action(mv(0, 7, 0, 6)).unwrap();
    assert_eq!(g.legal_actions().unwrap(), vec![mv(0, 2, 2, 4)]);
    g.apply_action(mv(0, 2, 2, 4)).unwrap();
    assert_eq!(g.side_to_move(), Color::Black);
}
#[test]
fn squire_transforms_after_capture_but_promotes_on_last_rank() {
    let mut g = game(&[
        (White, Squire, 4, 4),
        (Black, Pawn, 3, 3),
        (Black, King, 0, 7),
    ]);
    g.apply_action(mv(4, 4, 3, 3)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().kind, Knight);
    assert_eq!(g.piece(PieceId(1)).unwrap().origin, Some(Square::new(3, 3)));
    let mut g = game(&[
        (White, Squire, 1, 4),
        (Black, Pawn, 0, 3),
        (Black, King, 0, 7),
    ]);
    g.apply_action(mv(1, 4, 0, 3)).unwrap();
    assert_eq!(g.snapshot().pending_promotion, Some(PieceId(1)));
    g.apply_action(Action::Promote {
        piece: PieceId(1),
        into: Rook,
    })
    .unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().kind, Rook);
}
#[test]
fn standard_bearer_adds_sideways_pawn_movement_on_same_rank() {
    let g = game(&[
        (White, StandardBearer, 4, 0),
        (White, Pawn, 4, 4),
        (Black, Pawn, 4, 5),
        (White, Squire, 4, 6),
    ]);
    assert!(targets(&g, PieceId(1)).contains(&Square::new(4, 1)));
    assert!(targets(&g, PieceId(2)).contains(&Square::new(4, 5)));
    assert!(!targets(&g, PieceId(4)).contains(&Square::new(4, 5)));
    let mut s = g.snapshot();
    s.pieces[0]
        .statuses
        .push(Status::CannotCaptureUntilOwnerTurn {
            owner: Color::White,
            completed_turn: 1,
        });
    let g = GameState::from_snapshot(s).unwrap();
    assert!(!targets(&g, PieceId(2)).contains(&Square::new(4, 5)));
    assert!(targets(&g, PieceId(2)).contains(&Square::new(4, 3)));
}
fn large_game(kind: PieceKind, others: &[(Owner, PieceKind, u16, u16)]) -> GameState {
    let mut entries = vec![(White, kind, 4, 4)];
    entries.extend_from_slice(others);
    let mut s = game(&entries).snapshot();
    s.pieces[0].footprint = vec![
        Square::new(4, 4),
        Square::new(4, 5),
        Square::new(5, 4),
        Square::new(5, 5),
    ];
    s.pieces[0].hp = Some(if kind == Colossus { 3 } else { 2 });
    s.pieces[0].max_hp = s.pieces[0].hp;
    s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
    GameState::from_snapshot(s).unwrap()
}
#[test]
fn colossus_moves_one_shared_entity_and_sector_attack_does_not_move_it() {
    let mut g = large_game(Colossus, &[(Black, Pawn, 3, 4), (Black, King, 0, 0)]);
    let untouched = g.clone();
    g.apply_action(mv(4, 4, 3, 4)).unwrap();
    assert!(g.piece(PieceId(2)).is_none());
    assert_eq!(g.piece(PieceId(1)).unwrap().footprint.len(), 4);
    assert_eq!(
        g.board()
            .cells()
            .iter()
            .filter(|id| **id == Some(PieceId(1)))
            .count(),
        4
    );
    assert_eq!(
        untouched.piece(PieceId(1)).unwrap().anchor,
        Square::new(4, 4)
    );
    let mut g = large_game(
        Colossus,
        &[
            (Black, Pawn, 1, 6),
            (Black, Pawn, 2, 7),
            (Black, King, 0, 0),
        ],
    );
    g.apply_action(Action::AttackSector {
        piece: PieceId(1),
        sector: 0,
    })
    .unwrap();
    assert!(g.piece(PieceId(2)).is_none());
    assert!(g.piece(PieceId(3)).is_none());
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(4, 4));
    assert!(!g.piece(PieceId(1)).unwrap().moved);
}
#[test]
fn big_rook_can_trample_friends_and_stops_at_landing_capture() {
    let mut g = large_game(
        BigRook,
        &[
            (White, Pawn, 3, 4),
            (Black, Pawn, 3, 5),
            (Black, King, 0, 0),
        ],
    );
    assert!(targets(&g, PieceId(1)).contains(&Square::new(3, 4)));
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(2, 4)));
    g.apply_action(mv(4, 4, 3, 4)).unwrap();
    assert!(g.piece(PieceId(2)).is_none());
    assert!(g.piece(PieceId(3)).is_none());
}
#[test]
fn big_bishop_diagonal_footprint_captures_up_to_three_entities() {
    let mut g = large_game(
        BigBishop,
        &[
            (White, Pawn, 3, 3),
            (Black, Pawn, 3, 4),
            (Black, Pawn, 4, 3),
            (Black, King, 0, 7),
        ],
    );
    g.apply_action(mv(4, 4, 3, 3)).unwrap();
    assert_eq!(g.pieces().len(), 2);
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(3, 3));
    assert_eq!(
        GameState::from_canonical_json(&g.to_canonical_json().unwrap()).unwrap(),
        g
    );
}
#[test]
fn ordinary_attacks_damage_hp_without_advancing_even_on_lethal_hit() {
    let mut s = large_game(BigRook, &[(Black, Rook, 4, 0), (White, King, 7, 7)]).snapshot();
    s.turn.side = Color::Black;
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(4, 0, 4, 4)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().hp, Some(1));
    assert_eq!(g.piece(PieceId(2)).unwrap().anchor, Square::new(4, 0));
    g.apply_action(mv(7, 7, 7, 6)).unwrap();
    g.apply_action(mv(4, 0, 4, 4)).unwrap();
    assert!(g.piece(PieceId(1)).is_none());
    assert_eq!(g.piece(PieceId(2)).unwrap().anchor, Square::new(4, 0));
    assert!(!g.piece(PieceId(2)).unwrap().moved);
}
#[test]
fn canonical_v3_rejects_v2_and_normalizes_default_windmill_mode() {
    let g = game(&[(White, Windmill, 4, 4)]);
    let mut s = g.snapshot();
    s.pieces[0].windmill_mode = Some(WindmillMode::Bishop);
    assert_eq!(GameState::from_snapshot(s).unwrap(), g);
    let mut s = g.snapshot();
    s.schema_version = 2;
    assert!(GameState::from_snapshot(s).is_err());
    let mut s = g.snapshot();
    s.pieces[0].kind = Pawn;
    s.pieces[0].windmill_mode = Some(WindmillMode::Rook);
    assert!(GameState::from_snapshot(s).is_err());
}
#[test]
fn berserker_movement_depends_on_unique_allied_entity_count() {
    for count in [5, 6, 9, 10] {
        let mut entries = vec![(White, Berserker, 4, 4)];
        for i in 0..count - 1 {
            entries.push((White, Pawn, i / 8, i % 8));
        }
        let g = game(&entries);
        let out = targets(&g, PieceId(1));
        assert_eq!(out.contains(&Square::new(2, 3)), count <= 5);
        assert_eq!(out.contains(&Square::new(4, 0)), count <= 9);
        assert!(out.contains(&Square::new(3, 3)));
    }
}
#[test]
fn princess_uses_queen_movement_only_without_an_allied_queen() {
    let g = game(&[
        (White, Princess, 4, 4),
        (White, Queen, 7, 0),
        (Black, King, 0, 7),
    ]);
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(4, 0)));
    assert!(targets(&g, PieceId(1)).contains(&Square::new(3, 3)));
    let g = game(&[
        (White, Princess, 4, 4),
        (Black, Queen, 7, 0),
        (Black, King, 0, 7),
    ]);
    assert!(targets(&g, PieceId(1)).contains(&Square::new(4, 0)));
}
#[test]
fn clockwork_needs_an_allied_neighbor_and_preserves_source_threat_omission() {
    let g = game(&[(White, Clockwork, 4, 4), (Black, Pawn, 4, 5)]);
    assert!(targets(&g, PieceId(1)).is_empty());
    let mut g = game(&[
        (White, Clockwork, 4, 4),
        (White, Pawn, 4, 5),
        (Black, King, 0, 4),
    ]);
    assert!(targets(&g, PieceId(1)).contains(&Square::new(0, 4)));
    assert!(!g.is_in_check(Color::Black).unwrap());
    g.apply_action(mv(4, 4, 0, 4)).unwrap();
    assert!(g.is_terminal());
}
#[test]
fn checker_global_capture_force_also_blocks_other_checkers_quiet_moves() {
    let g = game(&[
        (White, Checker, 6, 0),
        (Black, Pawn, 5, 1),
        (White, CheckerKing, 4, 7),
        (White, Pawn, 6, 6),
    ]);
    assert_eq!(g.legal_actions().unwrap(), vec![mv(6, 0, 4, 2)]);
}
#[test]
fn bearer_sideways_move_to_ep_target_does_not_capture_the_ep_pawn() {
    let mut s = game(&[
        (White, StandardBearer, 2, 2),
        (Black, Pawn, 3, 3),
        (Black, King, 0, 7),
    ])
    .snapshot();
    s.pieces[1].moved = true;
    s.history.en_passant = Some(EnPassant {
        pawn: PieceId(2),
        target: Square::new(2, 3),
        available_to: Color::White,
    });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(2, 2, 2, 3)).unwrap();
    assert!(g.piece(PieceId(2)).is_some());
}
#[test]
fn wall_is_immobile_uncapturable_and_can_screen_cannons() {
    let g = game(&[
        (White, Cannon, 4, 0),
        (Owner::Neutral, Wall, 4, 2),
        (Black, Pawn, 4, 5),
    ]);
    assert_eq!(
        targets(&g, PieceId(1)),
        vec![Square::new(4, 3), Square::new(4, 4), Square::new(4, 5)]
    );
    let g = game(&[(White, Wall, 4, 2), (White, Rook, 4, 0)]);
    assert!(targets(&g, PieceId(1)).is_empty());
    assert!(!targets(&g, PieceId(2)).contains(&Square::new(4, 2)));
}
#[test]
fn herald_jumps_after_lock_expires_but_never_lands_on_a_piece() {
    let g = game(&[
        (White, Herald, 4, 0),
        (Black, Pawn, 4, 1),
        (Black, King, 0, 7),
    ]);
    assert!(targets(&g, PieceId(1)).contains(&Square::new(4, 3)));
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(4, 1)));
    let mut s = g.snapshot();
    s.pieces[0].herald_jump_locked = true;
    s.pieces[0].herald_jump_lock_turn = Some(0);
    let mut g = GameState::from_snapshot(s).unwrap();
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(4, 3)));
    g.apply_action(mv(4, 0, 3, 0)).unwrap();
    assert!(!g.piece(PieceId(1)).unwrap().herald_jump_locked);
    assert_eq!(g.piece(PieceId(1)).unwrap().herald_jump_lock_turn, Some(0));
    g.apply_action(mv(0, 7, 0, 6)).unwrap();
    g.apply_action(mv(3, 0, 4, 0)).unwrap();
    g.apply_action(mv(0, 6, 0, 7)).unwrap();
    assert!(targets(&g, PieceId(1)).contains(&Square::new(4, 3)));
}
#[test]
fn fresh_herald_agreement_wins_without_completing_turn() {
    let mut s = game(&[(White, Herald, 4, 0), (Black, King, 3, 3)]).snapshot();
    s.pieces[0]
        .statuses
        .push(Status::CannotCaptureUntilOwnerTurn {
            owner: Color::White,
            completed_turn: 1,
        });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(4, 0, 4, 3)).unwrap();
    assert_eq!(
        g.result(),
        Some(&GameResult::Win {
            winner: Color::White,
            reason: EndReason::HeraldAgreement
        })
    );
    assert_eq!(g.turn().completed.white, 0);
    assert_eq!(g.turn().move_count, 0);
    assert!(g.piece(PieceId(1)).unwrap().moved);
}
#[test]
fn king_moving_into_enemy_herald_agreement_loses() {
    let mut g = game(&[(White, King, 5, 3), (Black, Herald, 3, 3)]);
    g.apply_action(mv(5, 3, 4, 3)).unwrap();
    assert_eq!(
        g.result(),
        Some(&GameResult::Win {
            winner: Color::Black,
            reason: EndReason::HeraldAgreement
        })
    );
}
#[test]
fn native_pawn_variants_quiet_moves_do_not_reset_deathmatch() {
    for kind in [Squire, StandardBearer] {
        let mut s = game(&[(White, kind, 4, 0), (Black, King, 0, 7)]).snapshot();
        s.deathmatch = Some(Deathmatch {
            started_at_turn: 45,
            half_turns_since_progress: 4,
            interval_half_turns: 20,
            progress_this_turn: false,
        });
        let mut g = GameState::from_snapshot(s).unwrap();
        g.apply_action(mv(4, 0, 3, 0)).unwrap();
        assert_eq!(
            g.snapshot().deathmatch.unwrap().half_turns_since_progress,
            4
        );
    }
}

#[test]
fn recruiter_leaves_unmoved_pawn_with_deterministic_identity() {
    let mut g = game(&[
        (White, Recruiter, 4, 4),
        (Black, King, 0, 7),
        (Black, Pawn, 4, 5),
    ]);
    assert!(!targets(&g, PieceId(1)).contains(&Square::new(4, 5)));
    let before = g.snapshot();
    g.apply_action(mv(4, 4, 3, 4)).unwrap();
    let pawn = g.piece(PieceId(before.ids.next_piece)).unwrap();
    assert_eq!(pawn.kind, Pawn);
    assert_eq!(pawn.anchor, Square::new(4, 4));
    assert_eq!(pawn.origin, Some(pawn.anchor));
    assert!(!pawn.moved);
    assert_eq!(
        pawn.statuses,
        vec![Status::CannotCaptureUntilOwnerTurn {
            owner: Color::White,
            completed_turn: 1
        }]
    );
    assert_eq!(g.snapshot().ids.next_piece, before.ids.next_piece + 1);
    assert_eq!(g.snapshot().chance, before.chance);
    let mut replay = GameState::from_snapshot(before).unwrap();
    replay.apply_action(mv(4, 4, 3, 4)).unwrap();
    assert_eq!(g.snapshot(), replay.snapshot());
}

#[test]
fn recruiter_allocator_overflow_is_atomic() {
    let g = game(&[(White, Recruiter, 4, 4), (Black, King, 0, 7)]);
    let mut s = g.snapshot();
    s.ids.next_piece = u32::MAX;
    let mut g = GameState::from_snapshot(s.clone()).unwrap();
    assert!(g.apply_action(mv(4, 4, 3, 4)).is_err());
    assert_eq!(g.snapshot(), s);
}

fn funded(g: GameState, gold: u32) -> GameState {
    let mut s = g.snapshot();
    s.pieces
        .iter_mut()
        .find(|p| p.kind == Merchant)
        .unwrap()
        .gold = Some(gold);
    GameState::from_snapshot(s).unwrap()
}
#[test]
fn merchant_purchase_changes_owner_without_moving_or_capturing() {
    let mut g = funded(
        game(&[
            (White, Merchant, 4, 4),
            (Black, Guard, 3, 3),
            (Black, King, 0, 7),
        ]),
        2,
    );
    let action = Action::Purchase {
        merchant: PieceId(1),
        target: PieceId(2),
    };
    assert!(g.legal_actions().unwrap().contains(&action));
    g.apply_action(action).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(4, 4));
    assert_eq!(g.piece(PieceId(1)).unwrap().gold, Some(0));
    assert!(!g.piece(PieceId(1)).unwrap().moved);
    assert_eq!(g.piece(PieceId(2)).unwrap().owner, White);
    assert!(g.piece(PieceId(2)).unwrap().moved);
    g.apply_action(mv(0, 7, 0, 6)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().gold, Some(1));
}
#[test]
fn merchant_royal_purchase_preserves_target_and_turn_counters() {
    let mut g = funded(game(&[(White, Merchant, 4, 4), (Black, King, 0, 7)]), 20);
    g.apply_action(Action::Purchase {
        merchant: PieceId(1),
        target: PieceId(2),
    })
    .unwrap();
    assert_eq!(g.piece(PieceId(2)).unwrap().owner, Black);
    assert_eq!(g.snapshot().turn.completed.white, 0);
    assert_eq!(
        g.snapshot().result,
        Some(GameResult::Win {
            winner: Color::White,
            reason: EndReason::RoyalPurchase
        })
    );
}
#[test]
fn merchant_fresh_lock_blocks_purchase_and_gold_overflow_is_atomic() {
    let g = funded(
        game(&[
            (White, Merchant, 4, 4),
            (White, King, 7, 7),
            (Black, King, 0, 7),
        ]),
        20,
    );
    let mut s = g.snapshot();
    s.pieces[0]
        .statuses
        .push(Status::CannotCaptureUntilOwnerTurn {
            owner: Color::White,
            completed_turn: 1,
        });
    let g = GameState::from_snapshot(s).unwrap();
    assert!(
        !g.legal_actions()
            .unwrap()
            .iter()
            .any(|a| matches!(a, Action::Purchase { .. }))
    );
    let mut s = g.snapshot();
    s.turn.side = Color::Black;
    s.pieces[0].gold = Some(u32::MAX);
    let mut g = GameState::from_snapshot(s.clone()).unwrap();
    assert!(g.apply_action(mv(0, 7, 0, 6)).is_err());
    assert_eq!(g.snapshot(), s);
}
#[test]
fn capturing_merchant_and_herald_next_to_merchant_end_game() {
    let mut g = game(&[(White, Rook, 4, 0), (Black, Merchant, 4, 4)]);
    g.apply_action(mv(4, 0, 4, 4)).unwrap();
    assert!(g.snapshot().result.is_some());
    let mut g = game(&[(White, Herald, 4, 0), (Black, Merchant, 3, 3)]);
    g.apply_action(mv(4, 0, 4, 3)).unwrap();
    assert_eq!(
        g.snapshot().result,
        Some(GameResult::Win {
            winner: Color::White,
            reason: EndReason::HeraldAgreement
        })
    );
}

#[test]
fn merchant_purchase_retains_historical_en_passant_color() {
    let g = funded(
        game(&[
            (White, Merchant, 7, 0),
            (Black, Pawn, 3, 3),
            (Black, King, 0, 7),
        ]),
        2,
    );
    let mut s = g.snapshot();
    s.pieces[1].moved = true;
    s.history.en_passant = Some(EnPassant {
        pawn: PieceId(2),
        target: Square::new(2, 3),
        available_to: Color::White,
    });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(Action::Purchase {
        merchant: PieceId(1),
        target: PieceId(2),
    })
    .unwrap();
    assert_eq!(g.piece(PieceId(2)).unwrap().owner, White);
    assert_eq!(
        g.snapshot()
            .history
            .en_passant
            .as_ref()
            .unwrap()
            .available_to,
        Color::White
    );
    let mut restored = GameState::from_snapshot(g.snapshot()).unwrap();
    restored.apply_action(mv(0, 7, 0, 6)).unwrap();
    assert!(restored.snapshot().history.en_passant.is_some());
    restored.apply_action(mv(3, 3, 2, 3)).unwrap();
    assert!(restored.snapshot().history.en_passant.is_none());
}

#[test]
fn shield_blocks_direct_capture_without_moving_attacker_or_damaging_hp() {
    for kind in [Pawn, King, BigRook] {
        let g = game(&[
            (White, Rook, 4, 0),
            (Black, kind, 4, 4),
            (Black, King, 0, 7),
        ]);
        let mut s = g.snapshot();
        let target = &mut s.pieces[1];
        target.shielded = true;
        if kind == BigRook {
            target.footprint = vec![
                Square::new(4, 4),
                Square::new(4, 5),
                Square::new(5, 4),
                Square::new(5, 5),
            ];
            target.hp = Some(2);
            target.max_hp = Some(2);
        }
        s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
        let mut g = GameState::from_snapshot(s).unwrap();
        g.apply_action(mv(4, 0, 4, 4)).unwrap();
        assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(4, 0));
        assert!(!g.piece(PieceId(1)).unwrap().moved);
        let target = g.piece(PieceId(2)).unwrap();
        assert!(!target.shielded);
        assert_eq!(target.hp, if kind == BigRook { Some(2) } else { None });
        assert!(g.snapshot().result.is_none());
    }
}
#[test]
fn checker_breaks_shield_while_jumping_and_keeps_capture_chain() {
    let g = game(&[
        (White, CheckerKing, 4, 4),
        (Black, Pawn, 3, 3),
        (Black, King, 0, 7),
    ]);
    let mut s = g.snapshot();
    s.pieces[1].shielded = true;
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(4, 4, 2, 2)).unwrap();
    assert!(!g.piece(PieceId(2)).unwrap().shielded);
    assert_eq!(
        g.snapshot().turn.continuation,
        Some(Continuation::CheckerCapture { piece: PieceId(1) })
    );
    g.apply_action(mv(2, 2, 4, 4)).unwrap();
    assert!(g.piece(PieceId(2)).is_none());
}
#[test]
fn en_passant_breaks_shield_without_removing_or_moving_either_pawn() {
    let g = game(&[
        (White, Pawn, 3, 2),
        (Black, Pawn, 3, 3),
        (Black, King, 0, 7),
    ]);
    let mut s = g.snapshot();
    s.pieces[1].shielded = true;
    s.pieces[1].moved = true;
    s.history.en_passant = Some(EnPassant {
        pawn: PieceId(2),
        target: Square::new(2, 3),
        available_to: Color::White,
    });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(3, 2, 2, 3)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(3, 2));
    assert!(!g.piece(PieceId(2)).unwrap().shielded);
    assert!(g.snapshot().history.en_passant.is_none());
}

#[test]
fn large_landing_ignores_shield_but_colossus_sector_skips_it() {
    for kind in [BigRook, Colossus] {
        let g = game(&[
            (White, kind, 4, 4),
            (Black, Pawn, 3, 4),
            (Black, King, 0, 7),
        ]);
        let mut s = g.snapshot();
        s.pieces[0].footprint = vec![
            Square::new(4, 4),
            Square::new(4, 5),
            Square::new(5, 4),
            Square::new(5, 5),
        ];
        s.pieces[0].hp = Some(2);
        s.pieces[0].max_hp = Some(2);
        s.pieces[1].shielded = true;
        s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
        let mut g = GameState::from_snapshot(s).unwrap();
        g.apply_action(mv(4, 4, 3, 4)).unwrap();
        assert!(g.piece(PieceId(2)).is_none());
    }
    let g = game(&[
        (White, Colossus, 4, 4),
        (Black, Pawn, 2, 6),
        (Black, Pawn, 1, 6),
        (Black, King, 0, 0),
    ]);
    let mut s = g.snapshot();
    s.pieces[0].footprint = vec![
        Square::new(4, 4),
        Square::new(4, 5),
        Square::new(5, 4),
        Square::new(5, 5),
    ];
    s.pieces[0].hp = Some(3);
    s.pieces[0].max_hp = Some(3);
    s.pieces[1].shielded = true;
    s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(Action::AttackSector {
        piece: PieceId(1),
        sector: 0,
    })
    .unwrap();
    assert!(g.piece(PieceId(2)).unwrap().shielded);
    assert!(g.piece(PieceId(3)).is_none());
}

fn charged(g: GameState, mana: u32) -> GameState {
    let mut s = g.snapshot();
    let p = s.pieces.iter_mut().find(|p| p.kind == Wizard).unwrap();
    p.mana = Some(mana);
    p.max_mana = Some(5);
    GameState::from_snapshot(s).unwrap()
}
fn spell(wizard: u32, spell: WizardSpell, row: u16, col: u16) -> Action {
    Action::CastSpell {
        wizard: PieceId(wizard),
        spell,
        target: Square::new(row, col),
    }
}
#[test]
fn wizard_lightning_hits_after_enemy_action_before_turn_completion() {
    let mut g = charged(game(&[(White, Wizard, 4, 4), (Black, King, 0, 7)]), 1);
    g.apply_action(spell(1, WizardSpell::Lightning, 0, 6))
        .unwrap();
    assert_eq!(g.snapshot().delayed_spells.len(), 1);
    assert_eq!(g.snapshot().turn.completed.white, 1);
    let mut restored = GameState::from_snapshot(g.snapshot()).unwrap();
    restored.apply_action(mv(0, 7, 0, 6)).unwrap();
    assert!(restored.snapshot().delayed_spells.is_empty());
    assert_eq!(restored.snapshot().turn.completed.black, 0);
    assert_eq!(
        restored.snapshot().result,
        Some(GameResult::Win {
            winner: Color::White,
            reason: EndReason::RoyalCapture
        })
    );
}
#[test]
fn wizard_time_stop_spends_mana_without_ending_turn_and_defers_completion() {
    let mut g = charged(
        game(&[
            (White, Wizard, 4, 4),
            (White, Pawn, 6, 0),
            (Black, King, 0, 7),
        ]),
        5,
    );
    g.apply_action(spell(1, WizardSpell::TimeStop, 4, 4))
        .unwrap();
    assert_eq!(g.snapshot().time_stopped, vec![Color::Black]);
    assert_eq!(g.snapshot().turn.move_count, 0);
    g.apply_action(mv(6, 0, 5, 0)).unwrap();
    assert!(g.snapshot().time_stopped.is_empty());
    assert_eq!(g.snapshot().turn.completed.white, 0);
    assert_eq!(g.snapshot().turn.move_count, 0);
    assert_eq!(g.snapshot().turn.side, Color::White);
    g.apply_action(mv(5, 0, 4, 0)).unwrap();
    assert_eq!(g.snapshot().turn.completed.white, 1);
}
#[test]
fn wizard_shield_and_fresh_spell_cast_use_actual_target_entity() {
    let g = charged(
        game(&[
            (White, Wizard, 4, 4),
            (White, Pawn, 6, 0),
            (Black, Rook, 6, 7),
            (Black, King, 0, 7),
        ]),
        2,
    );
    let mut s = g.snapshot();
    s.pieces[0]
        .statuses
        .push(Status::CannotCaptureUntilOwnerTurn {
            owner: Color::White,
            completed_turn: 1,
        });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(spell(1, WizardSpell::Shield, 6, 0)).unwrap();
    assert!(g.piece(PieceId(2)).unwrap().shielded);
    g.apply_action(mv(6, 7, 6, 0)).unwrap();
    assert!(!g.piece(PieceId(2)).unwrap().shielded);
    assert_eq!(g.piece(PieceId(3)).unwrap().anchor, Square::new(6, 7));
}
#[test]
fn wizard_gets_mana_for_allied_loss_not_for_ordinary_turns() {
    let mut g = charged(
        game(&[
            (White, Rook, 4, 0),
            (Black, Pawn, 4, 3),
            (Black, Wizard, 6, 6),
            (Black, King, 0, 7),
        ]),
        0,
    );
    g.apply_action(mv(4, 0, 4, 3)).unwrap();
    assert_eq!(g.piece(PieceId(3)).unwrap().mana, Some(1));
    g.apply_action(mv(6, 6, 5, 6)).unwrap();
    assert_eq!(g.piece(PieceId(3)).unwrap().mana, Some(1));
}
#[test]
fn wizard_meteor_repeats_on_hp_cells_even_after_shield_break() {
    let g = charged(
        game(&[
            (White, Wizard, 7, 0),
            (Black, Colossus, 3, 3),
            (Black, King, 0, 7),
        ]),
        3,
    );
    let mut s = g.snapshot();
    s.pieces[1].footprint = vec![
        Square::new(3, 3),
        Square::new(3, 4),
        Square::new(4, 3),
        Square::new(4, 4),
    ];
    s.pieces[1].hp = Some(3);
    s.pieces[1].max_hp = Some(3);
    s.pieces[1].shielded = true;
    s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(spell(1, WizardSpell::Meteor, 3, 3)).unwrap();
    g.apply_action(mv(0, 7, 0, 6)).unwrap();
    assert!(g.piece(PieceId(2)).is_none());
}
#[test]
fn delayed_spell_survives_caster_death_and_validation_rejects_invalid_area() {
    let mut s = game(&[
        (White, Wizard, 4, 4),
        (Black, Rook, 4, 0),
        (Black, King, 0, 7),
    ])
    .snapshot();
    s.turn.side = Color::Black;
    s.delayed_spells.push(DelayedSpell {
        spell: DelayedSpellKind::Lightning,
        anchor: Square::new(0, 7),
        owner: Color::White,
        caster: Some(PieceId(1)),
    });
    let mut g = GameState::from_snapshot(s.clone()).unwrap();
    g.apply_action(mv(4, 0, 4, 4)).unwrap();
    assert!(g.piece(PieceId(1)).is_none());
    assert!(g.piece(PieceId(3)).is_none());
    assert!(g.snapshot().result.is_some());
    s.delayed_spells[0].spell = DelayedSpellKind::Meteor;
    s.delayed_spells[0].anchor = Square::new(7, 7);
    assert!(GameState::from_snapshot(s).is_err());
}

#[test]
fn time_stop_defers_incoming_spell_until_real_turn_completion() {
    let g = charged(
        game(&[
            (White, Wizard, 4, 4),
            (White, Pawn, 6, 0),
            (Black, King, 0, 7),
        ]),
        5,
    );
    let mut s = g.snapshot();
    s.delayed_spells.push(DelayedSpell {
        spell: DelayedSpellKind::Lightning,
        anchor: Square::new(4, 4),
        owner: Color::Black,
        caster: None,
    });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(spell(1, WizardSpell::TimeStop, 4, 4))
        .unwrap();
    g.apply_action(mv(6, 0, 5, 0)).unwrap();
    assert!(g.piece(PieceId(1)).is_some());
    assert_eq!(g.snapshot().delayed_spells.len(), 1);
    g.apply_action(mv(5, 0, 4, 0)).unwrap();
    assert!(g.piece(PieceId(1)).is_none());
    assert!(g.snapshot().delayed_spells.is_empty());
    assert_eq!(g.snapshot().turn.completed.white, 1);
}

fn log_dir(piece: u32, dr: i8, dc: i8) -> Action {
    Action::SetLogDirection {
        piece: PieceId(piece),
        direction: LogDirection { dr, dc },
    }
}
#[test]
fn log_direction_waits_until_next_own_turn_and_rolls_after_other_piece_moves() {
    let mut g = game(&[(White, Log, 4, 4), (White, Pawn, 6, 0), (Black, King, 0, 7)]);
    g.apply_action(log_dir(1, -1, 0)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(4, 4));
    assert_eq!(g.piece(PieceId(1)).unwrap().log_roll_after_turn, Some(1));
    g.apply_action(mv(0, 7, 0, 6)).unwrap();
    g.apply_action(mv(6, 0, 5, 0)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(3, 4));
    assert_eq!(g.piece(PieceId(1)).unwrap().log_roll_after_turn, Some(2));
}
#[test]
fn log_stops_at_shield_and_guard_without_removing_them() {
    for (kind, shield) in [(Pawn, true), (Guard, false), (Pawn, false)] {
        let mut s = game(&[
            (White, Log, 4, 4),
            (White, Pawn, 6, 0),
            (Black, kind, 3, 4),
            (Black, King, 0, 7),
        ])
        .snapshot();
        s.pieces[0].log_direction = Some(LogDirection { dr: -1, dc: 0 });
        s.pieces[2].shielded = shield;
        let mut g = GameState::from_snapshot(s).unwrap();
        g.apply_action(mv(6, 0, 5, 0)).unwrap();
        if shield || kind == Guard {
            assert!(g.piece(PieceId(1)).unwrap().log_direction.is_none());
            assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(4, 4));
            assert_eq!(g.piece(PieceId(3)).unwrap().shielded, shield);
        } else {
            assert!(g.piece(PieceId(3)).is_none());
            assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(3, 4));
        }
    }
}
#[test]
fn log_rolls_before_time_stop_but_only_once_per_completed_turn() {
    let mut s = game(&[(White, Log, 4, 4), (White, Pawn, 6, 0), (Black, King, 0, 7)]).snapshot();
    s.pieces[0].log_direction = Some(LogDirection { dr: -1, dc: 0 });
    s.time_stopped = vec![Color::Black];
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(6, 0, 5, 0)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(3, 4));
    assert_eq!(g.snapshot().turn.completed.white, 0);
    g.apply_action(mv(5, 0, 4, 0)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(3, 4));
    assert_eq!(g.snapshot().turn.completed.white, 1);
}
#[test]
fn log_royal_capture_precedes_turn_counters_and_invalid_direction_is_rejected() {
    let mut s = game(&[(White, Log, 4, 4), (White, Pawn, 6, 0), (Black, King, 3, 4)]).snapshot();
    s.pieces[0].log_direction = Some(LogDirection { dr: -1, dc: 0 });
    let mut g = GameState::from_snapshot(s.clone()).unwrap();
    g.apply_action(mv(6, 0, 5, 0)).unwrap();
    assert_eq!(g.snapshot().turn.completed.white, 0);
    assert!(g.snapshot().result.is_some());
    s.pieces[0].log_direction = Some(LogDirection { dr: 0, dc: 0 });
    assert!(GameState::from_snapshot(s).is_err());
}

#[test]
fn log_can_occupy_ep_landing_and_pawn_captures_both_entities() {
    let mut s = game(&[
        (White, Log, 6, 3),
        (White, Pawn, 6, 4),
        (Black, Pawn, 4, 3),
        (Black, King, 0, 7),
    ])
    .snapshot();
    s.pieces[0].log_direction = Some(LogDirection { dr: -1, dc: 1 });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(6, 4, 4, 4)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(5, 4));
    assert_eq!(
        g.snapshot().history.en_passant.as_ref().unwrap().target,
        Square::new(5, 4)
    );
    g.apply_action(mv(4, 3, 5, 4)).unwrap();
    assert!(g.piece(PieceId(1)).is_none());
    assert!(g.piece(PieceId(2)).is_none());
    assert_eq!(g.piece(PieceId(3)).unwrap().anchor, Square::new(5, 4));
}

#[test]
fn squire_captures_only_occupied_ep_landing_without_normal_pawn_priority() {
    let mut s = game(&[
        (White, Log, 6, 3),
        (White, Pawn, 6, 4),
        (Black, Squire, 4, 3),
        (Black, King, 0, 7),
    ])
    .snapshot();
    s.pieces[0].log_direction = Some(LogDirection { dr: -1, dc: 1 });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(6, 4, 4, 4)).unwrap();
    g.apply_action(mv(4, 3, 5, 4)).unwrap();
    assert!(g.piece(PieceId(1)).is_none());
    assert!(g.piece(PieceId(2)).is_some());
    assert_eq!(g.piece(PieceId(3)).unwrap().kind, Knight);
}

fn armed(g: GameState, ammo: u32) -> GameState {
    let mut s = g.snapshot();
    let p = s.pieces.iter_mut().find(|p| p.kind == ShotgunKing).unwrap();
    p.hp = Some(4);
    p.max_hp = Some(4);
    p.ammo = Some(ammo);
    p.max_ammo = Some(3);
    p.facing = Some(Facing::Up);
    GameState::from_snapshot(s).unwrap()
}
#[test]
fn shotgun_reload_and_diagonal_movement_update_resources_and_facing() {
    let mut g = armed(game(&[(White, ShotgunKing, 4, 4), (Black, King, 0, 7)]), 0);
    g.apply_action(Action::Reload { piece: PieceId(1) })
        .unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().ammo, Some(1));
    g.apply_action(mv(0, 7, 0, 6)).unwrap();
    g.apply_action(mv(4, 4, 5, 5)).unwrap();
    assert_eq!(g.piece(PieceId(1)).unwrap().facing, Some(Facing::Down));
}
#[test]
fn shotgun_blast_hits_friends_but_skips_guard_and_spends_two_ammo() {
    let mut g = armed(
        game(&[
            (White, ShotgunKing, 4, 4),
            (White, Pawn, 3, 3),
            (Black, Pawn, 3, 4),
            (Black, Guard, 3, 5),
            (Black, King, 0, 7),
        ]),
        3,
    );
    g.apply_action(Action::ShotgunBlast {
        piece: PieceId(1),
        direction: ShotgunDirection { dr: -1, dc: 0 },
    })
    .unwrap();
    assert!(g.piece(PieceId(2)).is_none());
    assert!(g.piece(PieceId(3)).is_none());
    assert!(g.piece(PieceId(4)).is_some());
    assert_eq!(g.piece(PieceId(1)).unwrap().ammo, Some(1));
}
#[test]
fn shotgun_snipe_breaks_shield_without_advancing() {
    let g = armed(
        game(&[
            (White, ShotgunKing, 4, 4),
            (Black, Pawn, 4, 7),
            (Black, King, 0, 7),
        ]),
        3,
    );
    let mut s = g.snapshot();
    s.pieces[1].shielded = true;
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(Action::ShotgunSnipe {
        piece: PieceId(1),
        target: Square::new(4, 7),
    })
    .unwrap();
    assert!(!g.piece(PieceId(2)).unwrap().shielded);
    assert_eq!(g.piece(PieceId(1)).unwrap().ammo, Some(0));
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(4, 4));
}
#[test]
fn shotgun_king_loses_on_lethal_hp_hit_without_attacker_advancing() {
    let g = armed(game(&[(White, Rook, 4, 0), (Black, ShotgunKing, 4, 4)]), 0);
    let mut s = g.snapshot();
    s.pieces[1].hp = Some(1);
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(4, 0, 4, 4)).unwrap();
    assert_eq!(
        g.snapshot().result,
        Some(GameResult::Win {
            winner: Color::White,
            reason: EndReason::RoyalCapture
        })
    );
    assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(4, 0));
}

#[test]
fn shotgun_disables_repetition_recording_and_loses_star_limit() {
    let g = armed(game(&[(White, ShotgunKing, 4, 4), (Black, King, 0, 7)]), 3);
    let mut s = g.snapshot();
    s.config.star_win_limit = 1;
    s.config.deathmatch_enabled = false;
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(4, 4, 5, 4)).unwrap();
    assert!(g.snapshot().history.position_counts.is_empty());
    g.apply_action(mv(0, 7, 0, 6)).unwrap();
    assert_eq!(
        g.snapshot().result,
        Some(GameResult::Win {
            winner: Color::Black,
            reason: EndReason::TurnLimitStars
        })
    );
}
#[test]
fn colossus_sector_stops_after_lethal_shotgun_royal_hp_hit() {
    let g = armed(
        game(&[
            (White, Colossus, 4, 4),
            (Black, ShotgunKing, 2, 6),
            (Black, King, 1, 6),
        ]),
        0,
    );
    let mut s = g.snapshot();
    s.pieces[0].footprint = vec![
        Square::new(4, 4),
        Square::new(4, 5),
        Square::new(5, 4),
        Square::new(5, 5),
    ];
    s.pieces[0].hp = Some(3);
    s.pieces[0].max_hp = Some(3);
    s.pieces[1].hp = Some(1);
    s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(Action::AttackSector {
        piece: PieceId(1),
        sector: 0,
    })
    .unwrap();
    assert!(g.piece(PieceId(2)).is_none());
    assert!(g.piece(PieceId(3)).is_some());
    assert!(g.snapshot().result.is_some());
}

#[test]
fn pegasus_teleports_to_empty_squares_but_only_captures_as_knight() {
    let mut g = game(&[
        (White, Pegasus, 4, 4),
        (White, Pawn, 3, 4),
        (Black, Pawn, 0, 0),
        (Black, Pawn, 2, 5),
        (Black, King, 0, 7),
    ]);
    let moves = targets(&g, PieceId(1));
    assert!(moves.contains(&Square::new(0, 1)));
    assert!(!moves.contains(&Square::new(0, 0)));
    assert!(moves.contains(&Square::new(2, 5)));
    assert!(!g.is_in_check(Color::Black).unwrap());
    g.apply_action(mv(4, 4, 2, 5)).unwrap();
    assert!(g.piece(PieceId(4)).is_none());
}
#[test]
fn fanatic_captures_forward_up_to_two_and_does_not_promote() {
    let mut g = game(&[
        (White, Fanatic, 2, 4),
        (Black, Rook, 0, 4),
        (Black, King, 0, 7),
    ]);
    assert!(g.legal_actions().unwrap().contains(&mv(2, 4, 0, 4)));
    g.apply_action(mv(2, 4, 0, 4)).unwrap();
    assert!(g.snapshot().pending_promotion.is_none());
    assert_eq!(g.piece(PieceId(1)).unwrap().kind, Fanatic);
    let g = game(&[
        (White, Fanatic, 4, 4),
        (White, Pawn, 3, 4),
        (Black, King, 0, 7),
    ]);
    assert!(targets(&g, PieceId(1)).is_empty());
}
#[test]
fn enemy_pending_spell_on_current_king_prevents_castling() {
    for anchor in [Square::new(7, 4), Square::new(7, 5)] {
        let mut s = game(&[
            (White, King, 7, 4),
            (White, Rook, 7, 7),
            (Black, King, 0, 7),
        ])
        .snapshot();
        s.delayed_spells.push(DelayedSpell {
            spell: DelayedSpellKind::Lightning,
            anchor,
            owner: Color::Black,
            caster: None,
        });
        let g = GameState::from_snapshot(s).unwrap();
        assert_eq!(
            g.legal_actions().unwrap().contains(&mv(7, 4, 7, 6)),
            anchor.col == 5
        );
    }
}

#[test]
fn big_rook_castles_on_both_sides_for_both_colors() {
    for owner in [White, Black] {
        for king_side in [false, true] {
            let row = if owner == White { 7 } else { 0 };
            let anchor_row = if owner == White { 6 } else { 0 };
            let corner = if king_side { 6 } else { 0 };
            let end = if king_side { 6 } else { 2 };
            let mut s = game(&[
                (owner, King, row, 4),
                (owner, BigRook, anchor_row, corner),
                (
                    if owner == White { Black } else { White },
                    King,
                    if owner == White { 0 } else { 7 },
                    4,
                ),
            ])
            .snapshot();
            s.turn.side = if owner == White {
                Color::White
            } else {
                Color::Black
            };
            s.pieces[1].hp = Some(2);
            s.pieces[1].max_hp = Some(2);
            s.pieces[1].footprint = vec![
                Square::new(anchor_row, corner),
                Square::new(anchor_row, corner + 1),
                Square::new(anchor_row + 1, corner),
                Square::new(anchor_row + 1, corner + 1),
            ];
            s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
            let mut g = GameState::from_snapshot(s).unwrap();
            g.apply_action(mv(row, 4, row, end)).unwrap();
            assert_eq!(g.piece(PieceId(1)).unwrap().anchor, Square::new(row, end));
            assert_eq!(
                g.piece(PieceId(2)).unwrap().anchor,
                Square::new(anchor_row, if king_side { 4 } else { 3 })
            );
            assert_eq!(g.piece(PieceId(2)).unwrap().footprint.len(), 4);
            assert!(g.piece(PieceId(2)).unwrap().moved);
        }
    }
}
#[test]
fn big_rook_castle_erases_friends_without_capture_mana_or_progress() {
    let mut s = game(&[
        (White, King, 7, 4),
        (White, BigRook, 6, 6),
        (White, Pawn, 6, 4),
        (White, Wizard, 4, 0),
        (Black, King, 0, 4),
    ])
    .snapshot();
    s.pieces[1].hp = Some(2);
    s.pieces[1].max_hp = Some(2);
    s.pieces[1].footprint = vec![
        Square::new(6, 6),
        Square::new(6, 7),
        Square::new(7, 6),
        Square::new(7, 7),
    ];
    s.pieces[3].mana = Some(0);
    s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
    s.deathmatch = Some(Deathmatch {
        started_at_turn: 45,
        half_turns_since_progress: 4,
        interval_half_turns: 20,
        progress_this_turn: false,
    });
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(7, 4, 7, 6)).unwrap();
    assert!(g.piece(PieceId(3)).is_none());
    assert_eq!(g.piece(PieceId(4)).unwrap().mana, Some(0));
    assert!(!g.snapshot().deathmatch.unwrap().progress_this_turn);
}
