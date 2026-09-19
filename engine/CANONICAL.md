# Canonical v3 — phase3_cardless

기계 검증 스키마는 `src/state.rs`의 `CanonicalState`와 각 nested DTO의 serde 정의 및 `GameState::from_snapshot` 불변식 검사다. 샘플은 `tests/fixtures/initial.canonical.json`이다. 이 v3는 **카드 없는 일반 8×8 실행 상태와 기존 state-only 표현**의 계약이며, 모든 카드가 포함된 JS 게임 상태 전체의 스키마라고 주장하지 않는다. 의미 상태를 추가할 때 스키마 버전과 fixture를 갱신하고 구버전 입력을 명시적으로 거절하거나 migration을 제공해야 한다.

- `schema_version=3`, `capability=phase3_cardless` (실행) 또는 `phase1_state_only` (표현 전용), `config.rules_profile=local_0dbbad68`. 원본 전체 SHA-256은 `SOURCE_SHA256` 및 oracle manifest에 고정한다.
- `board`: rows/cols 1…64, row-major `cells`의 길이는 rows×cols. `null`은 빈칸, 나머지는 양의 `PieceId`. 크기 확장은 상태 표현용이며 다른 게임 모드 구현을 뜻하지 않는다.
- `pieces`: ID 오름차순의 entity records. ID 중복/0/겹침/없는 참조 금지. anchor는 footprint 안에 있어야 하고 모든 칸은 보드 안이어야 한다. 각 footprint는 집합이며 row/col 순서로 정렬한다. 같은 entity는 정확히 한 번 serialize한다. 현재 위치에서 ID를 다시 만들지 않는다.
- `origin`: 최초 생성/변신 위치. nullable. 기물의 `hp`/`max_hp`는 둘 다 null 또는 0 < hp ≤ max인 쌍이다. `statuses`는 중복 없는 정렬 집합이다. `cannot_capture_until_owner_turn`은 지정 owner의 완료 턴 counter를 기준으로 하며 owner가 entity owner와 일치해야 한다.
- `players`: white/black의 color가 키와 일치한다. 일반 모드의 카드 슬롯 세 개는 순서를 유지하고 빈칸은 null이다. 현재 occupied 슬롯은 Unsupported이다. category와 activation은 독립 타입이며 RULE은 카드 category가 아니다.
- `turn`: side, 양측 completed, full_move(1부터), move_count, actions_remaining, continuation을 분리한다. action 수와 완료 턴 수를 등치시키지 않는다. continuation이 있으면 현재 actor의 살아 있는 entity를 참조한다. 카운터 관계에 기본 체스 공식(흑 1턴=정확히 2 actions 등)을 강제하지 않는다.
- `phase`: play 또는 terminal. `result=null`은 ongoing, draw record는 무승부, win record는 winner와 이유를 보존한다. phase/result 불일치는 오류다. decision은 Player/Terminal이다. 프로모션 보류 중에는 동일 플레이어가 선택하고 다른 행동은 금지한다.
- `history`: en-passant 참조, 양측 castled/castling_canceled, repetition_salt, position_counts를 보존한다. 반복 키는 opaque string이고 record는 key 순서로 정렬한다. 중복 key나 count=0은 거절한다. JS positionKey/recordPosition을 실행하며 생성 시 최초 위치 count=1을 기록한다. 키는 moved, en-passant와 entity ID를 포함하지 않는 원본 의미를 보존한다.
- `chance`: splitmix64_v1의 64-bit 상태는 high/low u32로 저장하여 JS JSON 정밀도 손실을 피한다. seed=0도 유효하다. tape_v1은 ordered `{candidates,index}` outcome과 cursor를 보존한다. 후보 수 불일치/소진/잘못된 입력은 오류이며 cursor/RNG는 변하지 않는다. 후보 정렬은 규칙 호출자의 책임이다. JS Math.random과 같은 seed를 준다고 같은 결과라고 간주하지 않는다.
- `ids`: next_piece/next_card/next_effect 양의 단조 증가 할당 경계. next_piece는 살아 있는 모든 ID보다 커야 한다. 초기 배치 32개 이후 33이다. ID 생성은 rules RNG를 소비하지 않는다.

