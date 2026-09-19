# Phase 3 — 변형 기물 (진행 중)

Phase 3 전체 완료 보고가 아니다. `PORT_RUST_ENGINE.md`와 `PORTING_PLAN.md`의 변형 기물 이식을 순차 진행한다. 기준은 SHA-256 `0dbbad680c6e8e2abcdb6e49817ff486aa2f8ef799a8035bd356baef8a2731ea`인 로컬 JS bundle이다. 카드 획득을 구현한 것으로 간주하지 않고 직접 setup한 기물의 행동을 비교한다.

## 현재 구현

- 기본 delta/ray: man, ferz, alfil, camel, eagle, amazon, windmill.
- knightmaster와 인접 폰 행마 교체 및 생성 턴 aura 잡기 제한.
- assassin의 왕 전용 직선/대각 포획, guard의 비포획·피포획 면역.
- cannon의 screen 필수 이동/공격 및 포 screen/포 포획 금지, grasshopper의 첫 장애물 직후 착지.
- hook의 한 번 꺾는 경로, cardinal의 가장자리/벽 반사, protestant의 장애물을 넘는 1…3칸 대각 도약.
- checker/checkerKing의 강제 잡기, 전체 아군 행동 제한, 연속 잡기와 승격.
- squire의 포획 후 나이트 변신, 마지막 rank 선택형 승격.
- standardBearer와 같은 rank의 일반 폰에 대한 좌우 행마·잡기 aura.
- colossus/bigRook/bigBishop의 공유 2×2 entity 이동·착지 포획·HP. bigRook/bigBishop은 아군도 밟으며 포획 개수 제한은 각각 2/3 entity다.
- colossus의 오른쪽/왼쪽 섹터 공격. 표시용 body 클릭을 제거하고 섹터 단위 Action으로 표현한다.
- berserker의 아군 entity 수(5/9 경계)에 따른 행마, princess의 아군 queen 부재 시 행마, clockwork의 인접 아군 필요 조건.
- herald의 최대 3칸 직선 비포획 이동, jump lock/만료, 행동자 우선 인접 왕 협정 승리.
- recruiter의 비포획 왕 행마와 출발 칸 폰 생성. 단조 증가 ID를 할당하고 rules RNG를 소비하지 않는다.
- merchant의 골드, 자기 턴 시작 보급, 기물별 가격과 매수, 왕 매수 승리. 소유권 변경 시 entity/footprint/HP를 유지한다.
- wizard 보호 주문의 선행 조건: shielded 실행 지원. 일반/앙파상 공격은 보호막만 제거하고 제자리, 체커는 점프하며 보호막을 제거하고 연쇄를 이어간다.
- wizard의 비포획 인접 이동, 아군 포획에 따른 마나(기본 상한 5), 번개/보호/메테오/시간 정지. 지연 주문은 시전자 사망 후에도 남고, 시간 정지는 완료 counter와 지연 주문을 보류한다.
- log의 방향 지정과 자기 행동 종료 때의 자동 이동. 방향 지정 당일에는 굴러가지 않고 이후 owner 완료 턴마다 한 번만 이동한다. 보드 순서, 경계·보호막·가드·HP 정지, 왕 포획을 보존한다.
- shotgunKing의 비포획 인접 이동, HP 왕 판정, 탄약/바라보는 방향, 장전·산탄·저격. 산탄은 아군도 타격하며 entity당 한 번, 저격은 광선의 첫 상대 기물만 대상으로 삼는다.
- pegasus의 빈 칸 전역 이동/나이트 포획, fanatic의 전방 최대 2칸 이동·포획과 승격 없음. 새 상태 필드 없이 기존 이동 규칙으로 구현한다.
- missionary의 대각선 한 칸 이동과 전향. 선교사는 제자리에 남고 대상 entity의 소유권·origin·moved를 갱신하며, 보호막과 자원은 유지한다. 대형 HP는 최대치로 복구하고 왕 전향은 즉시 승리한다.
- jester의 퀸 광선 이동, 왕/상인만 포획하는 제한과 왕 계열에게만 잡히는 면역.
- bat의 카드 없는 낮 행마(상하좌우 최대 2칸). 밤 행마는 blood-moon-night 카드 상태를 이식할 때 연결한다.
- vip의 킹 행마, 체크 대상과 포획 패배. 원본대로 왕 정체성과는 구분하여 암살자·광대·전령의 왕 전용 규칙에는 포함하지 않는다.
- primeMinister의 왕 행마 1~2회 이동·포획. 두 번째 걸음은 빈 중간 칸을 필요로 하며, 여러 경로의 같은 목적지는 한 행동이다.
- royalKnight의 기본 나이트 행마와 왕 포획·위협·협정·매수 판정. royalKnightKing/hillKing 카드 효과는 아직 지원하지 않는다.
- bear의 퀸 행마, 2회 반격 자원, 반격 직후 다음 자기 턴까지의 이동 잠금. 직접 포획·체커·거신병 섹터·마법사·샷건은 즉시 반격하고, 통나무 포획은 직렬화 가능한 대기열로 다음 해당 색 행동 종료까지 보존한다.
- hedgehog의 킹 행마와 같은 2회 반격. 곰과 달리 이동 잠금 중에는 위협도 억제되며 반격 복귀 뒤 종류를 유지한다.
- campfire의 상하좌우 한 칸 비포획 이동과 인접 4칸 아군 보호. 왕 계열은 보호 대상에서 제외하고, 대형 footprint·체커·섹터·통나무·마법·샷건도 같은 현재 보드 aura를 사용한다.
- lobster의 자기 전방 세 칸 이동·포획. 카드가 만드는 지연 소환 예약은 카드 단계에 남기고 직접 배치된 기물 규칙만 지원한다.
- slime의 직교 정확히 3칸 도약과 출발 칸 복제. 복제본은 단조 증가 ID, moved=true, origin=출발 칸과 다음 자기 턴까지의 포획 잠금을 갖는다.
- 지연 마법이 현재 왕 칸에 지정된 경우 캐슬링 금지. 통과/도착 칸에만 지정된 마법은 같은 금지 조건을 적용하지 않는다.
- bigRook 캐슬링: 공유 footprint를 경로에서 제외하고 새 2×2 착지의 아군을 포획 효과 없이 제거한다. 양색·양측을 지원한다.
- wall의 이동 없음·차단·포획 금지 및 cardinal 반사.

