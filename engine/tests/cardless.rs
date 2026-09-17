use augment_chess_engine::*;
fn new() -> GameState {
    GameState::new(GameConfig::cardless(), 42).unwrap()
}
fn mv(r: u16, c: u16, tr: u16, tc: u16) -> Action {
    Action::Move {
        from: Square::new(r, c),
        to: Square::new(tr, tc),
        route: vec![],
    }
}
fn setup(entries: &[(Color, PieceKind, u16, u16)]) -> GameState {
    let mut s = new().snapshot();
    s.pieces = entries
        .iter()
        .enumerate()
        .map(|(i, &(color, kind, r, c))| {
            Piece::new(PieceId(i as u32 + 1), color.into(), kind, Square::new(r, c))
        })
        .collect();
    s.ids.next_piece = s.pieces.len() as u32 + 1;
    s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
    s.history.position_counts.clear();
    GameState::from_snapshot(s).unwrap()
}
use Color::*;
use PieceKind::*;
#[test]
fn initial_moves_and_replay_preserve_chance() {
    let mut g = new();
    let mut copy = g.clone();
    let chance = g.snapshot().chance;
    assert_eq!(g.legal_actions().unwrap().len(), 20);
    for a in [mv(6, 4, 4, 4), mv(1, 3, 3, 3), mv(4, 4, 3, 3)] {
        g.apply_action(a.clone()).unwrap();
        copy.apply_action(a).unwrap();
        assert_eq!(g, copy);
        assert_eq!(g.snapshot().chance, chance);
        assert_eq!(
            GameState::from_canonical_json(&g.to_canonical_json().unwrap()).unwrap(),
            g
        );
    }
    assert_eq!(g.pieces().len(), 31);
    assert_eq!(g.turn().completed, Sides { white: 2, black: 1 });
    assert!(g.player(White).first_move_cards_forced);
    assert!(g.player(Black).first_move_cards_forced);
}
#[test]
fn blockers_boundaries_and_pawn_back_rank_are_reference_rules() {
    let g = setup(&[
        (White, Rook, 4, 4),
        (White, Pawn, 4, 6),
        (Black, Bishop, 4, 1),
    ]);
    let actions = g.legal_actions().unwrap();
    assert!(actions.contains(&mv(4, 4, 4, 1)));
    assert!(!actions.contains(&mv(4, 4, 4, 0)));
    assert!(actions.contains(&mv(4, 4, 4, 5)));
    assert!(!actions.contains(&mv(4, 4, 4, 6)));
    let g = setup(&[(White, Pawn, 7, 0), (Black, King, 0, 4)]);
    assert!(g.legal_actions().unwrap().contains(&mv(7, 0, 5, 0)));
    let mut s = g.snapshot();
    s.pieces[0].moved = true;
    assert!(
        !GameState::from_snapshot(s)
            .unwrap()
            .legal_actions()
            .unwrap()
            .contains(&mv(7, 0, 5, 0))
    );
}
#[test]
fn threats_are_advisory_and_kings_can_be_captured() {
    let mut g = setup(&[
        (White, King, 7, 4),
        (White, Rook, 6, 4),
        (Black, Rook, 0, 4),
        (Black, King, 0, 0),
    ]);
    assert!(!g.is_in_check(White).unwrap());
    g.apply_action(mv(6, 4, 6, 5)).unwrap();
    assert!(g.is_in_check(White).unwrap());
    assert!(!g.is_terminal());
    let before = g.turn().clone();
    g.apply_action(mv(0, 4, 7, 4)).unwrap();
    assert_eq!(
        g.result(),
        Some(&GameResult::Win {
            winner: Black,
            reason: EndReason::RoyalCapture
        })
    );
    assert_eq!(g.turn(), &before);
    assert!(g.legal_actions().unwrap().is_empty());
    // Reference returns before the capturing piece's moved flag is set.
    assert!(!g.piece(PieceId(3)).unwrap().moved);
}
#[test]
fn castle_moves_rook_and_king_and_checks_path_and_rights() {
    for col in [2, 6] {
        let mut g = setup(&[
            (White, King, 7, 4),
            (White, Rook, 7, 0),
            (White, Rook, 7, 7),
            (Black, King, 0, 4),
        ]);
        let mut canceled = g.snapshot();
        canceled.history.castling_canceled.white = true;
        assert!(
            !GameState::from_snapshot(canceled)
                .unwrap()
                .legal_actions()
                .unwrap()
                .contains(&mv(7, 4, 7, col))
        );
        g.apply_action(mv(7, 4, 7, col)).unwrap();
        assert!(g.snapshot().history.castled.white);
        let rook = if col == 2 { PieceId(2) } else { PieceId(3) };
        assert_eq!(
            g.piece(rook).unwrap().anchor,
            Square::new(7, if col == 2 { 3 } else { 5 })
        );
        assert!(g.piece(rook).unwrap().moved);
    }
    for attacked_col in [4, 5, 6] {
        let g = setup(&[
            (White, King, 7, 4),
            (White, Rook, 7, 7),
            (Black, Rook, 0, attacked_col),
        ]);
        assert!(!g.legal_actions().unwrap().contains(&mv(7, 4, 7, 6)));
    }
}
#[test]
fn en_passant_removes_correct_pawn_and_expires() {
    let mut g = new();
    for a in [
        mv(6, 4, 4, 4),
        mv(1, 0, 2, 0),
        mv(4, 4, 3, 4),
        mv(1, 3, 3, 3),
    ] {
        g.apply_action(a).unwrap();
    }
    let mut expired = g.clone();
    expired.apply_action(mv(7, 6, 5, 5)).unwrap();
    assert!(expired.snapshot().history.en_passant.is_none());
    g.apply_action(mv(3, 4, 2, 3)).unwrap();
    assert_eq!(g.board().at(Square::new(3, 3)).unwrap(), None);
    assert_eq!(g.pieces().len(), 31);
    assert!(g.snapshot().history.en_passant.is_none());
}
#[test]
fn promotion_is_a_serializable_choice_and_finishes_one_turn() {
    for into in [Queen, Rook, Bishop, Knight] {
        let mut g = setup(&[
            (White, Pawn, 1, 0),
            (White, King, 7, 4),
            (Black, King, 0, 7),
        ]);
        g.apply_action(mv(1, 0, 0, 0)).unwrap();
        assert_eq!(g.turn().move_count, 0);
        assert_eq!(g.side_to_move(), White);
        assert_eq!(g.snapshot().pending_promotion, Some(PieceId(1)));
        let choices = g.legal_actions().unwrap();
        assert_eq!(choices.len(), 4);
        let before = g.clone();
        assert!(g.apply_action(mv(7, 4, 6, 4)).is_err());
        assert_eq!(g, before);
        g = GameState::from_canonical_json(&g.to_canonical_json().unwrap()).unwrap();
        g.apply_action(Action::Promote {
            piece: PieceId(1),
            into,
        })
        .unwrap();
        assert_eq!(g.turn().move_count, 1);
        assert_eq!(g.side_to_move(), Black);
        let p = g.piece(PieceId(1)).unwrap();
        assert_eq!(p.kind, into);
        assert_eq!(p.origin, Some(Square::new(0, 0)));
        assert_eq!(
            p.statuses,
            vec![Status::CannotCaptureUntilOwnerTurn {
                owner: White,
                completed_turn: 1
            }]
        );
    }
}
#[test]
fn no_actions_is_a_loss_not_stalemate() {
    let mut g = setup(&[(White, King, 7, 4), (Black, Pawn, 7, 0)]);
    g.apply_action(mv(7, 4, 6, 4)).unwrap();
    assert_eq!(
        g.result(),
        Some(&GameResult::Win {
            winner: White,
            reason: EndReason::NoActions
        })
    );
}
#[test]
fn repetition_includes_initial_position_and_ignores_moved_flags() {
    let mut g = new();
    assert_eq!(g.snapshot().history.position_counts.len(), 1);
    for _ in 0..2 {
        for a in [
            mv(7, 6, 5, 5),
            mv(0, 6, 2, 5),
            mv(5, 5, 7, 6),
            mv(2, 5, 0, 6),
        ] {
            g.apply_action(a).unwrap();
        }
    }
    assert_eq!(
        g.result(),
        Some(&GameResult::Draw {
            reason: EndReason::RepetitionStars
        })
    );
    assert_eq!(g.turn().move_count, 8);
}
#[test]
fn fewer_stars_wins_and_equal_stars_draw() {
    assert_eq!(
        star_tiebreak(Sides { white: 3, black: 4 }, EndReason::RepetitionStars),
        GameResult::Win {
            winner: White,
            reason: EndReason::RepetitionStars
        }
    );
    assert_eq!(
        star_tiebreak(Sides { white: 4, black: 3 }, EndReason::TurnLimitStars),
        GameResult::Win {
            winner: Black,
            reason: EndReason::TurnLimitStars
        }
    );
    assert_eq!(
        star_tiebreak(Sides { white: 0, black: 0 }, EndReason::TurnLimitStars),
        GameResult::Draw {
            reason: EndReason::TurnLimitStars
        }
    );
}
#[test]
fn forty_five_turn_limit_starts_deathmatch_or_draws() {
    for enabled in [false, true] {
        let mut s = new().snapshot();
        s.config.deathmatch_enabled = enabled;
        s.turn.completed = Sides {
            white: 45,
            black: 44,
        };
        s.turn.side = Black;
        let mut g = GameState::from_snapshot(s).unwrap();
        g.apply_action(mv(0, 6, 2, 5)).unwrap();
        if enabled {
            assert_eq!(
                g.snapshot().deathmatch.unwrap(),
                Deathmatch {
                    started_at_turn: 45,
                    half_turns_since_progress: 0,
                    interval_half_turns: 20,
                    progress_this_turn: false
                }
            );
        } else {
            assert_eq!(
                g.result(),
                Some(&GameResult::Draw {
                    reason: EndReason::TurnLimitStars
                })
            );
        }
    }
}
#[test]
fn deathmatch_black_tick_and_progress_order() {
    let mut s = new().snapshot();
    s.deathmatch = Some(Deathmatch {
        started_at_turn: 45,
        half_turns_since_progress: 18,
        interval_half_turns: 20,
        progress_this_turn: false,
    });
    let mut g = GameState::from_snapshot(s.clone()).unwrap();
    g.apply_action(mv(7, 6, 5, 5)).unwrap();
    assert_eq!(
        g.snapshot().deathmatch.unwrap().half_turns_since_progress,
        18
    );
    let before = g.turn().clone();
    g.apply_action(mv(0, 6, 2, 5)).unwrap();
    assert!(g.is_terminal());
    assert_eq!(g.turn().side, Black);
    assert_eq!(g.turn().full_move, before.full_move);
    let mut g = GameState::from_snapshot(s).unwrap();
    g.apply_action(mv(6, 0, 5, 0)).unwrap();
    g.apply_action(mv(0, 6, 2, 5)).unwrap();
    let dm = g.snapshot().deathmatch.unwrap();
    assert_eq!(dm.half_turns_since_progress, 0);
    assert!(!dm.progress_this_turn);
    assert!(!g.is_terminal());
}
#[test]
fn invalid_routes_overflow_and_unsupported_entities_are_atomic() {
    let mut g = new();
    let before = g.clone();
    let mut a = mv(6, 4, 4, 4);
    if let Action::Move { route, .. } = &mut a {
        route.push(Square::new(5, 4));
    }
    assert!(g.apply_action(a).is_err());
    assert_eq!(g, before);
    let mut s = g.snapshot();
    s.turn.move_count = u32::MAX;
    let mut g = GameState::from_snapshot(s).unwrap();
    let before = g.clone();
    assert!(g.apply_action(mv(6, 4, 4, 4)).is_err());
    assert_eq!(g, before);
    let mut s = new().snapshot();
    s.pieces[0].kind = Colossus;
    let mut g = GameState::from_snapshot(s).unwrap();
    let before = g.clone();
    assert!(matches!(
        g.legal_actions(),
        Err(EngineError::Unsupported(_))
    ));
    assert!(g.apply_action(mv(6, 4, 4, 4)).is_err());
    assert_eq!(g, before);
}

