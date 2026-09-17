# Rust 포팅 계획 — Phase 0 산출물

## 현재 진행 상태

2026-09-18: **Phase 3 진행 중**. 단순/조건 행마, 기사단장·기수 aura, 체커 연속 잡기, 종자 변신, 대형 기물·HP/섹터 공격을 구현·검증하고 있다. 현황과 남은 타입은 `PHASE3_REPORT.md`에 기록한다. Phase 3 전체 완료를 뜻하지 않는다.

2026-09-18: Phase 2 카드 없는 기본 규칙 완료. `engine/`에서 행마·착수·프로모션·턴·종료를 실행하며 canonical v2를 사용한다. Rust 테스트 27개, JS와 고정 68개 시나리오 및 100 seeds 무작위 대국(총 10,560개 전이, 10,728개 상태 비교)이 통과했다. 대응/검증/미포팅 경계는 `PHASE2_REPORT.md`, `analysis/PHASE2_STATE_MAPPING.md`에 기록했다. 다음 단계는 Phase 3 변형 기물이다. Phase 1 기록과 아래 Phase 0 분석은 당시 경계에 대한 역사적 기록이다.

## 기준과 범위

기준 원본/해시 및 실제 동작은 `ENGINE_ANALYSIS.md`, 전체 카드·기물·RULE 목록은 `RULE_INVENTORY.md`에 고정했다. 아래는 실행 가능한 작은 작업 단위의 계획이며 구현 완료를 뜻하지 않는다. Phase 0에서 Rust 프로젝트는 만들지 않는다.

첫 호환 target은 이 번들의 최신 로컬 일반 8×8 게임이다. 카드 없음도 JS의 왕 포획·행동 불가 패배·반복/별 판정 규칙을 따른다. 표준 체스 라이브러리로 바꾸는 작업이 아니다. 온라인 v1–v19 profile, chaos/grand, GUN/캠페인/경매, 편집기는 명시적인 후속 compatibility 영역으로 둔다. API는 이 확장을 막지 않아야 하며, 지원하지 않는 설정/카드는 오류를 반환한다. 아무 효과 없이 성공 처리하지 않는다.

## Phase 0 완료 및 한계

- [x] 제공된 전체 번들의 구조와 핵심 호출 흐름 조사.
- [x] 세 분석 문서 작성: 상태/Action/순서/규칙/위험/단계별 검증.
- [x] 최종 분류·별점·passive patch를 반영한 241개 `CARD_DEFS` 추출 및 중복 확인.
- [x] 기물 label/이동 분기, 27개 RULE, 상태 효과·특수 행동·승리 조건 목록 작성.
- [x] Math.random 등장 82행과 간접 helper 사용, 상태 참조 인덱스 보존.
- [x] 출처 hash·정적 자료 추출 재현 및 문서 coverage 검사.
- [ ] JS headless 실행/oracle: Phase 1 선행 작업으로 남음.
- [ ] Rust 엔진/게임 규칙 테스트/differential/benchmark: Phase 1–9에서 수행.

ZIP, 외부 import 청크, HTML, Worker 및 서버는 입력에 없다. 앱 실행을 검증했다고 주장하지 않는다. 확보 가능한 순수 함수를 추출해서 oracle을 만들 수 있으며 브라우저 전체 시작 코드를 임의 mock으로 성공시켜 규칙이 검증됐다고 간주하지 않는다.

## 공통 설계 결정

