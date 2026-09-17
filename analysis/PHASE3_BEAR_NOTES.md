# 곰/고슴도치 반격 — 구현 전 조사

아직 구현하지 않았다. 기준은 PHASE3_REPORT.md의 고정 bundle이다.

- `septemberCounterLimit` (1028): bear/hedgehog만 2회. babyBear는 반격하지 않으며 별도 자동 이동/성장 기물이다.
- `getLegalMoves` (81902): owner turnsTaken이 bearMoveLockedUntilTurn 미만이면 이동 없음. bear는 queen 행마; hedgehog는 king 행마. 위협 쿼리의 잠금 적용은 별도로 원본 확인해야 한다.
- `capturePieceAt` (94057): victim owner 마나 지급 후 제거, `armBearRetaliation` 호출. attackerLanding이 있으면 지연, 없으면 같은 공격자에 대한 대기 반격을 즉시 처리한다.
- `armBearRetaliation` (93645): 적색 attacker identity가 있고 remaining > 0이어야 한다. 포획 당시 공격자 위치를 counterDestination으로 저장한다. bear 원본 clone, 원래 피포획 칸, attacker ID/색, remaining-1을 예약한다.
- `resolvePendingBearRetaliations` (93664): due를 queue에서 먼저 제거한다. 공격자 ID별 한 번 force removal. counterDestination이 비어 있지 않으면 피포획 칸으로 fallback; 둘 다 막혔으면 복구하지 않는다. 복구 bear는 moved=true, totalCaptures+1, remaining 감소, owner.completed+1까지 이동 잠금. 기존 fresh/origin 등 clone 필드는 유지한다.
- `forceRemovePieceAt` (91275): 보호막/HP를 무시하고 전체 entity를 제거한다. 왕 패배와 deathmatch progress를 처리하지만 wizard 마나를 지급하지 않는다. 일반 capturePieceAt 재호출이 아니므로 상대 곰의 재반격도 발생하지 않는다.
- `endMove` (79700): 이미 terminal이면 반격 전에 반환. 자기 색의 예약 반격은 Herald, 자동 Log 이동, 시간 정지 처리보다 앞선다. 따라서 Log가 이후에 만든 예약을 같은 endMove에서 즉시 처리한다고 추측하면 틀린다.
- 지연 마법 (81444): casterId가 현재 보드에 있을 때만 attacker를 전달한다. 죽은 시전자의 주문은 곰 반격을 예약하지 않는다.
- 샷건 산탄/저격 (94968/95035): attacker만 전달하고 attackerLanding 없음. 발사자 제거 후에도 원본 forEach/기존 object 갱신이 이어지는 경우를 비교해야 한다.
- 통나무 (94483): attackerLanding 있음. 이동 포획/체커/대형 착지와 함께 각각 실제 호출 옵션을 확인해야 한다.

필요한 구현은 구체적인 반격 queue와 기물의 반격 자원/잠금이다. future 카드용 범용 이벤트 DSL은 필요하지 않다. 승격·체커 연쇄 또는 Log 이동 뒤 queue가 행동 사이에 남으므로 로컬 임시 변수만으로 처리하면 저장/복원이 깨진다. 큐 직렬화 및 중간 상태 JS 투영도 함께 추가해야 한다.
