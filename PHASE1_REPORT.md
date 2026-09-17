# Phase 1 완료 기록 — Rust core skeleton

요청의 `POST_RUST_ENGINE.md`는 저장소의 `PORT_RUST_ENGINE.md`를 가리키는 것으로 해석했다. Phase 1의 상태 모델과 `PORTING_PLAN.md` 1A–1F에 따른 초기 reference 경계를 구현했다. 기본 체스 및 전체 게임 호환성은 완료 대상이 아니다.

## 구현과 JS 대응

| 작업 | 구현 / 근거 |
|---|---|
| 1A 제한된 실행 oracle | 원본 SHA-256 검사 후 allowlist 함수와 상수만 Node VM에서 실행. `piece` L52522, `createInitialBoard` L53243, `placeCampaignColossus` L53268, 일반 빈 deck slot 함수, reset literal counter를 fixture로 저장. 원본 변경 시 중단. |
| 1B 상태 필드 audit | 기존 292개 점 표기 필드 + reset 251개 + 동적 키 후보를 합친 295개 분류. `analysis/PHASE1_STATE_MAPPING.md`와 JSON에 대응/유보/제외 이유, dynamic write·alias·전역 resolution context 185곳 기록. `entry.state.toUpperCase`는 false positive로 확인. |
| 1C 모델/불변식 | Rust workspace, GameState/Board/Piece/Player/TurnState/Action, compact entity ID, neutral owner, anchor/footprint, HP/status, ongoing/win/draw. 보드-entity 양방향 일치, ID/allocator, owner, 중복 검사. |
| 1D canonical | 버전과 capability, 고정 profile, strict nested DTO, 정렬된 canonical JSON, validated restore. 빈 슬롯과 ordered 배열 보존. 스키마 범위는 `engine/CANONICAL.md`. |
| 1E chance seam | SplitMix64 v1의 저장 가능한 seed state와 `{candidates,index}` tape/cursor. 재생/중단 재개/오류 원자성. ID RNG/시간/UI I/O와 분리. 실제 게임 chance event 소비는 후속 규칙 구현 때 연결. |
| 1F API | new/side_to_move/decision/result/is_terminal/snapshot/canonical. ongoing의 legal_actions와 apply_action은 명시적인 Unsupported. terminal action 거절, 실패/조회 비변이. |

`GameState::new(GameConfig::cardless(), seed)`는 RULE·드래프트를 끈 8×8 초기 모델이다. JS의 resetGame 전체를 실행하거나 이후 recordPosition/타이머/드래프트를 흉내 내지 않는다. deck 슬롯 수와 기물 생성 순서는 원본과 일치한다. 게임 후반의 모든 의미 필드를 이 Phase 1 스키마로 가져올 수 있다는 의미가 아니다.

## 검증

- `cargo test --offline --workspace`: 13개 통합 테스트. JS 초기 배치/counter/slot projection 비교, 거신병 공유 entity, 독립 clone, neutral/status, canonical golden/roundtrip/정렬, malformed snapshot/미지 필드/중복/범위 오류, 실패한 행동의 원자성, terminal win/draw, chance replay/재개/고정 벡터, unsupported config.
- `cargo clippy --offline --workspace --all-targets -- -D warnings` 및 `cargo fmt --all -- --check`.
- `node tools/phase1_oracle.cjs`: 원본 hash와 checked-in fixture를 재생성 결과와 비교하며 미지 기물 필드 거절 검사. fixture 갱신은 `--write`를 명시해야 한다.
- `python3 tools/phase1_field_audit.py --check`: 분류/참조 인덱스 재현 확인.
- `cargo run --offline --example initial`: 브라우저 없는 실행으로 canonical 초기 상태 출력.

이는 **초기화 projection differential**이다. legal-action set, 착수 전이, 전체 canonical JS/Rust 비교 및 random game differential은 아직 수행할 구현이 없으며 통과했다고 표시하지 않는다.

## 미포팅 및 경계

Phase 2–9의 기본 행마·포획·캐슬링·앙파상 전이·승격·턴/승패 판정, 변형 기물 동작, Event/Effect/Rule 실행기, 모든 카드, 드래프트, RULE, fuzz/benchmark는 남아 있다. 그에 필요한 포획/부활/rollback 이력, pending choices, terrain, effect queues 등도 실제 규칙 이식과 함께 스키마에 추가한다. 미지원 config/행동/기물/필드는 오류를 내며 성공한 척하지 않는다.

누락 import는 `analysis/phase1_oracle_manifest.json`에 기록했다. 누락 청크·HTML·Worker·서버 없이 브라우저 앱 전체를 구동하지 않았다. oracle에 DOM stub이나 가짜 타이머 완료를 제공하지 않았다. UI event sink는 없고 허용 경로 밖 호출은 즉시 실패한다. 필드 audit는 보수적 정적 인덱스이며 전체 alias/dataflow 증명이 아니다. unresolved 또는 deferred 상태가 있는 JS snapshot을 가져오는 기능은 제공하지 않는다.

## JS 모호성/결정

- Observed JS behavior: `piece()`가 global Math.random으로 문자열 ID를 만든다. Expected rule: ID는 동일 entity 참조를 보존하며 게임 난수와 분리해야 한다. Likely bug: 규칙 버그로 판단하지 않으나 replay 소비 순서 결합 위험. Rust decision: 생성 순서의 단조 ID로 정규화하고 게임 chance 상태를 소비하지 않는다.
- Observed JS behavior: 거신병 네 칸이 동일 객체를 참조한다. Expected rule: 하나의 entity다. Likely bug: JSON clone에서 alias가 사라질 위험(Phase 0 기록). Rust decision: 하나의 record와 네 ID 참조로 보존; 이동/포획 효과는 아직 실행하지 않는다.
- Observed JS behavior: resetGame에는 UI/온라인/규칙 상태가 혼재하며 normal deck은 빈 슬롯 세 개다. Expected rule: 의미 상태를 잃지 않아야 한다. Likely bug: 버그로 단정하지 않음. Rust decision: 지원 subset을 capability로 명시하고 미지 필드를 거절한다. 이후 규칙 추가 시 스키마를 확장한다.

이번 단계에서 새로운 게임 규칙 버그를 확정하거나 Phase 0의 OPENING/반복/왕권 모호성을 임의로 수정하지 않았다.
