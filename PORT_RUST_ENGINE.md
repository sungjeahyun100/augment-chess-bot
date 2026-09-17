# 목표

첨부된 증강체스 웹사이트 코드 ZIP을 분석하여, 현재 브라우저 내부에서 동작하는 게임 규칙 엔진을 **독립적인 Rust headless 엔진**으로 포팅하라.

이 작업의 목표는 단순히 현재 게임이 동작하도록 복제하는 것이 아니다.

장기적으로 이 Rust 엔진은 다음 용도로 사용될 예정이다.

* AlphaZero 기반 MCTS 자기대국
* Policy/Value Network 학습
* 기보 평가
* 대량 시뮬레이션
* 봇
* 추후 WASM을 통한 웹사이트 게임 엔진 교체 가능성
* 추후 Python binding을 통한 PyTorch 학습 코드 연결

따라서 UI를 재현하지 말고, **결정론적이고 테스트 가능하며 대량 탐색에 적합한 게임 코어**를 만드는 것을 최우선 목표로 한다.

## augmentchess.org와의 경계

이 Rust 엔진은 augmentchess.org 웹사이트 내부 구현에 종속되어서는 안 된다.

현재 웹사이트의 JavaScript bundle은 **게임 규칙의 Reference Implementation**으로만 사용한다.

향후 실제 사이트와의 연결은 별도의 `Site Adapter` 계층이 담당한다.

```text
augmentchess.org
      │
      │ DOM observation / UI events
      ▼
 Site Adapter
      │
      │ canonical State / Action
      ▼
 Rust Engine
      │
      ├─ AlphaZero
      └─ 분석기
```

Rust core에는 다음을 포함하지 않는다.

* DOM selector
* browser event
* React state 접근
* 사이트 전용 UI 제어
* Chrome Extension 코드

즉, 사이트 구조가 변경되어도 Rust 엔진 자체는 수정할 필요가 없는 구조를 유지하라.

---

# 가장 중요한 원칙

## 1. 기존 JS 구현을 Reference Implementation으로 사용하라

첨부된 ZIP 안의 기존 웹사이트 코드는 현재 증강체스 규칙의 사실상 기준 구현이다.

규칙을 추측해서 다시 만들지 말라.

먼저 기존 코드를 충분히 조사하여 다음의 실제 동작을 확인하라.

* GameState에 해당하는 데이터
* 초기 게임 생성
* 기물 이동 생성
* 합법수 판정
* 착수 적용
* 잡기
* 특수 기물
* 카드 효과
* 턴 진행
* 체크 및 킹 관련 판정
* 승리/패배/무승부
* 드래프트
* RULE
* 랜덤 효과
* 기존 AI용 상태 snapshot
* 기존 시뮬레이션용 state clone
* 게임 중 발생하는 모든 중요한 상태 변화

현재 코드에는 다음과 같은 함수 및 구조가 존재하는 것으로 확인되어 있다.

* `createInitialBoard()`
* `getLegalMoves(row, col)`
* `collectValidAiActions(color)`
* `movePiece(...)`
* `movePieceCore(...)`
* `movePieceAttack(...)`
* `endMove(...)`
* `completeTurnAfterMove(...)`
* `cloneStateForKingThreatSimulation()`
* `aiWorkerStateSnapshot()`

실제 이름이 번들 과정에서 일부 변경되었거나 추가 함수가 존재할 수 있으므로, 위 목록만 믿지 말고 호출 관계를 직접 조사하라.

---

# 2. JS 코드를 Rust로 기계적으로 번역하지 말라

현재 JS 구현은 UI와 게임 규칙이 강하게 결합되어 있다.

예를 들어 게임 처리 과정에 다음 종류의 코드가 섞여 있을 수 있다.

* render
* DOM
* toast
* sound
* animation
* status display
* user input
* logging
* browser event
* timeout
* Web Worker communication

Rust 엔진에는 이러한 기능을 포함하지 않는다.

Rust 엔진은 오직 다음만 담당한다.

> 현재 GameState + Action → 다음 GameState

UI 표현은 완전히 엔진 외부의 책임으로 둔다.

---

# 3. 처음부터 확장 가능한 규칙 엔진으로 설계하라

증강체스에는 앞으로 계속 새로운 카드가 추가될 예정이다.

특히 많은 카드는 기존 체스의 규칙 자체를 깨거나 수정하는 형태가 될 가능성이 높다.

