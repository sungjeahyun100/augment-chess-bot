'use strict';
// Execute hash-pinned function bodies; never load the browser module or its imports.
const fs = require('node:fs'), path = require('node:path'), vm = require('node:vm');
const assert = require('node:assert/strict'), crypto = require('node:crypto');
const ROOT = path.resolve(__dirname,'..');
const source = fs.readFileSync(path.join(ROOT,'origin_code/main-DsoigPgV.js'),'utf8');
assert.equal(crypto.createHash('sha256').update(source).digest('hex'),'0dbbad680c6e8e2abcdb6e49817ff486aa2f8ef799a8035bd356baef8a2731ea');
const bodies = [...source.matchAll(/^function ([\w$]+)\([^]*?^\}$/gm)];
const functions = new Map(bodies.map(m=>[m[1],m[0]]));
function constant(name) {
  const marker = `\nconst ${name} = `, start=source.indexOf(marker)+1;
  assert(start>0,`missing constant ${name}`);
  for(let end=source.indexOf(';',start);end>=0;end=source.indexOf(';',end+1)){
    const body=source.slice(start,end+1);try{new vm.Script(body);return body;}catch{}
  }
  throw new Error('cannot extract '+name);
}
const constants = ['DEATHMATCH_WARNING_STATUS_TITLE','DEATHMATCH_WARNING_MESSAGE','DEATHMATCH_WARNING_HALF_TURNS','DEATHMATCH_START_TOAST','DEATHMATCH_START_STATUS_TEXT','DEATHMATCH_START_STATUS_TITLE','PIECE_NOTATION_CODES','tutorialState','CARD_CATEGORY_GROUPS','CARD_CATEGORY_BY_ID','SIREN_TURN_START_KEY','TYPE_LABELS','RULE_BOMB_COUNT','septemberBoardActionOrigins','MAD_AI_OPTION_DEFS','LOCAL_BOARD_HISTORY_LIMIT','REPLAY_FRAME_EXCLUDED_KEYS','INTERNAL_EIGHT_FLAGS','make','SEPTEMBER_CARD_DEFINITIONS','SEPTEMBER_PASSIVE_EFFECTS','ONLINE_STATE_KEYS','REPLAY_FRAME_KEYS','lightSquare','threeType','usesEnemyOnlyRadiance','SATURATION_CAPTURE_LIMIT','MOVING_STREAK_TARGET','COLORS','COLOR_LABELS','FILES','EXTENDED_FILES','NORMAL_DECK_SLOT_COUNT','CHAOS_DECK_SLOT_COUNT','GRAND_DECK_SLOT_COUNT','GAME_STYLE_IDS','DEFAULT_GAME_STYLE','PROMOTION_TYPES','MINOR_PROMOTION_TYPES','DEATHMATCH_DEFAULT_INTERVAL_TURNS'];
// Explicit UI/clock/replay boundaries. Rule functions are NOT automatically stubbed.
const sinks = ['toast','setStatus','renderAll','renderBoard','renderPromotionPanel','playSound','playMoveSound',
  'markPieceForAnimation','addLog','addPieceActionLog','addMovementLog','recordBoardHistory',
  'startClockForTurn','pauseMainClock','beginMoveReplayCapture','commitMoveReplayCapture',
  'queueMoveHistoryNotation','queueJumpCaptureHistoryNotation','queueSpecialHistoryNotation',
  'queueReplayVisual','amendPendingPromotionNotation','trackAccelerationTrail','setLastMove'];
