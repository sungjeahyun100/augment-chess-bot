# augmentchess.org — Rust 엔진 / AI 연동 레퍼런스

이 문서는 augmentchess.org와 별도의 Rust 게임 엔진 및 AI를 연동하거나, Rust 엔진과 실제 사이트의 동작을 비교 검증할 때 필요한 사이트 측 정보만 정리한다.

---

## 0. 핵심 원칙

사이트의 내부 `state`, React 내부 객체 등을 직접 읽거나 수정하는 방식에 의존하지 않는다.

기존 확장 개발 과정에서 내부 state를 직접 조작했을 때 사이트 화면이 멈추는 문제가 발생한 적이 있다.

따라서 사이트 연동 계층은 기본적으로 다음만 사용한다.

* 사이트가 렌더링한 DOM 관찰
* 실제 UI 이벤트를 통한 입력
* 기보 및 화면에 노출된 상태 읽기

Rust 엔진은 augmentchess.org와 독립적으로 유지한다.

권장 구조:

```text
augmentchess.org
      │
      │ DOM observation / UI events
      ▼
 Site Adapter
      │
      │ canonical Action / Observation
      ▼
 Rust Engine
      │
      ├─ AlphaZero / MCTS
      └─ 분석 및 differential testing
```

사이트 전용 selector나 클릭 동작은 Rust core에 포함하지 않는다.

---

# 1. 사이트 버전 주의

사이트는 배포 시 JavaScript bundle의 hash가 변경된다.

따라서 기존에 조사한:

* DOM id
* class
* aria-label
* 이벤트 처리 방식

등은 사이트 업데이트 이후 변경될 수 있다.

Site Adapter가 동작하지 않을 경우 우선 현재 배포된 사이트에서 selector와 이벤트 동작을 다시 검증한다.

---

# 2. 보드 읽기

## 보드 칸

각 보드 칸:

```css
button.square
```

각 칸의 `aria-label`에는 현재 좌표와 기물 정보가 포함된다.

예:

```text
e2 백 폰
e3 빈 칸
```

따라서 DOM으로부터 최소한 다음을 추출할 수 있다.

```text
square
piece color
piece display type
empty / occupied
```

이를 Rust 엔진의 내부 Piece ID와 직접 동일시하지 말고 Site Adapter에서 canonical representation으로 변환한다.

---

# 3. 사이트의 합법수 판정 읽기

기물을 선택하면 이동 가능한 칸에:

```css
.legal
```

class가 추가된다.

예:

```html
<button class="square light legal">
```

따라서 사이트 자체의 legal move 결과를 읽을 수 있다.

이 기능은 Rust 엔진 검증에 특히 중요하다.

예:

```text
사이트에서 기물 선택
        ↓
`.legal` square 수집

Rust Engine
        ↓
동일 state에서 legal actions 생성

        ↓
결과 비교
```

이를 이용해 실제 배포 사이트를 reference implementation으로 사용하는 differential test를 구축할 수 있다.

주의:

`.legal`은 현재 선택한 기물에 대해 사이트가 표시한 목적지를 나타낸다.

카드 사용, 드래프트, 복수 단계 Action 등 모든 종류의 legal action을 직접 나타내는 것은 아니다.

---

# 4. 현재 차례 판별

현재 차례인 플레이어는:

```css
.match-player-card.turn-active
```

로 표시된다.

플레이어 색은:

```text
data-match-player="white"
data-match-player="black"
```

으로 판별할 수 있다.

Site Adapter는 사이트에서 수를 실행하기 전에 실제 현재 차례와 Rust 엔진의 `side_to_move`가 일치하는지 확인하는 것이 좋다.

---

# 5. 보드 입력 방법

보드의 `button.square`는 일반적인:

```javascript
element.click()
```

만으로 정상적으로 동작하지 않을 수 있다.

기물 선택 상태가 pointer 기반 이벤트로 관리되는 것으로 확인되었다.

기존 확장에서 정상 동작한 이벤트 순서는:

```text
pointerdown
mousedown
pointerup
mouseup
click
```

이다.

따라서 Site Adapter에서는 이 동작을 하나의 함수로 캡슐화한다.

예:

```javascript
simulateBoardClick(squareElement)
```

AI나 자동 테스트에서 실제 착수를 실행할 때는:

```text
Action::Move { from, to }

        ↓

from square에 pointer sequence
        ↓
to square에 pointer sequence
```

형태로 변환한다.

사이트의 현재 이벤트 구현은 synthetic event의 `isTrusted === false`를 차단하지 않는 것으로 조사되어 있다.

사이트 업데이트 후에는 이를 다시 확인해야 한다.

---

# 6. 프로모션

프로모션 UI:

```css
#promotionPanel
```

프로모션 선택 버튼:

```css
#promotionChoices .promotion-choice
```

각 버튼의 `aria-label`에 선택할 기물의 한글명이 표시된다.

예:

```text
퀸
```

따라서 canonical Action이:

```text
Promote {
    from,
    to,
    piece
}
```

라면 Site Adapter가 다음 순서로 실행할 수 있다.

```text
1. from 클릭
2. to 클릭
3. promotionPanel 등장 확인
4. 원하는 promotion choice 클릭
```

---

# 7. 카드 타겟팅

특정 대상 선택이 필요한 카드가 활성화되면:

```css
#targetingPanel
```

이 표시된다.

취소 버튼:

```css
#targetingCancelButton
```

따라서 카드 Action이 복수 단계 UI를 요구하는 경우:

```text
카드 클릭
   ↓
targetingPanel 확인
   ↓
target 선택
```

으로 처리할 수 있다.

Rust 엔진에서는 이러한 UI 단계와 게임 Action을 혼동하지 않는다.

예를 들어 Rust에서는:

