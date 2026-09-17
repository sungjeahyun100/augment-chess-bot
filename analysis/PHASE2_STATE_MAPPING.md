# Phase 2 상태 및 실행 대응

기준 bundle SHA-256: `0dbbad680c6e8e2abcdb6e49817ff486aa2f8ef799a8035bd356baef8a2731ea`.

| JS 의미 | Canonical v2 / Rust |
|---|---|
| board의 기본 여섯 기물, id/color/type/moved/origin | Board + Piece, ID 유지, footprint 단일 칸 |
| getLegalMoves, pawnMoves, knightMoves, rayMoves, jumpMoves | movement::legal_actions, 기본 Move(from,to,route=[]) |
| isSquareAttacked, pieceAttacksSquare | movement::attacked, GameState::is_in_check |
| castleMoves, performCastle | 왕 이동으로 식별, 룩 재배치, moved/castled 갱신 |
| pawnEnPassantRightAfterAdvance, retainSameTurnEnPassant | EnPassant {pawn,target,available_to}; 다음 착수에 생성/해제 |
| movePieceAttack, capturePieceAt, resolveRoyalCapture | transition::apply, ID 제거, RoyalCapture 결과 |
| pendingPromotion, choosePromotion | pending_promotion ID, 네 개 Promote 행동, 선택 후 턴 완료 |
| markTransformedOrigin, markFreshNoCapture | origin 갱신 + owner.completed+1 capture deadline |
| moveCount, turnsTaken, fullMove, turn | TurnState; 서로 다른 정산 지점 보존 |
| forceFirstMoveCardsAfterMove (빈 덱) | 해당 첫 턴에 first_move_cards_forced=true |
| endMove, completeTurnAfterMove | victory::end_turn |
| recordPosition, positionKey | history.position_counts, 같은 JS key, 최초 count=1 |
| starWinLimit, deathmatchEnabled, deathmatchLimitTurns | GameConfig의 양의 정수 한도 및 bool |
| deathmatch 활성 record | Option<Deathmatch>, 네 가지 의미 필드 |
| markDeathmatchProgress, tickDeathmatchAfterTurn | 폰 이동/포획 progress; 흑 완료 때 +2 또는 reset |
| checkRepetitionOrStarLimit, resolveStarTiebreak | 반복/턴 한도/연장전 및 별 우열 함수 |
| checkNoActionLoss | 다음 플레이어가 수가 없으면 상대 승리 |
| mode=gameover, winner, endGame 호출 이유 | Phase::Terminal + GameResult/EndReason |

JS의 mode/status 문구나 UI 클릭 순서를 공개 Action으로 도입하지 않는다. 프로모션 외 다단계 선택은 아직 실행하지 않는다. recordPosition의 문자열은 향후 MCTS transposition hash로 재사용하면 안 된다.

## Oracle 경계

`tools/phase2_oracle.cjs`는 hash가 같은 원본에서 top-level 함수 선언, 명시한 constants, resetGame의 상태 객체 literal만 추출한다. import.meta를 쓰는 브라우저 함수는 제외한다. 모듈 import, module top-level 부작용, DOM 생성, timer, worker는 실행하지 않는다. 원본 규칙 함수 본문은 수정하지 않는다. 선언에 포함된 다른 카드 함수도 실행 경로가 들어가면 실제 함수이며, 없는 의존성은 ReferenceError다. 임의로 false를 반환하는 fallback은 없다.

교체점은 manifest의 sinks와 boundaries에 전부 기록한다. UI/표시/애니메이션/기보 기록은 sink, 시계는 꺼짐, 입력 권한은 로컬 양측 사용자, RULE 선택은 빈 목록, endGame은 winner/reason/mode 저장으로 교체한다. 초기 보드는 Phase 1의 별도 실제 createInitialBoard 검증으로 보장하고 여기서는 canonical에서 가져온다. 초기 RULE과 드래프트는 명시적으로 끈다. Math.random 호출은 오류다.

비교는 정규화한 **전체 Phase 2 canonical**과 action set 및 양쪽 왕 위협을 포함한다. 매 수 Rust 결과를 JS에 다시 주입하지 않는다. 이름 있는 시나리오/무작위 대국 각각에서 JS VM 상태와 Rust 상태가 독립적으로 이어진다. 기본 카드 없는 상태에 영향 없는 표시나 비활성 카드용 capture/history 누적값은 canonical에서 제외하므로, 임의의 카드가 있는 JS 상태의 전체 호환성을 의미하지 않는다.

왕 포획은 JS의 조기 return 때문에 이동 기물의 moved, EP, 턴 카운터가 이전 상태일 수 있다. canonical도 이를 유지한다. turn limit과 반복의 종료는 side 변경 뒤지만 deathmatch 만료는 흑 completed/move_count 갱신 뒤 side/full_move 변경 전에 발생한다.

v1 입력은 migration 없이 거절한다. Phase 1 audit는 당시 경계에 대한 역사적 기록이며, 위 대응이 그 deferred 항목 중 Phase 2 실행 부분을 대체한다. 포획 재활용·카드·추가 이동·예약 effects·RULE은 후속 단계에서 새로운 의미 상태와 검증을 추가해야 한다.