const reset = functions.get('resetGame');
const object = reset.slice(reset.indexOf('  state = {')+'  state = '.length, reset.indexOf('\n  };',reset.indexOf('  state = {'))+4);
const functionScript = new vm.Script([...functions.values()].filter(body=>!body.includes('import.meta')).join('\n'));
const constantScript = new vm.Script(constants.map(constant).join('\n'));
function makeContext() {
  const ctx = vm.createContext({}, {codeGeneration:{strings:false,wasm:false}});
  functionScript.runInContext(ctx,{timeout:3000});
  constantScript.runInContext(ctx,{timeout:1000});
  vm.runInContext(`
    let acgViewerSession=null; let freeMovePopulationBoard=null; let freeMoveResolution = null; let activeMetalMove = null; let aiSimulationDepth = 0; let kingThreatProbeDepth = 0; let localDeathmatchWarningNoticeKey = ""; let state=null; let playMode='offline';
    let selectedGameStyle='normal', localPlayMode='local', aiHumanColor='white', initialGameStyle='normal';
    let start=false, previousDevMode=false, selectedClockMs=0, selectedFischerIncrementMs=0;
    let betaFriendlyRoomActive=false, online={enabled:false}, fogVisibilityProbeDepth=0;
    let activeMoveReplayCapture=null, saturationAttackContext=null;
    let completeRandomEnabled=false,ruleOpeningEnabled=false,ruleSelectionEnabled=false,selectedRuleCardIds=[];
    let shotgunDlcEnabled=false,boardFlipEnabled=false,pieceCardFlipEnabled=false,deathmatchEnabled=true,checkAlertEnabled=true,draftDeleteEnabled=true;
    let selectedDeathmatchEnabled=true, selectedDeathmatchLimitTurns=10;
    Math.random=()=>{throw new Error('unexpected rules randomness')};
    validRuleSelectionIds=(ids)=>{if(ids.length)throw new Error('RULE unsupported');return [];};
    createClockState=()=>null; createSetupDraftClockState=()=>null; createMadAiState=()=>null;
    ${sinks.map(n=>`${n}=()=>{};`).join('\n')}
    commitClockElapsed=()=>true; clockIncrementMs=()=>0;
    blockAiAction=()=>false; blockOnlineAction=()=>false;
    endGame=(winner,reason)=>{state.winner=winner; state.mode='gameover'; state.oracleReason=reason;};
    createInitialBoard=()=>Array.from({length:8},()=>Array(8).fill(null));
    state=${object}; state.mode='play'; state.middleDraftDone=true; state.endDraftDone=true;
  `,ctx,{timeout:1000});
  return ctx;
}
function run(ctx,code) { return vm.runInContext(code,ctx,{timeout:3000}); }
function plain(ctx,code) { return JSON.parse(run(ctx,`JSON.stringify(${code})`)); }
function load(ctx, input) {
  ctx.input = structuredClone(input);
  run(ctx,`
    state.board=Array.from({length:8},()=>Array(8).fill(null));
    for (const p of input.pieces) {
      const item={id:String(p.id),color:p.owner,type:p.kind,moved:p.moved,shielded:p.shielded,
        origin:p.origin? squareName(p.origin.row,p.origin.col):null};
      for(const status of p.statuses) item.freshNoCaptureUntil=status.completed_turn;
      state.board[p.anchor.row][p.anchor.col]=item;
    }
    state.turn=input.turn.side; state.turnsTaken={...input.turn.completed}; state.fullMove=input.turn.full_move;
    state.moveCount=input.turn.move_count; state.actionsRemaining=input.turn.actions_remaining;
    state.firstMoveCardsForced={white:input.players.white.first_move_cards_forced,black:input.players.black.first_move_cards_forced};
    state.cardsUsedThisTurn={white:input.players.white.cards_used_this_turn,black:input.players.black.cards_used_this_turn};
    state.castled={...input.history.castled}; state.castlingCanceled={...input.history.castling_canceled};
    state.repetitionSalt=input.history.repetition_salt;
    state.positionCounts=new Map(input.history.position_counts.map(e=>[e.key,e.count]));
    const ep=input.history.en_passant;
    const pawn=ep&&input.pieces.find(p=>p.id===ep.pawn);
    state.enPassant=ep?{row:ep.target.row,col:ep.target.col,capturedRow:pawn.anchor.row,capturedCol:pawn.anchor.col,color:ep.available_to==='white'?'black':'white'}:null;
    state.starWinLimit=input.config.star_win_limit; state.deathmatchEnabled=input.config.deathmatch_enabled;
    state.deathmatchLimitTurns=input.config.deathmatch_limit_turns;
    const dm=input.deathmatch;
    state.deathmatch=dm?{active:true,startedAtTurn:dm.started_at_turn,halfTurnsSinceProgress:dm.half_turns_since_progress,
      intervalHalfTurns:dm.interval_half_turns,progressThisTurn:dm.progress_this_turn,warningKey:''}:null;
    const promotion=input.pieces.find(p=>p.id===input.pending_promotion);
    state.pendingPromotion=promotion?{row:promotion.anchor.row,col:promotion.anchor.col,color:promotion.owner,choices:PROMOTION_TYPES}:null;
    if(input.result){state.mode='gameover';state.winner=input.result.winner??null;}
  `);
}
function actions(ctx) {
  return plain(ctx,`(()=>{
    if(state.mode==='gameover')return [];
    if(state.pendingPromotion){const p=state.pendingPromotion;return p.choices.map(into=>({kind:'promote',piece:Number(state.board[p.row][p.col].id),into}));}
    const actions=[];
    forEachSquare((p,row,col)=>{if(p?.color===state.turn)for(const m of getLegalMoves(row,col))actions.push({kind:'move',from:{row,col},to:{row:m.row,col:m.col},route:[]});});
    return actions;
  })()`);
}
function project(ctx, input) {
  const output=structuredClone(input);
  const s=plain(ctx,`({board:state.board,turn:state.turn,turnsTaken:state.turnsTaken,fullMove:state.fullMove,moveCount:state.moveCount,
    actionsRemaining:state.actionsRemaining,firstMoveCardsForced:state.firstMoveCardsForced,cardsUsedThisTurn:state.cardsUsedThisTurn,
    castled:state.castled,castlingCanceled:state.castlingCanceled,enPassant:state.enPassant,counts:[...state.positionCounts],
    pendingPromotion:state.pendingPromotion,deathmatch:state.deathmatch,mode:state.mode,winner:state.winner,reason:state.oracleReason})`);
  output.pieces=[]; output.board.cells=[];
  s.board.forEach((row,r)=>row.forEach((p,c)=>{
    output.board.cells.push(p?Number(p.id):null); if(!p)return;
    const original=input.pieces.find(q=>q.id===Number(p.id)); assert(original,'unexpected spawn');
    const sq={row:r,col:c}, match=/^([a-h])([1-8])$/.exec(p.origin);
    output.pieces.push({...original,kind:p.type,moved:p.moved,anchor:sq,footprint:[sq],
      origin:match?{row:8-Number(match[2]),col:match[1].charCodeAt(0)-97}:null,
      statuses:p.freshNoCaptureUntil?[{kind:'cannot_capture_until_owner_turn',owner:p.color,completed_turn:p.freshNoCaptureUntil}]:[]});
  }));
  output.pieces.sort((a,b)=>a.id-b.id);
  Object.assign(output.turn,{side:s.turn,completed:s.turnsTaken,full_move:s.fullMove,move_count:s.moveCount,actions_remaining:s.actionsRemaining});
  for(const color of ['white','black'])Object.assign(output.players[color],{first_move_cards_forced:s.firstMoveCardsForced[color],cards_used_this_turn:s.cardsUsedThisTurn[color]});
  Object.assign(output.history,{castled:s.castled,castling_canceled:s.castlingCanceled,position_counts:s.counts.map(([key,count])=>({key,count})).sort((a,b)=>a.key<b.key?-1:a.key>b.key?1:0)});
  const ep=s.enPassant, epPawn=ep&&s.board[ep.capturedRow][ep.capturedCol];
  output.history.en_passant=ep&&epPawn?{pawn:Number(epPawn.id),target:{row:ep.row,col:ep.col},available_to:ep.color==='white'?'black':'white'}:null;
  output.pending_promotion=s.pendingPromotion?Number(s.board[s.pendingPromotion.row][s.pendingPromotion.col].id):null;
  const dm=s.deathmatch;
  output.deathmatch=dm?{started_at_turn:dm.startedAtTurn,half_turns_since_progress:dm.halfTurnsSinceProgress,interval_half_turns:dm.intervalHalfTurns,progress_this_turn:dm.progressThisTurn}:null;
  if(s.mode==='gameover'&&!input.result){
    const reason=s.reason.includes('동형반복')?'repetition_stars':s.reason.includes('별')?'turn_limit_stars':s.reason.includes('움직일')?'no_actions':'royal_capture';
    output.phase={kind:'terminal'};output.result=s.winner?{kind:'win',winner:s.winner,reason}:{kind:'draw',reason};
  }
  return output;
}
function createSession(input) {
  const ctx=makeContext(); load(ctx,input);
  let previous=structuredClone(input);
  return {step(action=null) {
    if(action){
      ctx.action=action;
      if(action.kind==='move')run(ctx,`{const a=action;const move=getLegalMoves(a.from.row,a.from.col).find(m=>m.row===a.to.row&&m.col===a.to.col);if(!move)throw new Error('illegal oracle move');movePieceAttack(a.from.row,a.from.col,a.to.row,a.to.col,move);}`);
      else if(action.kind==='promote')run(ctx,'choosePromotion(action.into)');
      else throw new Error('unsupported oracle action');
    }
    previous=project(ctx,previous);
    return {state:previous,actions:actions(ctx),check:plain(ctx,`({white:findPieces('white','king').some(p=>isSquareAttacked(p.row,p.col,'black')),black:findPieces('black','king').some(p=>isSquareAttacked(p.row,p.col,'white'))})`)};
  }};
}
function oracle(input,action) { return createSession(input).step(action); }
function starFixtures() {
  return [[0,0],[3,4],[4,3],[3,3]].map(([white,black])=>{
    const ctx=makeContext();ctx.totals={white:white/2,black:black/2};
    run(ctx,`state.deckSlots={white:[{id:'oracle-white',stars:totals.white},null,null],black:[{id:'oracle-black',stars:totals.black},null,null]};resolveStarTiebreak('3회 동형반복');`);
    const winner=plain(ctx,'state.winner');
    return {stars:{white,black},result:winner?{kind:'win',winner,reason:'repetition_stars'}:{kind:'draw',reason:'repetition_stars'}};
  });
}
function manifest() {
  return {
    source_sha256:crypto.createHash('sha256').update(source).digest('hex'),
    extraction:'top-level function declarations; import.meta functions excluded; explicit constants; resetGame state literal',
    extracted_function_count:functions.size,
    entry_points:['getLegalMoves','isSquareAttacked','movePieceAttack','choosePromotion','resolveStarTiebreak'],
    constants,sinks,
    boundaries:{
      clocks:'disabled; commitClockElapsed succeeds',
      ai_online:'local human play; input authorization wrappers disabled',
      rule_selection:'empty selection only, nonempty IDs throw',
      end_game:'records actual winner/reason and gameover; no browser UI, log export or timers',
      initialization:'resetGame literal; empty board replaced by canonical entity IDs; Phase 1 tests real createInitialBoard',
      randomness:'Math.random throws; cardless rules consume no randomness',
      continuation:'synchronous endMove/completeTurnAfterMove; explicit pending promotion selection',
      excluded_state:'UI/replay/clock fields and dormant card-specific capture/history fields',
      unknown_dependencies:'ReferenceError; no automatic rule mocks or browser/timer globals',
    },
    comparison:'full phase2 canonical state, legal action sets, and both king threat queries; JS state retained across each sequence',
  };
}
module.exports={oracle,createSession,starFixtures,manifest,functionSource:name=>functions.get(name),makeContext,load,project,actions,plain,run,constant};
if(require.main===module){
  if(process.argv.includes('--write-fixtures')){
    fs.writeFileSync(path.join(ROOT,'engine/tests/fixtures/js_star_tiebreak.json'),JSON.stringify(starFixtures(),null,2)+'\n');
    fs.writeFileSync(path.join(ROOT,'analysis/phase2_oracle_manifest.json'),JSON.stringify(manifest(),null,2)+'\n');
  }else if(process.argv.includes('--check-fixtures')){
    assert.deepEqual(JSON.parse(fs.readFileSync(path.join(ROOT,'engine/tests/fixtures/js_star_tiebreak.json'),'utf8')),starFixtures());
    assert.deepEqual(JSON.parse(fs.readFileSync(path.join(ROOT,'analysis/phase2_oracle_manifest.json'),'utf8')),manifest());
    console.log('Phase 2 JS star fixtures and extraction manifest passed');
  }else {const input=JSON.parse(fs.readFileSync(0,'utf8')); console.log(JSON.stringify(oracle(input.state??input,input.action)));}
}
