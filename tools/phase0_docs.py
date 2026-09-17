"""Regenerate/check the mechanically extracted appendix (Python standard library only)."""
import argparse
import hashlib
import json
import re
from collections import Counter
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--check', action='store_true')
args = parser.parse_args()
x = json.loads(Path('analysis/phase0_inventory.json').read_text())
source = Path(x['source']).read_text()
lines = source.splitlines()
assert hashlib.sha256(source.encode()).hexdigest() == x['sha256']
assert len(lines) == x['lines']
assert len(x['cards']) == len({c['id'] for c in x['cards']}) == 241
assert Counter(c['phase'] for c in x['cards']) == {'OPENING':37,'MIDDLE':86,'END':56,'PIECE':34,'RULE':27,'GUN':1}
assert len(x['random']) == sum('Math.random' in s for s in lines) == 82
assert all(c['sourceLines'] for c in x['cards'])

def cell(v):
    return str(v if v is not None else '—').replace('|', '\\|').replace('\n', ' ')

def loc(n):
    return f'[L{n}](origin_code/main-DsoigPgV.js#L{n})'

out = ['\n## 전체 카드 및 RULE 정의\n', '별점은 최종 patch 적용 값이다. 설명만으로 실제 capture/turn 처리 순서를 확정하지 않는다.\n']
for phase in ['OPENING','MIDDLE','END','PIECE','RULE','GUN']:
    cards = sorted((c for c in x['cards'] if c['phase']==phase),key=lambda c:c['id'])
    out += [f'\n### {phase} — {len(cards)}개\n', '| ID | 이름 | 별 | 활성화 | effect / target | 설명 | 정의 출처 |', '|---|---|---:|---|---|---|---|']
    for c in cards:
        out.append('| '+ ' | '.join([f"`{c['id']}`",cell(c['name']),cell(c['stars']),c['activation'],f"`{c['effect']}` / {cell(c.get('target'))}",cell(c['text']),loc(c['sourceLines'][0])])+' |')

out += ['\n## 기물 및 타입 표기 전체 목록\n', 'TYPE_LABELS 75개와 별도 `wall`을 합친 76개 표기다. 이는 76개 독립 playable type이라는 뜻이 아니다. `windmillBishop`/`windmillRook`은 표시용 모드이며 `wall`, `blackHole`, `monster` 등은 환경·자동 행동 경로를 함께 확인한다. 전체 타입 참조를 이 목록만으로 제한하지 않는다. 이동 분기는 `getLegalMoves`에서 직접 추출했으며 공통 modifier/포획 제한은 표 위의 설명과 분석 문서를 함께 적용한다.\n', '| 타입 | 이름 | 기본 이동 구현의 첫 식 또는 별도 경로 | 근거 |', '|---|---|---|---|']
start = source.index('function getLegalMoves(')
end = source.index('function finalizeLegalMoves(',start)
move_source = source[start:end]
# Collect grouped cases through each break, keeping the exact first movement expression.
branches = {}
for m in re.finditer(r'((?:\s*case "[^"]+":\s*)+)(.*?\bbreak;)',move_source,re.S):
    kinds = re.findall(r'case "([^"]+)"',m[1])
    code = re.sub(r'\s+',' ',m[2]).replace('break;','').strip()
    expr = code[:155] + ('…' if len(code)>155 else '')
    line = source[:start+m.start()].count('\n')+1
    for k in kinds: branches[k]=(expr,line)
labels = {**x['pieceLabels'],'wall':'엄폐물'}
for k,label in sorted(labels.items()):
    if k in branches:
        expr,n=branches[k]
        desc=f'`{cell(expr)}`'
    elif k=='football':
        desc='neutral 공을 현재 turn 소유자로 보고 footballMoves';n=81905
    elif k in ['windmillBishop','windmillRook']:
        desc='windmill의 표시 모드; 별도 switch 분기 없음';n=37726
    elif k=='babyBear':
        desc='직접 이동 불가; 턴 시작 자동 이동/성장';n=81904
    else:
        hits=[i+1 for i,s in enumerate(lines) if f'type === "{k}"' in s or f'case "{k}"' in s]
        n=hits[0] if hits else 37688
        desc='이동 switch 외 환경/자동 행동/차단 경로; 개별 helper 확인'
    out.append(f'| `{k}` | {label} | {desc} | {loc(n)} |')

