# Phase 0 — 규칙 목록

## 기준과 읽는 법

출처는 `origin_code/main-DsoigPgV.js`, 해시와 분석 범위는 `ENGINE_ANALYSIS.md` 참조. 모든 행 번호는 이 원본 기준이다. 아래 목록은 현재 로컬 카탈로그의 최종 category/별점/밸런스 patch를 반영한다. 과거 v1–v19 catalog의 같은 ID를 중복 집계하지 않는다.

`CARD_DEFS`는 241개: OPENING 37, MIDDLE 86, END 56, PIECE 34(일반 213), RULE 27, GUN 1. phase와 activation은 별도 축이다. 이름/설명은 표시 메타데이터이며 실행 규칙 전체의 증명이 아니다. 카드별 `effect` 및 source를 남겨 실제 분기·helper를 찾아갈 수 있게 했다. 모든 카드 조합의 실행 검증은 아직 하지 않았다.

출처 표의 첫 행은 정의 또는 해당 정의 묶음 안 ID 위치다. 상세 복수 위치는 `analysis/phase0_inventory.json`의 `sourceLines`. 후처리 공통 근거는 category L42039, passive L42717, 표시 별점 L708, 최종 balance L655–691/L42930이다. PASSIVE는 즉시 한 번만 실행된다는 의미가 아니다. OPENING ACTIVE도 첫 이동 후 자동 사용될 수 있다. RULE은 MATCH_RULE로 표시한다.

## 상태 효과와 예약 상태

아래는 의미별 목록이다. 작은 필드까지 모두 하나의 Status enum으로 만들라는 뜻은 아니다. 소유자 턴/전체 턴/action 경계를 구분해야 한다.

| 분류 | 발견된 필드/규칙 | 근거 및 포팅 주의 |
|---|---|---|
| 공통 생성/카드 잡기 금지 | `freshNoCaptureUntil`, `cardNoCaptureUntil`, herald jump lock, promotion rush | L52564–52585, L3259. 소환/변형/추가 행마·오프닝 예외. |
| 피해·보호 | `hp/maxHp`, `shielded`, `protected`, `outpostProtected`, encouragement, coronation, queensGambit, nullification | piece L52522, capture L94057, 보호 L95229 이후. 피해/포획/강제 삭제 구별. |
| 포획 대체/반응 | evasion, parry, recurrence, bear retaliation, trojanHorse, feudalContract, poisonedPawn, reaperCaptures | L94057–94200, L93800 이후. 실패한 포획이 attacker 이동/턴을 소비하는지도 확인. |
| 행동 금지·지형 제한 | frozen, card freeze, poisonStunTurns/Color, diceLocks, staked, severance, iceSheet/inertia, exhaustion, chainBonds, bloodCurse | getLegalMoves L81883, applyMoveRestrictions L84387. timer 단위가 서로 다름. |
| 잠복·기물 고유 | submerged, thief arrest, clockwork neighbor, bearMoveLockedUntilTurn, octopus/radiance/metal | L3694–4168, getLegalMoves, completeThreeTurn. |
| 정보·존재 | hiddenFrom, camouflageRule, quantum, timePhase, ghost, hallucination | L93450 이후, L95790 이후. shadow와 진짜 entity/플레이어 관측 구분. |
| 행마 변경 | pawnReverse/pawnQueen, sprint/leap/retreat, basicTraining, socialism, royalCommand, reversal, overtake, majesty, cornerKick, imperialStudies, parrotMovement | L81883–82217, L84387. 여러 이동 modifier를 합성. |
| 추가 행동 | extraMove, actionsRemaining, idolEncoreUsedByPiece, resolveMoveCredit, fileSurge, reposition, frenzyExtraMove, rookLiftChain, platform, skipTurn | endMove L79687, L81486 이후. 턴 유지/강제 이동/포기 가능한 이동을 분리. |
| 턴당 카드 제한 | cardsUsedThisTurn, freeCardUsed, recovering, turn-exclusive, time traveler card lock | L1891, L42423, L73524, L73639. |
| 패시브 보유 | 색별 boolean/카운터, clonedPassiveCards, passiveApplied | L52753, L55981. GameState flag와 등록된 modifier로 정리. |
| 지연 이동/공격 | pendingPawnStorm/Panic/FreeMoves/Icbm/Gales/Otherworld/TrojanHorse/BearRetaliations/Trolley/Scarecrows/Lobsters/Portals/RuleTickets | 초기 상태 L52753, endMove/completeTurnAfterMove. queue 순서 보존. |
| 지연 변신/부활/제거 | temporaryQueens, necromancy, undeadResurrections, pendingRecurrences, judgmentExiles, delayedHazards, holdoutPromotion, emptyLunchbox, prophecy | L52670, L79111 이후, L79958, L81162. 제거가 곧 영구 소실은 아님. |
| 연결/조건부 존재 | twinBondId/twinPartnerId, chain IDs, feudalContractId, vipInvitation, crownBearer, regencyHeir | 해당 생성 함수 및 포획/턴 hook. ID 참조 일관성 필요. |
| 보드 상태 | collapsedCells, periodicCollapse, ruleBombs, portalRule, crownRule, conveyorRule, blackHole, highGround, highway, winterKingdom, platformRule, palaces | RULE 표 및 L80015/L84387. |
| 규칙상 이력 | captures, capturedTypes, turnCaptures, lastTurnCaptures, moveReplay, firstMoveUndo, positionCounts/repetitionSalt | L52753, L81329, L86089 이후. UI 로그로 보고 삭제 금지. |

정적 `state` 필드 인덱스는 JSON에 전부 수록했다. `item2` 같은 지역 변수는 카드/기물/화면 객체에도 쓰이므로 그 변수의 모든 프로퍼티를 기물 Status라고 잘못 집계하지 않았다. 카드 표의 passive effect 이름도 지속 규칙 목록의 일부다.

## 특수 이동·잡기·선택

| 범주 | 발견된 동작 | 주요 근거 |
|---|---|---|
| 기본 체스 변형 | 캐슬링/무료 캐슬링/빅룩 캐슬링, 앙파상/롱파상/강제 앙파상, 승격/특진/승격 포기/재활용 | L79012–79528, pawnMoves L82530, castleMoves L82855 |
| 경로 변경 | 포탈 입구·출구 포획/통과, 고속도로, 대각선/단색/붕괴 보드, 궁전, 고지 | applyPortalMovesForPiece L83314 부근, restrictions L84387 |
| 도약/연쇄 | cannon screen, grasshopper hurdle, checker capture chain, thief jump, locust swarm | 기물 switch L81883, helper L83903 이후 |
| 교환/전향 | dragon swap, switcheroo/substitution/relay, symmetry, twins, merchant buy, missionary conversion, miracle | movePieceAttack L77923, relayMoves L81868, merchant L94200 이후 |
| 이동 없는 공격 | bishop snipe, shotgun blast/snipe/reload, wizard meteor/lightning/shield/timeStop, colossus body/sector attack | movePieceAttack 및 collectAiPieceActions L69827 |
| 다중 칸/다중 포획 | colossus/bigRook/bigBishop footprint, siegeRam path, 폭탄 rank/file, 블랙홀·붕괴 batch 제거 | L81713–81810, L82218, board hazards |
| 자동 행동 | log 방향 선택/턴말 전진, babyBear 이동/성장, conveyor, monsters, brutus 아군 왕 공격 | endMove/completeTurnAfterMove |
| 잡기 금지/강제 | 새 기물, 휴전, 보호/위엄/제네바, 포화 한도, 허수아비 강제 표적, fresh 행마 | finalizeLegalMoves, canCaptureTarget, capturePieceAt |
| 잡기 대체 | 회피, 패링, 실수 역전, shield/HP, bear/trojan/feudal 반격, recurrence/undead/coffin 부활, poison/reaper 연쇄 | L78416, L93800–95404 |
| 잡은 뒤 변신 | chameleon/trickster/squire/standardBearer/transcendence, slime 복제, queen disassembly | movePieceAttack 및 L88393 |
| 비결정적 관측 | quantum 진짜/허상, counterpart 이동, portal 관측 | L77318, L93450–93581 |
| 사용자 선택 | 카드 다중 타깃/방향, promotion, trolley 응답, rule-ticket/joker, fileSurgeSkip, draft/bundle/grand | targeting 및 AI action 관련 함수 |

이동 객체의 flags는 UI만의 장식이 아니다. 같은 from/to라도 캡처 경로·도약·포탈·교환이 달라질 수 있으므로 policy/action 정규화에서 보존한다.

## 승리·패배·무승부

| 조건 | 실제 관찰 | 근거 |
|---|---|---|
| 왕권 상실 | 왕 포획, 섭정 후계자/퀸, 상인, 귀빈. 민주주의·회귀·관 부활 예외 | resolveRoyalCapture L95195, capturePieceAt L94057 |
| 행동 불가 | 카드/합법 이동/특수 행동 모두 없으면 패배. 기본 stalemate draw 아님 | checkNoActionLoss L81099 |
| 전령 | 적 왕권 기물/상인 인접 협정 승리 | checkHeraldVictory L95600 |
| 더블 체크 | binaMate 활성 상태에서 서로 다른 공격자 2개 | checkDoubleCheckVictory L95614 |
| 레이싱 | 지정 왕이 목표 랭크 도착. macho/붕괴가 목표에 영향 | checkRacingKing L95665 부근 |
| 종교 | 아군 비숍이 상대보다 3개 이상 많음 | checkReligiousVictory L89163 |
| 오목 | 아군 기물 세로 5개 | checkGomokuVictory L79649 |
| 깃발 | 적 홈 깃발 점령의 턴 경계 정산. 양측 동시 패배 시 draw | septemberAdvanceFlags L1228, updateCaptureFlags L79673 |
| 왕관 | 10수 유지 | resolveCrownRuleAfterMove 및 endGame L88097 |
| 하이랜더 | 아군 종류 중복 없음, royal 종류 정규화 포함 | checkInternalHighlander L101109 부근, applyEightLocalCard L8695 |
| 사신 | 영혼 4개 | REAPER_CAPTURE_TARGET L1264, L93885/L93944 |
| 민주주의 | 폰 소멸에 따른 패배/동시 소멸 | checkDemocracyDefeat L94014, 인접 batch 함수 |
| 종전 | prophecy countdown 만료; 포획으로 취소 | tickPropheciesAfterTurn L81162 |
| 반복 | 3회 positionKey 반복 → 낮은 별 승, 동률 draw | L81304–81368 |
| 장기전 | 기본 45 공통 턴 → 데스매치 또는 별 판정. 데스매치 기본 10수 무진행, revelation 5수 | L81259–81329, L37420 |
| shotgun 패널티 | 반복/장기전 별 판정에서 shotgun 쪽 패배 | shotgunPenaltyColor L81365 |
| 동시 제거 | 양쪽 왕 깔림/양쪽 왕권 붕괴 등 draw | L87009, L94052, L95399 |
| 매수 | 왕권 기물 매수의 즉시 승리 경로 | L94259 |
| 캠페인 | 나이트 탐험/목표 달성, 시간 여행자 사망, 시간 카드의 적 전멸, 특수 적/왕권 | checkCampaignObjectives 계열 L68000 이후, L89016/L89066 |
| 경기 제어 | 시계 만료, 기권, 합의 무승부/온라인 이탈 | clock 함수 L52000 부근, resign L100137/L100201, 서버 전송 |

고전 50수·기물 부족 무승부·표준 체크메이트를 현재 로컬 기본 규칙이라고 가정하지 않는다. 여기서 승리 조건의 호출 우선순위까지 모두 독립적으로 교환 가능한 것은 아니다.

## 카탈로그 밖의 캠페인 카드

| ID/선택 | 효과 | 근거 |
|---|---|---|
| black-tower-legacy-magic | 캠페인 흑마법, 첫 이동 자동 사용/턴 소모 예외 | BLACK_TOWER_CARD L830, L74493 |
| time-phase-shift | 과거/미래 위상 교체 | TIME_TRAVELER_CARDS L39950 |
| time-clumsy-attack | 같은 위상 기물 포획 허용 | L39959 이후 |
| time-is-mine | 20수 이후 적 제거 | L39968 이후, L89016/L89066 |
| blood | 혈액 효과 선택용 카드; summon/veil/sunlight/curse/coffin | L39981–40019, makeBloodCard L88504 |
| knight-journey-hint-1/2/3 | 여행 힌트 카드/사용 횟수 | L40020, markCardUsed L73639 |

캠페인 시나리오·메뉴·업데이트 로그의 카드 모양 UI를 일반 카드 수에 더하지 않는다. 위는 실행에 사용하는 별도 카드군이다.

<!-- GENERATED_CATALOG -->

## 전체 카드 및 RULE 정의

별점은 최종 patch 적용 값이다. 설명만으로 실제 capture/turn 처리 순서를 확정하지 않는다.


### OPENING — 37개