따라서 다음처럼 구현해서는 안 된다.

```rust
if has_card(CardA) {
    ...
}

if has_card(CardB) {
    ...
}

if has_card(CardC) {
    ...
}
```

이런 예외를 `move_piece`, `capture`, `end_turn` 등에 계속 추가하는 구조는 금지한다.

대신 다음 개념을 중심으로 설계하라.

* GameState
* Action
* Event
* Effect
* Rule
* Trigger
* Condition
* Modifier
* Target

대략적으로 다음 방향을 지향한다.

```text
Action
  ↓
Validation
  ↓
Event / Rule processing
  ↓
Effects
  ↓
State mutation
  ↓
Generated events
  ↓
Resolution
  ↓
Next GameState
```

단, 처음부터 지나치게 추상적인 범용 게임 엔진을 만들지는 말라.

**현재 증강체스 규칙을 실제로 포팅하면서 필요한 abstraction만 도입하라.**

---

# 공식 카드 규칙

증강체스에는 다음 카드 분류가 존재한다.

* OPENING
* MIDDLE
* END
* PIECE
* RULE

또한 일반 카드는 작동 방식에 따라 다음으로 나뉜다.

* Passive
* Active

카드의 게임 단계 분류와 Activation 방식은 서로 다른 개념으로 취급하라.

예:

```rust
enum CardCategory {
    Opening,
    Middle,
    End,
    Piece,
}

enum ActivationType {
    Passive,
    Active,
}
```

RULE은 일반 카드와 동작 방식이 매우 다르므로 일반 카드 시스템에 억지로 포함시키지 말고 별도의 Match Rule / Rule Definition 계층으로 만드는 것을 우선 고려하라.

---

# 드래프트 공식 규칙

카드는 한 게임에서 총 세 번의 드래프트를 통해 획득한다.

### 첫 번째 드래프트

다음 카드가 등장할 수 있다.

* OPENING
* MIDDLE
* PIECE

### 두 번째 드래프트

* MIDDLE
* PIECE

### 세 번째 드래프트

* MIDDLE
* END

카드에는 별 등급이 존재한다.

높은 별 카드는 일반적으로 강하지만:

* 등장 확률이 낮을 수 있다.
* 장기전 종료 판정에서 불리할 수 있다.

`완전 무작위` 설정에서는 모든 카드가 동일한 등장 확률을 가진다.

드래프트는 UI 기능으로 취급하지 말라.

게임의 실제 상태 전이 일부이다.

따라서 필요하면 다음과 같은 Action으로 표현하라.

```rust
Action::ChooseDraftCard { ... }
```

AlphaZero가 최종적으로 드래프트 선택까지 탐색할 수 있는 구조여야 한다.

---

# OPENING 카드

OPENING 카드는 플레이어가 획득한 뒤,

> 해당 플레이어의 첫 번째 이동이 완료된 직후 자동 사용된다.

이를 UI callback 등으로 구현하지 말고 엔진의 이벤트/트리거 시스템에서 처리하라.

예:

```text
MoveResolved
    ↓
해당 플레이어의 첫 이동인가?
    ↓
Opening 카드 발동
```

---

# Passive 카드

Passive 카드는 플레이어가 별도로 사용하지 않는다.

단순히 획득 즉시 한 번 실행되는 효과라고 가정하지 말라.

Passive는 다음처럼 지속적인 규칙 변화 또는 Trigger일 수도 있다.

* 기물이 잡힐 때 발동
* 특정 이동을 막음
* 이동 규칙 변경
* 특정 기물 보호
* 턴 시작 효과
* 턴 종료 효과
* 승리 조건 변경

따라서 Passive는 필요에 따라 Rule / Trigger / Modifier로 등록될 수 있어야 한다.

---

# Active 카드

Active 카드는:

> 자신의 턴 안에서 원하는 시점에 사용할 수 있다.

따라서 Action system에서 일반 기물 이동과 동일한 1급 행동으로 취급해야 한다.

예:

```rust
enum Action {
    Move(...),
    ActivateCard(...),
    DraftChoice(...),
}
```

중요:

**Action과 Turn을 동일한 개념으로 가정하지 말라.**

한 턴 동안 Active 카드 사용 후 기물 이동을 수행하는 등 여러 Action이 존재할 수 있다.

