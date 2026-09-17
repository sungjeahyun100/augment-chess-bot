# Phase 0 — JS 엔진 분석

## 조사 범위와 기준

2026-09-18 작성. 입력은 ZIP이 아니라 `origin_code/main-DsoigPgV.js` 한 파일(103,874행)이다. SHA-256은 `0dbbad680c6e8e2abcdb6e49817ff486aa2f8ef799a8035bd356baef8a2731ea`이다. 원본은 수정하지 않았다. 아래 `L숫자`는 이 파일의 행 번호다.

번들 전체의 선언·상태 참조·난수 사용 위치를 검색하고, 초기화 → 드래프트 → 이동/카드 → 잡기 → 턴 → 종료 경로 및 특수 규칙의 연결부를 정적으로 조사했다. 카탈로그는 브라우저 앱 전체를 실행하지 않고 명시적으로 허용한 데이터 선언만 Node VM에서 평가해 추출했다. 개별 카드의 모든 조합을 실행 검증한 것은 아니다. Phase 0 결과는 분석 문서이며 Rust 구현과 differential 실행은 후속 단계다.

파일 L1부터 외부 JS 청크 `modulepreload-polyfill-COaX8i6R.js`, `zugzwang-BMan13jH.js`, `betaSupabaseAuth-BM9hgzr_.js`를 import하지만 해당 청크, HTML, 서버 및 Worker 구현은 제공되지 않았다. 현재 저장 폴더의 `.git`은 정상 Git 저장소가 아니므로 커밋 이력 비교는 할 수 없다. 원본 버전은 해시로 고정한다.

분석 결과의 상세 목록은 `RULE_INVENTORY.md`, 구현 순서와 검증 계약은 `PORTING_PLAN.md`, 재현 가능한 추출 자료는 `analysis/phase0_inventory.json`에 있다.

## 같은 번들 안의 서로 다른 엔진 경로

| 영역 | 근거 | 해석 |
|---|---|---|
| 공통 규칙 helper 및 최신 수정 | L597–4168 | 카탈로그 hash에 따른 동작 분기, 기물/패시브 함수. 최신 로컬 기본값과 구버전 온라인 규약을 구분해야 한다. |
| 권위 카드 manifest 및 catalog v1–v19 | L4170–7470, L11125 이후 | 과거 프로필을 재생하는 데이터가 공존한다. 모든 버전을 서로 다른 카드로 세면 안 된다. |
| 온라인 이벤트/선택 validation | L9039 이후, L11687 이후 | 카드 인스턴스, 타깃, 응답 창, domain event의 기존 계약을 참고할 수 있다. 서버의 실제 실행 결과와 동일하다고 단정하지 않는다. |
| 로컬 카드 및 UI 데이터 | L37287–42945 | `CARD_DEFS` → 분류/이름/별점/밸런스 patch. 현재 카탈로그 기준. |
| 로컬 상태 및 게임 실행 | L52522–56096, L77909 이후 | 이번 포팅의 첫 reference 대상. 전역 `state`를 직접 수정한다. |
| AI 및 위협 예측 | L69105–72690 | 탐색용 근사·가지치기·시뮬레이션이 섞임. 합법 행동의 완전한 oracle이 아니다. |
| 온라인 projection/전송/동기화 | L96555 이후, `startGame` L102996 | 서버 상태를 UI 상태로 투영한다. 로컬 `movePiece` 결과와 혼동하지 않는다. |

초기 호환 대상은 **이 해시의 최신 로컬 일반 게임**으로 고정하는 것이 적절하다. 온라인 profile별 버그 수정 플래그, 캠페인, chaos/grand, 편집기 및 AI 특혜는 별도 config와 지원 범위로 명시한다. 아직 어느 모드도 Rust로 구현하지 않았다.

## GameState와 Entity

`let state`는 L43298, `resetGame`은 L52718, 상태 리터럴은 L52753–53025에 있다. 구조는 JSON 같은 객체이지만 Set, Map, 공유 기물 참조도 포함한다. 초기 리터럴만 옮기면 나중에 생성되는 `captureTheFlag`, `pendingRecurrences`, `firstMoveUndo` 등의 상태를 놓친다. 전체 번들의 `state.x`/`state?.x` 정적 검색에서 292개 필드명을 발견했다. 이것은 동적 키·별칭을 포함한 완전한 schema가 아니며 JSON에 검색 결과를 보존했다.