현재 원본 대응은 `getLegalMoves`, 각 `*Moves` helper, `pieceAttacksSquare`, `movePieceAttack`, `markTransformedOrigin`, `crownCheckerIfNeeded`, `moveColossus`, `moveBigRook`, `attackColossusSectorAttack`, `damageHealthPiece`다.

## 필요한 시점의 구조 확장

- `PieceKind`에는 실제 구현한 기물만 추가했다. `Piece.windmill_mode`는 풍차를 이식하며 추가했다.
- 체커 연속 잡기에 `Continuation::CheckerCapture`를 추가했다. 일반 카드용 범용 continuation 실행기는 만들지 않았다.
- 거신병 공격에 `Action::AttackSector`를 추가했다. UI 클릭 좌표를 그대로 게임 행동으로 만들지 않았다.
- 기존 Board/footprint/HP 표현을 실제 대형 이동과 공격에 연결했다.
- 변신에 공통으로 필요한 origin/fresh 갱신만 `effects::transform`으로 묶었다. 범용 Effect DSL/카드 registry는 추가하지 않았다.
- Canonical v3, `phase3_cardless`. v1/v2는 거절한다. 기본 비숍 풍차 모드는 생략으로 정규화한다. 상세 계약은 `engine/CANONICAL.md`.

## 검증 도구

`engine/tests/variants.rs`: 기물별 테스트, fresh 제한, blocker, clone/replay, aura, 체커 강제 연쇄, 승격, 대형 entity ID와 HP 공격을 검사한다.