```text
ActivateCard {
    card,
    target
}
```

처럼 하나의 의미 있는 Action일 수 있지만,

Site Adapter가 이를 여러 UI 입력으로 변환할 수 있다.

---

# 8. 카드 식별

개발자 카드 라이브러리의 각 카드는:

```css
button.dev-card
```

형태이며 다음 속성을 가진다.

```text
data-card-id="dev-{실제 카드 id}"
```

카드 이름은 런타임에 표시명이 변경될 수 있으므로 **이름이 아니라 안정적인 card ID를 기준으로 식별한다.**

카드의 phase는 화면에서 다음 종류로 구분된다.

```text
OPENING
MIDDLE
END
PIECE
RULE
```

Rust 엔진에서도 외부 serialization용 Card ID는 사이트의 안정적인 ID와 대응 가능하도록 유지하는 것이 좋다.

전체 카드 목록은 Rust 포팅 과정에서 현재 JavaScript bundle을 기준으로 별도의 `RULE_INVENTORY.md`에 관리한다.

이 문서에는 193개 카드 전체 목록을 중복 저장하지 않는다.

---

# 9. 기보 읽기

사이트의 기보 표:

```css
#notationTableWrap table.notation-table
```

백 착수:

```css
td:nth-child(2)
```

흑 착수:

```css
td:nth-child(3)
```

일반 체스 착수:

```css
.notation-entry.notation-move
```

카드 사용 등의 기록은 다른 kind로 구분된다.

기보 DOM은 다음 용도로 사용할 수 있다.

* 상대가 방금 수행한 Action 감지 보조
* Rust shadow state와 사이트 진행 상태 동기화
* 기보 추출
* 분석기 입력
* 사이트와 Rust 엔진의 replay 비교

단, 화면 기보가 Rust GameState 전체를 복구할 수 있을 정도로 모든 내부 상태 변화를 기록한다고 가정하지 않는다.

---

# 10. Shadow State 방식

사이트의 모든 내부 상태가 DOM에 노출된다고 보장할 수 없다.

따라서 매 순간 DOM만으로 Rust `GameState` 전체를 새로 만드는 방법보다는, 게임 시작부터 Rust에서 동일한 게임을 병행 실행하는 방식을 우선 고려한다.

```text
사이트 게임                  Rust GameState
    │                              │
    ├── e2-e4 ────────────────────►│ apply(e2-e4)
    │                              │
    ├── 카드 사용 ────────────────►│ apply(card action)
    │                              │
    ├── Nc6 ──────────────────────►│ apply(Nc6)
    │                              │
    ▼                              ▼
```

DOM은 다음 목적으로 사용한다.

* 사이트 Action 관찰
* 사이트에 Action 입력
* 현재 보드 비교
* 현재 턴 비교
* legal move 비교
* 동기화 오류 감지

Rust 엔진이 실제 게임의 canonical state를 병행 유지한다.

---

# 11. Differential Testing

Rust 포팅 검증에서 실제 사이트를 oracle로 사용할 수 있다.

## Legal Move 비교

```text
1. 사이트에서 특정 기물 선택
2. `.legal` square 수집
3. Rust 엔진에서 동일 기물의 legal actions 생성
4. 비교
```

## State 비교

동일한 Action sequence를 사이트와 Rust 양쪽에 적용한다.

이후 최소한 다음 observable state를 비교한다.

```text
board
side to move
promotion state
game end state
기타 DOM에 명확히 노출되는 상태
```

Rust의 내부 canonical state 전체와 DOM state가 완전히 동일한 구조일 필요는 없다.

---

# 12. AI 연동

권장 흐름:

```text
augmentchess.org
      │
      │ 현재 Action / observable state
      ▼
Site Adapter
      │
      ▼
Rust GameState
      │
      ▼
AlphaZero MCTS
      │
      ▼
Canonical Action
      │
      ▼
Site Adapter
      │
      ▼
실제 UI 입력
```

AI가 사이트 DOM 자체를 이해하게 만들지 않는다.

AI는 오직 Rust Engine의 `GameState`와 `Action`만 사용한다.

---

# 13. 온라인 대전 관련 최소 정보

온라인 상태 여부는:

```javascript
document.body.classList.contains("online")
```

으로 확인할 수 있다.

현재 플레이어 및 턴은 앞서 설명한 DOM을 이용한다.

온라인 대전의:

* 채팅
* 방 코드
* 방 삭제
* 관전
* 방장 판별

등은 Rust 엔진과 AI 개발에 필요하지 않으므로 이 문서에서는 다루지 않는다.

사람과의 온라인 대전에서 AI를 사용하는 것은 사이트의 운영 규칙 및 공정성 정책을 별도로 확인해야 한다.

---

# 14. Site Adapter의 책임

Site Adapter는 다음만 담당한다.

```text
observe
- board
- turn
- visible game state
- promotion
- targeting
- notation

execute
- board square click
- card click
- target selection
- promotion selection

verify
- Rust board vs site board
- Rust turn vs site turn
- Rust legal moves vs site `.legal`
```

다음은 담당하지 않는다.

```text
체스 규칙 계산
카드 효과 계산
승패 계산
MCTS
신경망 추론
게임 State의 authoritative representation
```

이 기능들은 Rust Engine 또는 AI 계층의 책임이다.

---

# 15. 유지보수 원칙

사이트 업데이트에 의해 DOM 구조가 변경될 수 있으므로:

```text
Rust Engine
```

과

```text
Site Adapter
```

를 강하게 분리한다.

사이트 selector가 변경되었을 때 수정 대상은 Adapter로 한정되어야 한다.

Rust 엔진의 게임 규칙 구현은 augmentchess.org의 HTML 구조나 CSS class에 의존해서는 안 된다.