| 책임 | 실제 필드 | 포팅 요구 |
|---|---|---|
| 보드/기물 | `board`, 기물 `id/color/type/moved/shielded/origin` | 행·열 및 PieceId 분리. 검정 시작은 row 0/1, 흰색 row 7/6. |
| 턴 | `turn`, `mode`, `turnsTaken`, `fullMove`, `moveCount`, `actionsRemaining`, `effects.extraMove` | action 수, 완료 턴 수, 양측 공통 턴을 구분한다. |
| 카드 | `deckSlots`, `hands`, `playerCards`, `cardAcquisitionNonce`, `cardsUsedThisTurn`, `clonedPassiveCards` | `deckSlots[color]`의 슬롯 순서와 빈 슬롯도 보존. 정의 ID와 인스턴스 ID 분리. |
| 카드 인스턴스 | `used`, `firstTurnCard`, `passiveApplied`, `nextTurnPending`, `nextTurnPendingSinceTurn`, `acquiredOrder`, `recovering` | 획득/활성화/사용/회복을 분리한다. `usedAt`의 소비처는 규칙/표시 여부를 확인한다. |
| 드래프트 | `draft`, `draftBalance`, `draftResumeTurn`, `middleDraftDone`, `endDraftDone`, `endPhaseStartMove`, `cardBanIds` | 제시된 후보, 재추첨 밸런스 상태, 재개할 플레이어도 미래 전이에 필요하다. |
| 규칙 설정 | `appliedRuleCard`, `additionalRuleCards`, `selectedRuleCardIds`, `completeRandom`, `draftDelete`, `gameStyle`, feature flags | RuleSet과 호환 profile. `RULE`도 현재 JS에서는 `applyCard`로 실행하지만 Rust에서는 분리한다. |
| 효과/예약 | `pending*`, `delayedHazards`, `temporaryQueens`, `necromancy`, `undeadResurrections`, `chainBonds`, `feudalContracts` | 대상 ID·소유자·기한·발동 경계·등록 순서가 필요하다. |
| 보드 환경 | `collapsedCells`, `portalRule`, `ruleBombs`, `highGround`, `blackHole`, `platformRule`, `palaces`, `crownRule` | 단순 64칸 체스판만으로 표현할 수 없다. |
| 선택/연속 행동 | `pendingPromotion`, `activeTrolley`, `royalCommand`, 기물의 추가 이동 표식 | UI `targeting` 속 규칙상 선택은 PendingChoice로 승격한다. 단순 hover는 제외한다. |
| 포획/승리 | `captures`, `capturedTypes`, `turnCaptures`, `lastTurnCaptures`, `kingDead`, `regency`, `positionCounts`, `repetitionSalt`, `deathmatch`, `winner` | 잡힌 기물의 순서/속성은 재활용·부활에 필요. `winner=null`만으로 진행/무승부를 구별할 수 없다. |
| 되돌리기 | `moveReplay`, `firstMoveUndo` | 리플레이 카드는 실제 게임 효과다. 영상용 기록과 함께 삭제하면 안 된다. |
| 외부/표시 | `clock`, `draftClock`, `logs`, `boardHistory`, animation Sets, `dragging`, `onlineEvents` | 실제 시간은 외부 입력, 표시 데이터는 이벤트 소비자로 이동. 기보 중 규칙이 조회하는 부분은 별도 최소 이력으로 남긴다. |

`piece`(L52522)는 무작위 문자열 ID를 생성한다. 거신병은 HP 3, bigRook/bigBishop은 HP 2, shotgunKing은 HP 4·탄약 3·방향을 가진다. 생성 뒤 많은 필드가 동적으로 붙는다. `createInitialBoard`(L53243)는 기본 8×8, 캠페인은 `createEmptyBoard`, `setupWideBoardSide` 등으로 다른 크기도 만든다. 거신병은 동일 객체를 여러 칸에 넣는다(L53275 부근). Rust는 entity 저장소 + 칸별 ID + anchor/footprint를 사용해야 중복 이동·포획을 막을 수 있다. neutral 기물도 존재한다.

`cloneStateForSimulation`(L71808)은 JSON replacer/reviver로 Set/Map을 보존하지만 공유 객체 identity는 보존하지 않는다. `cloneStateForKingThreatSimulation`(L71820)은 표시 이력을 비우고 `structuredClone`을 우선 사용한다. 두 경로의 공유 참조 차이는 대형 기물 회귀 테스트 대상이다.