| 대상 | 시작 설계 | 제한/이유 |
|---|---|---|
| GameConfig / RulesProfile | rule/version, game style, draft/bans, deathmatch, board setup, 호환 flags | 실제 기본값과 테스트 config를 명시. UI 옵션을 암묵적 전역으로 읽지 않는다. |
| GameState | Board + EntityStore + Players + Turn + Draft + RuleRuntime + Pending + Outcome + RuleHistory + ChanceState | 292개 검색 필드를 그대로 struct 복사하지 않는다. 포함/제외 이유 매핑표를 먼저 만든다. |
| Board | 동적 rows/cols, Cell→PieceId, terrain, anchor/footprint | 2×2 공유 기물, neutral, 붕괴/포탈 지원. bitboard 최적화는 나중. |
| Entity/카드 | compact ID, typed kind/status, 외부 stable string ID | ID allocator는 RNG와 분리. card category/activation은 독립, RULE은 별도 정의. |
| Turn | actor, completed turns by color, fullMove, moveCount, action budget, continuation | action≠turn. owner-turn / shared-turn / action deadline을 typed 표현. |
| Action | Move(route 포함), Promotion, ActivateCard, DraftChoice, PieceAbility, ResolveChoice, EndOptionalContinuation | 외부 입력은 현재 action contract로 validation. from/to만으로 동치 판정하지 않는다. |
| Decision | Player(color), Chance(request), Terminal | pending 응답권자는 board turn과 다를 수 있다. |
| Event / Effect | 이동·포획·생성·변신·소유자 변경·status·rule·턴·종료 primitive | 핵심 함수 안 card ID 조건 난립 금지. registry가 event/condition/modifier를 등록. |
| 충돌 | phase + priority + registration sequence; prevention/replacement/after 구분 | 첫 범위는 현재 JS 순서를 재현. 무제한 범용 DSL은 만들지 않는다. |
| Native rule | 제한된 EngineContext의 query/emit/effect API | 복잡한 포탈·리플레이 등을 허용하되 내부 state 무제한 변경은 피한다. |
| 확률 | 명시 ChanceRequest/Outcome tape, production seed RNG state/version | 비교 시 같은 seed만으로 JS global RNG 소비 순서를 맞추려 하지 않는다. |
| observation | full canonical state와 color별 관측 분리 | 은신/위장 때문에 policy에 상대 비공개 정보를 무심코 노출하지 않는다. |
| undo | 처음은 clone/apply, 실패 시 원상태 보존 | 미래 undo를 위해 외부 side effect 없이 delta/event 경계를 유지. |

초기 디렉터리는 `engine/src/{lib,state,action,board,rules,cards,serialization}.rs` 정도로 시작하고 실제 책임이 커질 때 나눈다. tests/fixtures와 JS oracle은 엔진 밖에 둔다. 정확성 → 구조 → 성능 순서로 진행한다.

## Canonical v1 설계 작업

Phase 1에서 기계 검증 가능한 schema를 확정할 항목:

1. `schema_version`, `rules_profile`, config, support capabilities.
2. 보드 크기·terrain·칸별 entity ID, ID별 정규 기물 record, origin/anchor/footprint/traits/status/resource. 같은 entity는 한 번 serialize.
3. turn/decision owner/phase, 모든 의미 counter, 강제/선택 연속 행동, 다음 턴 카드 활성화.
4. 양측 카드 슬롯(빈칸 포함), card instance, acquire/use 순서, offer, 선택 이력, draftBalance, 재개 턴.
5. active rules/modifiers 및 효과 queues. queue는 처리 순서 유지, unordered set/map만 정렬.
6. en passant/castling, captures 및 재활용 가용 재료, 반복 counter/salt, deathmatch progress, 왕권/승계, 필요한 rollback 이력.
7. pending choice/request, chance tape cursor 또는 RNG state, 다음 entity/card/effect ID.
8. terminal result는 ongoing과 draw를 구분하며 winner + 안정적인 reason code. 한국어 표시 문구는 외부 매핑.

`undefined`/누락/null의 의미를 필드별로 정하고 default 정규화한다. 별점은 half-star 정수, 값의 부동소수 오차를 피한다. 원본의 랜덤 문자열 ID는 동일한 참조 테이블을 통해 재할당한다. 위치로 매번 ID를 새로 정하면 부활·쌍둥이·예약 효과 참조가 깨진다. 영상 로그/DOM/timer ID/벽시계는 제외하되 `moveReplay`/`firstMoveUndo` 같은 규칙 이력은 남긴다. `positionKey`는 reference 반복 규칙 전용이며 MCTS transposition key로 사용하지 않는다.

