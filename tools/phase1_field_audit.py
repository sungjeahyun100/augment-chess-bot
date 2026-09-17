"""Reproducible conservative field audit; no claim of a full JS state importer."""
import argparse
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
source = (ROOT / 'origin_code/main-DsoigPgV.js').read_text()
inventory = json.loads((ROOT / 'analysis/phase0_inventory.json').read_text())
assert hashlib.sha256(source.encode()).hexdigest() == inventory['sha256']
lines = source.splitlines()
start = source.index('  state = {', source.index('function resetGame('))
end = source.index('\n  };', start)
reset_fields = set(re.findall(r'^    (\w+):', source[start:end], re.M))
static = set(inventory['stateFields'])
# Literal keys missed by state.foo scanning, plus data-driven passive registration.
dynamic = set(re.findall(r'enableNewColorPassive\("(\w+)"', source))
dynamic.update(re.findall(r'restoreReplayColorState\(replay, color2, "(\w+)"', source))
dynamic.update(['falseStart', 'symmetry', 'mutation'])
# SEPTEMBER_PASSIVE_EFFECTS maps passive definitions to effect names. Preserve all
# candidate effect names conservatively, not only names already in state.foo.
for card in inventory['cards']:
    if card.get('passive') and card.get('effect'):
        dynamic.add(card['effect'])
all_fields = static | reset_fields | dynamic
mapped = {
 'board':'board + pieces (cell IDs / unique entities)',
 'turn':'turn.side', 'turnsTaken':'turn.completed', 'fullMove':'turn.full_move',
 'moveCount':'turn.move_count', 'actionsRemaining':'turn.actions_remaining',
 'deckSlots':'players.*.card_slots (empty slots only; occupied slots rejected)',
 'cardsUsedThisTurn':'players.*.cards_used_this_turn',
 'firstMoveCardsForced':'players.*.first_move_cards_forced',
 'mode':'phase (play/terminal only; idle/rule-event/draft not imported)',
 'winner':'result (explicit ongoing/win/draw)', 'winReason':'result.reason (typed subset)',
 'enPassant':'history.en_passant (typed entity/target reference; transition deferred)',
 'castled':'history.castled', 'castlingCanceled':'history.castling_canceled',
 'positionCounts':'history.position_counts (sorted unique records)',
 'repetitionSalt':'history.repetition_salt',
 'draftDelete':'config.draft_enabled (inverted; only disabled draft supported)',
 'ruleOpeningEnabled':'config.opening_rules_enabled (only false supported)',
}
ui = set('''animatedPieceIds forceAnimatedPieceIds boardFlip pieceCardFlip selected dragging
logs logHidden devLibraryOpen cardArchiveOpen cardArchiveAvailable opponentLibraryOpen
bloodMoonEffectLibraryOpen auctionInventoryOpen draftPreviewCardId draftPreviewBundleIndex
draftBoardPreview wizardPreview bigRookPreview barricadePreview shotgunPreview checkAlertEnabled
openingAutoNoticeShown notationEvent notationEvents notationTimeline pendingNotation pendingNotations
pendingReplayVisuals onlineEvents onlineEventNonce onlineGameStarted historyViewIndex
replayBaseFrame replayTailFrame replayEvents replayEventNonce replayStartedAt replayEndedAt
replayEndReason replayHistoryComplete replayCardHistoryComplete replayTimelineReady
simpleBoardEditorReturnHref activeHistoryMoveNumber brilliantMove'''.split())
derived = {'legalMoves', 'kingThreatCaptureCauses', 'kingThreatEffectCauses'}
unknown = set('''cardState authoritativeEndgame authoritativeRepetitionCount gameOver
castlingMoved fog fogOfWar fogWar serverCardAuthority wizardImpact targeting
lastMove boardHistory turnResolving timeErasedPieceIds timeErasingPieceIds timeIsMineSequence'''.split())
config = set('''gameStyle campaign localMode aiHumanColor madAi devMode simpleBoardEditor
simpleBoardEditorCardOverride shotgunDlc shotgunOpeningColor completeRandom ruleSelectionEnabled
selectedRuleCardId selectedRuleCardIds cardBanIds clock draftClock deathmatchEnabled
starWinLimit deathmatchLimitTurns'''.split())
profile = set('''cannonGhostScreen cannonScarecrowScreen internalSixFixes overtakeTurnOnly
vanguardDiagonalOnly parrotRookTarget extinctionMinorTargets unifiedJumpObstacles parrotBasicMovement
miracleSelectsBishop thiefQuietJump thiefRequiredJump thiefRemake zugzwangConsumesTurn'''.split())
first_refs = {}
reference_pattern = re.compile(r'\bstate\??\.([A-Za-z_$][\w$]*)')
for number, line in enumerate(lines, 1):
    for match in reference_pattern.finditer(line):
        first_refs.setdefault(match[1], number)
