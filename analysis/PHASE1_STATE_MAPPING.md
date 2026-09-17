# Phase 1 상태 필드 매핑

이 파일은 `tools/phase1_field_audit.py`로 재생성한다. 292개 점 표기 검색 결과에 reset 리터럴과 동적 passive 키를 합쳤다. 미지원 규칙 상태를 빈 기본값으로 바꾸는 JS snapshot importer는 제공하지 않는다. Rust canonical 입력은 미지 필드를 거절하며, 지원 범위는 `phase1_state_only`이다.

`rule`의 deferred는 규칙 데이터로 보수적으로 보존해야 한다는 뜻이며 구현 완료가 아니다. `unresolved`는 UI/규칙 여부를 단정하지 않는다. alias/dynamic write 위치와 출처는 JSON에 남겼다. 전체 dataflow 분석을 완료했다고 주장하지 않는다.

| JS 필드 | 분류 | Rust 대응 / 처리 |
|---|---|---|
| `acceleration` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `accelerationPendingFor` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `accelerationPendingTurns` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `accelerationStartsAfterBlackTurns` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `accelerationTrail` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `actionsRemaining` | rule | turn.actions_remaining |
| `activeHistoryMoveNumber` | presentation | excluded from core; no UI callback/timer serialization |
| `activeTrolley` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `additionalRuleCards` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `afterimageQueen` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `aiHumanColor` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `animatedPieceIds` | presentation | excluded from core; no UI callback/timer serialization |
| `appliedRuleCard` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `armistice` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `auctionInventoryOpen` | presentation | excluded from core; no UI callback/timer serialization |
| `authoritativeEndgame` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `authoritativeRepetitionCount` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `backwardKnight` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `barricadeDirectionChoice` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `barricadePreview` | presentation | excluded from core; no UI callback/timer serialization |
| `betaOptionalExtraMove` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `bigRookPreview` | presentation | excluded from core; no UI callback/timer serialization |
| `binaMate` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `bishopSnipe` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `blackHole` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `bloodMoonEffectLibraryOpen` | presentation | excluded from core; no UI callback/timer serialization |
| `board` | rule | board + pieces (cell IDs / unique entities) |
| `boardFlip` | presentation | excluded from core; no UI callback/timer serialization |
| `boardHistory` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `breakthroughPawns` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `brilliantMove` | presentation | excluded from core; no UI callback/timer serialization |
| `camouflageRule` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `campaign` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `cannonGhostScreen` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `cannonScarecrowScreen` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `captureTheFlag` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `capturedTypes` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `captures` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `cardAcquisitionNonce` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `cardArchiveAvailable` | presentation | excluded from core; no UI callback/timer serialization |
| `cardArchiveOpen` | presentation | excluded from core; no UI callback/timer serialization |
| `cardBanIds` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `cardState` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `cardsUsedThisTurn` | rule | players.*.cards_used_this_turn |
| `castled` | rule | history.castled |
| `castlingCanceled` | rule | history.castling_canceled |
| `castlingMoved` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `chainBonds` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `chaosNoCaptureUntilHalfTurn` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `checkAlertEnabled` | presentation | excluded from core; no UI callback/timer serialization |
| `clock` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `clonePassive` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `clonedPassiveCards` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `collapseDepth` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `collapsePending` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `collapsed` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `collapsedCells` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `completeRandom` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `conscription` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `conscriptionUsed` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `conveyorRule` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `coolGuy` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `cornerKick` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `coronation` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `crownRule` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `deathmatch` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `deathmatchEnabled` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `deathmatchLimitTurns` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `deck` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `deckSlots` | rule | players.*.card_slots (empty slots only; occupied slots rejected) |
| `delayedHazards` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `democracy` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `devLibraryOpen` | presentation | excluded from core; no UI callback/timer serialization |
| `devMode` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `diceLocks` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `disassembly` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `draft` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `draftBalance` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `draftBoardPreview` | presentation | excluded from core; no UI callback/timer serialization |
| `draftClock` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `draftDelete` | rule | config.draft_enabled (inverted; only disabled draft supported) |
| `draftLocked` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `draftPreviewBundleIndex` | presentation | excluded from core; no UI callback/timer serialization |
| `draftPreviewCardId` | presentation | excluded from core; no UI callback/timer serialization |
| `draftResumeTurn` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `dragging` | presentation | excluded from core; no UI callback/timer serialization |
| `drawOffer` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `drawRejection` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `earlyPromotion` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `effects` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `enPassant` | rule | history.en_passant (typed entity/target reference; transition deferred) |
| `enPassantFrenzy` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `encouragement` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `endDraftDone` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `endPhaseStartMove` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `exhaustion` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `extinctionMinorTargets` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `falseStart` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `fastGrowth` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `feudalContracts` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `fianchetto` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `fieldPromotion` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `fileSurge` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `finalWeapon` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `firstMoveCardsForced` | rule | players.*.first_move_cards_forced |
| `firstMoveUndo` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `fog` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `fogOfWar` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `fogWar` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `forceAnimatedPieceIds` | presentation | excluded from core; no UI callback/timer serialization |
| `freeCardUsed` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `freeCastling` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `freeMoveCaptureLock` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `frontlineResponse` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `fullMove` | rule | turn.full_move |
| `gameOver` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `gameStyle` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `genevaConvention` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `gomoku` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `gomokuVictoryCells` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `grandEndTurnCounts` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `hallucination` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `hands` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `highGround` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `highlander` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `highway` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `hillKing` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `historyViewIndex` | presentation | excluded from core; no UI callback/timer serialization |
| `idolEncoreUsedByPiece` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `imperialStudies` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `initiative` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `internalSixFixes` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `ironMonarch` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `jokerChoice` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `judgmentExiles` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `killerKing` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `kingDead` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `kingKnight` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `kingThreatCaptureCauses` | derived | recompute when owning rules are ported; not a canonical cache |
| `kingThreatEffectCauses` | derived | recompute when owning rules are ported; not a canonical cache |
| `knightInjury` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `knightmate` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `lastMove` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `lastTurnCaptures` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `legalMoves` | derived | recompute when owning rules are ported; not a canonical cache |
| `localMode` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `locustSwarm` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `logHidden` | presentation | excluded from core; no UI callback/timer serialization |
| `logs` | presentation | excluded from core; no UI callback/timer serialization |
| `longEnPassant` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `machoChess` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `madAi` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `madHorse` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `magicGirlSurge` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `magicGirlSurgeRefreshPending` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `majesty` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `middleDraftDone` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `miracleSelectsBishop` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `mistakeCard` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `mistakeRule` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `mode` | rule | phase (play/terminal only; idle/rule-event/draft not imported) |
| `monochromeChess` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `moveCount` | rule | turn.move_count |
| `moveReplay` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `moving` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `mutation` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `necromancy` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `notationEvent` | presentation | excluded from core; no UI callback/timer serialization |
| `notationEvents` | presentation | excluded from core; no UI callback/timer serialization |
| `notationTimeline` | presentation | excluded from core; no UI callback/timer serialization |
| `onlineEventNonce` | presentation | excluded from core; no UI callback/timer serialization |
| `onlineEvents` | presentation | excluded from core; no UI callback/timer serialization |
| `onlineGameStarted` | presentation | excluded from core; no UI callback/timer serialization |
| `openingAutoNoticeShown` | presentation | excluded from core; no UI callback/timer serialization |
| `opponentLibraryOpen` | presentation | excluded from core; no UI callback/timer serialization |
| `overtake` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `overtakeTurnOnly` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `overwhelm` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `palaces` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `parrotBasicMovement` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `parrotMovement` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `parrotRookTarget` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `pawnConversion` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pawnLeap` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pawnSprint` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingBearRetaliations` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingFeudalStrike` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingFreeMoves` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingGales` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingIcbm` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingLobsters` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingNotation` | presentation | excluded from core; no UI callback/timer serialization |
| `pendingNotations` | presentation | excluded from core; no UI callback/timer serialization |
| `pendingOtherworld` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingPanic` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingPawnStorm` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingPortals` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingPromotion` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingRecurrences` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingReplayVisuals` | presentation | excluded from core; no UI callback/timer serialization |
| `pendingRuleTickets` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingScarecrows` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingTrojanHorse` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pendingTrolley` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `periodicCollapse` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `pieceCardFlip` | presentation | excluded from core; no UI callback/timer serialization |
| `platformRule` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `playerCards` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `portalRule` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `positionCounts` | rule | history.position_counts (sorted unique records) |
| `proficiency` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `prophecy` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `quantumPending` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `queensGambitFiles` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `racingKing` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `radicalCharge` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `recycling` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `regency` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `relay` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `religiousVictory` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `repetitionSalt` | rule | history.repetition_salt |
| `replayBaseFrame` | presentation | excluded from core; no UI callback/timer serialization |
| `replayCardHistoryComplete` | presentation | excluded from core; no UI callback/timer serialization |
| `replayEndReason` | presentation | excluded from core; no UI callback/timer serialization |
| `replayEndedAt` | presentation | excluded from core; no UI callback/timer serialization |
| `replayEventNonce` | presentation | excluded from core; no UI callback/timer serialization |
| `replayEvents` | presentation | excluded from core; no UI callback/timer serialization |
| `replayHistoryComplete` | presentation | excluded from core; no UI callback/timer serialization |
| `replayStartedAt` | presentation | excluded from core; no UI callback/timer serialization |
| `replayTailFrame` | presentation | excluded from core; no UI callback/timer serialization |
| `replayTimelineReady` | presentation | excluded from core; no UI callback/timer serialization |
| `resolveMoveCredit` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `resolveReady` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `resolveSpentTurn` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `retreat` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `reversal` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `rookLift` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `royalCommand` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `royalKnightKing` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `ruleBombs` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `ruleOpeningEnabled` | rule | config.opening_rules_enabled (only false supported) |
| `ruleOpeningEvent` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `ruleSelectionEnabled` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `ruleTicketChoice` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `saturationRule` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `selected` | presentation | excluded from core; no UI callback/timer serialization |
| `selectedRuleCardId` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `selectedRuleCardIds` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `serverCardAuthority` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `shotgunAction` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `shotgunDlc` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `shotgunOpeningColor` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `shotgunOpeningForced` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `shotgunPreview` | presentation | excluded from core; no UI callback/timer serialization |
| `simpleBoardEditor` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `simpleBoardEditorCardOverride` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `simpleBoardEditorReturnHref` | presentation | excluded from core; no UI callback/timer serialization |
| `sirenExposure` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `skipTurn` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `socialism` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `starWinLimit` | rule | deferred configuration/external input; only explicit cardless local setup supported |
| `substitution` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `switcheroo` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `symmetry` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `targeting` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `taunt` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `temporaryQueens` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `thiefQuietJump` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `thiefRemake` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `thiefRequiredJump` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `timeErasedPieceIds` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `timeErasingPieceIds` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `timeIsMineSequence` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `toUpperCase` | false-positive | entry.state.toUpperCase at L24848, not game state |
| `transcendenceRule` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `trojanHorse` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `turn` | rule | turn.side |
| `turnCaptures` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `turnResolving` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `turnsTaken` | rule | turn.completed |
| `ultimatum` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `undeadResurrections` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `underpromotion` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `unifiedJumpObstacles` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `vanguard` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `vanguardDiagonalOnly` | rule | fixed latest local profile; behavior remains deferred to owning rules |
| `vanishing` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `winReason` | rule | result.reason (typed subset) |
| `winner` | rule | result (explicit ongoing/win/draw) |
| `winterKingdom` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `wizardImpact` | unresolved | reject full JS snapshot; investigate reads/writes before enabling this feature |
| `wizardPreview` | presentation | excluded from core; no UI callback/timer serialization |
| `wizardSpell` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `zugzwang` | rule | deferred rule state; NOT silently dropped by an importer; feature unsupported |
| `zugzwangConsumesTurn` | rule | fixed latest local profile; behavior remains deferred to owning rules |
