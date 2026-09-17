# Canonical v2 — phase2_cardless

기계 검증 스키마는 `src/state.rs`의 `CanonicalState`와 각 nested DTO의 serde 정의 및 `GameState::from_snapshot` 불변식 검사다. 샘플은 `tests/fixtures/initial.canonical.json`이다. 이 v2는 **카드 없는 일반 8×8 실행 상태와 기존 state-only 표현**의 계약이며, 모든 카드가 포함된 JS 게임 상태 전체의 스키마라고 주장하지 않는다. 의미 상태를 추가할 때 스키마 버전과 fixture를 갱신하고 구버전 입력을 명시적으로 거절하거나 migration을 제공해야 한다.

- `schema_version=2`, `capability=phase2_cardless` (실행) 또는 `phase1_state_only` (표현 전용), `config.rules_profile=local_0dbbad68`. 원본 전체 SHA-256은 `SOURCE_SHA256` 및 oracle manifest에 고정한다.
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

정규화 대상은 같은 entity ID를 사용하는 의미상 동등 상태다. 서로 다른 ID 배정의 상태를 graph-isomorphism으로 같게 만들지는 않는다. JS fixture는 생성 순서의 단일 ID mapping을 모든 cell/entity 참조에 적용한다. 캐시/DOM/표시 로그는 포함하지 않는다. 카드, 포획 이력, 예약 effect, rollback, terrain 등 아직 모델링하지 않은 실제 규칙 상태는 이 스키마로 가져올 수 없다. 이를 임의의 빈 배열로 대체하여 호환성을 주장하지 않는다.

불변식 검사는 **구조적 일관성**을 보장한다. 외부 snapshot이 실제 합법 행동으로 도달 가능한 상태인지 증명하거나, terminal 결과가 실제 규칙에 맞는지 재판정하지는 않는다. 새 행동 이후의 판정은 실행 엔진의 책임이다.

## v2에 추가된 실행 상태

- `config.star_win_limit`: 양측 완료 턴 중 작은 값이 도달해야 하는 양의 한도(기본 45).
- `config.deathmatch_enabled`: 한도 도달 시 연장 여부(기본 true).
- `config.deathmatch_limit_turns`: 연장전 진행 없음 제한(기본 10). 두 배가 u32 범위 안이어야 한다.
- `pending_promotion`: null 또는 현재 플레이어의 마지막 rank 폰 ID. terminal/다른 소유자/비폰/마지막 rank 외 위치는 거절한다. 이동 뒤 선택을 직렬화하며 Promote 후 null이 된다.
- `deathmatch`: null 또는 `{started_at_turn, half_turns_since_progress, interval_half_turns, progress_this_turn}`. 활성 상태만 객체로 표현한다. UI warning key는 제외한다. 흑 턴 완료 때만 +2하며, 폰 이동/포획은 progress를 표시하고 다음 흑 완료 때 해제한다. 만료는 side/full_move 전환보다 먼저 발생한다.

기본 실행은 8×8, 여섯 기본 기물, 단일 칸 footprint, 양측 소유자, HP/shield 없음, actions_remaining=1, continuation 없음에 한정한다. 다른 표현 가능한 상태는 실행 시 Unsupported다. fresh-capture deadline은 승격 시 owner.completed+1로 기록하며 만료 후에도 원본처럼 보존한다. 승격은 origin도 승격 칸으로 갱신한다.

앙파상은 실제 상대 폰 ID, 목표 칸과 available_to를 보존한다. cardless capability에서는 폰의 moved/두 칸 전진 도착 rank/목표 칸의 기하와 빈칸 여부를 검증한다. 왕 포획은 원본의 조기 반환을 보존하므로 착수자의 moved, 턴 카운터와 이전 앙파상 상태를 갱신하기 전에 종료할 수 있다. 제거된 entity를 가리키는 참조는 남기지 않는다.

스키마 v1에는 프로모션·데스매치·종료 설정이 없고 초기 반복 기록도 다르므로 자동 migration을 제공하지 않는다. `schema_version=1`은 Unsupported다. 규칙상 의미 없는 표시·기보·애니메이션, 비활성 카드용 누적 포획 이력은 이 cardless 계약에 포함하지 않으며, Phase 5에서 해당 이력이 필요한 카드를 추가할 때 모델을 확장해야 한다.