out += ['\n## 모든 Math.random 등장 행\n','아래 82행은 원본 전체 문자열 검색과 개수가 일치한다. 기본 인자 및 한 행의 복수 호출도 포함하므로 난수 draw 수와 다르다. 함수명은 가장 가까운 앞선 최상위 function 선언으로 붙인 위치 안내이며 중첩 함수의 정확한 AST owner를 보증하지 않는다. `createBoardEditor` 등은 상위 영역 이름이다. G=규칙/설정, I=의미 ID(allocator로 대체), U=표시/외부, A=AI 정책, H=공유 helper.\n','| 행 | 영역/함수 | 분류 | 코드 |','|---:|---|---|---|']
ui={21399,24699,32690,34270,45905,45935,49608,49650,53119,66786,74718,75615,92636,92842,92877,93155}
ai={67216,67222,67229,69148,69738}
helpers={54373,54379,95958}
for r in x['random']:
    n=r['line'];code=r['code']
    if n in ui:kind='U'
    elif n in ai:kind='A'
    elif n in helpers:kind='H'
    elif any(t in code for t in ['id:', 'instanceId:', 'const id =', 'const bondId =', 'item2.id =']):kind='I'
    else:kind='G'
    out.append(f"| {loc(n)} | `{r['owner']}` | {kind} | {cell(code)} |")

out += ['\n## 간접 난수 helper 사용 위치\n','다음은 `randomChoice`, `weightedChoice`, `shuffle` 이름의 호출/선언을 정적 검색한 전 위치다. 이 helper를 쓰는 신규 카드도 Chance audit 대상이다. 주입된 `random()` 인자 경로는 위 Math.random 기본 인자 함수와 함께 확인한다. 고정 결과를 먼저 만들고 UI만 난수로 연출하는 룰렛은 결과 draw와 표시 draw를 분리한다.\n']
for helper in ['randomChoice','weightedChoice','shuffle']:
    hits=[i+1 for i,s in enumerate(lines) if re.search(r'\b'+helper+r'\s*\(',s)]
    out.append(f"- `{helper}`: {len(hits)}행 — "+', '.join(str(n) for n in hits)+'.')
out += ['\n## 원본 ID 별칭\n','| 구 ID | 현재 ID |','|---|---|']
for a,b in x['aliases'].items():out.append(f'| `{a}` | `{b}` |')
out += ['\n## 재현 및 검증\n','저장소 루트에서 실행한다. Node/Python 표준 라이브러리만 사용하고 패키지 다운로드나 브라우저 초기화는 하지 않는다.\n','```sh\nnode tools/phase0_inventory.cjs\npython3 tools/phase0_docs.py --check\n```\n','원본 변경 뒤 목록을 의도적으로 재생성할 때는 `python3 tools/phase0_docs.py`를 실행한다. 현재 검사는 source hash, 행 수, 241개 ID 유일성/분류 수, 정의 출처, Math.random 82행 및 생성 문서 일치를 확인한다. 게임 규칙의 실행 테스트나 Rust differential test가 아니다. 제한된 data declaration evaluator는 이 고정 번들의 Phase 0 추출용이며 일반 JS parser가 아니다.']
marker='<!-- GENERATED_CATALOG -->'
p=Path('RULE_INVENTORY.md')
old=p.read_text()
assert marker in old
new=old.split(marker)[0]+marker+'\n'+'\n'.join(out)+'\n'
if args.check:
    assert old==new,'Generated inventory differs; run tools/phase0_docs.py'
else:p.write_text(new)
print(f'OK: 241 definitions, 27 RULEs, 76 type labels (including aliases/environment), 82 random rows; hash and documentation coverage verified.')
