# 빅룩 캐슬링 — 구현 및 원본 대응

원본 `castleMoves` (82854 이후)는 rook 또는 bigRook을 모두 허용한다. `engine/src/castling.rs`에서 일반/대형 룩 캐슬링을 함께 처리한다. 아래 전이를 구현했고 양색·양측 조합의 JS 비교를 통과했다.

- 홈 row의 코너 칸에 같은 색의 unmoved bigRook entity가 점유하면 대상이다. anchor가 코너일 필요는 없다.
- 왕과 코너 사이 clear 검사에서는 해당 bigRook의 점유 칸을 장애물로 세지 않는다.
- `bigRookCastleAnchorForKing` (82914): white anchor row=왕 row-1, black=row. 왕이 queenside 도착하면 col=왕 col+1, kingside면 col=왕 col-2.
- `canPlaceBigRookAfterCastle`: 새 footprint 네 칸이 범위 내이며 비어 있거나 자신/왕/아군이면 허용, 적 점유는 거절한다.
- `performCastle` (79014): 기존 빅룩을 비우고 왕 이동 → 새 footprint의 아군을 제거 → 빅룩 배치. 밟힌 아군이 대형이면 entity 전체를 제거한다.
- 이 아군 제거는 capturePieceAt 경로가 아니다. 마나 보급·포획 progress·royal capture 판정을 임의로 추가하면 안 된다. 현재 지원 타입에는 reaper 후속 반응이 없으나 reaper 이식 때 해당 훅이 필요하다.
- king/rook moved=true, castled=true, enPassant=null 뒤 herald/턴 처리는 기존 캐슬링과 동일하다.
- 양색×양측, 아군 점유/적 점유, shared footprint, 지연 마법과 위협 차단을 검증한다.