`tools/phase3_oracle.cjs`: Phase 2의 hash 고정 함수 추출기를 공유한다. 실제 JS 함수는 자동 stub하지 않는다. 추가 상수는 HOOK_ORTHOGONAL_DIRECTIONS, hookIcePathsByMove, MAX_NOTATION_TEXT. 거신병의 500ms UI 지연 callback은 즉시 실행하여 실제 endMove를 호출한다. HP 애니메이션/소리 전용 primeStatusMagicLoss와 scheduleStatusMagicSound, 기보 전용 queueHpAttackHistoryNotation는 표시 sink다. 추가 UI 의존성은 확인 후 표시 경계로만 분리한다. 난수는 여전히 발생하면 오류다.

`tools/phase3_differential.cjs`: 양쪽 색, 중앙/경계, 장애물, fresh 상태, 풍차 모드 조합의 합법수·왕 위협·전이를 비교한다. 각 초기 상태에서 해당 기물의 모든 생성 행동을 검사하며, 체커 연쇄/승격/aura/풍차 변경 등은 JS VM을 유지하는 연속 전이로도 검사한다. 전체 실행 보고는 `analysis/phase3_differential_report.json`, 선택 실행은 `phase3_subset_report.json`이다. 실패는 `phase3_failure.json`에 재현 입력과 양측 결과를 저장한다.

```sh
cargo test --offline --workspace
cargo build --offline --example oracle_bridge
node tools/phase3_differential.cjs
PHASE3_KINDS=checker,checkerKing node tools/phase3_differential.cjs
node tools/phase3_differential.cjs --sequences-only
# 실패가 발생한 경우
node tools/phase3_differential.cjs --replay analysis/phase3_failure.json
```

현재 묶음 검증:

- Rust 테스트 114개(기존 27 + Phase 3 87)가 통과했다. fmt, clippy 경고 0 검사 통과.
- 곰의 양색/중앙/경계/장애물/fresh/반격 자원·잠금 조합과 직접·왕·체커·대형 착지·섹터·통나무·마법사·샷건 시나리오는 **1,381개 비교, 1,138개 전이**를 통과했다. 이 중 고정 연속 시나리오는 **397개 비교, 226개 전이**다.
- 고슴도치와 캠프파이어의 양색/경계/장애물/fresh/반격 자원·잠금 조합, 반격·보호·왕 예외·매수 시나리오는 함께 **768개 비교, 496개 전이**를 통과했다. 확장된 전체 고정 시나리오는 **408개 비교, 232개 전이**다.
- 랍스터 조합은 **472개 비교, 271개 전이**, 슬라임의 도약·복제·ID·fresh 상태 조합은 **502개 비교, 300개 전이**를 통과했다. 현재 전체 고정 시나리오는 **414개 비교, 236개 전이**다.
- 전령: 선택 실행 **870개 상태/합법수/위협 비교, 759개 전이** 통과. 징집관과 연속 시나리오: **176개 비교, 131개 전이** 통과. 징집관 생성 fresh status는 위 명세 수정에 따른 의도적 차이로 별도 보정하며 나머지를 비교한다.
- 상인 추가 전 전체 실행: 27개 변형 기물에서 **6,595개 비교, 5,830개 전이** 통과. 이 실행은 징집관 fresh status 수정 전이며 해당 수정은 위 별도 실행으로 검증했다.
- 상인의 골드/색/경계/fresh 조합은 **262개 비교, 97개 전이** 통과. 이후 지원 기물 전체 매수 가격·소유권 변경·골드 보급·왕 매수·매수된 폰의 앙파상 이력을 포함한 연속 시나리오는 **156개 비교, 98개 전이** 통과. 실행 간 겹치는 시나리오가 있으므로 합산하지 않는다.
- 보호막의 일반/왕/HP/풍차/종자/체커/대형 착지/섹터/앙파상 조합 추가 후 연속 실행: **177개 비교, 109개 전이** 통과.
- 마법사 네 주문, 왕/아군/가드/HP/보호막 대상, 시전자 사망, 직접·체커·대형·HP 포획 및 매수의 마나 보급을 포함한 연속 실행은 시간 정지와 지연 마법의 상호작용까지 **235개 비교, 145개 전이** 통과. 별도 마법사 조합 실행은 **9,536개 비교, 9,347개 전이** 통과했다.
- 통나무 기본 조합은 **374개 비교, 259개 전이** 통과. 자동 이동·시간 정지·앙파상 점유·종자의 일반 포획 우선순위를 추가한 연속 실행은 **271개 비교, 166개 전이** 통과했다.
- 샷건 킹의 양색/탄약/경계/fresh 조합: **965개 비교, 763개 전이** 통과. 별도 공격/HP 왕/보호막/아군/장기전 시나리오를 포함한 연속 실행: **292개 비교, 178개 전이** 통과.
- 지연 마법 캐슬링과 신규 타입의 상인 매수까지 포함한 연속 시나리오는 **303개 비교, 184개 전이** 통과했다. pegasus/fanatic 조합 검사 **1,784개 비교, 1,620개 전이**도 통과했다.
- 빅룩 캐슬링의 양색×양측×빈 착지/아군/적군/대형 아군/이동 이력/지연 마법 조합 추가 후 연속 실행: **343개 비교, 196개 전이** 통과.
- Phase 2 기존 68개 시나리오 + 100 seeds(최대 200 decisions)도 재실행하여 **10,728개 상태 비교 / 10,560개 전이**가 일치했다.
- 상인/앙파상 변경 후 기본 68개 시나리오 회귀는 **126개 비교, 58개 전이** 통과(`analysis/phase3_basic_regression_report.json`).
- 이전 실패 fixture replay도 통과했고 Phase 1 oracle/field audit와 Phase 2 oracle manifest 검사를 유지했다.
- 아직 전체 변형 기물 random game fuzz가 아니다. 지원한 타입끼리의 모든 조합을 검증했다는 뜻도 아니다.