## Phase 1 — reference harness와 core skeleton

| 작은 작업 | JS 대응 | 검증/완료 gate |
|---|---|---|
| 1A 입력 의존성 목록 및 로컬 oracle 추출 | bundle imports, resetGame, piece, createInitialBoard | 누락 청크 목록 기록. 추출한 규칙 함수만 실행; UI 이벤트 sink와 timer continuation을 명시. 원본 hash 검사. |
| 1B 상태 필드 매핑 | resetGame + runtime assignments + snapshots | 정적 292필드에 규칙/표시/파생/미확인 분류; alias/dynamic writes 추가 감사. unknown 필드 손실을 잡는 fixture. |
| 1C Rust 타입/불변식 | board, IDs, turns, deck, terminal | 8×8 초기 배치 32 entity, 중복 ID 금지, footprint 일관성, clone 독립성, neutral 처리. |
| 1D serialization/canonical | Set/Map, entity alias, profile | serialize→deserialize 동치, key 순서 안정성, 배열 순서 보존, invalid input 거절. |
| 1E Chance/외부 I/O seam | random helpers, Date.now, UI callbacks | 고정 outcome tape 재생, ID와 UI 난수 분리, input 없는 외부 시간 접근 금지. |
| 1F 공개 API | new/legal_actions/apply_action/is_terminal/result/side_to_move | 잘못된 행동에서 상태 불변, terminal에서 행동 거절, 조회가 state/RNG를 변경하지 않음. |

초기 API는 unsupported 상태에 대한 명확한 오류를 허용한다. encode/policy_index/Python/WASM은 아직 만들지 않는다. canonical field audit와 정상적인 JS oracle이 없으면 “differential 통과”라 표시하지 않는다.

## Phase 2 — 카드 없는 기본 규칙

2A pawn/knight/ray/king move + 직접 포획 → 2B 캐슬링/앙파상 → 2C 승격·보류 선택 → 2D 턴·왕권·행동 불가·반복/별/데스매치.

JS 대응: `getLegalMoves`, `pawnMoves`, `rayMoves`, `jumpMoves`, `castleMoves`, `movePieceAttack`, `choosePromotion`, `endMove`, `completeTurnAfterMove`, `resolveRoyalCapture`, `checkNoActionLoss`, `checkRepetitionOrStarLimit`.

필수 fixture: 기본 시작 수, blocker/경계, 왕을 위협에 노출하는 수, 실제 왕 포획, 캐슬링 공격 경로/권리, 앙파상 생성·만료, underpromotion 선택, 움직일 수 없는 쪽의 패배, 별 동률/우열, 45턴 및 데스매치 black tick. 표준 체스 perft는 이 게임의 합법수 정답이 아니다. 최소 전이마다 action set + canonical state 비교가 통과해야 한다.

## Phase 3 — 변형 기물

카드로 획득하는 효과와 기물 자체 행동을 분리해 직접 setup fixture로 시작한다.

1. 단순 delta/ray: man/ferz/alfil/camel/eagle, amazon, knightmaster, windmill.
2. 조건/도약/원거리: cannon/grasshopper/hook/cardinal/protestant/herald, assassin/guard/recruiter, checker 연속 잡기.
3. entity/크기/자원: colossus/bigRook/bigBishop, merchant, wizard, shotgunKing, log.
4. trigger/변신/복제: squire/standardBearer, idol/lobster/bear, missionary/siegeRam/magicGirl/berserker/slime/siren/trickster/undead/thief, paladin/octopus/brutus/clockwork/parrot, campfire/hedgehog/princess.
5. royal/neutral/캠페인: royalKnight/primeMinister/vip/crown, football/monster/blackHole/wall, darkWizard/timeTraveler/vampireLord/bat/coffin 등.

목록 문서의 모든 type/alias를 지원표에 연결한다. 기물마다 이동·포획·금지·보호·왕권·생성 당일 금지·경계·특수 상태 최소 fixture를 둔다. 2×2 entity는 이동 한 번, 포획 한 번, clone 뒤 ID 보존을 확인한다. 공통 effect가 필요한 작은 기능은 Phase 4 primitive를 앞당기되 대규모 카드 switch로 우회하지 않는다.

