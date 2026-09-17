use augment_chess_engine::*;
use serde_json::{Value, json};

fn game() -> GameState {
    GameState::new(GameConfig::cardless(), 0).unwrap()
}
fn move_action() -> Action {
    Action::Move {
        from: Square::new(6, 4),
        to: Square::new(4, 4),
        route: vec![],
    }
}

#[test]
fn initial_state_matches_executed_js_projection() {
    let expected: Value = serde_json::from_str(include_str!("fixtures/js_initial.json")).unwrap();
    let g = game();
    let s = g.snapshot();
    assert_eq!(serde_json::to_value(&s.board).unwrap(), expected["board"]);
    assert_eq!(serde_json::to_value(&s.pieces).unwrap(), expected["pieces"]);
    assert_eq!(json!(s.turn.side), expected["reset"]["turn"]);
    assert_eq!(json!(s.turn.completed), expected["reset"]["turnsTaken"]);
    assert_eq!(json!(s.turn.full_move), expected["reset"]["fullMove"]);
    assert_eq!(json!(s.turn.move_count), expected["reset"]["moveCount"]);
    assert_eq!(
        json!(s.turn.actions_remaining),
        expected["reset"]["actionsRemaining"]
    );
    for (color, key) in [(Color::White, "white"), (Color::Black, "black")] {
        let p = g.player(color);
        assert_eq!(json!(p.card_slots), expected["deck_slots"][key]);
        assert_eq!(
            json!(p.cards_used_this_turn),
            expected["reset"]["cardsUsedThisTurn"][key]
        );
        assert_eq!(
            json!(p.first_move_cards_forced),
            expected["reset"]["firstMoveCardsForced"][key]
        );
    }
    assert_eq!(g.pieces().len(), 32);
    assert_eq!(s.ids.next_piece, 33);
    assert_eq!(s.chance, ChanceState::seeded(0)); // 32 ID allocations consume no rules RNG.
}

#[test]
fn shared_colossus_is_one_entity_and_clone_is_independent() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/js_colossus.json")).unwrap();
    let mut s = game().snapshot();
    s.board = serde_json::from_value(fixture["board"].clone()).unwrap();
    s.pieces = serde_json::from_value(fixture["pieces"].clone()).unwrap();
    s.ids.next_piece = 2;
    let parent = GameState::from_snapshot(s).unwrap();
    assert_eq!(parent.pieces().len(), 1);
    for square in [
        Square::new(2, 3),
        Square::new(2, 4),
        Square::new(3, 3),
        Square::new(3, 4),
    ] {
        assert_eq!(parent.board().at(square).unwrap(), Some(PieceId(1)));
    }
    let child = parent.clone();
    let mut changed = child.snapshot();
    changed.pieces[0].hp = Some(2);
    let changed = GameState::from_snapshot(changed).unwrap();
    assert_eq!(changed.pieces()[0].hp, Some(2));
    assert_eq!(parent.pieces()[0].hp, Some(3));
    assert_eq!(child, parent);
}

#[test]
fn canonical_roundtrip_and_golden_bytes() {
    let g = game();
    let text = g.to_canonical_json().unwrap();
    assert_eq!(
        text,
        include_str!("fixtures/initial.canonical.json").trim_end()
    );
    assert_eq!(GameState::from_canonical_json(&text).unwrap(), g);
    assert_eq!(game().to_canonical_json().unwrap(), text);
    let mut s = g.snapshot();
    s.pieces.reverse();
    assert_eq!(
        GameState::from_snapshot(s)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
        text
    );
}