#[test]
fn malformed_pending_and_en_passant_states_fail_validation() {
    let mut s = new().snapshot();
    s.pending_promotion = Some(PieceId(18));
    assert!(GameState::from_snapshot(s).is_err());
    let mut g = new();
    g.apply_action(mv(6, 4, 4, 4)).unwrap();
    for target in [Square::new(4, 4), Square::new(5, 3), Square::new(3, 4)] {
        let mut s = g.snapshot();
        s.history.en_passant.as_mut().unwrap().target = target;
        assert!(GameState::from_snapshot(s).is_err());
    }
    let mut s = new().snapshot();
    s.config.deathmatch_limit_turns = u32::MAX;
    assert!(GameState::from_snapshot(s).is_err());
    let mut s = new().snapshot();
    s.deathmatch = Some(Deathmatch {
        started_at_turn: 45,
        half_turns_since_progress: 21,
        interval_half_turns: 20,
        progress_this_turn: false,
    });
    assert!(GameState::from_snapshot(s).is_err());
}

#[test]
fn star_resolution_matches_executed_js_fixtures() {
    #[derive(serde::Deserialize)]
    struct Fixture {
        stars: Sides<u32>,
        result: GameResult,
    }
    let fixtures: Vec<Fixture> =
        serde_json::from_str(include_str!("fixtures/js_star_tiebreak.json")).unwrap();
    for fixture in fixtures {
        assert_eq!(
            star_tiebreak(fixture.stars, EndReason::RepetitionStars),
            fixture.result
        );
    }
}