출력은 compact UTF-8 JSON, 재귀적으로 알파벳순 object key, 정수만 사용한다. enum은 안정적인 문자열 이름을 쓴다. 구현은 serde_json의 기본 BTreeMap object ordering에 의존하므로 `preserve_order` 기능을 활성화하지 않는다. `pieces`/footprint/status/repetition 같은 집합만 정렬하고 카드 슬롯, chance outcome, action route/target 등 순서가 의미 있는 배열은 그대로 둔다. JSON object 입력 순서는 무관하다. Option 필드의 누락은 null로 정규화하고, 그 외 필수 필드 누락 및 모든 미지 필드를 거절한다. 중복 struct key도 오류다.

정규화 대상은 같은 entity ID를 사용하는 의미상 동등 상태다. 서로 다른 ID 배정의 상태를 graph-isomorphism으로 같게 만들지는 않는다. JS fixture는 생성 순서의 단일 ID mapping을 모든 cell/entity 참조에 적용한다. 캐시/DOM/표시 로그는 포함하지 않는다. 카드, 포획 이력, 곰 이외의 예약 effect, rollback, terrain 등 아직 모델링하지 않은 실제 규칙 상태는 이 스키마로 가져올 수 없다. 이를 임의의 빈 배열로 대체하여 호환성을 주장하지 않는다.

불변식 검사는 **구조적 일관성**을 보장한다. 외부 snapshot이 실제 합법 행동으로 도달 가능한 상태인지 증명하거나, terminal 결과가 실제 규칙에 맞는지 재판정하지는 않는다. 새 행동 이후의 판정은 실행 엔진의 책임이다.

## v2에 추가된 실행 상태

- `config.star_win_limit`: 양측 완료 턴 중 작은 값이 도달해야 하는 양의 한도(기본 45).
- `config.deathmatch_enabled`: 한도 도달 시 연장 여부(기본 true).
- `config.deathmatch_limit_turns`: 연장전 진행 없음 제한(기본 10). 두 배가 u32 범위 안이어야 한다.
- `pending_promotion`: null 또는 현재 플레이어의 마지막 rank 폰 ID. terminal/다른 소유자/비폰/마지막 rank 외 위치는 거절한다. 이동 뒤 선택을 직렬화하며 Promote 후 null이 된다.
- `deathmatch`: null 또는 `{started_at_turn, half_turns_since_progress, interval_half_turns, progress_this_turn}`. 활성 상태만 객체로 표현한다. UI warning key는 제외한다. 흑 턴 완료 때만 +2하며, 폰 이동/포획은 progress를 표시하고 다음 흑 완료 때 해제한다. 만료는 side/full_move 전환보다 먼저 발생한다.

기본 실행은 8×8, actions_remaining=1에 한정한다. 기본 기물과 현재 이식한 변형 기물은 `PHASE3_REPORT.md`를 따른다. 대형 기물은 anchor부터 오른쪽/아래의 정확한 2×2 footprint와 HP 쌍이 필요하다. 벽은 중립 소유자로도 배치할 수 있다. 일반 ExtraMove continuation은 아직 실행하지 않는다. 다른 표현 가능한 상태는 실행 시 Unsupported다. fresh-capture deadline은 승격 시 owner.completed+1로 기록하며 만료 후에도 원본처럼 보존한다. 승격은 origin도 승격 칸으로 갱신한다.

앙파상은 실제 폰 ID, 목표 칸과 전진 당시 반대 색인 available_to를 보존한다. 매수 후에는 현재 소유자가 전진 당시 색과 다를 수 있다. cardless capability에서는 폰의 moved/두 칸 전진 도착 rank/목표 칸의 기하를 검증한다. 통나무 자동 이동으로 목표 칸이 점유될 수 있다. 왕 포획은 원본의 조기 반환을 보존하므로 착수자의 moved, 턴 카운터와 이전 앙파상 상태를 갱신하기 전에 종료할 수 있다. 제거된 entity를 가리키는 참조는 남기지 않는다.