---

# PIECE 및 변형 기물

일부 카드는 기존 체스에 존재하지 않는 새로운 기물을 생성할 수 있다.

생성된 기물을 카드 내부의 특수 데이터로 따로 관리하지 말고 정상적인 Piece Entity로 취급하라.

공식 공통 규칙:

> 새로 생성된 기물은 생성된 그 턴 동안 기물을 잡을 수 없다.

단순 이동은 가능하다.

이 규칙을 개별 카드마다 반복해서 구현하지 말고 공통 상태 또는 상태 효과로 처리하라.

예:

```rust
Status::CannotCaptureUntilNextTurn
```

또는 동등한 구조.

---

# RULE

RULE은 일반 카드와 완전히 다르게 동작한다.

* 일반 드래프트에는 등장하지 않는다.
* 일반 설정에서는 게임 시작 시 일정 확률로 자동 적용된다.
* 직접 RULE 선택 모드에서는 원하는 RULE 또는 없음 상태를 선택할 수 있다.
* 멀티플레이에서는 방 생성자의 설정을 사용한다.
* RULE은 게임 전체의 규칙을 변경한다.

따라서 RULE은 가급적 다음과 같은 수준에서 다뤄라.

```text
Game configuration
      ↓
Rule set 생성
      ↓
GameState 생성
```

RULE은 다음 영역까지 변경할 가능성이 있다고 가정하고 구조를 설계한다.

* 이동 규칙
* 보드 규칙
* 승리 조건
* 턴 진행
* 카드
* 드래프트
* 기물 행동

---

# Effect 설계

카드가 `GameState`를 임의로 직접 수정하는 구조를 피하라.

가능하면 카드나 Rule은 작은 Engine Primitive 또는 Effect를 생성하게 하라.

예:

```text
MoveEntity
RemoveEntity
CreateEntity
TransformEntity
ChangeOwner
AddStatus
RemoveStatus
AddRule
RemoveRule
SetValue
EmitEvent
```

카드의 고수준 개념 하나마다 Rust enum variant를 추가하는 방식은 피한다.

예를 들어:

```rust
Effect::ResurrectLastCapturedPiece
```

보다는

```text
History 조회
→ CreateEntity
→ Position 설정
→ Event 발생
```

처럼 기존 primitive 조합으로 표현할 수 있으면 그렇게 한다.

---

# Native escape hatch

모든 미래 카드를 DSL이나 공통 Effect만으로 표현할 수 있다고 가정하지 말라.

증강체스에서는 기존 규칙의 틀을 깨는 매우 특수한 카드가 계속 추가될 수 있다.

따라서 필요하다면 다음 두 경로를 허용하라.

```text
일반 카드
→ declarative definition / rule/effect composition

매우 특수한 카드
→ native Rust implementation
```

단, Native 구현도 가능한 한 공통 Engine API를 사용하고 GameState 내부 구조를 무제한으로 직접 조작하지 않도록 한다.

---

# Event system

다음과 같은 의미 있는 게임 상태 전이를 Event 후보로 고려하라.

* ActionRequested
* ActionValidated
* BeforeMove
* AfterMove
* BeforeCapture
* AfterCapture
* BeforeSpawn
* AfterSpawn
* BeforeRemove
* AfterRemove
* TurnStarted
* TurnEnding
* TurnEnded
* GameEnding
* GameEnded

필요하면 현재 JS 규칙을 분석하면서 추가한다.

의미 없는 지나친 이벤트 세분화는 피하라.

---

# Rule conflict

향후 카드끼리 이런 충돌이 발생할 수 있다.

* 이 기물은 죽지 않는다.
* 이 효과로 제거된 기물은 보호 효과를 무시한다.
* 모든 보호 효과를 무효화한다.

따라서 규칙 간 충돌 처리를 우연한 코드 실행 순서에 맡기지 않는다.

필요한 경우 다음과 같은 명시적 개념을 도입하라.

* priority
* replacement effect
* prevention effect
* override
* after-effect

현재 게임 규칙에 실제로 필요한 수준부터 구현하고 확장 가능성을 남긴다.

---

# Determinism

AlphaZero와 기보 재생 때문에 매우 중요하다.

가능한 한 다음이 성립해야 한다.

```text
같은 GameState
+
같은 Action
+
같은 명시적 random/chance 입력
=
항상 같은 Next GameState
```

