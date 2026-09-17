# 마법사 원본 분석 및 구현 대응

마법사 타입/네 주문과 보호막 선행 처리를 구현했다. Rust 개별 테스트와 JS 연속 시나리오 검증은 통과했고 전체 마법사 조합 9,536개 비교/9,347개 전이도 통과했다. 아래는 hash 고정 `origin_code/main-DsoigPgV.js`에서 확인한 구현 근거다.

- `wizardMoves` (84240): 인접 8칸의 빈 칸으로만 이동한다.
- `grantWizardMana` (81416): 같은 색 마법사 모두 +amount, maxMana 기본 5, mana 기본 0. 턴마다 보급하는 것이 아니라 아군 기물의 포획 시 +1이다. `capturePieceAt` (94057), `damageHealthPiece` (943xx)의 실제 제거 순서에 연결해야 한다. 일반 포획에서는 제거 전에, HP 사망에서는 제거 후 호출한다.
- `wizardSpellInfo` (92296): lightning 1, shield 2, meteor 3, timeStop 5.
- `castWizardSpell` (92419): shield는 아군이고 wall/scarecrow가 아닌 기물에 shielded=true. lightning/meteor는 상대 색이 행동을 끝낼 때 발동하는 delayedHazards를 생성한다. 시전자 ID는 사망 후에도 이력으로 남을 수 있다.
- `wizardPreviewCells` (92444): meteor는 우하단 클릭을 row/col 최대 6으로 보정한 2×2, row-major 순서다. Action은 실제 영역의 좌상단으로 중복 제거 가능하다.
- `chooseWizardSpell` (92198): timeStop은 마나 5를 쓰고 상대 skipTurn=true만 설정한다. 즉시 턴을 끝내지 않는다.
- `handleWizardSpellTarget` (92316): 나머지 주문은 cast 후 비용 차감하고 endMove 호출한다. fresh 포획 잠금을 주문 시전에 검사하지 않는다.
- `retainTimeStopAsSameTurn` (79661): 다음 행동의 endMove에서 상대 skipTurn을 false로 소비하고 enPassant=null. 같은 턴 유지, moveCount/completed 증가 없음, 지연 주문도 아직 발동하지 않는다. 기존 ExtraMove 표현을 범용 실행기로 만들 필요 없이 실제 필요한 두 색 boolean만 추가하면 된다.
- `endMove` (79762–79800): timeStop 확인 → delayed hazards → terminal/Herald 확인 → moveCount 증가 → completeTurnAfterMove. 지연 마법으로 왕이 죽으면 완료 counter 증가 전 종료다.
- `applyDelayedHazards` (81422): due 목록을 먼저 상태에서 제거한 뒤 등록 순서대로 실행한다. wall 제외, 아군도 대상. shield면 제거하고 해당 칸 종료. meteor는 HP entity를 점유 칸마다 타격하며 HP 피해는 기존 구현과 같이 1이다. 일반 entity는 ID 중복 제거한다. 시전자 생존 여부와 fresh 때문에 대상이 자동 면역이 되지 않는다. guard의 일반 포획 면역도 이 helper에서는 별도 검사하지 않는다.
- 지연 마법의 forEach는 왕 포획 후에도 남은 칸/주문을 실행한다. 대형 착지에서처럼 종료 판정 순서를 검증해야 한다.

실제로 추가한 최소 확장: 마법사 mana/max_mana, 주문 종류와 실제 target을 가진 Action, ordered delayed hazards, timeStop boolean 상태. 전체 이벤트 버스나 미래 카드용 효과 DSL은 만들지 않았다. 구현은 `engine/src/wizard.rs`에 있다.