`aiWorkerStateSnapshot`(L69311)은 수동 필드 whitelist이며 `aiSearchNoCards: true`, `skipRandomVanishing: true`를 주입한다. 완전한 재생 snapshot이나 canonical state로 재사용하면 안 된다.

의미 있는 임시 context도 전역에 남아 있다: `activeMetalMove`, `saturationAttackContext`, `activeMoveReplayCapture`, `freeMovePopulationBoard` 및 `aiSimulationDepth`. 앞 네 항목의 처리 문맥은 ActionResolution에 넣고 AI 근사 모드는 게임 상태 전이에서 분리한다. state 객체만 복제한다고 실행 문맥까지 독립적인 것은 아니다.

## 초기화와 주요 호출 관계

```text
resetGame
  → createInitialBoard → piece
  → maybeApplyOpeningRuleEvent → applyCard(rule)
  → beginInitialGameFlow / startDraft(white)
startDraft → drawPhaseChoices → 후보 생성
finishDraftSelection → addCardToPlayerDeck → applyPassiveCardOnDraft
  → (현재는 timer) completeDraftStep → black draft / play
getLegalMoves → 기물별 생성 → 공통/보드/포탈 modifier
  → applyMoveRestrictions → finalizeLegalMoves
movePiece → movePieceCore → movePieceAttack
  → 특수 행동 / 포획 / 이동 / 변신 / 프로모션 선택
  → endMove → first-move 카드 → completeTurnAfterMove
applyCard → applyCardEffect → 개별 효과/helper
  → finishCard → 사용 기록·후처리 → 같은 턴 또는 endMove
capturePieceAt → 방지/대체 → 제거/기록 → 반격·부활·왕권 판정
endGame → mode=gameover, winner, reason + UI/clock/replay
```

이것은 핵심 경로 지도이며 모든 함수가 무조건 순서대로 호출되는 단일 파이프라인은 아니다. 특수 행동은 조기 반환하거나 추가 선택을 만든다.

## 이동, 합법 행동, 잡기

`getLegalMoves`(L81883)는 anchor 정규화, 강제 후속 이동, frozen/poison/dice/arrest 등 이동 금지, 기물별 생성, 추가 행마, 포탈, 전역 제한을 합성한다. 잠시 양자 shadow를 보드에 materialize하는 경로도 있다. `applyMoveRestrictions`(L84387)는 붕괴·고지·시간 위상·궁전·마초·도발·얼음·위엄·사슬 등을 제한한다. `finalizeLegalMoves`(L82218)는 예약 칸·휴전·대형 footprint·포화·양자 counterpart 및 허수아비 강제 포획을 반영한다.

조회는 현재 순수하지 않다. 예를 들어 민주주의의 zugzwang flag 정리, `monoShade` 갱신(L84455 부근), 양자 shadow 생성/정리가 있다. Rust `legal_actions(&self)`는 상태·RNG·이력을 변경하지 않아야 한다.

**일반 체스 합법수 검사와 다르다.** `jumpMoves`(L83903)와 위 두 필터는 모든 자기 체크 수를 일괄 제거하지 않는다. 왕 위협 예측은 별도 시뮬레이션이며 `requestKingThreatConfirmation`(L72615 부근)은 “그 수 두기”를 허용하는 UI 경고다. 반면 `castleMoves`(L82855)는 미이동 여부, 경로, 공격받는 출발/통과 칸, 마법사 위험 및 포탈 제한을 실제로 검사한다. 따라서 표준 chess 라이브러리의 legal move/perft 결과를 그대로 정답으로 삼지 않는다.

`collectValidAiActions`(L69740)는 기본값에서 카드 행동을 넣지 않으며, `isAiUsefulSpecialMove`와 `collectAiMovesForPiece` 등의 선택적 필터를 사용한다. `collectAiWizardActions`(L69838)는 점수 임계값과 top-k를 사용한다. `exhaustiveCards` 옵션만으로 전체 행동 공간이 완전해지는 것도 아니다. differential oracle은 기물 generator와 모든 카드/선택의 유효성 검사를 조합해 별도로 만든다.

