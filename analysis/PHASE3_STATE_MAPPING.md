# Phase 3 상태 대응 (진행 중)

기준 source hash와 전체 지원표는 `../PHASE3_REPORT.md`를 따른다. 기존 Phase 1/2 field audit는 당시 경계의 기록이며, 아래 필드를 실제 변형 기물 실행 상태로 승격했다.

| JS 필드/경로 | Rust canonical / 동작 | 적용 시점 |
|---|---|---|
| piece.gold / grantMerchantGold | Piece.gold: optional u32 | 자기 턴 시작에 +1, 누락은 0으로 동작 |
| move.merchantBuy | Action::Purchase {merchant,target} | 공유 footprint 클릭은 entity 단위로 중복 제거 |
| merchantBuy의 왕 대상 | EndReason::RoyalPurchase | 비용 지불 후 소유권/턴 counter를 바꾸지 않고 승리 |
| piece.shielded | Piece.shielded | 직접/앙파상 공격 시 제거; 체커는 제거 후 점프 착지; 대형 착지는 무시, 섹터는 대상 제외 |
| piece.mana/maxMana | Piece.mana/max_mana | 마법사 전용 optional u32; 기본 0/5 |
| state.delayedHazards | delayed_spells ordered Vec | 번개/메테오 종류, anchor, owner, historical caster ID; 상대 행동 완료 직전 순서대로 실행 |
| state.skipTurn | time_stopped color set | 시간 정지 예약; 상대 표시를 다음 자기 행동 후 소비하여 같은 턴 유지 |
| wizardSpellInfo + target | Action::CastSpell | 표시 선택 상태 없이 실제 주문/대상으로 한 행동 표현 |
| piece.logDir/logRollAfterTurn | log_direction/log_roll_after_turn | 방향 지정은 owner.completed+1; 자동 이동은 보드 순서로 자기 행동 종료 때, deadline 도달 시 한 번 |
| move.setLogDirection | Action::SetLogDirection | 목적 칸 클릭을 이동으로 처리하지 않음 |
| piece.ammo/maxAmmo/facing | ammo/max_ammo/facing | 샷건 킹 전용; 이동·산탄은 방향 갱신, 저격/장전은 방향 보존 |
| shotgun action UI modes | Reload/ShotgunBlast/ShotgunSnipe | 표시 모드 없이 실행 가능한 행동들의 합집합 |
| shotgunPenaltyColor | 살아 있는 샷건 킹 검색 | 반복 기록 중지 및 장기전 패널티(white 우선) |
| primeMinisterMoves | 기존 Action::Move | 빈 중간 칸을 거치는 최대 두 왕 걸음, 경로 중복 제거; 추가 상태 없음 |
| royalKnight / isRoyalKing | PieceKind::RoyalKnight / royal() | 카드 없는 기본 나이트 행마 및 왕 판정; royalKnightKing/hillKing은 해당 카드 단계 |
| piece.type | PieceKind의 실제 지원된 string ID | 생성/변신 |
| piece.heraldJumpLockTurn / heraldJumpUnlocked===false | herald_jump_lock_turn / herald_jump_locked | 전령 전용; 이동 시 legacy 잠금 해제, owner completed와 deadline 비교 |
| resolveHeraldThreats | EndReason::HeraldAgreement | 턴 완료 전 행동자 우선 인접 왕 확인 |
| piece() identity | ids.next_piece | 징집관 폰 생성 시 할당; RNG 소비 없음 |
| piece.windmillMode | windmill_mode: rook 또는 생략(기본 bishop) | 이동 완료 후 토글, 왕 직접 포획 조기 반환 전에는 토글 안 함 |
| piece.checkerChainCapture | turn.continuation.kind=checker_capture, piece ID | 연속 jump capture가 남으면 같은 턴 유지 |
| piece.origin | Piece.origin | 공통 transform 시 현재 칸으로 갱신 |
| piece.freshNoCaptureUntil | cannot_capture_until_owner_turn status | transform 시 owner.completed+1; 만료 후 기록 유지 |
| piece.anchorRow/anchorCol | Piece.anchor | 대형 기물의 정규 출발점 |
| state.board의 공유 object | Board.cells의 같은 PieceId 네 칸 + Piece.footprint | clone/serialize/apply에서도 한 entity 유지 |
| piece.hp/maxHp | Piece.hp/max_hp | 일반 공격마다 HP 1 감소; HP가 0이면 entity 전체 제거 |
| move.colossusBody | 제외 | 선택/표시용 클릭으로 게임 전이 없음 |
| move.colossusMove/bigRookMove | Action::Move | 대상 footprint의 entity별 포획 후 anchor 이동 |
| move.colossusAttack/sectorCells | Action::AttackSector {piece,sector} | 같은 섹터의 4개 표시 좌표는 한 행동 |
| uniqueAlliedPieceCount | pieces 중 같은 owner의 개수 | 버서커 query마다 계산; footprint 중복 미계산 |
| septemberPrincessHasQueenMovement | 같은 owner의 Queen entity 존재 | 프린세스 query마다 계산 |
| clockworkHasNeighbor | 인접 8칸에서 다른 아군 entity 존재 | 태엽인형 이동 query마다 계산 |
| state.moveCount/turnsTaken | turn.move_count/completed | 체커 연쇄의 각 jump는 move_count, 연쇄 종료 때만 completed |

