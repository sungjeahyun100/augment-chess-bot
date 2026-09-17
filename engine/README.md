# Augment Chess Rust core — Phase 1

브라우저 없이 동작하는 상태 모델과 canonical/replay 경계다. 기준은 `../PORT_RUST_ENGINE.md`와 `../PORTING_PLAN.md`이며, 완료 기록은 `../PHASE1_REPORT.md`에 있다.

```sh
# 저장소 루트에서
cargo test --offline --workspace
cargo clippy --offline --workspace --all-targets -- -D warnings
cargo run --offline --example initial
node tools/phase1_oracle.cjs
python3 tools/phase1_field_audit.py --check
```

의존성이 캐시에 없으면 처음에는 `--offline`을 빼고 Cargo.lock에 고정된 패키지를 받는다. Node oracle은 npm 의존성이 없다.

```rust
use augment_chess_engine::{GameConfig, GameState};

let state = GameState::new(GameConfig::cardless(), 42)?;
let saved = state.to_canonical_json()?;
let restored = GameState::from_canonical_json(&saved)?;
assert_eq!(state, restored);
# Ok::<(), augment_chess_engine::EngineError>(())
```

`GameConfig::cardless()`는 드래프트와 시작 RULE을 명시적으로 끈 초기 배치다. 사이트의 기본 게임 시작 전체를 실행한 상태가 아니다. Phase 1에서는 ongoing 상태의 `legal_actions()`와 모든 `apply_action()`이 `Unsupported`를 반환한다. 따라서 합법수 미구현을 빈 행동 목록/패배로 해석하지 않는다. terminal 입력은 `legal_actions() = []`, `apply_action() = Terminal`이다. 어떤 실패도 상태를 바꾸지 않는다.

`CanonicalState`는 공개 DTO이고, `GameState::from_snapshot()` 또는 `from_canonical_json()`이 불변식을 검사한다. `GameState` 자체는 임의 역직렬화나 mutable reference를 노출하지 않는다. `snapshot()`은 독립 복사본을 반환한다. DTO 단독 역직렬화는 의미 검증을 대체하지 않는다.

현재 모델은 기본 여섯 기물, 거신병의 다중 칸 entity, neutral wall, HP, moved/shielded, 생성 직후 잡기 금지 deadline을 **표현**한다. 행마나 상태 효과 실행은 아직 구현하지 않았다. 미지 기물/카드/설정은 JSON 또는 capability 경계에서 거절한다. 게임 전체 JS snapshot importer, 카드 registry, draft/RULE 실행기, MCTS/학습 binding은 후속 단계다.

큰 파일을 미리 세분화하거나 더미 Effect/Rule variant를 만들지 않았다. 실제 실행 규칙이 생길 때 `state`/`action` 경계에서 확장한다. UI/DOM/사이트 adapter/벽시계/전역 RNG 의존성은 없다.