`movePiece`(L77909)는 metal 이동 context를 설정하고 `movePieceCore`의 saturation 포획 context를 통해 `movePieceAttack`으로 간다. 이 거대 함수에는 통나무 방향 지정, 포탈 입구/출구 검증, 양자 관측, 공성추 경로, 아군 잡기, 원거리 사격, 캐슬링, 피해/보호, 변신, 추가 행동, 승격 등이 섞여 있다. UI가 넘겨 준 move flags를 신뢰하는 부분은 Rust 공개 API에서 생성된 합법 행동과 재검증해야 한다.

`capturePieceAt`(L94057)의 순서는 특히 중요하다. 양자 관측 → nullification/잠복/포화/휴전/압도/동결/위상 등 방지 → 특수 대상 검사 → 회피 → 패링 대체 → 포획 종류 기록/마나/보드 제거 → 봉건계약/곰/트로이 반격 → 직접 잡기/독 → 군주 부활 → 새 카드 반응/종전 취소/잡힌 목록 → 왕권/민주주의 → 사신 연쇄/반격 → 예고장/왕관이다. `forceCapture`는 일부 앞단 방지만 우회한다. 모든 보호를 무시하는 보편적인 삭제 플래그로 해석하지 않는다. HP·shield 처리는 호출 측에도 있으므로 `capturePieceAt`만 복제해서는 부족하다.

`markFreshNoCapture`(L52564)는 `turnsTaken[color]+1`을 저장한다. 소환뿐 아니라 `markTransformedOrigin`을 통한 변신에도 적용되며 오프닝 passive/전향 예외가 있다. `cardNoCaptureUntil`, 신규 행마 금지, herald jump lock은 서로 구분한다.

## 턴과 효과 정산 순서

`endMove`(L79687)와 `completeTurnAfterMove`(L80015)를 하나로 합치지 않는다.

1. `endMove` 앞부분: 회귀, 주도권, 폭탄, 민주주의, 곰/트로이, 쌍둥이, 레이싱, 잠복 갱신, 깃발, 전령 승리.
2. 자유 이동 처리 중이면 국소 정산 후 반환한다. 일반 경로는 블랙홀, 통나무, 궁전 등을 정산한다.
3. 아이돌 앙코르, 결의 credit, `extraMove`, `actionsRemaining`, 시간 정지 중 하나가 턴을 유지할 수 있다. 이때 후반부와 `turnsTaken` 증가는 실행되지 않는다.
4. 실제 턴 종료로 진행하면 도적 체포, 임시 퀸/강령/지연 위험을 처리한다. `countMove!==false`일 때 `moveCount` 증가, 랍스터/언데드/오목/이세계/괴물 등을 처리한다.
5. 효과 소모와 종교 승리, 이동 표식 정리 뒤 첫 이동 OPENING 자동 사용, 폰 스톰, 전령, 휴전을 처리한다.
6. `completeTurnAfterMove`: 외부 시계 반영, 컨베이어(흑), `turnsTaken[movingColor]++`, 찬합/발판/왕관/돌풍/상태 만료/데스매치/종전 등을 처리한다.
7. 흑 완료 시 `fullMove++`, 겨울/붕괴/최후통첩, 포획 set 교체, 예약 자유 이동 및 턴당 카드 수 초기화 후 `turn=opponent`.
8. 새 차례의 귀빈/포탈/ICBM/붕괴/세이렌/실종/아기곰/카드 활성화/RULE 티켓/박스/패닉/트롤리/시간 여행/브루투스/가속/골드/징집을 처리한다. 이후 반복·장기전, 10/20턴 드래프트, 행동 불가 패배.

중간 단계마다 gameover로 조기 종료할 수 있다. Effect queue에는 phase, priority, 등록 순서와 명시적인 interruption을 두고 이 순서를 fixture로 먼저 고정한다. `moveCount`, `fullMove`, 소유자 완료 턴, 양측 최소 완료 턴은 대체 가능한 시계가 아니다.

## 카드 획득·사용

카드 정의는 L37766, category 재분류 L42039, passive 집합 L42717, 최종 밸런스 L42930 부근이다. 최종 241개 정의에는 RULE 27개와 GUN 1개도 포함된다. 일반 분류와 활성화 방식은 서로 독립적이다.