rows = []
for field in sorted(all_fields):
    if field in mapped:
        category, disposition = 'rule', mapped[field]
    elif field == 'toUpperCase':
        category, disposition = 'false-positive', 'entry.state.toUpperCase at L24848, not game state'
    elif field in ui:
        category, disposition = 'presentation', 'excluded from core; no UI callback/timer serialization'
    elif field in derived:
        category, disposition = 'derived', 'recompute when owning rules are ported; not a canonical cache'
    elif field in unknown:
        category, disposition = 'unresolved', 'reject full JS snapshot; investigate reads/writes before enabling this feature'
    elif field in config:
        category, disposition = 'rule', 'deferred configuration/external input; only explicit cardless local setup supported'
    elif field in profile:
        category, disposition = 'rule', 'fixed latest local profile; behavior remains deferred to owning rules'
    else:
        category, disposition = 'rule', 'deferred rule state; NOT silently dropped by an importer; feature unsupported'
    rows.append(dict(field=field, category=category, disposition=disposition,
                     in_static_inventory=field in static, in_reset=field in reset_fields,
                     dynamic_candidate=field in dynamic, first_reference=first_refs.get(field)))
# Dynamic writes and aliases are an audit index, not proof of complete dataflow.
patterns = [r'\bstate\[', r'Object\.assign\(state\b', r'\b(?:const|let) \w+ = state;',
            r'\b(?:stateRef|sourceState|parentState|previousState|visibleState|resolvingState)\??\.',
            r'\b(?:activeMetalMove|saturationAttackContext|activeMoveReplayCapture|freeMovePopulationBoard)\b']
sites = [dict(line=i+1, code=line.strip()[:300]) for i,line in enumerate(lines)
         if any(re.search(pattern,line) for pattern in patterns)]
result = dict(source_sha256=inventory['sha256'], static_count=len(static), reset_count=len(reset_fields),
              fields=rows, dynamic_and_alias_sites=sites,
              limitation='Conservative lexical audit, not full alias/dataflow proof. Unresolved and deferred features cannot be imported.')
md = '''# Phase 1 상태 필드 매핑

이 파일은 `tools/phase1_field_audit.py`로 재생성한다. 292개 점 표기 검색 결과에 reset 리터럴과 동적 passive 키를 합쳤다. 미지원 규칙 상태를 빈 기본값으로 바꾸는 JS snapshot importer는 제공하지 않는다. Rust canonical 입력은 미지 필드를 거절하며, 지원 범위는 `phase1_state_only`이다.

`rule`의 deferred는 규칙 데이터로 보수적으로 보존해야 한다는 뜻이며 구현 완료가 아니다. `unresolved`는 UI/규칙 여부를 단정하지 않는다. alias/dynamic write 위치와 출처는 JSON에 남겼다. 전체 dataflow 분석을 완료했다고 주장하지 않는다.

| JS 필드 | 분류 | Rust 대응 / 처리 |
|---|---|---|
'''
md += ''.join(f"| `{r['field']}` | {r['category']} | {r['disposition']} |\n" for r in rows)
check = argparse.ArgumentParser()
check.add_argument('--check', action='store_true')
args = check.parse_args()
for file, content in [('analysis/phase1_field_audit.json', json.dumps(result,ensure_ascii=False,indent=2)+'\n'),
                      ('analysis/PHASE1_STATE_MAPPING.md',md)]:
    path = ROOT/file
    if args.check:
        assert path.read_text() == content, f'stale audit: {file}'
    else:
        path.write_text(content)
print(f'Field audit: {len(static)} static, {len(reset_fields)} reset, {len(rows)} union, {len(sites)} dynamic/context sites')
