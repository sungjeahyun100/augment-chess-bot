# Phase 2 — 카드 없는 기본 규칙

요청의 `POST_RUST_ENGINE.md`는 저장소에 없으므로 실제 지침 파일 `PORT_RUST_ENGINE.md`의 Phase 2와 `PORTING_PLAN.md`의 2A–2D를 기준으로 구현했다. 기준은 로컬 bundle `main-DsoigPgV.js`이며 표준 체스의 체크메이트 규칙으로 대체하지 않았다.

## 구현

- 여섯 기본 기물 이동과 직접 포획, blocker/경계 검사, 자기 왕 노출 허용 및 별도 위협 조회.
- 양쪽 캐슬링: moved/취소 권리, 빈 경로, 출발·통과·도착 공격 검사, 룩과 왕 동시 이동.
- 폰의 두 칸 전진, 앙파상 생성·포획·만료. 원본처럼 초기 rank뿐 아니라 미이동 폰의 맨 뒤 rank 두 칸 전진도 보존.
- 프로모션 보류 상태 및 퀸/룩/비숍/나이트 선택. 이동과 선택은 별도 Action이며 선택 완료에 한 턴을 정산.
- 턴 카운터, 첫 이동의 빈 OPENING 처리 플래그, 왕 포획, 행동 불가 패배, 최초 상태를 포함한 3회 반복.
- 별 합계가 낮은 쪽 승리, 동률 무승부; 카드 없는 게임은 0:0. 기본 45 공통 턴 후 10수 데스매치, 흑 완료마다 +2, 폰 이동/포획 progress reset.
- Canonical v2: 종료 설정, pending_promotion, deathmatch 추가. v1은 명시적으로 거절. 지원하지 않는 기물/효과 실행과 잘못된 행동을 거절하고 실패 시 원래 상태를 보존.

책임은 `movement.rs`(행마/위협), `transition.rs`(검증된 착수 적용), `victory.rs`(턴/종료)로 나눴다. GameState 공개 API로 생성·legal_actions·apply_action·clone·canonical replay·terminal/result를 사용할 수 있다. 구현하지 않은 카드 이름 분기나 범용 Effect DSL을 미리 추가하지 않았다. Event/Effect/Rule은 Phase 4에서 실제 요구에 맞춰 구축한다.

JS 함수 및 상태별 대응은 `analysis/PHASE2_STATE_MAPPING.md`, 추출과 UI/clock 교체 경계는 `analysis/phase2_oracle_manifest.json`에 기록했다.

## 검증

- Rust 테스트 27개: 기존 13개 + Phase 2 14개. 실제 JS 별 우열 fixture, replay/clone, 입력 검증 및 실패 원자성 포함.
- `cargo fmt --all -- --check`, `cargo clippy --offline --workspace --all-targets -- -D warnings`, `git diff --check` 통과.
- Phase 1 JS 초기 배치/공유 entity oracle 및 field audit 유지 통과.
- 고정 시나리오 68개, 58개 행동 전이: 기본 배치, 앙파상과 만료, 양쪽 캐슬링 및 공격 경로, 양쪽 네 프로모션, 자기 체크와 왕 이동/포획, 행동 불가, 반복, 45턴, 데스매치, 여섯 기물의 blocker와 fresh status, 연장 중 포획. 126개 상태에서 canonical/action set/왕 위협을 함께 비교.
- 무작위 대국 seeds 1…100, 최대 200 decisions: 10,502개 무작위 행동 전이 통과. 고정 시나리오를 합쳐 **10,560개 전이 / 10,728개 상태 비교**, 불일치 0. JS VM을 대국 전체에서 유지해 Rust와 독립적으로 진행했다.
- 별도로 매 수 canonical snapshot을 재주입하는 방식도 100 seeds를 검증하여 같은 10,502개 무작위 전이가 일치했다. 이 실행은 초기 고정 시나리오 41개를 포함하여 총 10,557개 전이 / 10,698개 상태를 비교했다.
- 최종 수치와 고정 시나리오 목록은 `analysis/phase2_differential_report.json`에 저장했다.

재실행:

```sh
cargo test --offline --workspace
cargo build --offline --example oracle_bridge
node tools/phase2_oracle.cjs --check-fixtures
node tools/phase2_differential.cjs
```

기본은 seeds 1…100, 대국당 최대 200 decisions다. `PHASE2_SEEDS=0`은 고정 시나리오만 실행한다. 비교 실패 시 `analysis/phase2_failure.json`에 초기 상태·seed·행동 이력·현재 입력·양측 출력·최초 불일치 필드를 저장한다. `node tools/phase2_differential.cjs --replay analysis/phase2_failure.json`으로 재현한다. 브라우저나 npm 의존성 없이 JS VM과 Rust JSONL bridge를 사용한다.

## 발견한 원본 동작과 결정

| Observed JS behavior | Expected rule / Likely bug | Rust decision |
|---|---|---|
| 자기 왕을 노출하는 수, 공격받는 칸으로 왕 이동 가능 | 사이트의 경고 UX이며 표준 체스와 다름. 버그로 단정하지 않음 | 그대로 허용; 캐슬링만 공격 검사 |
| 초기 rank와 맨 뒤 rank의 미이동 폰 모두 두 칸 전진 가능 | 표준 체스보다 넓은 조건; 편집/생성 기물 의도 가능 | 원본 보존 및 테스트 |
| 반복 키가 moved/EP/ID를 생략 | 서로 다른 미래 행동이 같은 key가 될 수 있는 잠재적 결함 | JS key 보존, 완전 상태 직렬화와 구분 |
| 왕 포획 후 착수자의 moved/EP/턴 정산 전에 반환 | 종료 상태 bookkeeping 비대칭. 이미 terminal이므로 이후 수에는 영향 없음 | canonical에서도 원본 순서 보존 |
| deathmatch는 흑 완료에만 +2, 백 progress는 흑 완료까지 남음 | halfTurns 변수명만으로 매 Action 증가라고 해석하면 틀림 | 원본 정산 순서 보존 |
| 프로모션이 origin과 freshNoCaptureUntil도 변경 | 단순 type 교체 이상의 상태 전이 | origin/deadline 저장, replay 검증 |

## 후속 범위

변형 기물은 Phase 3, Event/Effect/Rule 공통 primitive는 Phase 4, 카드/드래프트/RULE은 Phase 5–7이다. 온라인/chaos/grand/캠페인/시간패/사이트 adapter 및 MCTS binding은 구현하지 않았다. Phase 8의 10,000 seeds 규모 fuzz와 Phase 9 성능 측정도 이번 완료 주장에 포함하지 않는다. 현재 비교는 카드 없는 profile의 의미 상태이며 arbitrary JS snapshot importer가 아니다.