`addCardToPlayerDeck`(L55822 부근)은 빈 슬롯에 복제하고 획득 순번을 기록한다. 첫 드래프트의 OPENING은 `firstTurnCard`를 설정한다. MIDDLE/END 드래프트에서 얻은 해당 분류 카드는 `nextTurnPending`으로 지연될 수 있다. passive 획득은 즉시 효과 실행/지속 플래그 등록/다음 턴까지 보류 등의 경로를 가진다. 특히 `applyPassiveCardOnDraft`(L55981)는 `firstTurnCard`를 검사하지 않아 OPENING passive도 획득 시 적용하고 `used=true`로 만든다. 이후 첫 이동 자동 사용은 `!used`만 고르므로 이미 적용된 passive는 건너뛴다. 공식 요약의 “OPENING은 첫 이동 직후”와 충돌하는 별도 확인 항목이다. `activatePendingDraftCardsForTurn`(L55999)은 획득 시 저장한 완료 턴 수보다 일반 모드에서 1, grand END에서 3 이상 진행했을 때 잠금을 해제한다. 단순히 다음 turn 이벤트를 한 번 받으면 푸는 것과 다를 수 있다.

`forceFirstMoveCardsAfterMove`(L74469)는 완료 턴 0이고 아직 강제 사용하지 않았을 때 발동한다. 대상은 `autoTargetForFirstMoveCard`가 자동·무작위로 정할 수 있다. `forceFirstMoveCard`(L74507)는 첫 이동 후 적용 실패 시 저장된 첫 이동을 되돌리고 다시 적용하는 경로가 있다. 문서의 “첫 이동 직후”라는 문장만으로 이 rollback 예외를 없애면 호환성이 깨진다.

Active는 `applyCard → applyCardEffect`(L86302) 후 `finishCard`(L73524)가 사용 여부, 턴당 사용 수, 반복 salt, 승리/환경 반응을 정산한다. 대부분 턴 유지지만 shotgun-king, summon-colossus, trolley, premove, miracle, 최신 zugzwang 등은 턴을 소비한다. 턴 독점 카드 제한도 있다. 성공 전후와 실패 경로를 나누고 실패 입력에 대한 원자성을 확보해야 한다.

## 드래프트

일반 게임은 처음/양측 10턴 완료/양측 20턴 완료에 진행한다(`maybeStartMilestoneDraft`, L81371). 기본 후보 3장 중 1장, 백 선택 후 흑 선택, 저장된 `draftResumeTurn`으로 복귀한다. `finishDraftSelection`(L54064)이 상태를 수정한 뒤 timer로 `completeDraftStep`을 호출하므로 headless adapter에서는 타이머를 명시적 완료 처리로 바꿔야 한다.

- 첫 후보: OPENING/MIDDLE/PIECE. 오프닝 가중치 보정이 있다.
- 둘째 후보: 기본 3장일 때 MIDDLE 2장+PIECE 1장 계획과 부족분 fallback.
- 셋째 후보: MIDDLE/END.
- 낮은 별(score≤5, 별≤2.5)은 가중치 1.125. 높은 별(score 6…10)은 1…0.5. 별은 0.5 단위 정수로 표현한다.
- `draftBalance`에 따라 상대 제시 품질 보정/재추첨 가능. bans, 이미 선택한 카드, 상호 배타 관계, 대상 존재, RULE, AI 모드에 따른 필터가 추가된다.
- `completeRandom`은 단계별 pool 대신 RULE/shotgun 제외 등의 조건을 통과한 pool을 shuffle한다. 모든 정의가 무조건 등장하는 모드는 아니다. 호환성 필터 뒤 최종 카드 집합까지 수학적으로 균일하다고 단정하지 않는다.
- chaos는 3묶음×2장 선택, grand는 별도 공유 draft/pick 순서와 END 3턴 잠금이다. 일반 드래프트와 분리한다.

## RULE

`RULE_OPENING_CHANCE=0.4`(L37287). `maybeApplyOpeningRuleEvent`(L53115)는 설정이 켜졌을 때 일반 모드에서 40% 판정, 선택 모드에서는 선택 후보들(없음 포함)에서 하나를 고른다. 복수 선택은 모든 RULE의 동시 적용이 아니다. 시작 RULE은 임시로 백을 actor로 잡아 `applyCard`를 호출하고 `appliedRuleCard`에 기록한다. 화면 이벤트 시간/nonce는 규칙에서 제외한다.

`pendingRuleTickets` → `tickPendingRuleTicketsForTurnStart` → `applyAdditionalRuleCard`(L53174–53242)는 게임 도중 RULE을 추가한다. 따라서 시작 때 불변인 RuleSet만으로 부족하다. config에서 초기 RuleSet을 만들되 실행 중 AddRule도 가능해야 한다. 온라인은 방 설정/서버 projection 경로를 별도 검증한다.