스키마 v1에는 프로모션·데스매치·종료 설정이 없고 초기 반복 기록도 다르므로 자동 migration을 제공하지 않는다. `schema_version=1`과 `schema_version=2`는 Unsupported다. 규칙상 의미 없는 표시·기보·애니메이션, 비활성 카드용 누적 포획 이력은 이 cardless 계약에 포함하지 않으며, Phase 5에서 해당 이력이 필요한 카드를 추가할 때 모델을 확장해야 한다.


## v3에 추가된 변형 기물 상태

- `PieceKind`: 실제 이식된 식별자만 추가한다. 미구현 기물 식별자는 deserialize 오류다.
- `Piece.windmill_mode`: `rook` 또는 생략. 입력의 명시적 `bishop`은 생략으로 정규화하며 의미는 비숍 모드다. 다른 기물에 모드를 지정하면 거절한다.
- `Continuation::CheckerCapture {piece}`: 살아 있는 현재 플레이어 체커에 대한 강제 연속 잡기. 선택 종료 행동은 없다. move_count는 잡기마다 증가하지만 completed는 연쇄가 끝날 때 증가한다.
- `Action::AttackSector {piece,sector}`: 거신병의 두 전방 섹터 중 하나를 공격한다. `sector=0`은 오른쪽, `1`은 왼쪽. 표시용 본체 클릭은 행동이 아니다. 같은 섹터 내 여러 UI 클릭을 하나의 행동으로 정규화한다.
- 대형 entity의 네 칸은 하나의 PieceId다. 일반 HP 공격은 한 번에 HP 1을 줄이고 공격자는 움직이지 않는다. 대형 착지 포획은 HP 공격과 별도다.
- 변신(폰 승격/종자/킹 체커)은 origin과 fresh deadline을 함께 갱신한다. 만료된 deadline도 원본처럼 보존한다.

Phase 3은 진행 중이며 카드나 미래 기물의 필드는 미리 추가하지 않는다.

- 전령 전용 `herald_jump_lock_turn`(optional u32), `herald_jump_locked`(false 생략)는 원본의 turn 기반 잠금과 legacy 잠금을 보존한다. 다른 기물에 설정하면 오류다. `herald_agreement`는 인접 왕에 의한 승리 이유다.
- 징집관이 남기는 폰은 `ids.next_piece`를 할당하며 overflow 시 전체 행동을 거절한다. origin은 출발 칸, moved=false다. 원본이 누락한 공통 생성 턴 포획 금지는 owner.completed+1의 기존 fresh status로 적용한다.

- 상인 전용 `gold`는 optional u32이며 생략 시 0으로 동작한다. 자기 턴 시작에 1 증가한다. `Purchase {merchant,target}`는 entity ID 두 개를 받으며 이동이나 일반 포획을 수행하지 않는다. 왕 매수는 `royal_purchase`로 즉시 종료한다. 매수된 기물의 fresh deadline은 보존하되 owner는 새 소유자로 바뀐다.

- `shielded`는 이제 실행 상태다. 직접/앙파상 공격은 방패만 제거하고 공격자가 제자리에 남는다. 체커 점프는 방패를 제거하면서 착지한다. 대형 착지 포획과 섹터 공격은 원본의 별도 보호막 처리 경로를 따른다.

- 마법사 전용 `mana`/`max_mana`는 optional u32, 기본 0/5이며 mana≤max_mana다. 아군 포획마다 상한까지 +1, 턴 시작 보급은 없다.
- `CastSpell {wizard,spell,target}`는 lightning/shield/meteor/time_stop을 받는다. 메테오 target은 0…6의 좌상단, 보호는 아군 entity anchor, 시간 정지는 시전자 anchor로 표현한다.
- `delayed_spells`는 등록 순서가 의미 있는 배열이며 빈 배열은 생략한다. 각 항목은 `{spell:lightning|meteor,anchor,owner,caster}`다. caster는 null 또는 이미 할당된 양의 ID이며 생존을 요구하지 않는다. owner의 상대가 행동을 완료하기 직전에 발동한다.
- `time_stopped`는 중복 없는 색 집합이며 white/black 순으로 정렬하고 빈 집합은 생략한다. 시전 자체는 턴을 끝내지 않고, 다음 행동 후 상대 색의 예약을 소비해 completed/move_count 증가 및 지연 주문 발동 없이 같은 턴을 유지한다.