Rust 엔진 내부에서 아무 곳에서나 global RNG를 호출하지 말라.

기존 JS 코드의 `Math.random()` 사용 위치를 모두 조사하라.

랜덤성이 게임 규칙에 영향을 준다면 다음 중 적절한 방식을 설계하라.

* RNG state를 GameState 일부로 저장
* seed 기반 deterministic RNG
* Chance Event를 명시적인 state transition으로 표현

기보 replay와 differential test가 가능해야 한다.

---

# Action 모델

Action을 처음부터 명확한 자료형으로 만든다.

예시:

```rust
enum Action {
    Move {
        from: Square,
        to: Square,
    },

    Promote {
        from: Square,
        to: Square,
        piece: PieceType,
    },

    ActivateCard {
        card: CardInstanceId,
        targets: Vec<Target>,
    },

    ChooseDraftCard {
        card: CardId,
    },

    // 실제 게임을 분석하여 필요한 Action 추가
}
```

UI의 클릭 순서를 Action으로 착각하지 말라.

게임 규칙상 필요한 선택을 표현해야 한다.

다단계 선택이 필요한 경우에는 필요하면 `pending choice`를 GameState에 명시적으로 저장하라.

---

# AlphaZero 요구사항

최종 엔진 API는 최소한 다음 기능을 지원해야 한다.

```rust
GameState::new(...)
GameState::legal_actions()
GameState::apply_action(...)
GameState::is_terminal()
GameState::result()
GameState::side_to_move()
```

추후 다음을 추가하기 쉽게 설계한다.

```rust
GameState::encode()
Action::policy_index()
```

MCTS에서 매우 많은 상태 전이가 발생하므로 성능을 고려한다.

그러나 초기 구현 단계에서는:

**정확성 > 구조 > 성능**

순으로 우선한다.

처음부터 bitboard, unsafe, custom allocator 등 과도한 최적화를 하지 말라.

---

# Clone / Apply / Undo

초기 구현은 단순성을 위해:

```rust
let child = state.clone();
child.apply_action(action);
```

형태여도 된다.

다만 장기적으로 MCTS 성능을 위해 다음 구조를 적용할 수 있도록 내부 설계를 막지 말라.

```rust
let undo = state.apply(action);

search(state);

state.undo(undo);
```

현재 단계에서 무리하게 undo 시스템까지 완성할 필요는 없지만, 불필요한 전역 상태나 외부 side effect 때문에 향후 구현이 불가능해지는 구조는 피한다.

---

# 내부 데이터 표현

성능이 중요한 엔진 내부에서는 문자열 비교 남발을 피한다.

예:

```rust
enum PieceType
enum CardId
enum RuleId
enum Color
```

또는 compact integer ID를 활용한다.

다만 외부 serialize/deserialize, debug, 테스트에서는 사람이 읽을 수 있는 안정적인 string ID를 제공하라.

---

# Canonical State

JS 엔진과 Rust 엔진이 동일한 결과를 내는지 자동 검증할 수 있도록 **Canonical State representation**을 정의하라.

예:

```json
{
  "turn": "white",
  "board": [],
  "cards": [],
  "effects": [],
  "rules": [],
  "draft": {},
  "winner": null
}
```

실제 게임에 필요한 모든 의미 있는 상태를 포함한다.

내부 Rust 자료구조와 동일할 필요는 없다.

중요한 것은:

> 의미상 동일한 게임 상태라면 JS와 Rust가 동일한 canonical representation을 만들 수 있어야 한다.

정렬 순서 등 직렬화 결과도 안정적으로 만든다.

---

# Differential Testing

이 포팅에서 가장 중요한 검증 방법 중 하나다.

기존 JS 엔진과 Rust 엔진에 동일한 상태를 주고 비교하라.

### 1. Legal Action 비교

```text
JS legal actions
vs
Rust legal actions
```

순서와 관계없이 의미상 같은 action set인지 비교한다.

### 2. State Transition 비교

동일한 Action을 양쪽에 적용한다.

```text
JS State
 + Action
→ JS Next State

Rust State
 + Action
→ Rust Next State
```

Canonical State가 동일해야 한다.

### 3. Random game differential test

가능하면 자동으로 무작위 합법수를 선택하여 상당히 많은 게임을 진행한다.

각 ply마다:

* legal actions
* resulting state
* turn
* effects
* victory state