## Phase 4 — Event/Effect/Rule 실행기

4A Action validation과 atomic commit → 4B capture prevention/replacement/after → 4C spawn/transform/owner/status → 4D turn clocks/예약 queue → 4E dynamic Rule registry와 제한된 native handler.

실제 사례로 추상화를 검증한다: shield vs HP vs forceCapture, 독 폰의 expiry, 패링/곰/트로이 반격, 회귀/언데드 부활, 쌍둥이 이동, 포탈 경로, 허수아비 강제 잡기. queue 중복/사이클은 진단 오류로 보고하며 임의 깊이에서 규칙을 조용히 생략하지 않는다. winner 결정 후 추가 mutation 중단, 동시 제거는 batch 결과로 판정한다. Event 순서와 state diff를 fixtures에서 함께 비교한다.

## Phase 5 — 카드 배치별 이식

| 배치 | 카드/JS 예시 | 완료 gate |
|---|---|---|
| 5A 단순 modifier passive | sprint, retreat, early-promotion, frontline-response | 획득·등록·지속 효과, 여러 카드 합성, next-turn delay. |
| 5B 단순 active | shield/status/transform, armistice, poisoned-pawn | 타깃 validation, 실패 원자성, used/독점/턴 소모. |
| 5C OPENING | forceFirstMoveCardsAfterMove/forceFirstMoveCard | OPENING passive 획득 즉시 적용과 공식 규칙 충돌을 명시적으로 결정; active 첫 이동 직후, 실패 rollback, random auto target, 추가 행동과 first-turn 경계. |
| 5D PIECE | summon-colossus, 변형 카드들 | 공통 fresh capture lock 및 예외, 대형 공간 검사/왕 깔림. |
| 5E trigger/승리 | conscription, recurrence, undead, gomoku, highlander | 우선순위·동시 제거·예약 시계·terminal interruption. |
| 5F 복합/규칙 변경 | portal-gun, premove, trolley, rule-ticket, replay, quantum, boxes | pending actor, 카드 연쇄/추가 RULE, rollback 최소 이력, chance trace. |
| 5G 특수 모드 | GUN, TIME/BLOOD/HINT, chaos/grand 연동 | 별도 capabilities/profile fixture; 일반 지원 완료 수에 섞지 않음. |

각 card ID는 `미착수/구현/fixture/JS differential/상호작용 검증` 상태와 근거를 가진다. 213개 일반 카드와 나머지 RULE/GUN의 총계를 별도로 관리한다. 문구만 보고 구현하지 않고 effect handler·helper·획득 경로·turn hook을 함께 읽는다. 매 배치마다 미포팅 ID와 버그 결정 기록을 갱신한다.

## Phase 6 — Draft

6A 일반 첫/10/20턴 제시 및 양측 선택 → 6B pool eligibility/bans/중복·상호배타/가중치·균형 보정 → 6C next-turn passive/active 활성화 및 원 turn 복귀 → 6D completeRandom → 6E chaos/grand.

기존 함수: `drawPhaseChoicesRaw`, `draftPoolForCategories`, `draftCardWeight`, `balanceDraftChoices`, `finishDraftSelection`, `completeDraftStep`.

고정 chance tape로 offer 자체부터 비교한다. 기본 중간 후보가 MIDDLE 2 + PIECE 1, pool 부족 fallback, 빈 pool skip, 잘못된 인스턴스 선택 실패, 동등 별 분포, draft 없이 시작, 9→10/19→20 경계, 추가 이동 도중 draft 지연을 검증한다. completeRandom은 제약조건이 있는 shuffle임을 보존한다. 통계 검사는 정확한 단일 전이 검증 이후 보조적으로 한다.

## Phase 7 — Match RULE