- 통나무 전용 `log_direction`은 null/생략 또는 `{dr,dc}`이며 각 성분 -1…1, 둘 다 0은 금지다. `log_roll_after_turn`은 optional u32다. `SetLogDirection {piece,direction}`은 제자리에서 방향과 owner.completed+1의 deadline을 기록하고 턴을 끝낸다. 자동 이동은 시간 정지와 지연 주문보다 먼저 실행하며 경계/막힘/HP 충돌 때 방향과 deadline을 함께 지운다.

- 샷건 킹은 단일 칸 footprint와 HP 쌍을 갖는 왕이다. HP가 0이 되면 포획한 색이 승리한다(아군 피해면 반대 색 승리). `ammo`/`max_ammo`는 optional u32, 기본 0/3이며 ammo≤max_ammo다. `facing`은 optional up/down/left/right다. 샷건 필드는 다른 타입에서 거절한다.
- `Reload {piece}`, `ShotgunBlast {piece,direction:{dr,dc}}`, `ShotgunSnipe {piece,target}`를 지원한다. 이동은 빈 인접 칸만, 장전은 +1, 산탄/저격 비용은 각각 2/3이다. 산탄 방향은 보드 내 클릭으로 지정 가능한 8방향으로 한정한다. HP 피해는 기존 규칙처럼 1이다.
- 샷건 킹이 존재하면 반복 position_counts를 갱신하지 않는다. 장기전 판정에서는 샷건 킹 소유자가 패배하며, 양쪽에 있으면 white가 패널티 대상이다. 카드 획득 이력에 의한 패널티는 Phase 5 범위다.

- 빅룩 캐슬링도 기존 왕의 `Move`로 표현한다. 코너의 공유 ID로 빅룩을 찾고 새 2×2 footprint를 배치한다. 새 점유 칸의 아군은 entity 전체를 제거하지만 포획 효과를 발생시키지 않는다. 새 필드나 캐슬링 전용 Action은 필요하지 않다.

- 곰/고슴도치 전용 `bear_retaliations_remaining`은 0…2, `bear_move_locked_until_turn`은 owner completed 기준 deadline이다. 이동 잠금 중 곰의 퀸 위협은 유지하고 고슴도치의 킹 위협은 억제한다.
- `pending_bear_retaliations`는 순서가 의미 있는 배열이며 빈 배열은 생략한다. 각 항목은 포획된 곰 또는 고슴도치 entity clone, 공격자 ID/색, 포획자 색, 피포획 칸, 포획 시 공격자 위치와 감소한 잔여 횟수를 보존한다. 해당 공격자 색의 행동 종료 또는 즉시 공격 경로에서 처리하며 공격자를 HP/방패와 무관하게 entity 전체 제거한다. 복귀 칸이 막히면 피포획 칸을 사용하고 둘 다 막히면 반격 기물은 복귀하지 않는다.
- 대형 기물 착지는 원본의 처리 순서 때문에 곰/고슴도치 반격의 최종 효과가 남지 않는다. 이 경로는 대기열을 만들지 않는다. 거신병 섹터 공격은 일반 즉시 반격 경로를 따른다.
- 캠프파이어는 별도 상태 필드가 없다. 현재 board에서 같은 색 캠프파이어와 직교 인접한 footprint를 조회해 왕 계열이 아닌 아군 entity의 포획을 막는다.
- 랍스터는 별도 상태 없이 소유자 방향의 바로 앞 세 칸으로 이동·포획한다. 카드의 지연 소환 예약은 이 cardless 계약에 포함하지 않는다.
- 슬라임은 직교 방향으로 정확히 세 칸을 뛰며 중간 점유는 보지 않는다. 이동할 때 `ids.next_piece`로 출발 칸에 새 슬라임을 할당하고 moved=true, origin=출발 칸, owner.completed+1의 기존 포획 잠금 status를 설정한다. allocator overflow는 행동 전체를 거절한다.