카드 획득 상태, 드래프트, RULE, 누적 captures/capturedTypes, 예약 반격/부활, 골드/마나/탄약 외 기물 자원은 해당 기물/카드 이식 시점에 추가한다. 지금 비어 있다고 가정한 채 미구현 기능을 지원한다고 표시하지 않는다. 기물 속성을 임의 JSON map으로 저장하거나 앞으로 생길 카드용 범용 modifier DSL을 만들지 않았다.

## JS oracle 표시 경계

Phase 2 추출 함수 목록/출처 hash/UI sink를 재사용한다. Phase 3에 추가된 순수 상수는 `HOOK_ORTHOGONAL_DIRECTIONS`, `hookIcePathsByMove`, `MAX_NOTATION_TEXT`다. `primeStatusMagicLoss`, `scheduleStatusMagicSound`, `queueHpAttackHistoryNotation`는 각각 HP 애니메이션/소리/기보 출력이므로 sink다. 실제 HP 감소/포획/턴 함수는 원본을 실행한다. 거신병 지연은 `window.setTimeout(callback)`을 즉시 실행해 같은 실제 continuation을 처리한다. 동작에 쓰이는 난수 호출은 숨기지 않고 예외 처리한다.

징집관 oracle은 원본 `piece` 함수의 opaque ID 생성식만 단조 증가 ID로 치환한다. 나머지 생성자와 `leaveRecruiterPawn`은 원본 그대로 실행하며 규칙 난수 호출은 계속 오류로 처리한다. 신규 폰은 기본 canonical 레코드를 만든 후 실제 JS 필드를 투영한다. 다른 생성 타입은 명시적으로 거절한다.

명세 수정: 징집관이 생성한 폰에는 원본이 누락한 공통 fresh 제한을 적용한다. differential의 `check`는 원본의 빈 status를 먼저 확인한 뒤 이 명시적 차이만 기대 상태에 추가한다. 원본 함수와 oracle 출력은 수정하지 않으며 보고서 `intentional_corrections`에 이 차이를 기록한다. 현재 이동은 턴을 종료하므로 다음 행동 시 이미 만료되지만 canonical 기록은 유지한다.

상인 가격 조회용 `ENCYCLOPEDIA_PIECE_VALUES`와 `CHESS_N_POW_30_TYPE_ALIASES` 상수를 추가 추출한다. 매수 시 raw JS 소유권을 투영하고 앙파상 로더는 현재 폰 색 대신 과거 `available_to`를 사용한다.

통나무 표시 전용 `addAutoLogMoveHighlight`는 oracle sink에 추가했다. 방향 지정·자동 이동·HP 피해·포획·턴 함수는 원본을 실행한다.

샷건 비교는 SHOTGUN_BLAST_AMMO_COST/SHOTGUN_SNIPE_AMMO_COST 상수를 추가 추출하고, 원본 UI 모드별 getLegalMoves 결과를 명시적 공격 Action으로 정규화한다. 실제 발사/장전 함수는 원본 그대로 실행한다.
