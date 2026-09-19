# 곰 반격 — 구현 기록

곰과 고슴도치를 구현했다. 아기곰은 아직 지원하지 않는다. 기준은 PHASE3_REPORT.md의 고정 bundle이다.

- `septemberCounterLimit` (1028): bear/hedgehog만 2회. babyBear는 반격하지 않으며 별도 자동 이동/성장 기물이다.
- `getLegalMoves` (81902): owner turnsTaken이 bearMoveLockedUntilTurn 미만이면 이동 없음. bear는 queen 행마다. `pieceAttacksSquare`에는 이 잠금 검사가 없어 잠긴 곰도 퀸 위협을 유지한다.
- `capturePieceAt` (94057): victim owner 마나 지급 후 제거, `armBearRetaliation` 호출. attackerLanding이 있으면 지연, 없으면 같은 공격자에 대한 대기 반격을 즉시 처리한다.
- `armBearRetaliation` (93645): 적색 attacker identity가 있고 remaining > 0이어야 한다. 포획 당시 공격자 위치를 counterDestination으로 저장한다. bear 원본 clone, 원래 피포획 칸, attacker ID/색, remaining-1을 예약한다.
- `resolvePendingBearRetaliations` (93664): due를 queue에서 먼저 제거한다. 공격자 ID별 한 번 force removal. counterDestination이 비어 있지 않으면 피포획 칸으로 fallback; 둘 다 막혔으면 복구하지 않는다. 복구 bear는 moved=true, totalCaptures+1, remaining 감소, owner.completed+1까지 이동 잠금. 기존 fresh/origin 등 clone 필드는 유지한다.
- `forceRemovePieceAt` (91275): 보호막/HP를 무시하고 전체 entity를 제거한다. 왕 패배와 deathmatch progress를 처리하지만 wizard 마나를 지급하지 않는다. 일반 capturePieceAt 재호출이 아니므로 상대 곰의 재반격도 발생하지 않는다.
- `endMove` (79700): 이미 terminal이면 반격 전에 반환. 자기 색의 예약 반격은 Herald, 자동 Log 이동, 시간 정지 처리보다 앞선다. 따라서 Log가 이후에 만든 예약을 같은 endMove에서 즉시 처리한다고 추측하면 틀린다.
- 지연 마법 (81444): casterId가 현재 보드에 있을 때만 attacker를 전달한다. 살아 있는 시전자는 즉시 반격 대상이고, 죽은 시전자의 주문은 곰 반격을 예약하지 않는다.
- 샷건 산탄/저격 (94968/95035): attacker만 전달하고 attackerLanding 없음. 반격은 발사자를 즉시 제거한다.
- 통나무 (94483): attackerLanding이 있어 곰 반격이 endMove 뒤까지 대기한다. 통나무 자동 이동이 반격 처리보다 뒤이므로 이 대기열은 다음 해당 색 행동까지 canonical에 남는다.
- 일반 이동과 체커 착지는 endMove에서 반격한다. 거신병 섹터 공격은 attackerLanding이 없어 즉시 반격한다.
- 대형 기물 착지는 `attackerLandingCells`만 전달해 `capturePieceAt`이 반격을 즉시 처리하지만, 뒤의 `moveBigRook`/`moveColossus`가 제거된 공격자 object를 다시 배치하면서 복귀한 곰을 덮어쓴다. 최종 의미 상태에 반격이 남지 않으므로 Rust는 이 경로에서 queue를 만들지 않는다.

구현은 구체적인 `PendingBearRetaliation` queue와 곰/고슴도치가 실제 공유하는 두 필드만 추가했다. future 카드용 범용 이벤트 DSL은 만들지 않았다. Log 이동 뒤 queue가 행동 사이에 남으므로 canonical 직렬화와 JS oracle 중간 상태 투영을 함께 추가했다. 곰 선택 실행 1,381개 비교/1,138개 전이와 고슴도치·캠프파이어 결합 실행 768개 비교/496개 전이가 통과했다.