## 난수와 결정론

전체 파일에 `Math.random` 문자열을 포함한 **82개 행**을 발견했다(호출 수가 아니라 기본 인자/같은 행의 복수 호출 포함). 전 위치·코드는 JSON, 위치별 분류는 목록 문서에 있다. `randomChoice`/`weightedChoice`/`shuffle` 간접 호출도 별도 인덱스로 남긴다.

규칙 난수: RULE 채택/선택, 드래프트 및 밸런스 재시도, Chess960/무작위 배치, 변형/박스/룰렛, 겨울/실종/패닉, 자동 타깃, 주사위, 실수·패링·양자, 괴물/발판/깃발/왕관, 캠페인 경매. ID 난수: 기물·카드·예약 효과/유대. 외부 난수: UI 룰렛 연출/카드 회전/로그 ID, 계정·진단·AI 정책.

공용 global Math.random에서 UI 소비까지 섞이면 같은 seed만 넣어도 브라우저와 headless 소비 순서가 달라진다. typed chance request + 기록된 결과 tape를 우선 differential 계약으로 삼는다. 순수 Rust playout은 버전을 고정한 seed RNG/state와 고정 후보 순서를 사용한다. EntityId는 별도 단조 증가 allocator로 생성하고 예전 ID는 모든 참조에 일관되게 remap한다. 벽시계 Date.now/new Date는 결과 기록이나 외부 ClockInput으로 분리한다.

AI 시뮬레이션은 `crownReplacementSquare`(L87895), `applyTranscendenceCaptureUpgrade`(L88393)에서 실제 난수를 moveCount 기반 값으로 대체한다. 위 snapshot은 실종도 생략한다. 실제 경기와 AI simulation을 같은 oracle로 취급하지 않는다.

## 승패 및 UI 경계

`resolveRoyalCapture`(L95195)는 왕 포획, 섭정 승계, 상인/귀빈, 민주주의, 회귀 예외를 처리한다. `checkNoActionLoss`(L81099)는 카드·기물 이동·특수 행동이 모두 없으면 패배시키며 일반 스테일메이트 무승부와 다르다. pending 선택이나 서버 경로에서는 생략된다.

그 밖에 전령 인접, 더블 체크, 레이싱, 종교(비숍 3개 차이), 오목(세로 5개), 왕관 유지, 깃발 점령, 하이랜더, 사신 영혼, 종전, 동시 왕권 붕괴, 캠페인 목표가 있다. 정확한 지점은 목록 문서에 수록한다.

반복 키 `positionKey`(L81336)는 turn/salt 및 기물 위치·색·종류·HP·ammo·frozen만 포함한다. 3회 반복은 별 판정, 별이 같으면 무승부, 적으면 승리, shotgun은 별도 패널티다. 45 공통 턴 도달 시 기본 데스매치가 켜져 있으면 연장전으로 간다. 기본 10수 동안 진행이 없으면 별 판정, revelation은 5수. 데스매치 tick은 흑 완료에 +2하므로 변수명만 보고 매 action마다 증가시키면 안 된다.

`endGame`(L95707)은 winner와 mode 외에 시계, toast, DOM, 기보 저장까지 호출한다. 엔진은 TerminalResult와 의미 이벤트만 반환해야 한다. 화면의 확인·타이머·온라인 Promise 완료에 걸려 있는 드래프트/프로모션/트롤리/카드 후처리를 엔진 continuation으로 옮긴다. 위협 경고는 엔진 legality와 구별한다. 은신/위장/안개는 표시만의 문제가 아니라 공격 위협·AI 관측에 영향을 주므로 전체 상태와 플레이어별 Observation을 분리한다.

## AlphaZero에 필요한 Action과 상태 계약