## 관측한 동작과 결정

| Observed JS behavior | Expected rule / Likely bug | Rust decision |
|---|---|---|
| fresh 기물은 pieceAttacksSquare에서도 제외 | 이전 Phase 2의 threat 구현에서 누락 | 공통 위협 필터를 수정하고 캐슬링 회귀 테스트 추가 |
| 킹 체커 승격이 fresh 금지를 새로 설정 | 같은 턴의 뒤쪽 추가 잡기가 종료됨. 일반 체커 규칙으로 추측하면 틀림 | 원본 보존; 다음 자기 턴에 뒤로 포획 가능 |
| 종자 변신도 origin뿐 아니라 fresh를 설정 | 단순 kind 변경만으로는 replay 상태 불일치 | 공통 transform primitive 사용 |
| HP 공격은 피해량 인자를 무시하고 항상 1, 치명타여도 공격자는 이동하지 않음 | piecePower 값으로 피해량을 추측하면 틀림 | 원본 보존 |
| 빅룩/빅숍 threat query는 네 점유 칸에서 각각 행마를 계산 | 실제 이동 anchor보다 넓은 위협이 생기는 원본의 잠재 버그 | 호환성 보존, 정상 이동과 위협 query를 분리 |
| 대형 착지 포획은 왕 포획 후에도 forEach가 계속되어 마지막 왕의 endGame이 덮어씀 | 양쪽 왕 동시 포획에서 순서 의존하는 잠재 버그 | 실제 footprint 순서를 명시적으로 보존하여 비교 |
| clockwork는 합법 포획이 있어도 pieceAttacksSquare switch에 없어 threat가 false | 위협 누락으로 보이는 원본 버그 | 합법 포획과 별개로 advisory query는 원본 보존 |
| 징집관의 leaveRecruiterPawn은 markFreshNoCapture를 호출하지 않음 | 명시된 공통 생성 턴 포획 금지 규칙을 누락한 버그 | 기존 fresh status를 적용; JS 비교에서 recruiter_spawn_fresh_status라는 의도적 차이만 별도 보정하고 기록 |
| 종자/기수의 비포획 이동은 deathmatch progress를 설정하지 않음 | 원본 movedAsType===pawn 조건 | 일반 폰에만 progress 설정 |
| 상인 가격은 piecePower가 아닌 ENCYCLOPEDIA_PIECE_VALUES 사용, 퍼즈 가격 없음 | 최소 2골드이나 미등록 타입은 매수 불가 | 별도 가격 함수와 전체 지원 타입 매수 비교 |
| 매수는 enPassant를 지우지 않고 전진했던 색의 다음 이동도 이력을 유지 | 현재 소유자와 전진 당시 색이 다를 수 있음 | available_to로 과거 전진 방향을 검증하고 retainSameTurnEnPassant 보존 |
| 대형 착지 포획은 shielded를 무시하지만 거신병 섹터는 shielded 대상을 제외 | 일반 공격의 보호막 제거와 다른 원본 경로 | 원본 보존; 직접 공격·체커·착지·섹터를 각각 비교 |
| 시간 정지는 마나만 소비하고 턴을 즉시 종료하지 않음; 다음 행동도 completed/moveCount 없이 같은 턴 유지 | 단순히 상대 턴 counter를 증가시키면 다른 효과 시점이 달라짐 | 두 색의 예약 상태를 저장하고 delayed spell보다 먼저 소비 |
| 번개/메테오는 fresh 포획 제한과 guard 포획 면역을 검사하지 않음 | 일반 이동 포획과 다른 주문 효과 경로 | 원본 보존; fresh 주문/guard 피해 비교 |
| 통나무 자동 이동은 시간 정지/지연 주문보다 먼저, 보드 row-major 순으로 처리 | ID 순서나 완료 counter 증가 뒤 처리하면 충돌 순서가 달라짐 | 시작 시 위치 목록을 고정하고 roll deadline을 적용 |
| 자동 이동으로 앙파상 도착 칸이 점유될 수 있음 | 기본 체스의 빈 도착 칸 불변식이 성립하지 않음 | 상태 검증을 확장; 일반 폰의 이중 포획과 종자/기수의 일반 포획 우선순위를 구분 |
| 샷건 킹이 있으면 반복 횟수를 기록하지 않고 장기전에서 해당 색이 패배 | 양쪽 샷건 킹이면 COLORS 순서의 white가 패널티 대상 | 원본 보존; 턴 제한과 반복 기록 회귀 테스트 |
| 산탄은 아군도 맞고 HP entity당 1회; 메테오는 점유 칸마다 HP 피해 | 같은 광역 공격으로 통합하면 규칙이 달라짐 | 별도 주문/산탄 순서와 중복 제거 유지 |
| 빅룩 캐슬링의 아군 제거는 capturePieceAt을 호출하지 않음 | 일반 착지 포획과 달리 마나/포획 progress/왕 패배를 발생시키지 않음 | 캐슬링 전이를 분리하고 실제 entity 전체만 제거 |
| 대형 기물 착지가 곰/고슴도치를 잡으면 즉시 반격으로 제거된 공격자 객체를 이어지는 착지 코드가 다시 배치 | 최종 보드에서는 공격자가 살아 있고 반격 기물이 사라지는 원본 순서 의존 동작 | 대형 착지에는 반격 대기열을 만들지 않아 최종 의미 상태를 보존; 섹터 공격은 정상 즉시 반격 |
| 반복 key는 풍차 모드를 포함하지 않음 | 미래 행동이 다른 상태를 같은 반복으로 셀 수 있음 | 원본 반복 key 보존; canonical은 모드 포함 |