27개 전체 ID를 별도 registry로 옮긴다. 시작 40%/직접 선택 후보/없음/bans 및 초기 actor를 먼저 구현한다. 다음으로 배치 변경(960/344200/diagonal/N^30), 이동 제한/가속, 보드 환경(포탈/붕괴/폭탄/컨베이어/겨울), 포획 변경, 별/목표 승리를 순서대로 이식한다.

RULE 티켓의 턴 시작 적용과 추가 RULE 목록을 포함한다. 시작만 검사해서 끝내지 않는다. 동시 효과 예: 겨울+포탈, 가속+첫 이동 카드, 붕괴+대형 entity, 왕관+부활, 폭탄+양쪽 왕 제거. 설정/등록 순서/처리 순서가 fixture에 드러나야 한다.

## Phase 8 — Differential 및 random testing

reference 실행기는 UI/AI 정책과 분리한 진짜 규칙 경로를 사용한다. 프로토콜은 state/config + action + chance outcomes → legal actions/next canonical/events/result로 둔다.

- Action set은 정규화된 구조의 집합으로 비교한다. 경로/특수 subtype/프로모션 선택을 제거하지 않는다.
- 하나의 action마다 deterministic outcome을 양쪽에 주고 모든 의미 필드 비교. pending 상태도 비교한다.
- 종료 없는 효과 cycle, 빈 action set, same-turn 연속 행동을 별도 검증한다.
- 실패 보고: source/profile/schema 버전, seed, initial state, action history, chance tape, JS/Rust legal sets/states, first divergent JSON path. 최소 반례로 축소한다.
- 초기 smoke는 100 seeds×최대 200 decisions, 배치별 확대 후 최종 목표 10,000 seeds×최대 1,000 decisions. 숫자는 향후 실행 목표이며 현재 달성한 수가 아니다.
- 한도 도달은 draw가 아니라 테스트 중단으로 기록. 선택 노드와 chance 노드, board ply 수를 따로 센다.
- source 동작이 의심스러우면 Observed JS / Expected / Likely bug / Rust decision 형식으로 기록하고 compatibility 모드 또는 명세 수정으로 처리한다.

보조 invariant: 보드↔entity 양방향 일치, 존재하지 않는 pending target 처리 계약, legal query 비변이성, 같은 입력 재현, clone 분기 독립성, 적용 실패 시 상태 동일. 카드 효과를 흉내만 내는 테스트 대신 실제 경계/상호작용을 검증한다.

## Phase 9 — Benchmark 및 학습 연결 준비

정확성 gate 통과 후 release build에서 초기 상태 생성, legal action, apply, clone, canonical, random playout을 측정한다. 일반/대형 기물/다중 effect/드래프트 상태를 fixture로 고정하고 seed/profile/CPU/compiler를 기록한다. transitions/sec와 allocation/profile을 보고 병목을 찾는다. 그 후에만 compact storage/undo를 검토한다.

`encode`는 관측 가능한 정보와 전체 상태를 구분하고, `policy_index`는 고정된 action 계약의 충돌/역변환을 검증한 뒤 도입한다. 숨은 정보와 chance가 있는 모드에서 순수 완전정보 AlphaZero를 그대로 적용할지 별도 결정한다. Python/WASM binding은 core 안정화 이후다.

## 다음 구현 착수 조건

Phase 2A–2D와 JS 실행 differential을 완료했다. 다음 작업은 Phase 3의 변형 기물이다. 현재 cardless canonical/differential 경계를 유지하면서 기물별 원본 generator/전이 및 필요한 entity 속성을 순차 추가하고 각 기물에 테스트를 작성한다. 현재 결과를 카드·RULE·온라인 프로필까지의 전체 게임 호환성으로 확대 해석하지 않는다.

각 Phase 2 상태 전이에 필요한 의미 필드는 audit의 deferred 항목에서 꺼내 canonical schema에 추가하고 기존 상태의 migration 또는 버전 거절 계약을 갱신한다. 매 단계의 완료 보고에는 구현한 ID/JS 대응/실행한 테스트/미포팅 목록/버그·모호성 결정을 함께 남긴다.