#[test]
fn sets_normalize_and_semantic_array_order_is_preserved() {
    let mut s = game().snapshot();
    s.history.position_counts = vec![
        RepetitionEntry {
            key: "z".into(),
            count: 2,
        },
        RepetitionEntry {
            key: "a".into(),
            count: 1,
        },
    ];
    s.chance = ChanceState::tape(vec![
        ChanceOutcome {
            candidates: 3,
            index: 2,
        },
        ChanceOutcome {
            candidates: 3,
            index: 0,
        },
    ])
    .unwrap();
    let a = GameState::from_snapshot(s.clone()).unwrap();
    s.history.position_counts.reverse();
    let b = GameState::from_snapshot(s).unwrap();
    assert_eq!(
        a.to_canonical_json().unwrap(),
        b.to_canonical_json().unwrap()
    );
    let mut reversed = a.snapshot();
    if let ChanceState::TapeV1 { outcomes, .. } = &mut reversed.chance {
        outcomes.reverse();
    }
    assert_ne!(GameState::from_snapshot(reversed).unwrap(), a);
    assert_eq!(
        GameState::from_canonical_json(&a.to_canonical_json().unwrap()).unwrap(),
        a
    );
    let action = Action::ActivateCard {
        card: CardInstanceId(9),
        targets: vec![
            Target::Square {
                square: Square::new(1, 2),
            },
            Target::Square {
                square: Square::new(2, 1),
            },
        ],
    };
    assert_eq!(
        serde_json::from_str::<Action>(&serde_json::to_string(&action).unwrap()).unwrap(),
        action
    );
}

#[test]
fn malformed_external_states_are_rejected() {
    let base: Value = serde_json::from_str(&game().to_canonical_json().unwrap()).unwrap();
    let cases: Vec<(&str, Value)> = vec![
        ("/schema_version", json!(999)),
        ("/config/rules_profile", json!("online_v1")),
        ("/board/rows", json!(0)),
        ("/board/cols", json!(65)),
        ("/board/cells", json!([])),
        ("/board/cells/0", json!(999)),
        ("/pieces/0/id", json!(0)),
        ("/pieces/1/id", json!(1)),
        ("/pieces/0/anchor/row", json!(90)),
        ("/pieces/0/footprint", json!([])),
        ("/pieces/0/owner", json!("red")),
        ("/pieces/0/kind", json!("future_piece")),
        ("/pieces/0/hp", json!(2)),
        ("/ids/next_piece", json!(32)),
        ("/ids/next_effect", json!(0)),
        ("/turn/full_move", json!(0)),
        ("/players/white/color", json!("black")),
        ("/players/white/card_slots/0", json!(7)),
        ("/phase", json!({"kind":"terminal"})),
        (
            "/turn/continuation",
            json!({"kind":"extra_move","piece":999,"optional":true}),
        ),
        (
            "/history/en_passant",
            json!({"pawn":1,"target":{"row":2,"col":0},"available_to":"white"}),
        ),
        (
            "/chance",
            json!({"algorithm":"tape_v1","cursor":1,"outcomes":[]}),
        ),
    ];
    for (pointer, value) in cases {
        let mut bad = base.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(
            GameState::from_canonical_json(&bad.to_string()).is_err(),
            "accepted {pointer}"
        );
    }
    let mut duplicate_footprint = base.clone();
    duplicate_footprint["pieces"][0]["footprint"] = json!([{"row":0,"col":0},{"row":0,"col":0}]);
    assert!(GameState::from_canonical_json(&duplicate_footprint.to_string()).is_err());
    for pointer in [
        "",
        "/config",
        "/pieces/0",
        "/players/white",
        "/turn",
        "/history",
        "/chance",
    ] {
        let mut bad = base.clone();
        bad.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("new_rule_field".into(), json!(true));
        assert!(
            GameState::from_canonical_json(&bad.to_string()).is_err(),
            "unknown field at {pointer}"
        );
    }
    let text = game().to_canonical_json().unwrap();
    let duplicated = text.replacen(
        "\"schema_version\":2",
        "\"schema_version\":2,\"schema_version\":2",
        1,
    );
    assert!(GameState::from_canonical_json(&duplicated).is_err());
    assert!(GameState::from_canonical_json("{}").is_err());
    assert!(GameState::from_canonical_json(&(text + " null")).is_err());
}