| 제안 Action | 근거/선택 내용 |
|---|---|
| Move | from/to 외에 route, 포탈, 도약 잡기, 대형 anchor, 교환, 양자 출발을 구분. 같은 from/to라도 효과가 다를 수 있다. |
| Promote / ResolvePromotion | 기본 승격, 특진, 보류 승격, 재활용 재료, 포기. 가능한 경우 이동+승격을 atomic action으로 정규화. |
| ActivateCard | CardInstanceId + 타깃의 순서/묶음/방향/선택. UI 클릭 횟수와 분리. |
| ChooseDraftCard / ChooseDraftBundle | 이미 생성된 offer의 인스턴스를 선택. grand의 draft actor는 board turn과 구별. |
| PieceAbility | wizard 주문·mana, shotgun fire/snipe/reload, 상인 매수, 통나무 방향 등. 생성된 move flag와 중복 없이 정규화. |
| ResolveChoice | trolley 응답, RULE 티켓, joker 카드 선택, barricade 방향, trickster 행마 등 필요한 규칙 선택. 선택권자가 항상 `turn`과 같지는 않다. |
| FinishOptionalContinuation | `fileSurgeSkip`/추가 이동 포기. 강제 연속 행동에는 허용하지 않음. |
| Resign / DrawResponse | 외부 경기 제어. 학습 action space 포함 여부를 config로 명시. |
| ResolveChance | 플레이어 policy 선택과 분리된 확률 노드. 자동 오프닝 타깃을 임의의 전략 선택으로 바꾸지 않는다. |

phase를 Setup/Draft/Play/PendingChoice/Chance/Terminal로 명시하고 actor를 별도로 둔다. `side_to_move`만으로 chance/응답 상황을 표현하지 말고 decision owner API를 추가한다. canonical proposal은 rules profile/config, board+entities, turn counters, ordered deck/draft, status, pending queues, 최소 규칙 이력, terminal result, RNG/chance cursor 및 ID allocator를 포함한다. map/set은 키순 정렬하고 배열은 의미 순서를 보존한다. schema version을 둔다. 구현 전 각 필드를 포함/제외/파생으로 검토하는 명세는 Phase 1 gate다.

## 발견된 불일치·위험 및 결정

| 관찰된 JS 동작 | 기대 규칙/위험 | 버그 판단 | Rust 결정(아직 구현 전) |
|---|---|---|---|
| 체크 위협 확인 후 위험한 수를 계속 둘 수 있음 | 표준 체스 library가 이를 금지할 수 있음 | 변형 규칙/UX, 버그로 단정하지 않음 | 로컬 reference 보존. 캐슬링의 실제 공격 검사와 구분. |
| OPENING passive가 획득 시 적용되고 used 처리됨(L55981) | 공식 OPENING 첫 이동 직후 규칙과 불일치 | 명세/구현 충돌, 의도 확인 필요 | 획득 직후 fixture로 관찰을 고정. 일반 규칙과 호환 profile의 처리 결정을 명시하며 조용히 수정하지 않음. |
| OPENING 실패 시 첫 이동 rollback/retry | 공식 요약에는 없는 예외 | 의도 불명 | 보존 fixture + 명시적 compatibility 규칙. |
| `positionKey`가 moved/enPassant/여러 status를 생략 | 서로 다른 미래 합법수가 같은 반복 키가 될 수 있음 | 잠재적 결함, 실행 재현 필요 | JS 반복 키와 완전 canonical hash를 분리. 조용히 고치지 않음. |
| AI generator top-k, simulation 난수 대체 | 모든 합법 행동/정확한 상태 전이가 아님 | 의도된 근사 | AI를 oracle로 쓰지 않음. |
| JSON clone의 공유 참조 손실 | 동일 ID 대형 기물의 객체 `===` 검사 위험 | 잠재적 결함 | ID 정규화 및 두 clone 경로 fixture. |
| `resolveRoyalCapture`에 `if (isDemocracyProtectedMerchant(state.democracy)) ;` | 조건 결과에 따른 본문 없음 | 의심스러운 no-op, 확정 버그 아님 | 실제 상인/민주주의 경로 재현 후 결정. |
| 데이터 정의 이후 phase/stars/passive 수정 | 초기 문구나 오래된 catalog와 다른 현재 동작 | 버전 차이 | 최종 patch/profile 고정, 원 ID 보존. |
| 은신/위장/안개 및 상대 후보 정보 | 완전정보 AlphaZero 전제를 깨뜨릴 수 있음 | 모델링 제약 | State와 Observation 분리, 초기 학습의 정보 설정 명시. |

이 단계에서 발견한 의심 항목을 수정하거나 실제 경기 버그로 확정하지 않았다. 가장 큰 후속 위험은 누락된 런타임 의존성, UI continuation에 숨어 있는 상태 전이, 처리 순서 충돌, RNG 소비 차이, AI oracle의 불완전성이다.