를 비교한다.

불일치가 발생하면:

* random seed
* action history
* JS state
* Rust state
* first divergent field

를 출력하여 재현할 수 있게 한다.

---

# 기존 JS의 버그를 무조건 복제하지 말라

Reference Implementation에 명백한 버그가 발견될 수 있다.

이 경우 조용히 Rust 쪽에서 수정하지 말고 반드시 기록하라.

다음처럼 분류한다.

```text
Observed JS behavior:
Expected rule:
Likely bug:
Rust decision:
```

규칙 의도가 불분명하면 기존 JS 동작을 우선 보존하고 TODO로 명시한다.

---

# 권장 Rust 프로젝트 구조

실제 분석 후 변경해도 되지만 다음과 비슷한 구조를 우선 고려한다.

```text
engine/
  src/
    lib.rs

    state/
    action/
    board/
    piece/

    rules/
    events/
    effects/
    conditions/
    targets/

    cards/
      definitions/
      native/

    draft/
    turn/
    victory/

    serialization/
    testing/
```

과도하게 파일을 세분화하지 말고 실제 책임 단위로 정리하라.

---

# 작업 순서

한 번에 모든 규칙을 Rust로 옮기려고 하지 말라.

## Phase 0 — 분석

먼저 ZIP을 분석해서 다음 문서를 작성한다.

### ENGINE_ANALYSIS.md

포함할 것:

* 기존 JS 게임 상태 구조
* 주요 함수와 호출 관계
* 이동 처리 흐름
* 턴 처리 흐름
* 카드 처리 흐름
* 드래프트 흐름
* RULE 흐름
* 랜덤 요소
* 승패 처리
* UI와 엔진이 결합된 지점
* AlphaZero에 필요한 Action 종류
* Rust 포팅 시 주요 위험 요소

### RULE_INVENTORY.md

현재 코드에서 발견할 수 있는 모든:

* 카드
* 기물
* Rule
* 상태 효과
* 특수 이동
* 특수 잡기
* 승리 조건

을 가능한 범위에서 목록화한다.

### PORTING_PLAN.md

Rust 구현을 작은 단위로 쪼갠다.

---

## Phase 1 — Rust core skeleton

먼저 최소한 다음을 만든다.

* GameState
* Board
* Piece
* Player
* Action
* Turn state
* deterministic serialization
* canonical state

아직 모든 카드를 구현하지 않는다.

---

## Phase 2 — 기본 체스

우선 카드가 없는 상태에서 기본 게임을 구현한다.

* 기본 기물 이동
* 잡기
* 체크 관련 규칙
* 캐슬링
* 앙파상
* 프로모션
* 턴 진행
* 종료 판정

그리고 JS와 differential test를 수행한다.

---

## Phase 3 — 변형 기물

웹 코드에 존재하는 특수/변형 기물을 단계적으로 포팅한다.

각 기물마다 반드시 테스트를 추가한다.

---

## Phase 4 — Event / Effect / Rule

카드를 본격적으로 옮기기 전에 카드가 필요로 하는 공통 primitive와 이벤트 시스템을 구축한다.

기존 JS 카드들을 실제 사례로 사용하여 abstraction이 현실적으로 맞는지 검증한다.

---

## Phase 5 — 카드

단순 카드부터 시작한다.

권장 순서:

1. 단순 Passive
2. 단순 Active
3. OPENING
4. PIECE 생성 카드
5. 복잡한 Trigger 카드
6. 규칙 변경 카드
7. 매우 특수한 카드

각 카드를 구현할 때 기존 JS 동작을 테스트 fixture로 만든다.

---

## Phase 6 — Draft

세 번의 드래프트 및 등장 풀을 포팅한다.

완전 무작위 옵션도 포함한다.

---

## Phase 7 — RULE

게임 시작 설정 및 Match Rule 구조를 구현한다.

---

## Phase 8 — Differential fuzz/random testing

대량의 랜덤 게임을 JS와 Rust에서 함께 돌려 불일치를 찾는다.

---

## Phase 9 — Benchmark

정확성이 충분히 검증된 후에만 성능을 측정한다.

최소 측정 대상:

* initial state 생성
* legal action generation
* action apply
* state clone
* 전체 random playout
* 초당 state transition 수

성능 병목을 실제 profile 결과 없이 추측해서 대규모 리팩터링하지 말라.