## 남은 작업

아직 이식하지 않은 타입은 아래 지원표에 모두 표시한다. 남은 변형 기물과 자원/선택/반격/생성/턴 트리거를 구현하고 개별 및 JS 비교 테스트를 추가해야 Phase 3가 완료된다. 캠페인 전용 타입과 표시용 별칭은 별도 경계로 정리해야 한다. 미구현 타입을 일반 기물처럼 조용히 실행하지 않는다.

| 타입 | 상태 |
|---|---|
| `alfil` | 기본/현재 구현 |
| `amazon` | 기본/현재 구현 |
| `assassin` | 기본/현재 구현 |
| `babyBear` | 미구현 |
| `bat` | 낮 행마 구현; blood-moon-night의 밤 행마는 카드 단계 |
| `bear` | 퀸 행마·2회 반격·이동 잠금·대기열 구현 |
| `berserker` | 기본/현재 구현 |
| `bigBishop` | 기본/현재 구현 |
| `bigRook` | 이동·포획·HP·대형 캐슬링 구현 |
| `bishop` | 기본/현재 구현 |
| `blackHole` | 미구현 |
| `brutus` | 미구현 |
| `camel` | 기본/현재 구현 |
| `campfire` | 직교 한 칸 비포획 이동·인접 아군 보호 구현 |
| `cannon` | 기본/현재 구현 |
| `cardinal` | 기본/현재 구현 |
| `checker` | 기본/현재 구현 |
| `checkerKing` | 기본/현재 구현 |
| `clockwork` | 기본/현재 구현 |
| `coffin` | 미구현 |
| `colossus` | 기본/현재 구현 |
| `crown` | 미구현 |
| `darkWizard` | 미구현 |
| `dragon` | 미구현 |
| `eagle` | 기본/현재 구현 |
| `fanatic` | 현재 구현 |
| `ferz` | 기본/현재 구현 |
| `football` | 미구현 |
| `grasshopper` | 기본/현재 구현 |
| `guard` | 기본/현재 구현 |
| `hedgehog` | 킹 행마·2회 반격·이동/위협 잠금 구현 |
| `herald` | 현재 구현 |
| `hook` | 기본/현재 구현 |
| `idol` | 미구현 |
| `jester` | 현재 구현 |
| `king` | 기본/현재 구현 |
| `knight` | 기본/현재 구현 |
| `knightmaster` | 기본/현재 구현 |
| `lobster` | 전방 세 칸 이동·포획 구현; 지연 소환은 카드 단계 |
| `log` | 현재 구현 |
| `magicGirl` | 미구현 |
| `man` | 기본/현재 구현 |
| `merchant` | 현재 구현 |
| `missionary` | 현재 구현 |
| `monster` | 미구현 |
| `octopus` | 미구현 |
| `paladin` | 미구현 |
| `parrot` | 미구현 |
| `pawn` | 기본/현재 구현 |
| `pegasus` | 현재 구현 |
| `primeMinister` | 카드 없는 행마 구현; 포탈은 후속 카드 단계 |
| `princess` | 기본/현재 구현 |
| `protestant` | 기본/현재 구현 |
| `queen` | 기본/현재 구현 |
| `reaper` | 미구현 |
| `recruiter` | 현재 구현 |
| `rook` | 기본/현재 구현 |
| `royalKnight` | 기본 나이트 행마·왕권 구현; 추가 카드 행마는 미구현 |
| `scarecrow` | 미구현 |
| `shotgunKing` | 현재 구현 |
| `siegeRam` | 미구현 |
| `siren` | 미구현 |
| `slime` | 직교 3칸 도약·출발 칸 fresh 복제 구현 |
| `squire` | 기본/현재 구현 |
| `standardBearer` | 기본/현재 구현 |
| `thief` | 미구현 |
| `timeTraveler` | 미구현 |
| `trickster` | 미구현 |
| `undead` | 미구현 |
| `vampireLord` | 미구현 |
| `vip` | 현재 구현 |
| `wall` | 기본/현재 구현 |
| `windmill` | 기본/현재 구현 |
| `windmillBishop` | 풍차 표시 별칭 (별도 entity 아님) |
| `windmillRook` | 풍차 표시 별칭 (별도 entity 아님) |
| `wizard` | 현재 구현 |


## 다음 구현 지점

반격/생성/부활 계열과 남은 조건부 행마를 원본으로 확인하고 이식한다. 대형 기물과 일반 포획, 변신·생성의 공통 처리는 실제로 공유가 필요해질 때만 작은 primitive로 추출한다. 현재 `Action`, `TurnState`, `Piece`, `effects`를 미래 카드에 맞춰 미리 범용화하지 않는다.