| ID | 이름 | 별 | 활성화 | effect / target | 설명 | 정의 출처 |
|---|---|---:|---|---|---|---|
| `apprentice-knights` | 견습 기사단 | 1.5 | ACTIVE | `apprenticeKnights` / — | 아군 양쪽 두 번째 파일의 폰을 종자로 변경합니다. | [L39049](origin_code/main-DsoigPgV.js#L39049) |
| `big-bishop` | BISHOP | 4 | PASSIVE | `bigBishop` / — | 조금 큰 비숍을 가지고 게임을 시작합니다. | [L994](origin_code/main-DsoigPgV.js#L994) |
| `big-rook` | ROOK | 4 | PASSIVE | `bigRook` / — | 조금 큰 룩을 가지고 게임을 시작합니다. | [L2146](origin_code/main-DsoigPgV.js#L2146) |
| `calling-card` | 예고장 | 3 | ACTIVE | `callingCard` / — | 상대 무작위 비폰 기물 하나에 예고장을 보냅니다. 해당 기물이 잡힐 경우, 다른 기물 하나를 추가로 제거합니다. | [L39456](origin_code/main-DsoigPgV.js#L39456) |
| `checker` | 체커 | 2 | ACTIVE | `checker` / — | 적군 양 끝 파일의 폰을 체커로 변경합니다. | [L1451](origin_code/main-DsoigPgV.js#L1451) |
| `democracy` | 민주주의 | 4 | PASSIVE | `democracy` / — | 킹이 잡혀도 패배하지 않습니다. 모든 폰이 잡히면 패배합니다. | [L2006](origin_code/main-DsoigPgV.js#L2006) |
| `dutch` | 네덜란드 | 4 | PASSIVE | `dutch` / — | 모든 아군 룩과 마이너 피스를 풍차로 바꾸고 시작합니다. | [L38040](origin_code/main-DsoigPgV.js#L38040) |
| `early-promotion` | 조기 진급 | 3.5 | PASSIVE | `earlyPromotion` / — | 아군 폰의 프로모션 랭크가 2칸 내려옵니다. | [L37966](origin_code/main-DsoigPgV.js#L37966) |
| `elephant-escape` | 코끼리 탈출 | 2 | ACTIVE | `elephantEscape` / — | 알필 두 마리를 추가합니다. | [L38853](origin_code/main-DsoigPgV.js#L38853) |
| `false-start` | 부정출발 | 3 | PASSIVE | `falseStart` / — | 모든 아군 기물이 2칸 전진된 상태로 시작합니다. | [L3822](origin_code/main-DsoigPgV.js#L3822) |
| `fast-growth` | 성급한 승진 | 3.5 | PASSIVE | `fastGrowth` / — | 아군 폰의 프로모션 랭크가 3칸 내려옵니다. 단, 메이저 피스로는 변할 수 없습니다. | [L37985](origin_code/main-DsoigPgV.js#L37985) |
| `fianchetto` | 피앙케토 | 3.5 | PASSIVE | `fianchetto` / — | 아군 비숍이 피앙케토 대각선 안에 들어갈 경우, 상대 폰이 그 안에 진입할수 없습니다. | [L38031](origin_code/main-DsoigPgV.js#L38031) |
| `field-promotion` | 특진 | 3.5 | PASSIVE | `fieldPromotion` / — | 아군 폰이 기물을 2개 잡으면 즉시 프로모션할 수 있습니다. | [L37774](origin_code/main-DsoigPgV.js#L37774) |
| `geneva-convention` | 제네바 협약 | 3 | PASSIVE | `genevaConvention` / — | 상대방 퀸이 아군 폰을 잡을 수 없습니다. | [L37789](origin_code/main-DsoigPgV.js#L37789) |
| `guard` | 근위병 | 2 | ACTIVE | `guard` / — | 아군 킹 파일의 폰을 근위병으로 변경합니다. | [L1448](origin_code/main-DsoigPgV.js#L1448) |
| `holdout` | 존버 | 3 | ACTIVE | `holdout` / own-pawn | 폰 하나를 지정합니다. 해당 폰은 14수 이후 퀸으로 프로모션합니다. | [L37975](origin_code/main-DsoigPgV.js#L37975) |
| `horde` | 호드 | 4 | PASSIVE | `horde` / — | 킹을 제외한 모든 기물을 제거하고 호드 배치로 전환합니다. | [L38067](origin_code/main-DsoigPgV.js#L38067) |
| `initiative` | 선공권 | 3 | PASSIVE | `initiative` / — | 상대방이 첫 10수 동안 기물을 잡을 수 없게 됩니다. 단, 기물이 잡히거나 체크시 제한이 해제됩니다. | [L38086](origin_code/main-DsoigPgV.js#L38086) |
| `iron-monarch` | 친정 | 4.5 | PASSIVE | `ironMonarch` / — | 아군 킹이 상대방 폰 혹은 광신도를 잡을 경우 한 번 더 움직일 수 있습니다. | [L1979](origin_code/main-DsoigPgV.js#L1979) |
| `king-of-the-hill` | 언덕의 왕 | 2.5 | PASSIVE | `kingOfTheHill` / — | 킹이 보드 중앙 4칸으로 들어올 경우 아마존처럼 움직일 수 있습니다. | [L1978](origin_code/main-DsoigPgV.js#L1978) |
| `knightmate` | 나이트메이트 | 3.5 | ACTIVE | `knightmate` / — | 킹을 로얄 나이트로, 모든 나이트를 만으로 변경합니다. | [L1987](origin_code/main-DsoigPgV.js#L1987) |
| `last-stand` | 집단 광기 | 2 | ACTIVE | `lastStand` / — | 모든 아군 폰을 광신도로 변이시킵니다. | [L37957](origin_code/main-DsoigPgV.js#L37957) |
| `locust-swarm` | 메뚜기떼 | 3 | PASSIVE | `locustSwarm` / — | 모든 아군 비폰 기물이 시작 위치에서 움직이지 않았을 경우 1회에 한해 그래스호퍼처럼 이동할 수 있습니다. | [L3824](origin_code/main-DsoigPgV.js#L3824) |
| `london-system` | 런던 시스템 | 2.5 | PASSIVE | `londonSystem` / — | 런던 시스템의 첫 7수가 진행된 상태로 내 진영을 배치합니다. | [L2147](origin_code/main-DsoigPgV.js#L2147) |
| `loyalist` | 충신 | 3 | ACTIVE | `loyalist` / own-loyalist-piece | 아군 기물 하나를 지정합니다. 해당 기물은 위치에 관계없이 턴을 소모해 킹 주변 칸으로 이동할 수 있습니다. | [L1750](origin_code/main-DsoigPgV.js#L1750) |
| `martyrdom` | 순교 | 3 | ACTIVE | `martyrdom` / — | 비숍을 전부 희생하여 모든 아군 폰에 가호를 부여합니다. | [L38483](origin_code/main-DsoigPgV.js#L38483) |
| `merchant-guild` | 상인 조합 | 5 | ACTIVE | `merchantGuild` / — | 킹을 2칸 전진시킨 뒤 상인으로 변경합니다. 상인은 골드로 상대 기물을 매수합니다. | [L37912](origin_code/main-DsoigPgV.js#L37912) |
| `otherworld` | 이세계 | 3.5 | ACTIVE | `otherworld` / — | 무작위 아군 폰 하나를 제거한 뒤 14수 후 동일한 위치에 마법사를 소환합니다. 소환 위치가 막혀있을 경우 해당 기물을 잡으며 소환됩니다. | [L39417](origin_code/main-DsoigPgV.js#L39417) |
| `pawn-conversion` | 전환 | 2 | PASSIVE | `pawnConversion` / — | 모든 아군 폰의 전진 및 대각선 행마가 뒤바뀝니다. | [L38049](origin_code/main-DsoigPgV.js#L38049) |
| `princess` | 프린세스 | 3.5 | ACTIVE | `princess` / own-rook | 룩 하나를 선택해 프린세스로 변경합니다. | [L949](origin_code/main-DsoigPgV.js#L949) |
| `queen-cavalry` | 퀸의 기병대 | 2 | ACTIVE | `queenCavalry` / — | 아군 퀸 파일의 폰을 나이트로 바꿉니다. | [L38368](origin_code/main-DsoigPgV.js#L38368) |
| `queens-gambit` | 퀸즈 갬빗 | 3.5 | ACTIVE | `queensGambit` / own-queen | 퀸을 갬빗합니다. 이후 퀸 파일 및 무작위 파일의 아군 폰에 반영구적인 보호를 부여합니다. | [L38473](origin_code/main-DsoigPgV.js#L38473) |
| `qxe1` | Qxe1!! | 3 | ACTIVE | `qxe1` / — | 아군 퀸이 킹을 잡고 왕위를 찬탈합니다. 해당 퀸은 킹으로 판정되며, 잡힐 경우 패배하게 됩니다. | [L1988](origin_code/main-DsoigPgV.js#L1988) |
| `sprint` | 질주 | 3 | PASSIVE | `pawnSprint` / — | 폰이 첫번째 이동으로 3칸까지 이동할 수 있습니다. | [L38022](origin_code/main-DsoigPgV.js#L38022) |
| `summon-colossus` | 메가체스트론 | 5 | ACTIVE | `summonColossus` / — | 아군 폰 6개를 희생해 체크메이트의 거신병을 소환합니다. | [L37858](origin_code/main-DsoigPgV.js#L37858) |
| `thief` | 도적 | 3 | ACTIVE | `thief` / — | 퀸을 도적으로 변경하고 잠복을 부여합니다. 도적은 내 턴이 끝날때 잠복이 없다면 체포되어 사라집니다. | [L649](origin_code/main-DsoigPgV.js#L649) |
| `vanguard` | 선봉 | 3 | PASSIVE | `vanguard` / — | 제일 선두에 있는 아군 폰이 하나라면, 해당 폰은 대각선으로도 이동할 수 있습니다. | [L968](origin_code/main-DsoigPgV.js#L968) |

### MIDDLE — 86개

| ID | 이름 | 별 | 활성화 | effect / target | 설명 | 정의 출처 |
|---|---|---:|---|---|---|---|
| `alekhine-machine-gun` | 알레킨의 머신건 | 2.5 | ACTIVE | `alekhineMachineGun` / — | 아군 기물이 알레킨의 총을 이루는 상태에서 사용한다면 세 기물이 모두 가호를 받습니다. | [L38397](origin_code/main-DsoigPgV.js#L38397) |
| `armistice` | 휴전 | 3.5 | ACTIVE | `armistice` / — | 아군 및 적군 기물이 다음 2수동안 서로 잡을 수 없습니다. | [L37792](origin_code/main-DsoigPgV.js#L37792) |
| `backward-knight` | 뒤로 가는 나이트 | 3 | PASSIVE | `backwardKnight` / — | 나이트가 뒤로 가며 기물을 잡으면 자신의 턴을 유지합니다. | [L38993](origin_code/main-DsoigPgV.js#L38993) |
| `basic-training` | 기초 교습 | 3.5 | ACTIVE | `basicTraining` / own-basic-training-piece | 아군 기물 하나를 지정해 폰의 행마법을 추가적으로 부여합니다. | [L38283](origin_code/main-DsoigPgV.js#L38283) |
| `black-box` | 검은 상자 | 2.5 | ACTIVE | `blackBox` / — | 사용시 무작위 액티브 증강 카드 하나를 발동시킵니다. 선택이 필요한 카드일 경우 완전히 무작위로 선택됩니다. | [L39255](origin_code/main-DsoigPgV.js#L39255) |
| `breakthrough-order` | 돌파 | 3 | ACTIVE | `breakthroughOrder` / — | 이번 턴 동안 아군 폰이 바로 앞의 적 기물을 잡으며 이동할 수 있습니다. | [L39180](origin_code/main-DsoigPgV.js#L39180) |
| `castling` | 캐슬링 | 3.5 | ACTIVE | `freeCastling` / own-castling-rook | 킹과 같은 직선 혹은 대각선에 있는 룩 하나를 선택합니다. 해당 룩과 킹은 캐슬링하며, 도착 위치에 기물이 있을 경우 자동으로 잡습니다. | [L1989](origin_code/main-DsoigPgV.js#L1989) |
| `chameleon-mutation` | 카멜레온 변이 | 1.5 | ACTIVE | `chameleonMutation` / own-piece | 아군 기물 최대 3개에게 잡은 기물로 변신하는 특성을 부여합니다. | [L38377](origin_code/main-DsoigPgV.js#L38377) |
| `chimera` | 키메라 | 3 | ACTIVE | `chimera` / own-minor | 마이너 피스 하나에게 키메라 효과를 부여합니다. | [L1755](origin_code/main-DsoigPgV.js#L1755) |
| `cleanup` | 클린업 | 2 | ACTIVE | `cleanupPieces` / own-cleanup-pieces | 아군 기물을 최대 3개까지 선택해 보드에서 삭제시킵니다. | [L37775](origin_code/main-DsoigPgV.js#L37775) |
| `conversion` | 기도 | 2.5 | PASSIVE | `conversion` / — | 획득 시 아군 나이트를 모두 비숍으로 변경합니다. | [L38274](origin_code/main-DsoigPgV.js#L38274) |
| `corner-kick` | 코너킥 | 3.5 | PASSIVE | `cornerKick` / — | 아군 나이트가 보드의 구석 2x2 자리에 위치할 경우 비숍처럼도 움직일 수 있습니다. | [L38265](origin_code/main-DsoigPgV.js#L38265) |
| `dice` | 주사위 | 2 | ACTIVE | `dice` / — | 주사위를 굴려 나온 눈에 따라 상대 기물을 2턴간 봉인합니다. | [L38682](origin_code/main-DsoigPgV.js#L38682) |
| `disarm` | 무장해제 | 4 | ACTIVE | `disarm` / enemy-piece | 킹을 제외한 상대 기물 하나를 선택합니다. 해당 기물은 이번 턴에 기물을 잡을 수 없습니다. | [L3543](origin_code/main-DsoigPgV.js#L3543) |
| `disassembly` | 분해 | 2 | ACTIVE | `disassembly` / — | 사용한 턴에 퀸을 움직일 경우 아군 퀸을 룩과 비숍으로 분해합니다. 퀸이 이동한 위치에 비숍을, 원래 위치에 룩을 소환합니다. | [L3821](origin_code/main-DsoigPgV.js#L3821) |
| `emergency-evacuation` | 긴급 피난 | 2.5 | ACTIVE | `emergencyEvacuation` / own-evacuation-piece | 아군 기물 최대 3개를 선택해 자기 진영 방향으로 1칸 후퇴시킵니다. 피난한 기물은 이번 턴에 기물을 잡을 수 없습니다. | [L39300](origin_code/main-DsoigPgV.js#L39300) |
| `en-passant-bang` | 앙파상 | 3 | ACTIVE | `enPassantBang` / — | 이번 턴 동안 아군 폰이 옆에있는 모든 기물을 상대로 앙파상할 수 있습니다. | [L38095](origin_code/main-DsoigPgV.js#L38095) |
| `encouragement` | 독려 | 3 | PASSIVE | `encouragement` / — | 킹 주변 아군에게 보호를 부여합니다. | [L38464](origin_code/main-DsoigPgV.js#L38464) |
| `evasion` | 회피 | 2.5 | ACTIVE | `evasion` / — | 무작위 아군 기물 하나에게 회피를 부여합니다. | [L3466](origin_code/main-DsoigPgV.js#L3466) |
| `exhaustion` | 탈진 | 3 | PASSIVE | `exhaustion` / — | 상대는 같은 기물을 4번 연속 움직일 수 없습니다. 킹은 예외입니다. | [L37821](origin_code/main-DsoigPgV.js#L37821) |
| `fanatical-ritual` | 광신적 의식 | 3 | ACTIVE | `fanaticalRitual` / — | 킹을 제외한 상대 무작위 기물 하나를 광신도로 변환합니다. | [L39020](origin_code/main-DsoigPgV.js#L39020) |
| `feudal-contract` | 봉건 계약 | 3.5 | ACTIVE | `feudalContract` / feudal-contract | 아군 폰 혹은 광신도와 비폰 기물 하나를 계약합니다. 폰이 잡히면 계약 기물이 반격합니다. | [L38862](origin_code/main-DsoigPgV.js#L38862) |
| `file-surge` | 급수 | 3 | PASSIVE | `fileSurge` / — | 나이트가 양 끝 파일로 이동할 경우 한번 더 이동할 수 있습니다. | [L39002](origin_code/main-DsoigPgV.js#L39002) |
| `freeze` | 빙결 | 3 | ACTIVE | `freeze` / — | 무작위 상대 기물 3개를 3턴 동안 얼립니다. 얼어붙은 기물은 움직이거나 공격받지 않습니다. | [L3536](origin_code/main-DsoigPgV.js#L3536) |
| `frontline-response` | 불가침 | 3.5 | PASSIVE | `frontlineResponse` / — | 아군 룩이 가로세로로 잡히지 않습니다. | [L37770](origin_code/main-DsoigPgV.js#L37770) |
| `gale` | 강풍 | 2.5 | ACTIVE | `gale` / — | 사용 3턴 뒤, 아군과 인접하지 않은 모든 기물은 강풍에 날아가 제거됩니다. | [L39388](origin_code/main-DsoigPgV.js#L39388) |
| `ghost` | 고스트 | 4.5 | PASSIVE | `ghost` / — | 획득 시 모든 아군 폰에 고스트 효과를 부여합니다. | [L1752](origin_code/main-DsoigPgV.js#L1752) |
| `horse-riding` | 승마 | 2.5 | PASSIVE | `horseRiding` / — | 획득 시 모든 아군 나이트를 삭제하며, 아군 킹이 킹과 나이트를 합친 행마법을 가지게 됩니다. | [L1980](origin_code/main-DsoigPgV.js#L1980) |
| `ice-sheet` | 빙판 | 2.5 | ACTIVE | `iceSheet` / — | 상대방의 모든 원거리기물이 다음 3수동안 최대 사거리로만 이동할 수 있습니다. | [L39162](origin_code/main-DsoigPgV.js#L39162) |
| `imperial-studies` | 제왕학 | 4 | PASSIVE | `imperialStudies` / — | 아군 킹이 기물을 잡을 경우 잡은 기물의 행마법을 습득합니다. 습득한 행마법은 중첩됩니다. | [L1983](origin_code/main-DsoigPgV.js#L1983) |
| `inertia` | 관성 | 3.5 | ACTIVE | `inertia` / enemy-ranged | 상대방 원거리 기물 중 하나를 지정합니다. 해당 기물은 이제 한 칸씩 움직일 수 없습니다. | [L1768](origin_code/main-DsoigPgV.js#L1768) |
| `infiltration` | 잠입 | 3.5 | PASSIVE | `infiltration` / — | 아군 기물이 상대 홈 랭크까지 침투할 경우, 주변에 기물이 없다면 잠복을 얻습니다. | [L884](origin_code/main-DsoigPgV.js#L884) |
| `injury` | 부상 | 4 | PASSIVE | `injury` / — | 상대방의 나이트 계열 기물이 더이상 상하좌우 4칸을 뛰어넘을 수 없습니다. | [L39171](origin_code/main-DsoigPgV.js#L39171) |
| `insight` | 통찰 | 3.5 | ACTIVE | `insight` / — | 상대방의 모든 긍정적 효과를 제거하고, 아군의 모든 부정적인 효과를 제거합니다. | [L39218](origin_code/main-DsoigPgV.js#L39218) |
| `killer-king` | 킬러 킹 | 4 | PASSIVE | `killerKing` / — | 아군 킹이 상대방 킹을 잡을 땐 룩처럼도 움직일 수 있습니다. | [L894](origin_code/main-DsoigPgV.js#L894) |
| `leap` | 도약 | 4 | PASSIVE | `pawnLeap` / — | 아군 폰이 상대 폰과 마주보았을 때 한 칸 뛰어넘어 전진할 수 있습니다. 해당 이동으로는 기물을 잡지 못합니다. | [L38058](origin_code/main-DsoigPgV.js#L38058) |
| `long-en-passant` | 롱파상 | 2 | PASSIVE | `longEnPassant` / — | 아군 폰이 먼 거리에 있는 폰을 상대로도 앙파상 할 수 있습니다. | [L3825](origin_code/main-DsoigPgV.js#L3825) |
| `mad-horse` | 광마 | 3.5 | PASSIVE | `madHorse` / — | 나이트가 아군 기물을 잡을 수 있습니다. 아군을 잡으면 나이트가 한 번 더 움직입니다. | [L38172](origin_code/main-DsoigPgV.js#L38172) |
| `majesty` | 위엄 | 3 | PASSIVE | `majesty` / — | 적군 메이저 피스가 아군 킹 주변 8칸으로 올 수 없습니다. | [L874](origin_code/main-DsoigPgV.js#L874) |
| `metal` | 메탈 | 3.5 | ACTIVE | `metal` / own-metal | 아군 원거리기물 하나를 선택해 금속화 효과를 부여합니다. | [L3697](origin_code/main-DsoigPgV.js#L3697) |
| `miracle` | 기적 | 3.5 | ACTIVE | `miracle` / own-plain-bishop | 아군 비숍을 하나 선택해 현재 잡을 수 있는 기물들을 모두 전향시킵니다. 이후 해당 비숍은 희생됩니다. | [L986](origin_code/main-DsoigPgV.js#L986) |
| `mistake-card` | 실수 | 3.5 | ACTIVE | `mistakeCard` / — | 이번 턴에 상대방이 기물을 잡을 경우 50% 확률로 실수합니다. 실수할 경우 반대로 상대 기물이 잡히게 됩니다. | [L37840](origin_code/main-DsoigPgV.js#L37840) |
| `mongolian-gambit` | 몽골리안 갬빗 | 5 | PASSIVE | `mongolianGambit` / — | 획득 시 킹을 제외한 모든 아군 기물을 나이트로 바꿉니다. | [L37921](origin_code/main-DsoigPgV.js#L37921) |
| `nullification` | 상쇄 | 3.5 | ACTIVE | `nullification` / own-piece | 원하는 아군 기물 하나에 상쇄를 부여합니다. | [L931](origin_code/main-DsoigPgV.js#L931) |
| `outpost` | 전초기지 | 3.5 | ACTIVE | `outpost` / own-outpost-piece | 상대 진영에 더 가까운 비킹 기물 하나를 선택해 보호를 부여합니다. 보호는 다음 움직임 전까지 유지됩니다. | [L913](origin_code/main-DsoigPgV.js#L913) |
| `overtake` | 추월 | 4 | ACTIVE | `overtake` / — | 이번 턴에 룩이 아군 기물을 뛰어넘으며 이동할 수 있습니다. | [L1004](origin_code/main-DsoigPgV.js#L1004) |
| `overwhelm` | 압도 | 4.5 | PASSIVE | `overwhelm` / — | 상대방 킹이 아군 킹과 퀸을 잡을 수 없습니다. | [L38890](origin_code/main-DsoigPgV.js#L38890) |
| `panic` | 패닉 | 2 | ACTIVE | `panic` / enemy-panic | 적군 기물 2개를 선택합니다. 다음 상대방 턴에 해당 기물들이 이동범위 내 무작위 칸 중 하나로 이동한 채 시작합니다. | [L39189](origin_code/main-DsoigPgV.js#L39189) |
| `parry` | 패링 | 4 | ACTIVE | `parry` / own-parry-piece | 아군 기물 하나를 지정합니다. 해당 기물은 다음 공격을 받을 때 40% 확률로 반격합니다. | [L1749](origin_code/main-DsoigPgV.js#L1749) |
| `pawn-storm` | 폰 스톰 | 2.5 | ACTIVE | `pawnStorm` / own-pawn-storm | 아군 폰을 원하는 만큼 선택해 즉시 한 칸씩 전진시킵니다. 이렇게 움직인 폰은 이번 턴에 기물을 잡을 수 없습니다. | [L39227](origin_code/main-DsoigPgV.js#L39227) |
| `poisoned-pawn` | 독이 든 폰 | 3 | ACTIVE | `poisonedPawn` / own-unpoisoned-pawn | 아군 폰 하나를 선택해 독을 부여합니다. 해당 폰을 먹은 적 기물은 2수동안 움직이지 못합니다. | [L37801](origin_code/main-DsoigPgV.js#L37801) |
| `premove` | 프리 무브 | 3.5 | ACTIVE | `freeMove` / own-premove-piece | 아군 기물을 최대 3개 지정해 '순서대로' 이동을 선언합니다. 선언은 상대의 다음 수 뒤에 반영되며, 그후 해당 턴에는 기물을 잡을 수 없습니다. | [L38123](origin_code/main-DsoigPgV.js#L38123) |
| `proficiency` | 숙련 | 2.5 | PASSIVE | `proficiency` / — | 내 폰이 프로모션 직전 랭크에 있다면 전진으로 적을 잡을 수 있습니다. | [L3823](origin_code/main-DsoigPgV.js#L3823) |
| `promotion-rush` | 승진 체험 | 3 | ACTIVE | `promotionRush` / own-promotion-rush-piece | 아군 비폰 기물 하나를 지정합니다. 해당 기물은 이번 턴 첫 이동 때 퀸처럼도 움직일 수 있지만, 기물을 잡을 수 없습니다. | [L38227](origin_code/main-DsoigPgV.js#L38227) |
| `quantum-mechanics` | 양자 역학 | 2.5 | ACTIVE | `quantumMechanics` / — | 이번 턴에 이동하는 기물의 위치가 무작위 도착지 한 곳과 중첩됩니다. 단, 해당 이동으로는 기물을 잡을 수 없습니다. | [L39273](origin_code/main-DsoigPgV.js#L39273) |
| `queen-afterimage` | 잔상 | 3.5 | PASSIVE | `queenAfterimage` / — | 아군 퀸으로 상대방 퀸을 잡을 경우 이동 전 위치에 퀸을 하나 더 소환합니다. | [L39237](origin_code/main-DsoigPgV.js#L39237) |
| `radical-charge` | 난폭한 돌진 | 4.5 | PASSIVE | `radicalCharge` / — | 아군 나이트가 상하좌우의 적 기물을 뛰어넘으면 그 기물을 자동으로 잡습니다. | [L38531](origin_code/main-DsoigPgV.js#L38531) |
| `random-roulette` | 랜덤룰렛 | 3.5 | ACTIVE | `randomRoulette` / any-major-piece | 아군 혹은 상대방 메이저 피스를 하나 지정합니다. 해당 기물을 변형 기물을 포함한 모든 기물 중 하나로 변경합니다. | [L38113](origin_code/main-DsoigPgV.js#L38113) |
| `relay` | 교대 | 2.5 | ACTIVE | `relay` / — | 이번 턴 동안 아군 기물이 턴을 소모하여 같은 행 또는 열의 아군과 위치를 바꿀 수 있습니다. | [L37772](origin_code/main-DsoigPgV.js#L37772) |
| `religious-victory` | 종교 승리 | 3.5 | PASSIVE | `religiousVictory` / — | 비숍이 상대방보다 3개 더 많으면 즉시 승리합니다. | [L38881](origin_code/main-DsoigPgV.js#L38881) |
| `reposition` | 재배치 | 2.5 | ACTIVE | `reposition` / — | 사용할 경우 이번 턴에 기물을 잡지 못하는 대신 같은 기물을 2번 움직일수 있습니다. | [L39291](origin_code/main-DsoigPgV.js#L39291) |
| `resolve` | 결의 | 3.5 | PASSIVE | `resolve` / — | 아군 폰이 잡힌 턴에 가장 먼저 움직인 폰은 첫번째 이동에 한해 턴을 소모하지 않습니다. | [L958](origin_code/main-DsoigPgV.js#L958) |
| `retreat` | 퇴각 | 2.5 | PASSIVE | `retreat` / — | 아군 폰이 뒤로도 움직일 수 있습니다. | [L38013](origin_code/main-DsoigPgV.js#L38013) |
| `rook-lift` | 룩 리프트 | 3.5 | PASSIVE | `rookLift` / — | 룩이 체스판 맨 구석 4칸에 도착하면 한번 더 이동할 수 있습니다. | [L39011](origin_code/main-DsoigPgV.js#L39011) |
| `royal-command` | 왕명 | 3 | ACTIVE | `royalCommand` / — | 다음 자기 턴부터 2수동안 모든 아군 기물이 킹처럼 이동할 수 있습니다. | [L39133](origin_code/main-DsoigPgV.js#L39133) |
| `royal-shield` | 가호 | 3 | ACTIVE | `royalShield` / — | 무작위 아군 기물 하나에게 가호를 부여합니다. | [L38927](origin_code/main-DsoigPgV.js#L38927) |
| `rule-ticket` | 규칙 티켓 | 2.5 | ACTIVE | `ruleTicket` / — | 활성화 시 원하는 규칙 하나를 추가할 수 있습니다. 규칙은 다음 내 턴부터 반영됩니다. | [L39209](origin_code/main-DsoigPgV.js#L39209) |
| `sacrifice` | 희생 | 3.5 | ACTIVE | `sacrifice` / own-sacrifice-piece | 아군 기물 하나를 처형하고, 그보다 가치가 낮은 기물 하나를 선택해 3턴동안 보호를 부여합니다. | [L38190](origin_code/main-DsoigPgV.js#L38190) |
| `scarecrow` | 허수아비 | 3.5 | ACTIVE | `scarecrow` / scarecrow-square | 원하는 칸에 3수 뒤 허수아비를 설치합니다. | [L3098](origin_code/main-DsoigPgV.js#L3098) |
| `schrodinger-pawns` | 슈뢰딩거의 폰 | 3.5 | ACTIVE | `schrodingerPawns` / — | 무작위 아군 폰 4개가 바로 뒷자리와 중첩됩니다. 해당 폰들은 이번 턴에 기물을 잡을 수 없습니다. | [L39282](origin_code/main-DsoigPgV.js#L39282) |
| `severance` | 절단 | 3 | ACTIVE | `severance` / enemy-ranged | 상대 원거리 기물 중 하나를 선택합니다. 해당 기물은 다음 2수 동안 한 칸씩만 이동할 수 있습니다. | [L3550](origin_code/main-DsoigPgV.js#L3550) |
| `snipe` | 저격 | 3 | ACTIVE | `bishopSnipe` / — | 이번 턴 동안 비숍이 기물 하나를 뛰어넘어 공격할 수 있습니다. | [L38964](origin_code/main-DsoigPgV.js#L38964) |
| `socialism` | 사회주의 | 4 | ACTIVE | `socialism` / — | 다음 턴에 킹을 제외한 상대의 모든 기물은 원래 행마를 잃고 폰처럼 움직입니다. | [L38626](origin_code/main-DsoigPgV.js#L38626) |
| `stealth` | 은신 | 4 | ACTIVE | `stealth` / own-bishop | 아군 비숍 하나를 지정합니다. 해당 비숍은 상대에게 보이지 않습니다. | [L3531](origin_code/main-DsoigPgV.js#L3531) |
| `submerge` | 잠복 | 2 | ACTIVE | `submerge` / own-submerge-piece | 아군 기물 하나를 선택해 잠복시킵니다. | [L3535](origin_code/main-DsoigPgV.js#L3535) |
| `suicide-bomber` | 자폭병 | 3.5 | ACTIVE | `suicideBomber` / own-pawn-or-fanatic | 폰 혹은 광신도 하나에게 잡힐 경우 3x3 반경으로 폭발하는 특성을 부여합니다. 폭발은 아군도 제거합니다. | [L38416](origin_code/main-DsoigPgV.js#L38416) |
| `suspicious-potion` | 수상한 물약 | 3 | ACTIVE | `suspiciousPotion` / any-potion-piece | 아군 혹은 적 기물 하나에 완전한 무작위 효과 하나를 부여합니다. | [L37773](origin_code/main-DsoigPgV.js#L37773) |
| `switcheroo` | 바꿔치기 | 1.5 | ACTIVE | `switcheroo` / — | 이번 턴 동안 킹이 턴을 소모하여 아군 폰의 위치로 순간이동할 수 있으며, 해당 폰은 제거됩니다. | [L1982](origin_code/main-DsoigPgV.js#L1982) |
| `symmetry` | 대칭 | 3.5 | ACTIVE | `symmetry` / — | 이번 턴 동안 아군 기물이 턴을 소모하여 중앙선을 기준으로 한 좌우대칭점으로 이동할 수 있습니다. | [L3977](origin_code/main-DsoigPgV.js#L3977) |
| `taunt` | 도발 | 2.5 | ACTIVE | `taunt` / — | 모든 상대방 기물이 이번 턴에 뒤로 갈 수 없습니다. | [L38293](origin_code/main-DsoigPgV.js#L38293) |
| `trojan-horse` | 트로이 목마 | 3.5 | ACTIVE | `trojanHorse` / own-knight | 아군 나이트 하나를 비밀리에 지정합니다. 해당 나이트가 잡힐 경우 공격한 기물을 즉시 되잡으며 아군 폰을 생성합니다. | [L38162](origin_code/main-DsoigPgV.js#L38162) |
| `trolley` | 트롤리 | 5 | ACTIVE | `trolley` / — | 사용 시 다음 턴에 상대방에게 트롤리를 보냅니다. 상대는 큰 딜레마에 빠질 겁니다... | [L39264](origin_code/main-DsoigPgV.js#L39264) |
| `twins` | 환상의 콤비 | 3 | ACTIVE | `twins` / own-twin-pair | 아군 기물 둘을 연결하고, 둘 중 하나가 이동할 때마다 서로 위치를 변경합니다. | [L39426](origin_code/main-DsoigPgV.js#L39426) |
| `vortex` | 소용돌이 | 3 | ACTIVE | `vortex` / — | 킹과 폰을 제외한 모든 적 기물의 위치를 서로 뒤섞습니다. | [L39522](origin_code/main-DsoigPgV.js#L39522) |
| `white-box` | 하얀 상자 | 3 | PASSIVE | `whiteBox` / — | 획득 시 무작위 패시브 증강 카드 하나를 발동합니다. | [L39246](origin_code/main-DsoigPgV.js#L39246) |
| `witch-trial` | 마녀재판 | 3 | ACTIVE | `witchTrial` / enemy-piece | 킹을 제외한 상대 기물 하나를 의심합니다. 3턴 안에 아군 기물을 잡지 못하면 제거되지만, 잡으면 가호를 얻습니다. | [L38973](origin_code/main-DsoigPgV.js#L38973) |

### END — 56개

| ID | 이름 | 별 | 활성화 | effect / target | 설명 | 정의 출처 |
|---|---|---:|---|---|---|---|
| `baby-bear` | 아기곰 | 4 | ACTIVE | `babyBear` / own-queen | 퀸을 희생한 뒤 그 자리에 아기곰을 소환합니다. 아기곰은 7수 뒤 곰으로 변화합니다. | [L39494](origin_code/main-DsoigPgV.js#L39494) |
| `barricade` | 엄폐물 설치 | 2.5 | ACTIVE | `barricade` / empty | 빈 칸을 골라 가로 또는 세로 3칸짜리 엄폐물을 설치합니다. | [L38358](origin_code/main-DsoigPgV.js#L38358) |
| `bina-mate` | 더블 체크 | 3.5 | PASSIVE | `binaMate` / — | 서로 다른 2개 이상의 아군 기물이 동시에 상대 킹을 공격하면 즉시 승리합니다. | [L38899](origin_code/main-DsoigPgV.js#L38899) |
| `black-magic` | 흑마법 | 4 | ACTIVE | `blackMagic` / — | 사용시 내 킹 옆에 아군 측 괴물을 소환합니다. | [L1986](origin_code/main-DsoigPgV.js#L1986) |
| `blue-jeans` | 청바지 | 5 | PASSIVE | `blueJeans` / — | 획득 시 모든 아군과 적군의 메이저 피스를 게임에서 추방합니다. | [L38320](origin_code/main-DsoigPgV.js#L38320) |
| `bribe` | 삼일천하 | 4 | ACTIVE | `bribe` / own-knight | 나이트 하나를 3턴 동안 아마존으로 변경합니다. 시간이 지나면 다시 나이트가 됩니다. | [L38426](origin_code/main-DsoigPgV.js#L38426) |
| `brutus` | 브루투스 | 4.5 | PASSIVE | `brutus` / — | 획득시 무작위 아군 룩 하나를 브루투스로 변경합니다. | [L650](origin_code/main-DsoigPgV.js#L650) |
| `canceling` | 캔슬링 | 5 | ACTIVE | `canceling` / — | 모든 상황을 무시하고 상대 킹을 즉시 시작위치로 돌려보냅니다. 단, 상대가 캐슬링하지 않았다면 사용할 수 없습니다. | [L38955](origin_code/main-DsoigPgV.js#L38955) |
| `chain` | 사슬 | 3 | ACTIVE | `chain` / enemy-chain-pair | 서로 2칸 안에 있는 상대 기물 둘을 지정합니다. 두 기물이 모두 살아있을 경우 서로 2칸보다 멀어질 수 없습니다. | [L38246](origin_code/main-DsoigPgV.js#L38246) |
| `charge` | 마지막 질주 | 4 | ACTIVE | `charge` / own-pawn | 아군 폰을 하나 선택합니다. 해당 폰은 이번 턴에 최대 3칸 전진할 수 있으며, 프로모션하지 못할 경우 사망합니다. | [L39123](origin_code/main-DsoigPgV.js#L39123) |
| `cleanup-sacrifice` | 강제 교환 | 1.5 | ACTIVE | `cleanupSacrifice` / own-cleanup-sacrifice | 아군 기물 하나를 지정해 희생합니다. 해당 기물과 같은 종류의 무작위 상대 기물이 즉시 사망합니다. | [L39368](origin_code/main-DsoigPgV.js#L39368) |
| `clone` | 복제 | 5 | PASSIVE | `clonePassive` / — | 상대가 이미 가진 패시브 카드와 이후 획득하는 패시브 카드의 효과를 모두 사용할 수 있습니다. | [L38200](origin_code/main-DsoigPgV.js#L38200) |
| `collapse` | 붕괴 | 4.5 | ACTIVE | `collapse` / — | 다음 턴에 양 끝쪽 파일과 랭크가 붕괴합니다. 프로모션 랭크는 각 진영의 끝에서 두 번째 랭크가 됩니다. | [L38635](origin_code/main-DsoigPgV.js#L38635) |
| `conscription` | 징집 | 4 | ACTIVE | `conscription` / — | 아군 두 번째 랭크의 빈 중앙 4개 파일에 폰을 즉시 소환합니다. | [L38339](origin_code/main-DsoigPgV.js#L38339) |
| `coronation` | 대관식 | 3.5 | PASSIVE | `coronation` / — | 퀸으로 프로모션하는 모든 아군 기물이 다음 자기 턴이 돌아올 때까지 보호를 얻습니다. | [L39086](origin_code/main-DsoigPgV.js#L39086) |
| `death-squad` | 결사대 | 2.5 | ACTIVE | `deathSquad` / own-pawn-file | 선택한 파일에 존재하는 아군 폰들을 전부 광신도로 변환합니다. | [L39339](origin_code/main-DsoigPgV.js#L39339) |
| `desperado` | 데스페라도 | 3.5 | ACTIVE | `desperado` / own-desperado-piece | 아군 기물 하나를 선택해 두 번 움직입니다. 이동이 끝나면 해당 기물은 사망하며, 이 상태에서는 상대 킹을 잡을 수 없습니다. | [L39349](origin_code/main-DsoigPgV.js#L39349) |
| `empty-lunchbox` | 빈 찬합 | 3.5 | ACTIVE | `emptyLunchbox` / enemy-empty-lunchbox-piece | 킹을 제외한 상대 기물을 하나 지정합니다. 다음 3수 안에 대상이 자기 킹과 인접하지 못하면 자결합니다. | [L37788](origin_code/main-DsoigPgV.js#L37788) |
| `exile` | 유배 | 2.5 | ACTIVE | `exile` / enemy-exile | 상대 기물 하나를 시작 위치로 되돌립니다. 킹에게는 사용할 수 없습니다. | [L38917](origin_code/main-DsoigPgV.js#L38917) |
| `extinction` | 멸종 | 4 | ACTIVE | `extinction` / own-extinction | 아군 마이너 피스 하나를 선택합니다. 해당 기물과 동일한 종류의 모든 기물을 진영을 가리지 않고 삭제합니다. | [L3826](origin_code/main-DsoigPgV.js#L3826) |
| `final-weapon` | 최종병기 | 3 | PASSIVE | `finalWeapon` / — | 아군의 프로모션 랭크가 마지막 랭크로 최종 확정됩니다. 폰이 끝까지 도달할 경우 아마존으로 승진할 수 있습니다. | [L39114](origin_code/main-DsoigPgV.js#L39114) |
| `fleeting-dream` | 일장춘몽 | 4 | ACTIVE | `fleetingDream` / — | 사용시 상대방의 모든 프로모션 기물을 폰으로 되돌립니다. | [L37787](origin_code/main-DsoigPgV.js#L37787) |
| `frenzy` | 광란 | 4 | ACTIVE | `frenzy` / own-frenzy-pawn | 아군 폰 하나를 선택합니다. 해당 폰은 이번 턴에 기물을 잡을 때마다 한번 더 이동할 수 있습니다. | [L3493](origin_code/main-DsoigPgV.js#L3493) |
| `gomoku` | 오목 | 4 | PASSIVE | `gomoku` / — | 이 카드를 획득한 상태에서 아군 기물이 세로로 5개 세워지는 순간 게임에서 승리합니다. | [L37781](origin_code/main-DsoigPgV.js#L37781) |
| `hallucination` | 환각 | 2 | ACTIVE | `hallucination` / — | 5수동안 모든 아군 기물이 상대방에게 퀸으로 표시됩니다. | [L38311](origin_code/main-DsoigPgV.js#L38311) |
| `highlander` | 하이랜더 | 5 | PASSIVE | `highlander` / — | 폰을 포함해 아군 진영에 중복되는 기물이 하나도 없을 경우 즉시 승리합니다. | [L3819](origin_code/main-DsoigPgV.js#L3819) |
| `homecoming` | 귀환 | 3 | ACTIVE | `homecoming` / own-homecoming-piece | 시작 위치가 비어있는 아군 비킹 기물 하나를 시작점으로 되돌립니다. 귀환 후 바로 기물을 잡을 수 있습니다. | [L39407](origin_code/main-DsoigPgV.js#L39407) |
| `hypocrisy` | 위선 | 4 | ACTIVE | `hypocrisy` / hypocrisy-squares | 원하는 빈 위치 4곳에 적 폰을 소환합니다. | [L37771](origin_code/main-DsoigPgV.js#L37771) |
| `icbm` | ICBM | 2 | ACTIVE | `icbm` / own-queen | 아군 퀸을 하나 지정합니다. 다음 내 턴이 시작될 때 상대 퀸에게 날아가 주위 3x3 구역을 폭발시킵니다. | [L38133](origin_code/main-DsoigPgV.js#L38133) |
| `joker` | 조커 | 4.5 | ACTIVE | `joker` / — | 이미 사용한 액티브 카드 하나를 다시 사용할 수 있게 되돌립니다. | [L38209](origin_code/main-DsoigPgV.js#L38209) |
| `judgment` | 레드카드 | 3 | ACTIVE | `judgment` / judgment-piece | 진영을 불문하고 게임에서 가장 많은 기물을 잡은 비킹 기물 중 하나를 선택해 다음 드래프트까지 추방합니다. | [L39436](origin_code/main-DsoigPgV.js#L39436) |
| `last-resistance` | 마지막 저항 | 3.5 | ACTIVE | `lastResistance` / — | 아군 킹이 3턴동안 보호를 얻습니다. | [L1985](origin_code/main-DsoigPgV.js#L1985) |
| `lobster` | 랍스터 | 3 | ACTIVE | `lobster` / empty-lobster-square | 원하는 빈 공간에 한 수 뒤 랍스터를 소환합니다. | [L1457](origin_code/main-DsoigPgV.js#L1457) |
| `moving` | 무빙 | 4 | PASSIVE | `moving` / — | 같은 아군 기물을 5번 연속 움직이면 해당 기물이 회피를 얻습니다. | [L39359](origin_code/main-DsoigPgV.js#L39359) |
| `mutation` | 변이 | 2.5 | PASSIVE | `mutation` / — | 아군 폰들이 마지막 랭크에 도달할 경우 괴물로 프로모션할 수 있습니다. 해당 괴물은 아군으로 취급됩니다. | [L3980](origin_code/main-DsoigPgV.js#L3980) |
| `necromancy` | 빙의 | 3.5 | ACTIVE | `necromancy` / own-necromancy-pawn | 아군 폰 하나를 선택해 4수동안 이미 제거된 무작위 아군 기물 중 하나로 변경합니다. 시간이 지나면 다시 폰이 됩니다. | [L39329](origin_code/main-DsoigPgV.js#L39329) |
| `othello` | 오델로 | 3.5 | ACTIVE | `othello` / enemy-othello | 연속한 아군 기물 2개 사이에 끼인 상대 기물을 하나 지정해 아군 기물로 전향시킵니다. | [L39319](origin_code/main-DsoigPgV.js#L39319) |
| `palace` | 궁성 | 3.5 | ACTIVE | `palace` / — | 상대 킹을 중심으로 3x3 크기의 궁성을 짓습니다. 상대 킹은 직접 움직여 궁성을 나갈 수 없습니다. | [L38608](origin_code/main-DsoigPgV.js#L38608) |
| `pegasus` | 유니콘 | 3.5 | ACTIVE | `pegasus` / own-rook | 룩 하나를 유니콘으로 교환합니다. | [L1436](origin_code/main-DsoigPgV.js#L1436) |
| `portal-gun` | 포탈 건 | 4 | ACTIVE | `portalGun` / portal-square | 체스판의 두 칸을 선택하여 다음 자기 턴에 포탈을 설치합니다. | [L37811](origin_code/main-DsoigPgV.js#L37811) |
| `prophecy` | 종전 | 4.5 | ACTIVE | `prophecy` / — | 다음 3수동안 체스보드 안의 그 어떤 기물도 잡히지 않으면 게임에서 즉시 승리합니다. | [L38653](origin_code/main-DsoigPgV.js#L38653) |
| `racing-king` | 레이싱 킹 | 3 | PASSIVE | `racingKing` / — | 아군 킹이 반대편 끝 랭크에 도달하면 즉시 승리합니다. | [L1981](origin_code/main-DsoigPgV.js#L1981) |
| `recurrence` | 회귀 | 3.5 | ACTIVE | `recurrence` / — | 무작위 아군 비폰 기물 하나에게 회귀를 부여합니다. | [L940](origin_code/main-DsoigPgV.js#L940) |
| `replay` | 리플레이 | 4 | ACTIVE | `replayMove` / — | 사용 즉시 자신의 마지막 기물 이동을 취소합니다. | [L37769](origin_code/main-DsoigPgV.js#L37769) |
| `reversal` | 반전 | 4 | ACTIVE | `reversal` / — | 사용한 턴에 아군 룩과 비숍의 행마가 뒤바뀝니다. | [L978](origin_code/main-DsoigPgV.js#L978) |
| `reverse-pawns` | 폰 방향 반전 | 2.5 | ACTIVE | `reversePawns` / — | 상대 폰의 전진 방향을 세 번의 이동 동안 반대로 바꿉니다. | [L38302](origin_code/main-DsoigPgV.js#L38302) |
| `spy` | 스파이 | 3 | ACTIVE | `spy` / enemy-pawn | 상대 폰을 최대 2개까지 비밀리에 지정합니다. 그 폰들은 프로모션하면 내 기물이 됩니다. | [L38329](origin_code/main-DsoigPgV.js#L38329) |
| `stake` | 말뚝 | 2.5 | ACTIVE | `stake` / own-piece | 아군 기물 하나를 지정합니다. 해당 기물은 4수 동안 움직일 수 없으며, 4수 뒤 가호를 얻습니다. | [L3539](origin_code/main-DsoigPgV.js#L3539) |
| `substitution` | 치환 | 3 | ACTIVE | `substitution` / — | 이번 턴 동안 아군 기물이 턴을 소모하여 같은 종류의 상대 말과 위치를 바꿀 수 있습니다. | [L38237](origin_code/main-DsoigPgV.js#L38237) |
| `traitor` | 변절자 | 4 | ACTIVE | `traitor` / — | 폰을 포함해 상대의 가장 약한 기물 하나를 내 편으로 전향시킵니다. | [L39310](origin_code/main-DsoigPgV.js#L39310) |
| `ultimatum` | 최후 통첩 | 4.5 | ACTIVE | `ultimatum` / — | 4수 뒤, 킹을 제외하고 마지막 4수 동안 한 번도 움직이지 않은 모든 기물이 제거됩니다. | [L38644](origin_code/main-DsoigPgV.js#L38644) |
| `underground-bunker` | 지하벙커 | 2.5 | PASSIVE | `undergroundBunker` / — | 다음 턴부터 아군 킹에게 5만큼의 HP를 부여합니다. 더이상 킹을 움직일 수 없습니다. | [L1984](origin_code/main-DsoigPgV.js#L1984) |
| `underpromotion` | 언더프로모션 | 3 | PASSIVE | `underpromotion` / — | 폰이 마이너 피스로 승진한다면 승진한 기물을 즉시 움직일 수 있습니다. | [L39095](origin_code/main-DsoigPgV.js#L39095) |
| `vanish` | 소멸 | 4.5 | ACTIVE | `vanish` / — | 사용한 뒤 턴이 돌아올때마다 해당 플레이어의 기물이 무작위로 하나씩 사라집니다. | [L38218](origin_code/main-DsoigPgV.js#L38218) |
| `vip` | 귀빈 | 3 | ACTIVE | `vip` / any-vip-pawn | 아군 혹은 상대방 폰 하나를 지정합니다. 3수 뒤 해당 폰이 살아있다면 해당 플레이어의 턴에 귀빈으로 변화합니다. | [L39484](origin_code/main-DsoigPgV.js#L39484) |
| `zugzwang` | 추크츠방 | 3 | ACTIVE | `zugzwang` / — | 이번 턴에 상대방은 반드시 자신의 킹을 움직여야 합니다. 상대 킹이 물리적으로 갇혀있다면 사용할 수 없습니다. | [L39475](origin_code/main-DsoigPgV.js#L39475) |

### PIECE — 34개

| ID | 이름 | 별 | 활성화 | effect / target | 설명 | 정의 출처 |
|---|---|---:|---|---|---|---|
| `amazon` | 아마존 | 4.5 | ACTIVE | `amazon` / own-queen | 나이트 하나를 희생해 퀸을 아마존으로 변환합니다. | [L1434](origin_code/main-DsoigPgV.js#L1434) |
| `assassin` | 암살자 | 3 | ACTIVE | `assassin` / own-knight | 나이트 하나를 선택해 암살자로 변경시킵니다. | [L1444](origin_code/main-DsoigPgV.js#L1444) |
| `berserker` | 버서커 | 3 | ACTIVE | `berserker` / own-rook | 아군 룩 하나를 버서커로 변경합니다. | [L1462](origin_code/main-DsoigPgV.js#L1462) |
| `campfire` | 캠프파이어 | 4 | ACTIVE | `campfire` / own-rook | 아군 룩 하나를 지정해 캠프파이어로 변화시킵니다. | [L904](origin_code/main-DsoigPgV.js#L904) |
| `clockwork` | 태엽인형 | 4 | ACTIVE | `clockwork` / own-minor | 마이너 피스 하나를 선택해 태엽인형으로 변경합니다. | [L651](origin_code/main-DsoigPgV.js#L651) |
| `constitutional-monarchy` | 입헌군주제 | 2 | ACTIVE | `constitutionalMonarchy` / own-queen | 아군 퀸을 국무총리로 변경합니다. | [L37994](origin_code/main-DsoigPgV.js#L37994) |
| `dragon` | 드래곤 | 3.5 | ACTIVE | `dragon` / own-rook | 룩 하나를 선택하여 드래곤으로 변화시킵니다. | [L1442](origin_code/main-DsoigPgV.js#L1442) |
| `eagle` | 알리바바 | 3.5 | ACTIVE | `eagle` / — | 모든 아군 나이트를 개당 알리바바 2개로 교환합니다. | [L1433](origin_code/main-DsoigPgV.js#L1433) |
| `eastern-policy` | 동방견문록 | 3.5 | ACTIVE | `easternPolicy` / own-minor | 아군 마이너 기물 하나를 포로 변경합니다. | [L38406](origin_code/main-DsoigPgV.js#L38406) |
| `grasshopper` | 그래스호퍼 | 3.5 | ACTIVE | `grasshopper` / own-minor | 마이너 피스 하나를 선택해 그래스호퍼로 변화시킵니다. | [L1441](origin_code/main-DsoigPgV.js#L1441) |
| `hedgehog` | 고슴도치 | 3.5 | ACTIVE | `hedgehog` / own-queen | 퀸을 고슴도치로 변경합니다. | [L922](origin_code/main-DsoigPgV.js#L922) |
| `herald` | 전령 | 3.5 | ACTIVE | `herald` / own-rook | 룩 하나를 선택해 전령으로 교체합니다. | [L1429](origin_code/main-DsoigPgV.js#L1429) |
| `hook` | 구행 | 4.5 | ACTIVE | `hook` / own-queen | 퀸과 룩을 1개씩 선택합니다. 선택한 룩을 희생하고 퀸이 구행이 됩니다. | [L1440](origin_code/main-DsoigPgV.js#L1440) |
| `idol` | 아이돌 | 4 | ACTIVE | `idol` / own-queen | 아군 퀸을 아이돌로 변경합니다. | [L1456](origin_code/main-DsoigPgV.js#L1456) |
| `jester` | 광대 | 2 | ACTIVE | `jester` / own-queen | 아군 퀸을 광대로 변경합니다. | [L1437](origin_code/main-DsoigPgV.js#L1437) |
| `knightmaster` | 기사단장 | 2.5 | ACTIVE | `knightmaster` / own-knight | 아군 나이트 하나를 기사단장으로 변경합니다. | [L1446](origin_code/main-DsoigPgV.js#L1446) |
| `local-conscription` | 현지군 징병 | 4 | ACTIVE | `localConscription` / own-queen | 아군 퀸을 징집관으로 교체합니다. | [L38348](origin_code/main-DsoigPgV.js#L38348) |
| `log` | 통나무 | 2 | ACTIVE | `log` / own-pawn | 폰 하나를 선택해 통나무로 교체합니다. | [L1439](origin_code/main-DsoigPgV.js#L1439) |
| `magic-girl` | 마법소녀 | 3.5 | ACTIVE | `magicGirl` / own-rook | 아군 룩 하나를 마법소녀로 변경합니다. | [L37776](origin_code/main-DsoigPgV.js#L37776) |
| `missionary` | 선교사 | 3 | ACTIVE | `missionary` / own-plain-bishop | 비숍 하나를 선택해 선교사 2개로 변경합니다. | [L1425](origin_code/main-DsoigPgV.js#L1425) |
| `octopus` | 문어 | 3.5 | ACTIVE | `octopus` / own-rook | 룩 하나를 선택해 문어로 변경합니다. | [L2901](origin_code/main-DsoigPgV.js#L2901) |
| `ordination` | 서품 | 2 | ACTIVE | `ordination` / own-plain-bishop | 비숍 하나를 선택합니다. 남은 비숍을 희생하고 선택한 비숍을 추기경으로 변경합니다. | [L38502](origin_code/main-DsoigPgV.js#L38502) |
| `paladin` | 팔라딘 | 3 | ACTIVE | `paladin` / own-knight | 나이트 하나를 선택해 팔라딘으로 변경합니다. | [L653](origin_code/main-DsoigPgV.js#L653) |
| `parrot` | 앵무새 | 3.5 | ACTIVE | `parrot` / own-rook | 아군 룩 하나를 선택해 앵무새로 변경합니다. | [L652](origin_code/main-DsoigPgV.js#L652) |
| `reaper` | 사신 | 3.5 | ACTIVE | `reaper` / own-queen | 아군 퀸을 사신으로 변경합니다. | [L1445](origin_code/main-DsoigPgV.js#L1445) |
| `reformation` | 종교 개혁 | 3 | ACTIVE | `reformation` / — | 모든 아군 비숍을 프로테스탄트로 변경합니다. | [L37930](origin_code/main-DsoigPgV.js#L37930) |
| `siege-ram` | 공성추 | 2.5 | ACTIVE | `siegeRam` / own-rook | 아군 룩 하나를 공성추로 변경합니다. | [L37768](origin_code/main-DsoigPgV.js#L37768) |
| `siren` | 세이렌 | 3 | ACTIVE | `siren` / own-queen | 아군 퀸 하나를 세이렌으로 변경합니다. | [L1464](origin_code/main-DsoigPgV.js#L1464) |
| `slime` | 슬라임 | 3.5 | ACTIVE | `slime` / own-rook | 아군 룩 하나를 슬라임으로 변경합니다. | [L1463](origin_code/main-DsoigPgV.js#L1463) |
| `standard-bearer` | 기수 | 2.5 | ACTIVE | `standardBearer` / own-pawn | 아군 폰 하나를 기수로 변경합니다. | [L39039](origin_code/main-DsoigPgV.js#L39039) |
| `trickster` | 트릭스터 | 4 | ACTIVE | `trickster` / own-rook | 아군 룩 하나를 트릭스터로 변경합니다. | [L1465](origin_code/main-DsoigPgV.js#L1465) |
| `undead` | 언데드 | 4 | ACTIVE | `undead` / own-queen | 아군 퀸 하나를 언데드로 변경합니다. | [L2977](origin_code/main-DsoigPgV.js#L2977) |
| `windmill` | 풍차 | 1.5 | ACTIVE | `windmill` / windmill-pair | 비숍과 룩 하나를 선택해 풍차 2개로 변경합니다. | [L1455](origin_code/main-DsoigPgV.js#L1455) |
| `wizard` | 마법사 | 5 | ACTIVE | `wizard` / own-queen | 퀸을 마법사로 대체합니다. 마법사는 마법을 사용할 수 있습니다. | [L1453](origin_code/main-DsoigPgV.js#L1453) |

### RULE — 27개

| ID | 이름 | 별 | 활성화 | effect / target | 설명 | 정의 출처 |
|---|---|---:|---|---|---|---|
| `acceleration` | 가속 | — | MATCH_RULE | `acceleration` / — | 적용 뒤 세번째 흑 차례부터 모든 플레이어가 한 턴에 2번 행동합니다. | [L38579](origin_code/main-DsoigPgV.js#L38579) |
| `black-hole` | 블랙홀 | — | MATCH_RULE | `blackHole` / — | 중앙 2x2 구역에 블랙홀이 생성됩니다. 블랙홀 위치에 도착한 기물은 즉시 사망합니다. | [L3688](origin_code/main-DsoigPgV.js#L3688) |
| `camouflage-color` | 위장색 | — | MATCH_RULE | `camouflageRule` / — | 킹을 제외한 모든 기물이 자기 색과 같은 색 타일에 섰을때 은신합니다. | [L38844](origin_code/main-DsoigPgV.js#L38844) |
| `capture-the-flag` | 깃발 뽑기 | — | MATCH_RULE | `captureTheFlag` / — | 양측 아군 홈 랭크에 랜덤으로 깃발 칸이 생깁니다. 적 기물이 깃발 칸에 들어온 뒤 1턴이 지나면 패배합니다. | [L1015](origin_code/main-DsoigPgV.js#L1015) |
| `chess-344200` | 344200 체스 | — | MATCH_RULE | `chess344200` / — | 폰을 포함한 모든 양쪽 기물을 무작위로 재배치합니다. 킹은 항상 홈 랭크에 배치됩니다. | [L37894](origin_code/main-DsoigPgV.js#L37894) |
| `chess-960` | 960 체스 | — | MATCH_RULE | `chess960` / — | 폰을 제외한 양쪽 기물을 무작위로 재배치합니다. | [L37876](origin_code/main-DsoigPgV.js#L37876) |
| `chess-n-pow-30` | 혼돈의 체스 | — | MATCH_RULE | `chessNPow30` / — | 킹을 제외한 모든 기물을 랜덤하게 바꾼 뒤 시작합니다. 첫 2수동안은 기물을 잡을 수 없습니다. | [L38835](origin_code/main-DsoigPgV.js#L38835) |
| `conveyor` | 컨베이어 | — | MATCH_RULE | `conveyorRule` / — | 체스판 가장자리에 컨베이어를 설치합니다. 컨베이어 위에 있는 기물은 매 수마다 시계방향으로 한 칸씩 이동합니다. | [L38727](origin_code/main-DsoigPgV.js#L38727) |
| `cool-guy` | 매너 | — | MATCH_RULE | `coolGuy` / — | 한번 말을 잡은 기물은 다시 평범한 이동을 하기 전까진 더이상 기물을 잡을 수 없습니다. | [L37903](origin_code/main-DsoigPgV.js#L37903) |
| `crown` | 왕관 | — | MATCH_RULE | `crownRule` / — | 중앙 4칸 중 한 칸에 왕관을 추가합니다. 한쪽이 왕관을 10수 이상 들고 있다면 승리합니다. | [L2920](origin_code/main-DsoigPgV.js#L2920) |
| `diagonal-chess` | 대각선 체스 | — | MATCH_RULE | `diagonalChess` / — | 대각선 배치로 시작합니다. 프로모션이 조금 더 쉬워지겠네요. | [L38826](origin_code/main-DsoigPgV.js#L38826) |
| `football` | 축구공 | — | MATCH_RULE | `football` / — | 게임 중앙 4칸 중 한 칸에 축구공을 추가합니다. | [L3685](origin_code/main-DsoigPgV.js#L3685) |
| `high-ground` | 고지전 | — | MATCH_RULE | `highGround` / — | 보드 중앙부 중 무작위 8칸이 고지로 선정됩니다. | [L39504](origin_code/main-DsoigPgV.js#L39504) |
| `highway` | 고속도로 | — | MATCH_RULE | `highway` / — | 체스판 양쪽 두 번째 파일에 고속도로를 설치합니다. | [L39513](origin_code/main-DsoigPgV.js#L39513) |
| `macho-chess` | 상남자 모드 | — | MATCH_RULE | `machoChess` / — | 기물들이 뒤로 갈 수 없으며, 옆으로는 잡으면서만 이동할 수 있습니다. 킹이 보드 반대편에 도달하면 승리하고 앙파상이 강제입니다. | [L38754](origin_code/main-DsoigPgV.js#L38754) |
| `mistake` | 실수 | — | MATCH_RULE | `mistake` / — | 기물이 상대 말을 잡을때 20% 확률로 실수합니다. 실수할 경우 반대로 아군 기물이 잡히게 됩니다. | [L38799](origin_code/main-DsoigPgV.js#L38799) |
| `monochrome-chess` | 단색 체스 | — | MATCH_RULE | `monochromeChess` / — | 기물이 다른 색 칸으로 이동할 수 없습니다. 나이트가 낙타로 변경됩니다. | [L37885](origin_code/main-DsoigPgV.js#L37885) |
| `monster` | 괴물 | — | MATCH_RULE | `monsterRule` / — | 중앙 4칸 중 무작위 위치에 괴물을 소환합니다. 괴물은 매 수 무작위 방향으로 한 칸 움직입니다. | [L3686](origin_code/main-DsoigPgV.js#L3686) |
| `periodic-collapse` | 붕괴 | — | MATCH_RULE | `periodicCollapse` / — | 20수마다 보드의 가장 바깥쪽 한 겹이 붕괴합니다. | [L38781](origin_code/main-DsoigPgV.js#L38781) |
| `platform` | 발판 | — | MATCH_RULE | `platformRule` / — | 5턴마다 무작위 빈칸 하나가 발판으로 지정됩니다. 발판을 밟은 기물은 한 번 더 움직일 수 있습니다. | [L37790](origin_code/main-DsoigPgV.js#L37790) |
| `portal` | 포탈 | — | MATCH_RULE | `portal` / — | 보드에 서로 연결된 포탈을 설치합니다. 포탈은 타고 이동하거나 한번에 통과할 수 있습니다. | [L38790](origin_code/main-DsoigPgV.js#L38790) |
| `recycling` | 재활용 | — | MATCH_RULE | `recycling` / — | 모든 폰은 이미 잡힌 아군 기물로만 프로모션할 수 있습니다. 이는 변형 기물을 포함합니다. | [L38763](origin_code/main-DsoigPgV.js#L38763) |
| `revelation` | 계시 | — | MATCH_RULE | `revelation` / — | 연장전 시에 적용되는 판정 주기를 10수에서 5수로 줄입니다. | [L38691](origin_code/main-DsoigPgV.js#L38691) |
| `rule-bombs` | 폭탄 | — | MATCH_RULE | `ruleBombs` / — | 무작위 중앙 3칸에 폭탄을 설치합니다. 밟으면 해당 랭크와 파일의 모든 기물이 사라집니다. | [L38772](origin_code/main-DsoigPgV.js#L38772) |
| `saturation` | 포화 | — | MATCH_RULE | `saturation` / — | 기물을 3개 이상 잡은 기물은 더 이상 기물을 잡을 수 없습니다. | [L38808](origin_code/main-DsoigPgV.js#L38808) |
| `transcendence` | 초월 | — | MATCH_RULE | `transcendence` / — | 아군 기물이 기물을 직접 잡을 경우 다음 단계로 초월합니다. 퀸이나 변형 기물은 초월할 수 없습니다. | [L38817](origin_code/main-DsoigPgV.js#L38817) |
| `winter-kingdom` | 겨울 왕국 | — | MATCH_RULE | `winterKingdom` / — | 3턴마다 흑과 백의 기물이 3개씩 얼어붙습니다. 얼어붙은 기물은 3턴간 움직일 수 없고 공격 대상이 되지 않습니다. | [L38745](origin_code/main-DsoigPgV.js#L38745) |

### GUN — 1개

| ID | 이름 | 별 | 활성화 | effect / target | 설명 | 정의 출처 |
|---|---|---:|---|---|---|---|
| `shotgun-king` | 샷건킹 | 5 | ACTIVE | `shotgunKing` / — | 킹을 샷건 킹으로 변경합니다. | [L38004](origin_code/main-DsoigPgV.js#L38004) |

## 기물 및 타입 표기 전체 목록

TYPE_LABELS 75개와 별도 `wall`을 합친 76개 표기다. 이는 76개 독립 playable type이라는 뜻이 아니다. `windmillBishop`/`windmillRook`은 표시용 모드이며 `wall`, `blackHole`, `monster` 등은 환경·자동 행동 경로를 함께 확인한다. 전체 타입 참조를 이 목록만으로 제한하지 않는다. 이동 분기는 `getLegalMoves`에서 직접 추출했으며 공통 modifier/포획 제한은 표 위의 설명과 분석 문서를 함께 적용한다.

| 타입 | 이름 | 기본 이동 구현의 첫 식 또는 별도 경로 | 근거 |
|---|---|---|---|
| `alfil` | 알필 | `moves = jumpMoves(row, col, item2.color, alfilDeltas());` | [L81959](origin_code/main-DsoigPgV.js#L81959) |
| `amazon` | 아마존 | `moves = uniqueMoves([ ...rayMoves(row, col, item2.color, queenDirections()), ...jumpMoves(row, col, item2.color, knightDeltasForMove(row, col, item2.color)…` | [L82035](origin_code/main-DsoigPgV.js#L82035) |
| `assassin` | 암살자 | `moves = assassinMoves(row, col, item2.color);` | [L81941](origin_code/main-DsoigPgV.js#L81941) |
| `babyBear` | 아기곰 | `moves = [];` | [L82032](origin_code/main-DsoigPgV.js#L82032) |
| `bat` | 박쥐 | `moves = batMoves(row, col, item2.color);` | [L81971](origin_code/main-DsoigPgV.js#L81971) |
| `bear` | 곰 | `moves = rayMoves(row, col, item2.color, queenDirections());` | [L82028](origin_code/main-DsoigPgV.js#L82028) |
| `berserker` | 버서커 | `moves = berserkerMoves(row, col, item2);` | [L82107](origin_code/main-DsoigPgV.js#L82107) |
| `bigBishop` | 빅숍 | `moves = bigRookMoves(row, col, item2.color);` | [L82094](origin_code/main-DsoigPgV.js#L82094) |
| `bigRook` | 빅룩 | `moves = bigRookMoves(row, col, item2.color);` | [L82094](origin_code/main-DsoigPgV.js#L82094) |
| `bishop` | 비숍 | `moves = state.bishopSnipe[item2.color] ? uniqueMoves([...rayMoves(row, col, item2.color, state.reversal?.[item2.color] ? rookDirections() : bishopDirection…` | [L82004](origin_code/main-DsoigPgV.js#L82004) |
| `blackHole` | 블랙홀 | 이동 switch 외 환경/자동 행동/차단 경로; 개별 helper 확인 | [L66083](origin_code/main-DsoigPgV.js#L66083) |
| `brutus` | 브루투스 | `moves = hookMoves(row, col, item2.color);` | [L81986](origin_code/main-DsoigPgV.js#L81986) |
| `camel` | 낙타 | `moves = jumpMoves(row, col, item2.color, camelDeltas());` | [L81956](origin_code/main-DsoigPgV.js#L81956) |
| `campfire` | 캠프파이어 | `moves = wizardMoves(row, col, item2.color).filter((move) => move.row === row \|\| move.col === col);` | [L82126](origin_code/main-DsoigPgV.js#L82126) |
| `cannon` | 포 | `moves = cannonMoves(row, col, item2.color);` | [L82025](origin_code/main-DsoigPgV.js#L82025) |
| `cardinal` | 추기경 | `moves = cardinalMoves(row, col, item2.color);` | [L82007](origin_code/main-DsoigPgV.js#L82007) |
| `checker` | 체커 | `moves = checkerMoves(row, col, item2.color, item2.type);` | [L81925](origin_code/main-DsoigPgV.js#L81925) |
| `checkerKing` | 킹 체커 | `moves = checkerMoves(row, col, item2.color, item2.type);` | [L81925](origin_code/main-DsoigPgV.js#L81925) |
| `clockwork` | 태엽인형 | `moves = rayMoves(row, col, item2.color, queenDirections());` | [L81989](origin_code/main-DsoigPgV.js#L81989) |
| `coffin` | 오래된 관 | `moves = [];` | [L81974](origin_code/main-DsoigPgV.js#L81974) |
| `colossus` | 거신병 | `moves = colossusMoves(row, col, item2.color);` | [L82091](origin_code/main-DsoigPgV.js#L82091) |
| `crown` | 왕관 | `moves = jumpMoves(row, col, item2.color, kingDeltas(), "king");` | [L82053](origin_code/main-DsoigPgV.js#L82053) |
| `darkWizard` | 흑마법사 | `moves = jumpMoves( row, col, item2.color, item2.darkMagicCircle ? rookDirections() : kingDeltas(), "darkWizard" );` | [L82067](origin_code/main-DsoigPgV.js#L82067) |
| `dragon` | 드래곤 | `moves = dragonMoves(row, col, item2.color);` | [L82044](origin_code/main-DsoigPgV.js#L82044) |
| `eagle` | 알리바바 | `moves = jumpMoves(row, col, item2.color, eagleDeltas());` | [L81977](origin_code/main-DsoigPgV.js#L81977) |
| `fanatic` | 광신도 | `moves = fanaticMoves(row, col, item2.color);` | [L81929](origin_code/main-DsoigPgV.js#L81929) |
| `ferz` | 페르즈 | `moves = jumpMoves(row, col, item2.color, bishopDirections());` | [L81962](origin_code/main-DsoigPgV.js#L81962) |
| `football` | 축구공 | neutral 공을 현재 turn 소유자로 보고 footballMoves | [L81905](origin_code/main-DsoigPgV.js#L81905) |
| `grasshopper` | 그래스호퍼 | `moves = grasshopperMoves(row, col, item2.color);` | [L81998](origin_code/main-DsoigPgV.js#L81998) |
| `guard` | 근위병 | `moves = jumpMoves(row, col, item2.color, kingDeltas());` | [L81951](origin_code/main-DsoigPgV.js#L81951) |
| `hedgehog` | 고슴도치 | `moves = jumpMoves(row, col, item2.color, kingDeltas());` | [L82119](origin_code/main-DsoigPgV.js#L82119) |
| `herald` | 전령 | `moves = heraldMoves(row, col, item2.color);` | [L82022](origin_code/main-DsoigPgV.js#L82022) |
| `hook` | 구행 | `moves = hookMoves(row, col, item2.color);` | [L82019](origin_code/main-DsoigPgV.js#L82019) |
| `idol` | 아이돌 | `moves = idolMoves(row, col, item2.color);` | [L82085](origin_code/main-DsoigPgV.js#L82085) |
| `jester` | 광대 | `moves = jesterMoves(row, col, item2.color);` | [L82047](origin_code/main-DsoigPgV.js#L82047) |
| `king` | 킹 | `moves = [ ...jumpMoves(row, col, item2.color, kingDeltas(), "king"), ...state.hillKing?.[item2.color] && isCenterTwoByTwoCell(row, col, boardRowCount(), bo…` | [L82057](origin_code/main-DsoigPgV.js#L82057) |
| `knight` | 나이트 | `moves = uniqueMoves([ ...knightMoves(row, col, item2.color), ...state.cornerKick?.[item2.color] && isBoardCorner(row, col, boardRowCount(), boardColCount()…` | [L81932](origin_code/main-DsoigPgV.js#L81932) |
| `knightmaster` | 기사단장 | `moves = jumpMoves(row, col, item2.color, bishopDirections());` | [L81938](origin_code/main-DsoigPgV.js#L81938) |
| `lobster` | 랍스터 | `moves = lobsterMoves(row, col, item2.color);` | [L82088](origin_code/main-DsoigPgV.js#L82088) |
| `log` | 통나무 | `moves = logMoves(row, col, item2.color);` | [L82129](origin_code/main-DsoigPgV.js#L82129) |
| `magicGirl` | 마법소녀 | `moves = state.magicGirlSurge?.[item2.color] ? uniqueMoves([ ...rayMoves(row, col, item2.color, queenDirections()), ...jumpMoves(row, col, item2.color, knig…` | [L82101](origin_code/main-DsoigPgV.js#L82101) |
| `man` | 만 | `moves = jumpMoves(row, col, item2.color, kingDeltas());` | [L81951](origin_code/main-DsoigPgV.js#L81951) |
| `merchant` | 상인 | `moves = merchantMoves(row, col, item2.color);` | [L82132](origin_code/main-DsoigPgV.js#L82132) |
| `missionary` | 선교사 | `moves = missionaryMoves(row, col, item2);` | [L82001](origin_code/main-DsoigPgV.js#L82001) |
| `monster` | 괴물 | 이동 switch 외 환경/자동 행동/차단 경로; 개별 helper 확인 | [L8481](origin_code/main-DsoigPgV.js#L8481) |
| `octopus` | 문어 | `moves = jumpMoves(row, col, item2.color, queenDirections(), "octopus");` | [L81983](origin_code/main-DsoigPgV.js#L81983) |
| `paladin` | 팔라딘 | `moves = jumpMoves(row, col, item2.color, [[2, 1], [2, -1], [-2, 1], [-2, -1], [1, 2], [1, -2], [-1, 2], [-1, -2]], "paladin");` | [L81980](origin_code/main-DsoigPgV.js#L81980) |
| `parrot` | 앵무새 | `moves = parrotBaseMoves({ state, memory: state.parrotMovement?.[item2.color], row, col, color: item2.color, moved: item2.moved, rows: boardRowCount(), cols…` | [L81992](origin_code/main-DsoigPgV.js#L81992) |
| `pawn` | 폰 | `moves = pawnMoves(row, col, item2.color);` | [L81918](origin_code/main-DsoigPgV.js#L81918) |
| `pegasus` | 유니콘 | `moves = pegasusMoves(row, col, item2.color);` | [L82041](origin_code/main-DsoigPgV.js#L82041) |
| `primeMinister` | 국무총리 | `moves = primeMinisterMoves(row, col, item2.color);` | [L82050](origin_code/main-DsoigPgV.js#L82050) |
| `princess` | 프린세스 | `moves = princessMoves(row, col, item2.color);` | [L82123](origin_code/main-DsoigPgV.js#L82123) |
| `protestant` | 프로테스탄트 | `moves = protestantMoves(row, col, item2.color);` | [L82010](origin_code/main-DsoigPgV.js#L82010) |
| `queen` | 퀸 | `moves = rayMoves(row, col, item2.color, queenDirections());` | [L82028](origin_code/main-DsoigPgV.js#L82028) |
| `reaper` | 사신 | `moves = jumpMoves(row, col, item2.color, kingDeltas());` | [L81951](origin_code/main-DsoigPgV.js#L81951) |
| `recruiter` | 징집관 | `moves = jumpMoves(row, col, item2.color, kingDeltas(), item2.type);` | [L82076](origin_code/main-DsoigPgV.js#L82076) |
| `rook` | 룩 | `moves = rayMoves(row, col, item2.color, state.reversal?.[item2.color] ? bishopDirections() : rookDirections());` | [L82013](origin_code/main-DsoigPgV.js#L82013) |
| `royalKnight` | 로얄 나이트 | `moves = uniqueMoves([ ...jumpMoves(row, col, item2.color, knightDeltasForMove(row, col, item2.color), "king"), ...state.royalKnightKing[item2.color] ? jump…` | [L81944](origin_code/main-DsoigPgV.js#L81944) |
| `scarecrow` | 허수아비 | 이동 switch 외 환경/자동 행동/차단 경로; 개별 helper 확인 | [L1125](origin_code/main-DsoigPgV.js#L1125) |
| `shotgunKing` | 샷건 킹 | `moves = shotgunKingMoves(row, col, item2);` | [L82079](origin_code/main-DsoigPgV.js#L82079) |
| `siegeRam` | 공성추 | `moves = siegeRamMoves(row, col, item2);` | [L82098](origin_code/main-DsoigPgV.js#L82098) |
| `siren` | 세이렌 | `moves = sirenMoves(row, col, item2);` | [L82113](origin_code/main-DsoigPgV.js#L82113) |
| `slime` | 슬라임 | `moves = slimeMoves(row, col, item2);` | [L82110](origin_code/main-DsoigPgV.js#L82110) |
| `squire` | 종자 | `moves = pawnMoves(row, col, item2.color);` | [L81921](origin_code/main-DsoigPgV.js#L81921) |
| `standardBearer` | 기수 | `moves = pawnMoves(row, col, item2.color);` | [L81921](origin_code/main-DsoigPgV.js#L81921) |
| `thief` | 도적 | `moves = jumpMoves(row, col, item2.color, thiefOffsets()).filter((to) => thiefBaseMoveAllowed({ row, col }, to, (r, c) => state.board[r]?.[c], state)).map((…` | [L81995](origin_code/main-DsoigPgV.js#L81995) |
| `timeTraveler` | 시간 여행자 | `moves = timeTravelerMoves(row, col, item2);` | [L81965](origin_code/main-DsoigPgV.js#L81965) |
| `trickster` | 트릭스터 | `moves = tricksterMoves(row, col, item2);` | [L82116](origin_code/main-DsoigPgV.js#L82116) |
| `undead` | 언데드 | `moves = jumpMoves(row, col, item2.color, kingDeltas());` | [L82119](origin_code/main-DsoigPgV.js#L82119) |
| `vampireLord` | 뱀파이어 군주 | `moves = vampireLordMoves(row, col, item2.color);` | [L81968](origin_code/main-DsoigPgV.js#L81968) |
| `vip` | 귀빈 | `moves = jumpMoves(row, col, item2.color, kingDeltas(), "king");` | [L82053](origin_code/main-DsoigPgV.js#L82053) |
| `wall` | 엄폐물 | 이동 switch 외 환경/자동 행동/차단 경로; 개별 helper 확인 | [L2442](origin_code/main-DsoigPgV.js#L2442) |
| `windmill` | 풍차 | `moves = rayMoves(row, col, item2.color, windmillDirections(item2));` | [L82016](origin_code/main-DsoigPgV.js#L82016) |
| `windmillBishop` | 풍차 | windmill의 표시 모드; 별도 switch 분기 없음 | [L37726](origin_code/main-DsoigPgV.js#L37726) |
| `windmillRook` | 풍차 | windmill의 표시 모드; 별도 switch 분기 없음 | [L37726](origin_code/main-DsoigPgV.js#L37726) |
| `wizard` | 마법사 | `moves = wizardMoves(row, col, item2.color);` | [L82082](origin_code/main-DsoigPgV.js#L82082) |

## 모든 Math.random 등장 행

아래 82행은 원본 전체 문자열 검색과 개수가 일치한다. 기본 인자 및 한 행의 복수 호출도 포함하므로 난수 draw 수와 다르다. 함수명은 가장 가까운 앞선 최상위 function 선언으로 붙인 위치 안내이며 중첩 함수의 정확한 AST owner를 보증하지 않는다. `createBoardEditor` 등은 상위 영역 이름이다. G=규칙/설정, I=의미 ID(allocator로 대체), U=표시/외부, A=AI 정책, H=공유 helper.

| 행 | 영역/함수 | 분류 | 코드 |
|---:|---|---|---|
| [L1528](origin_code/main-DsoigPgV.js#L1528) | `chessNPow30RandomUnit` | G | function chessNPow30RandomUnit(random = Math.random) { |
| [L1529](origin_code/main-DsoigPgV.js#L1529) | `chessNPow30RandomUnit` | G | const raw = Number(typeof random === "function" ? random() : Math.random()); |
| [L1568](origin_code/main-DsoigPgV.js#L1568) | `arrangeChessNPow30Lineup` | G | function arrangeChessNPow30Lineup(lineup, random = Math.random, pieceValues = CHESS_N_POW_30_PIECE_VALUES) { |
| [L1595](origin_code/main-DsoigPgV.js#L1595) | `chessNPow30Lineup` | G | function chessNPow30Lineup(totalValue, random = Math.random, pieceValues = CHESS_N_POW_30_PIECE_VALUES, ways = CHESS_N_POW_30_LINEUP_WAYS) { |
| [L1622](origin_code/main-DsoigPgV.js#L1622) | `generateChessNPow30Setup` | G | function generateChessNPow30Setup(random = Math.random, { legacyChaosPieceValues = false, legacyRandomPools = false, legacyMissionaryValue = false } = {}) { |
| [L4028](origin_code/main-DsoigPgV.js#L4028) | `applyFiveLocalCard` | G | function applyFiveLocalCard(state2, card2, target, color2, { isRoyal, random = Math.random } = {}) { |
| [L21399](origin_code/main-DsoigPgV.js#L21399) | `createFriendlyChatMessageId` | U | bytes[index] = Math.floor(Math.random() * 256); |
| [L24699](origin_code/main-DsoigPgV.js#L24699) | `createBoardEditor` | U | return globalThis.crypto?.randomUUID?.() \|\| `board-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`; |
| [L32690](origin_code/main-DsoigPgV.js#L32690) | `createAutoDiagnostics` | U | random = Math.random, |
| [L34270](origin_code/main-DsoigPgV.js#L34270) | `multiplayerRecordHistoryAvailable` | U | const probeKey = `${MULTIPLAYER_RECORD_HISTORY_STORAGE_KEY}.probe.${Date.now()}.${Math.random()}`; |
| [L45905](origin_code/main-DsoigPgV.js#L45905) | `generateProfileUsername` | U | bytes[index] = Math.floor(Math.random() * 256); |
| [L45935](origin_code/main-DsoigPgV.js#L45935) | `defaultProfileAvatarToken` | U | const seed = user?.id \|\| user?.email \|\| window.crypto?.randomUUID?.() \|\| String(Math.random()); |
| [L49608](origin_code/main-DsoigPgV.js#L49608) | `appendUpdateLogItems` | U | const detailId = `update-log-detail-${Math.random().toString(36).slice(2)}`; |
| [L49650](origin_code/main-DsoigPgV.js#L49650) | `createUpdateLogCardTerm` | U | const previewCard = { ...card2, instanceId: `update-inline-${card2.id}-${Math.random().toString(36).slice(2)}` }; |
| [L52528](origin_code/main-DsoigPgV.js#L52528) | `piece` | I | id: `${color2}-${type}-${Math.random().toString(36).slice(2)}` |
| [L52678](origin_code/main-DsoigPgV.js#L52678) | `scheduleUndeadResurrection` | I | id: `undead-${captured.id \|\| Date.now()}-${Math.random().toString(36).slice(2, 8)}`, |
| [L52944](origin_code/main-DsoigPgV.js#L52944) | `resetGame` | G | shotgunOpeningColor: shotgunDlcEnabled && initialGameStyle !== "grand" ? Math.random() < 0.5 ? "white" : "black" : null, |
| [L53119](origin_code/main-DsoigPgV.js#L53119) | `maybeApplyOpeningRuleEvent` | U | nonce: `rule-${createdAt}-${Math.random().toString(36).slice(2)}`, |
| [L53123](origin_code/main-DsoigPgV.js#L53123) | `maybeApplyOpeningRuleEvent` | G | if (!state.ruleSelectionEnabled && Math.random() >= RULE_OPENING_CHANCE) { |
| [L53189](origin_code/main-DsoigPgV.js#L53189) | `scheduleRuleTicket` | I | id: `rule-ticket-${Date.now()}-${Math.random().toString(36).slice(2)}`, |
| [L54373](origin_code/main-DsoigPgV.js#L54373) | `randomChoice` | H | return items[Math.floor(Math.random() * items.length)] ?? null; |
| [L54379](origin_code/main-DsoigPgV.js#L54379) | `weightedChoice` | H | let roll = Math.random() * total; |
| [L55048](origin_code/main-DsoigPgV.js#L55048) | `findTrolleyBundlePair` | G | if (pair) return Math.random() < 0.5 ? pair : [pair[1], pair[0]]; |
| [L55057](origin_code/main-DsoigPgV.js#L55057) | `buildTrolleyDilemmaForColor` | I | id: `trolley-${Date.now()}-${Math.random().toString(36).slice(2)}`, |
| [L55945](origin_code/main-DsoigPgV.js#L55945) | `applyClonedPassiveCard` | I | instanceId: `clone-${recipientColor}-${sourceCard.id}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, |
| [L56096](origin_code/main-DsoigPgV.js#L56096) | `cloneCard` | I | return { ...card2, instanceId: `${card2.id}-${Math.random().toString(36).slice(2)}` }; |
| [L66568](origin_code/main-DsoigPgV.js#L66568) | `timeTravelerDeck` | I | instanceId: `${card2.id}-${Math.random().toString(36).slice(2)}`, |
| [L66577](origin_code/main-DsoigPgV.js#L66577) | `knightJourneyHintDeck` | I | instanceId: `${card2.id}-${Math.random().toString(36).slice(2)}`, |
| [L66786](origin_code/main-DsoigPgV.js#L66786) | `showCampaignStartIntro` | U | const nonce = `campaign-start-${Date.now()}-${Math.random().toString(36).slice(2)}`; |
| [L67071](origin_code/main-DsoigPgV.js#L67071) | `auctionLotProps` | G | const trialRoll = Math.random(); |
| [L67075](origin_code/main-DsoigPgV.js#L67075) | `auctionLotProps` | G | if (auctionCanGainStealth(spec, props) && Math.random() < AUCTION_STEALTH_CHANCE) { |
| [L67106](origin_code/main-DsoigPgV.js#L67106) | `auctionWeightedChoice` | G | let roll = Math.random() * Math.max(1e-4, total); |
| [L67119](origin_code/main-DsoigPgV.js#L67119) | `auctionLotCount` | G | if (maxCount === 3 && minCount === 2) return Math.random() < 0.45 ? 3 : 2; |
| [L67120](origin_code/main-DsoigPgV.js#L67120) | `auctionLotCount` | G | if (maxCount === 2) return Math.random() < (round > 8 && spec.score <= 3 ? 0.32 : 0.22) ? 2 : 1; |
| [L67121](origin_code/main-DsoigPgV.js#L67121) | `auctionLotCount` | G | return Math.random() < 0.12 ? 2 : 1; |
| [L67216](origin_code/main-DsoigPgV.js#L67216) | `maybeAuctionAiBid` | A | auction.aiBidAt = Date.now() + 650 + Math.random() * 1200; |
| [L67222](origin_code/main-DsoigPgV.js#L67222) | `maybeAuctionAiBid` | A | if (amount <= value * pressure && Math.random() < 0.86) { |
| [L67229](origin_code/main-DsoigPgV.js#L67229) | `maybeAuctionAiBid` | A | auction.aiBidAt = Date.now() + 2200 + Math.random() * 2e3; |
| [L67289](origin_code/main-DsoigPgV.js#L67289) | `grantAuctionLot` | I | id: `${lot.id}-${Math.random().toString(36).slice(2)}`, |
| [L68979](origin_code/main-DsoigPgV.js#L68979) | `applyMadAiSyntheticCard` | I | const card2 = { ...cloneCard(template), instanceId: `mad-ai-${cardId}-${Date.now()}-${Math.random().toString(36).slice(2)}`, devCard: true }; |
| [L69148](origin_code/main-DsoigPgV.js#L69148) | `pickAiOpeningFirstMoveAction` | A | const selected = choices[Math.floor(Math.random() * choices.length)]; |
| [L69738](origin_code/main-DsoigPgV.js#L69738) | `evaluateDraftCardForAi` | A | return (Number(card2?.stars) \|\| 0) * 2 + (CARD_CATEGORY_BY_ID[card2?.id] === "PIECE" ? 0.8 : 0) + (effectDraftBonus[card2?.effect] \|\| 0) + aiDraftCardContextBonus(card2, draftColor) + Math.random() * 0.25; |
| [L74598](origin_code/main-DsoigPgV.js#L74598) | `autoTargetForFirstMoveCard` | G | return targets[Math.floor(Math.random() * targets.length)]; |
| [L74605](origin_code/main-DsoigPgV.js#L74605) | `randomAutoTargetForCard` | G | const count = Math.max(1, Math.floor(Math.random() * Math.min(3, targets.length)) + 1); |
| [L74621](origin_code/main-DsoigPgV.js#L74621) | `randomAutoTargetForCard` | G | const count = Math.max(1, Math.floor(Math.random() * targets.length) + 1); |
| [L74631](origin_code/main-DsoigPgV.js#L74631) | `randomSquare` | G | return candidates.length ? candidates[Math.floor(Math.random() * candidates.length)] : null; |
| [L74718](origin_code/main-DsoigPgV.js#L74718) | `createHistoryNotationId` | U | return `${kind}-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`; |
| [L75615](origin_code/main-DsoigPgV.js#L75615) | `historyCardAnimationFromLaunch` | U | nonce: `card-${Date.now()}-${Math.random().toString(36).slice(2)}`, |
| [L77318](origin_code/main-DsoigPgV.js#L77318) | `applyQuantumAfterMove` | G | const chosen = pool[Math.floor(Math.random() * pool.length)]; |
| [L78416](origin_code/main-DsoigPgV.js#L78416) | `movePieceAttack` | G | if (mistakeCandidate && mistakeRollTriggers(Math.random(), state.mistakeCard?.[moving2.color] ? MISTAKE_CARD_CHANCE : void 0) && resolveMistakeReversal( |
| [L78661](origin_code/main-DsoigPgV.js#L78661) | `movePieceAttack` | I | id: `slime-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, |
| [L80801](origin_code/main-DsoigPgV.js#L80801) | `chooseVanishingPiece` | G | let roll = Math.random() * totalWeight; |
| [L85118](origin_code/main-DsoigPgV.js#L85118) | `gale` | I | id: `gale-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, |
| [L85312](origin_code/main-DsoigPgV.js#L85312) | `otherworld` | I | id: `otherworld-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, |
| [L85418](origin_code/main-DsoigPgV.js#L85418) | `twins` | I | const bondId = `twins-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`; |
| [L85578](origin_code/main-DsoigPgV.js#L85578) | `judgment` | I | id: `judgment-exile-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, |
| [L85626](origin_code/main-DsoigPgV.js#L85626) | `lobster` | I | id: `lobster-pending-${Date.now()}-${Math.random().toString(36).slice(2)}`, |
| [L86250](origin_code/main-DsoigPgV.js#L86250) | `hypocrisy` | I | id: `hypocrisy-${Date.now()}-${index}-${Math.random().toString(36).slice(2, 7)}`, |
| [L86356](origin_code/main-DsoigPgV.js#L86356) | `applyCardEffect` | G | state.captureTheFlag = { flags: { white: { row: boardRowCount() - 1, col: Math.floor(Math.random() * boardColCount()) }, black: { row: 0, col: Math.floor(Math.random() * boardColCount()) } }, occupations: { white: null, black: null } }; |
| [L86823](origin_code/main-DsoigPgV.js#L86823) | `portalGun` | I | id: `portal-gun-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`, |
| [L86846](origin_code/main-DsoigPgV.js#L86846) | `missionary` | G | const cell = candidates[Math.floor(Math.random() * candidates.length)]; |
| [L87283](origin_code/main-DsoigPgV.js#L87283) | `placeChess960Bishops` | G | const firstShade = Math.random() < 0.5 ? 0 : 1; |
| [L87452](origin_code/main-DsoigPgV.js#L87452) | `dice` | G | const roll = 1 + Math.floor(Math.random() * 6); |
| [L87746](origin_code/main-DsoigPgV.js#L87746) | `footballSpawnSquare` | G | tie: Math.random() |
| [L87895](origin_code/main-DsoigPgV.js#L87895) | `crownReplacementSquare` | G | const index = aiSimulationDepth > 0 ? Math.max(0, Number(state.moveCount) \|\| 0) % pool.length : Math.floor(Math.random() * pool.length); |
| [L88393](origin_code/main-DsoigPgV.js#L88393) | `applyTranscendenceCaptureUpgrade` | G | const roll = aiSimulationDepth > 0 ? ((Number(state.moveCount) \|\| 0) + row * 7 + col * 11) % 2 * 0.5 : Math.random(); |
| [L88504](origin_code/main-DsoigPgV.js#L88504) | `makeBloodCard` | I | instanceId: `blood-${Math.random().toString(36).slice(2)}`, |
| [L89113](origin_code/main-DsoigPgV.js#L89113) | `feudalContract` | I | const id = `feudal-${Math.random().toString(36).slice(2)}`; |
| [L90237](origin_code/main-DsoigPgV.js#L90237) | `icbm` | I | id: `icbm-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, |
| [L90535](origin_code/main-DsoigPgV.js#L90535) | `freeMove` | I | id: `premove-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, |
| [L90659](origin_code/main-DsoigPgV.js#L90659) | `scarecrow` | I | id: `scarecrow-pending-${Date.now()}-${Math.random().toString(36).slice(2)}`, |
| [L90751](origin_code/main-DsoigPgV.js#L90751) | `trolley` | I | id: `trolley-pending-${Date.now()}-${Math.random().toString(36).slice(2)}`, |
| [L91634](origin_code/main-DsoigPgV.js#L91634) | `ensureChainPieceId` | I | if (!item2.id) item2.id = `${item2.color \|\| "piece"}-${item2.type \|\| "unknown"}-chain-${Math.random().toString(36).slice(2)}`; |
| [L91656](origin_code/main-DsoigPgV.js#L91656) | `chain` | I | id: `chain-${Date.now()}-${Math.random().toString(36).slice(2)}`, |
| [L92636](origin_code/main-DsoigPgV.js#L92636) | `randomRouletteDisplayType` | U | return choices[Math.floor(Math.random() * choices.length)]; |
| [L92842](origin_code/main-DsoigPgV.js#L92842) | `startRandomRouletteEffect` | U | let cursor = Math.floor(Math.random() * RANDOM_ROULETTE_PIECE_TYPES.length); |
| [L92877](origin_code/main-DsoigPgV.js#L92877) | `settleRandomRouletteEffect` | U | const shouldRebound = Math.random() < 0.65; |
| [L93155](origin_code/main-DsoigPgV.js#L93155) | `animateCard` | U | const spinValue = Math.random() > 0.5 ? 13 : -13; |
| [L93554](origin_code/main-DsoigPgV.js#L93554) | `resolveEnemyQuantumCapture` | G | const illusion = forceIllusion \|\| Math.random() < 0.7; |
| [L94026](origin_code/main-DsoigPgV.js#L94026) | `armParryRetaliation` | G | if (!parryRollTriggers(Math.random(), chance)) { |
| [L95958](origin_code/main-DsoigPgV.js#L95958) | `shuffle` | H | const j = Math.floor(Math.random() * (i + 1)); |
| [L100528](origin_code/main-DsoigPgV.js#L100528) | `updateSettingsClockPanelVisibility` | G | state.shotgunOpeningColor = shotgunDlcEnabled ? state.shotgunOpeningColor \|\| (Math.random() < 0.5 ? "white" : "black") : null; |

## 간접 난수 helper 사용 위치

다음은 `randomChoice`, `weightedChoice`, `shuffle` 이름의 호출/선언을 정적 검색한 전 위치다. 이 helper를 쓰는 신규 카드도 Chance audit 대상이다. 주입된 `random()` 인자 경로는 위 Math.random 기본 인자 함수와 함께 확인한다. 고정 결과를 먼저 만들고 UI만 난수로 연출하는 룰렛은 결과 draw와 표시 draw를 분리한다.

- `randomChoice`: 49행 — 52113, 52123, 52124, 53134, 53388, 53467, 53562, 54372, 54378, 55046, 55047, 59439, 67057, 79627, 79997, 80211, 85071, 85310, 85682, 85785, 85924, 85950, 85973, 86334, 86894, 86952, 87286, 87287, 87294, 87296, 87297, 87390, 87549, 88512, 88562, 88593, 89501, 89574, 90171, 90311, 90329, 90333, 90910, 90914, 91029, 91469, 91703, 92644, 94522.
- `weightedChoice`: 3행 — 54357, 54375, 77476.
- `shuffle`: 31행 — 53573, 53579, 54326, 54986, 55036, 61003, 67473, 68920, 74603, 74609, 74614, 74619, 87264, 87427, 87429, 87501, 87513, 87759, 88252, 88253, 88303, 88304, 88572, 88810, 89642, 90147, 90759, 91105, 91122, 95901, 95955.

## 원본 ID 별칭

| 구 ID | 현재 ID |
|---|---|
| `free-move` | `premove` |
| `chess-45-pow-30` | `chess-n-pow-30` |

## 재현 및 검증

저장소 루트에서 실행한다. Node/Python 표준 라이브러리만 사용하고 패키지 다운로드나 브라우저 초기화는 하지 않는다.

```sh
node tools/phase0_inventory.cjs
python3 tools/phase0_docs.py --check
```

원본 변경 뒤 목록을 의도적으로 재생성할 때는 `python3 tools/phase0_docs.py`를 실행한다. 현재 검사는 source hash, 행 수, 241개 ID 유일성/분류 수, 정의 출처, Math.random 82행 및 생성 문서 일치를 확인한다. 게임 규칙의 실행 테스트나 Rust differential test가 아니다. 제한된 data declaration evaluator는 이 고정 번들의 Phase 0 추출용이며 일반 JS parser가 아니다.