#[test]
fn neutral_entities_and_capture_deadline_roundtrip() {
    let mut s = game().snapshot();
    let mut wall = Piece::new(
        PieceId(33),
        Owner::Neutral,
        PieceKind::Wall,
        Square::new(3, 3),
    );
    wall.origin = None;
    s.pieces.push(wall);
    s.pieces[1]
        .statuses
        .push(Status::CannotCaptureUntilOwnerTurn {
            owner: Color::White,
            completed_turn: 1,
        });
    s.ids.next_piece = 34;
    s.board = Board::from_pieces(8, 8, &s.pieces).unwrap();
    let g = GameState::from_snapshot(s).unwrap();
    assert_eq!(
        GameState::from_canonical_json(&g.to_canonical_json().unwrap()).unwrap(),
        g
    );
    assert_eq!(g.piece(PieceId(33)).unwrap().owner, Owner::Neutral);
}

#[test]
fn queries_and_all_rejected_actions_are_non_mutating() {
    let mut g = game();
    let before = g.to_canonical_json().unwrap();
    assert_eq!(g.legal_actions().unwrap().len(), 20);
    assert!(!g.is_terminal());
    assert_eq!(g.result(), None);
    assert_eq!(g.side_to_move(), Color::White);
    assert_eq!(
        g.decision(),
        Decision::Player {
            color: Color::White
        }
    );
    let actions = vec![
        Action::Move {
            from: Square::new(6, 4),
            to: Square::new(3, 4),
            route: vec![],
        },
        Action::Move {
            from: Square::new(90, 0),
            to: Square::new(0, 0),
            route: vec![],
        },
        Action::Promote {
            piece: PieceId(18),
            into: PieceKind::Queen,
        },
        Action::ActivateCard {
            card: CardInstanceId(1),
            targets: vec![],
        },
        Action::ChooseDraftCard {
            offer: 1,
            card: CardInstanceId(1),
        },
        Action::ResolveChoice {
            request: 1,
            option: 0,
        },
        Action::FinishOptionalContinuation,
    ];
    for action in actions {
        assert!(g.apply_action(action).is_err());
        assert_eq!(g.to_canonical_json().unwrap(), before);
    }
    assert!(serde_json::from_str::<Action>(r#"{"kind":"move","from":{"row":1,"col":1},"to":{"row":2,"col":1},"route":[],"force_capture":true}"#).is_err());
}

#[test]
fn terminal_win_and_draw_are_distinct_and_block_actions() {
    for result in [
        GameResult::Win {
            winner: Color::White,
            reason: EndReason::RoyalCapture,
        },
        GameResult::Draw {
            reason: EndReason::RepetitionStars,
        },
    ] {
        let mut s = game().snapshot();
        s.phase = Phase::Terminal;
        s.result = Some(result.clone());
        let mut g = GameState::from_snapshot(s).unwrap();
        assert!(g.is_terminal());
        assert_eq!(g.result(), Some(&result));
        assert_eq!(g.decision(), Decision::Terminal);
        assert_eq!(g.legal_actions().unwrap(), vec![]);
        let before = g.clone();
        assert_eq!(g.apply_action(move_action()), Err(EngineError::Terminal));
        assert_eq!(g, before);
        assert_eq!(
            GameState::from_canonical_json(&g.to_canonical_json().unwrap()).unwrap(),
            g
        );
    }
}

#[test]
fn chance_tape_replay_resume_and_errors_are_atomic() {
    let mut tape = ChanceState::tape(vec![
        ChanceOutcome {
            candidates: 5,
            index: 4,
        },
        ChanceOutcome {
            candidates: 2,
            index: 0,
        },
    ])
    .unwrap();
    let before = tape.clone();
    assert_eq!(tape.choose(0), Err(EngineError::InvalidChance));
    assert_eq!(tape.choose(3), Err(EngineError::InvalidChance));
    assert_eq!(tape, before);
    assert_eq!(tape.choose(5).unwrap(), 4);
    let mut resumed: ChanceState =
        serde_json::from_str(&serde_json::to_string(&tape).unwrap()).unwrap();
    assert_eq!(resumed.choose(2).unwrap(), 0);
    assert_eq!(tape.choose(2).unwrap(), 0);
    assert_eq!(resumed, tape);
    let exhausted = tape.clone();
    assert_eq!(tape.choose(2), Err(EngineError::ChanceExhausted));
    assert_eq!(tape, exhausted);
    assert!(
        ChanceState::tape(vec![ChanceOutcome {
            candidates: 2,
            index: 2
        }])
        .is_err()
    );
}

#[test]
fn seeded_rng_has_fixed_vectors_and_resumes_exactly() {
    let mut rng = ChanceState::seeded(0);
    // Fixed SplitMix64 seed-zero outputs modulo 2^32-1, independently frozen.
    let actual: Vec<_> = (0..4).map(|_| rng.choose(u32::MAX).unwrap()).collect();
    assert_eq!(
        actual,
        [1_564_374_505, 271_713_375, 2_261_623_399, 1_792_555_669]
    );
    let mut restored: ChanceState =
        serde_json::from_str(&serde_json::to_string(&rng).unwrap()).unwrap();
    for bound in [1, 2, 3, 7, 32, 10000, u32::MAX] {
        assert_eq!(rng.choose(bound), restored.choose(bound));
    }
    assert_eq!(rng, restored);
}

#[test]
fn unsupported_configuration_fails_explicitly() {
    let mut config = GameConfig::cardless();
    config.draft_enabled = true;
    assert!(matches!(
        GameState::new(config, 0),
        Err(EngineError::Unsupported(_))
    ));
    let mut config = GameConfig::cardless();
    config.opening_rules_enabled = true;
    assert!(matches!(
        GameState::new(config, 0),
        Err(EngineError::Unsupported(_))
    ));
}

#[test]
fn board_overlap_dangling_refs_and_duplicate_sets_fail() {
    let mut s = game().snapshot();
    s.pieces[1].footprint = s.pieces[0].footprint.clone();
    s.pieces[1].anchor = s.pieces[0].anchor;
    assert!(Board::from_pieces(8, 8, &s.pieces).is_err());
    let mut s = game().snapshot();
    s.history.position_counts = vec![
        RepetitionEntry {
            key: "same".into(),
            count: 1
        };
        2
    ];
    assert!(GameState::from_snapshot(s).is_err());
    let mut s = game().snapshot();
    s.players.white.card_slots.pop();
    assert!(GameState::from_snapshot(s).is_err());
    let mut s = game().snapshot();
    s.pieces[0].statuses = vec![
        Status::CannotCaptureUntilOwnerTurn {
            owner: Color::Black,
            completed_turn: 1
        };
        2
    ];
    assert!(GameState::from_snapshot(s).is_err());
    let mut s = game().snapshot();
    s.pieces[0].statuses = vec![Status::CannotCaptureUntilOwnerTurn {
        owner: Color::White,
        completed_turn: 1,
    }];
    assert!(GameState::from_snapshot(s).is_err());
}

#[test]
fn noncanonical_object_keys_and_footprints_normalize() {
    let initial = game();
    let text = initial.to_canonical_json().unwrap();
    let reordered = format!(
        "{{\"schema_version\":2,{}",
        text.trim_start_matches('{')
            .replace("\"schema_version\":2,", "")
    );
    assert_eq!(
        GameState::from_canonical_json(&reordered)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
        text
    );
    let fixture: Value = serde_json::from_str(include_str!("fixtures/js_colossus.json")).unwrap();
    let mut s = initial.snapshot();
    s.board = serde_json::from_value(fixture["board"].clone()).unwrap();
    s.pieces = serde_json::from_value(fixture["pieces"].clone()).unwrap();
    let before = GameState::from_snapshot(s.clone()).unwrap();
    s.pieces[0].footprint.reverse();
    assert_eq!(GameState::from_snapshot(s).unwrap(), before);
}