---

# 코드 품질 원칙

* 외부 입력을 신뢰하지 않는다.
* GameState invariant를 명확히 한다.
* UI와 엔진을 분리한다.
* global mutable state를 피한다.
* 결정론적 테스트를 우선한다.
* 새로운 카드 하나 때문에 unrelated core module 여러 개를 수정해야 하는 구조를 피한다.
* 하드코딩된 카드 이름 검사 대신 Rule/Effect/Trigger 시스템을 우선한다.
* 지나친 추상화도 피한다.
* 현재 실제 게임 규칙을 표현하기 위해 필요한 abstraction을 만든다.

---

# 중요: 현재 JS 구조를 그대로 재현하지 말 것

기존 JS 코드에 거대한 `movePieceAttack()` 같은 함수가 존재한다고 해서 Rust에서도 같은 구조를 만들지 말라.

기존 구현의 **행동 의미는 보존하되 구조적 문제까지 복제하지 않는다.**

예:

```text
기존 JS

movePieceAttack()
  ├ movement
  ├ capture
  ├ portal
  ├ promotion
  ├ card
  ├ special piece
  ├ UI
  ├ sound
  └ turn handling
```

Rust에서는 책임을 적절히 분리한다.

---

# 최종 목표 API 예시

사용자가 궁극적으로 다음 수준으로 엔진을 사용할 수 있는 상태를 목표로 한다.

```rust
let mut game = GameState::new(config, seed)?;

let actions = game.legal_actions();

game.apply_action(actions[0].clone())?;

if game.is_terminal() {
    println!("{:?}", game.result());
}
```

그리고 추후 Python에서는 다음처럼 사용할 수 있어야 한다.

```python
state = engine.GameState(...)

actions = state.legal_actions()
state.apply_action(actions[0])
```

Python binding 자체는 현재 포팅보다 우선순위가 낮다.

먼저 Rust core를 독립적으로 완성하고 검증한다.

---

# 완료 조건

이 프로젝트의 첫 번째 큰 완료 기준은:

1. Rust 엔진이 브라우저 없이 실행된다.
2. UI 코드가 전혀 없어도 게임 상태 전이가 가능하다.
3. 주요 기존 기물 및 카드 규칙이 동작한다.
4. 세 번의 드래프트가 엔진 내부에서 동작한다.
5. RULE이 엔진 규칙으로 적용된다.
6. 같은 seed와 선택을 주면 동일한 게임을 재현할 수 있다.
7. JS와 Rust의 주요 테스트 상태에서 legal action이 일치한다.
8. 동일한 action sequence에 대해 canonical state가 일치한다.
9. 랜덤 differential test를 반복 실행할 수 있다.
10. AlphaZero MCTS에서 사용할 수 있는 clean GameState/Action API가 존재한다.

---

# 작업 방식

먼저 기존 코드 전체를 조사하라.

즉시 대규모 구현에 들어가지 말고 우선:

1. 기존 엔진 구조 파악
2. 규칙 목록화
3. state/action 모델 설계
4. 포팅 단계 정의

를 수행한다.

그 결과를 문서로 남긴 후 구현을 시작한다.

각 단계가 끝날 때마다:

* 무엇을 구현했는지
* 기존 JS의 어느 동작과 대응되는지
* 어떤 테스트를 추가했는지
* 아직 포팅되지 않은 규칙이 무엇인지
* 발견된 JS 버그 또는 모호성이 무엇인지

를 명확하게 기록하라.

기존 코드를 단순 번역하는 것이 아니라,

> **현재 증강체스의 동작을 정확하게 보존하면서 미래 카드 확장과 AlphaZero 대량 탐색에 적합한 독립 Rust 게임 엔진으로 재구축하는 것**

이 이 작업의 최종 목적이다.

# 참고사항

외부 사이트 연동은 Rust core 포팅 범위에 포함하지 않는다. SITE-REFERENCE.md는 추후 별도의 Site Adapter를 구현할 때 참고한다. Site Adapter는 DOM 관찰 및 UI 입력을 canonical State/Action으로 변환하며, DOM selector나 브라우저 이벤트를 Rust core에 도입하지 않는다. 현재 JavaScript bundle이 게임 규칙 호환성의 primary reference이고 실제 사이트의 observable behavior는 추후 secondary validation source로 사용할 수 있다.

