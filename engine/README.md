# Augment Chess Rust core — Phase 2

카드 없는 일반 8×8 증강체스를 브라우저 없이 실행한다. 기준은 `PORT_RUST_ENGINE.md`와 로컬 JS bundle이며, 구현 대응과 검증 범위는 `PHASE2_REPORT.md`에 기록한다.

```sh
# 저장소 루트에서
cargo test --offline --workspace
cargo clippy --offline --workspace --all-targets -- -D warnings
cargo run --offline --example initial
node tools/phase1_oracle.cjs
python3 tools/phase1_field_audit.py --check
cargo build --offline --example oracle_bridge
node tools/phase2_differential.cjs
# 빠른 고정 시나리오만 실행
PHASE2_SEEDS=0 node tools/phase2_differential.cjs
```

Cargo 의존성이 캐시에 없으면 최초 설치 시 `--offline`을 뺀다. JS 비교 도구는 Node.js 18+만 사용하며 npm 설치가 필요 없다. 기본 무작위 비교는 100개 시드, 최대 200 decisions다. `PHASE2_SEEDS`, `PHASE2_DECISIONS`로 조절하고, 불일치는 `analysis/phase2_failure.json`에 시드·행동 이력·양쪽 상태·최초 불일치 필드로 저장한다. `node tools/phase2_differential.cjs --replay analysis/phase2_failure.json`으로 재현한다.

```rust
use augment_chess_engine::{GameConfig, GameState};

let mut state = GameState::new(GameConfig::cardless(), 42)?;
let actions = state.legal_actions()?;
state.apply_action(actions[0].clone())?;
let saved = state.to_canonical_json()?;
let restored = GameState::from_canonical_json(&saved)?;
assert_eq!(state, restored);
# Ok::<(), augment_chess_engine::EngineError>(())
```

일반 여섯 기물의 이동/포획, 캐슬링, 앙파상, 선택형 프로모션, 턴, 왕 포획/행동 불가 패배, 3회 반복과 장기전 판정 및 데스매치를 지원한다. 이 게임은 표준 체스와 다르다. 자기 왕을 노출하는 수와 공격받는 칸으로 왕을 움직이는 수를 허용한다. `is_in_check(color)`는 위협 정보이고, 캐슬링의 출발·통과·도착 칸만 공격 검사를 강제한다. 체크메이트/스테일메이트/50수/기물 부족 판정은 없다.

`GameConfig::cardless()`는 드래프트와 RULE을 끈다. 별 판정은 빈 덱의 0:0 무승부이며 기본 45 공통 턴 후 10수 데스매치가 적용된다. `star_win_limit`, `deathmatch_enabled`, `deathmatch_limit_turns`로 설정한다. 덱의 별 우열 판정은 `star_tiebreak` 함수로 분리되어 있다.

프로모션 칸으로 이동하면 `pending_promotion`을 저장하고 턴을 유지한다. `legal_actions()`는 네 가지 `Action::Promote`만 반환한다. 선택이 끝나면 한 턴을 완료한다. 기본 이동의 `route`는 빈 배열이어야 한다. 잘못된 행동이나 카운터 overflow는 원래 상태를 바꾸지 않는다. terminal에서 legal actions는 빈 목록, apply는 `Terminal` 오류다.

`CanonicalState`는 공개 DTO다. `GameState::from_snapshot()` 또는 `from_canonical_json()`으로 검증하며, `snapshot()`은 독립 복사본이다. v2 스키마는 v1을 명시적으로 거절한다. 거신병·벽·HP·shield 등 기존 상태 표현은 유지하지만 그 상태에서 규칙 실행은 `Unsupported`다. 카드·변형 기물·드래프트·RULE·온라인 프로필·사이트 adapter는 후속 단계다.
