'use strict';
// Reuse the hash-pinned extraction and Phase 2's explicit browser boundaries.
const base=require('./phase2_oracle.cjs');
function createSession(input) {
  const ctx=base.makeContext();
  base.run(ctx,base.constant('HOOK_ORTHOGONAL_DIRECTIONS')+'\n'+base.constant('hookIcePathsByMove')+'\n'+base.constant('MAX_NOTATION_TEXT')+'\n'+base.constant('ENCYCLOPEDIA_PIECE_VALUES')+'\n'+base.constant('CHESS_N_POW_30_TYPE_ALIASES')+'\n'+base.constant('SHOTGUN_BLAST_AMMO_COST')+'\n'+base.constant('SHOTGUN_SNIPE_AMMO_COST')+'\n'+base.constant('directions'));
  base.load(ctx,input);
  // Replace only opaque identity randomness; execute the real piece constructor.
  ctx.oracleNextPieceId=input.ids.next_piece;
  const identity='`${color2}-${type}-${Math.random().toString(36).slice(2)}`';
  const constructor=base.functionSource('piece');
  if(!constructor.includes(identity))throw new Error('piece identity expression changed');
  base.run(ctx,constructor.replace(identity,'String(oracleNextPieceId++)'));
  const slimeIdentity='`slime-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`';
  const moveAttack=base.functionSource('movePieceAttack');
  if(!moveAttack.includes(slimeIdentity))throw new Error('slime identity expression changed');
  base.run(ctx,moveAttack.replace(slimeIdentity,'String(oracleNextPieceId++)'));

  // Execute the real 500ms animation continuation synchronously, without clocks.
  base.run(ctx,`globalThis.window={setTimeout:(callback)=>callback()};primeStatusMagicLoss=()=>null;scheduleStatusMagicSound=()=>{};queueHpAttackHistoryNotation=()=>{};addAutoLogMoveHighlight=()=>{};showMistakeEffect=()=>{};showParryEffect=()=>{};`);
  base.run(ctx,`for(const p of input.pieces) {
    if(input.turn.continuation?.kind==='checker_capture' && input.turn.continuation.piece===p.id) state.board[p.anchor.row][p.anchor.col].checkerChainCapture=true;
    const item=state.board[p.anchor.row][p.anchor.col];
    if(p.herald_jump_lock_turn!==undefined)item.heraldJumpLockTurn=p.herald_jump_lock_turn;
    if(p.herald_jump_locked)item.heraldJumpUnlocked=false;
    if(p.gold!==undefined)item.gold=p.gold;
    if(p.log_direction!==undefined)item.logDir=p.log_direction;
    if(p.log_roll_after_turn!==undefined)item.logRollAfterTurn=p.log_roll_after_turn;
    if(p.ammo!==undefined)item.ammo=p.ammo;
    if(p.max_ammo!==undefined)item.maxAmmo=p.max_ammo;
    if(p.facing!==undefined)item.facing=p.facing;
    if(p.mana!==undefined)item.mana=p.mana;
    if(p.max_mana!==undefined)item.maxMana=p.max_mana;
    if(p.bear_retaliations_remaining!==undefined)item.bearRetaliationsRemaining=p.bear_retaliations_remaining;
    if(p.bear_move_locked_until_turn!==undefined)item.bearMoveLockedUntilTurn=p.bear_move_locked_until_turn;
    if(p.hp!==null){item.hp=p.hp;item.maxHp=p.max_hp;}
    if(p.footprint.length===4){item.anchorRow=p.anchor.row;item.anchorCol=p.anchor.col;for(const sq of p.footprint)state.board[sq.row][sq.col]=item;}
    if(p.windmill_mode) state.board[p.anchor.row][p.anchor.col].windmillMode=p.windmill_mode;
  }`);
  base.run(ctx,`state.skipTurn={white:(input.time_stopped||[]).includes('white'),black:(input.time_stopped||[]).includes('black')};
    state.delayedHazards=(input.delayed_spells||[]).map(h=>({type:h.spell,cells:wizardPreviewCells(h.spell,h.anchor.row,h.anchor.col),owner:h.owner,triggerAfter:opponent(h.owner),casterId:h.caster===null?'':String(h.caster)}));`);
  base.run(ctx,`state.pendingBearRetaliations=(input.pending_bear_retaliations||[]).map(e=>({
    color:e.color,square:{...e.square},attackerId:String(e.attacker),attackerColor:e.attacker_color,capturedBy:e.captured_by,
    counterDestination:e.counter_destination?{...e.counter_destination}:null,remaining:e.remaining,
    bear:{id:String(e.bear.id),color:e.bear.owner,type:e.bear.kind,moved:e.bear.moved,shielded:e.bear.shielded,
      origin:e.bear.origin?squareName(e.bear.origin.row,e.bear.origin.col):null,
      bearRetaliationsRemaining:e.bear.bear_retaliations_remaining}
  }));`);
  let previous=structuredClone(input);
  return {step(action=null) {
    if(action){
      ctx.action=action;
      if(action.kind==='move')base.run(ctx,`{const a=action;const move=getLegalMoves(a.from.row,a.from.col).find(m=>m.row===a.to.row&&m.col===a.to.col);if(!move)throw new Error('illegal oracle move');movePieceAttack(a.from.row,a.from.col,a.to.row,a.to.col,move);}`);
      else if(action.kind==='set_log_direction')base.run(ctx,`{const found=findPieceById(String(action.piece));const d=action.direction;movePieceAttack(found.row,found.col,found.row+d.dr,found.col+d.dc,{row:found.row+d.dr,col:found.col+d.dc,setLogDirection:d});}`);
      else if(['reload','shotgun_blast','shotgun_snipe'].includes(action.kind))base.run(ctx,`{const p=findPieceById(String(action.piece)).item;if(action.kind==='reload')reloadShotgunKing(p);else if(action.kind==='shotgun_blast')fireShotgunBlast(p,[action.direction.dr,action.direction.dc]);else fireShotgunSnipe(p,action.target.row,action.target.col);}`);
      else if(action.kind==='attack_sector')base.run(ctx,`{const found=findPieceById(String(action.piece));if(!found)throw new Error('missing sector attacker');attackColossusSector(found.item,colossusAttackSectors(found.row,found.col,found.item.color)[action.sector]);}`);
      else if(action.kind==='purchase')base.run(ctx,`{const buyer=findPieceById(String(action.merchant)),target=findPieceById(String(action.target));merchantBuy(buyer.row,buyer.col,target.row,target.col);}`);
      else if(action.kind==='cast_spell')base.run(ctx,`{const found=findPieceById(String(action.wizard));state.selected={row:found.row,col:found.col};chooseWizardSpell(action.spell==='time_stop'?'timeStop':action.spell);if(action.spell!=='time_stop')handleWizardSpellTarget(action.target.row,action.target.col);}`);
      else if(action.kind==='promote')base.run(ctx,'choosePromotion(action.into)');
      else throw new Error('unsupported oracle action');
    }
    // Seed canonical defaults for newly allocated identities, then project actual JS fields.
    const spawned=base.plain(ctx,'state.board.flat().filter(Boolean)');
    for(const p of spawned)if(!previous.pieces.some(q=>q.id===Number(p.id))){
      const pending=(previous.pending_bear_retaliations||[]).find(e=>e.bear.id===Number(p.id));
      if(pending&&['bear','hedgehog'].includes(p.type))previous.pieces.push(structuredClone(pending.bear));
      else {
        if(!['pawn','slime'].includes(p.type))throw new Error('unsupported spawned type '+p.type);
        const slime=p.type==='slime';
        previous.pieces.push({id:Number(p.id),owner:p.color,kind:p.type,anchor:{row:0,col:0},footprint:[],origin:slime?structuredClone(action.from):null,moved:slime,shielded:false,hp:null,max_hp:null,statuses:[]});
      }
    }
    previous.ids.next_piece=ctx.oracleNextPieceId;
    previous=base.project(ctx,previous);
    const board=base.plain(ctx,'state.board');
    previous.pieces=[...new Map(previous.pieces.map(p=>[p.id,p])).values()];
    for(const p of previous.pieces){
      const squares=[];let item;
      board.forEach((row,r)=>row.forEach((q,c)=>{if(q&&Number(q.id)===p.id){squares.push({row:r,col:c});item=q;}}));
      delete p.log_direction;delete p.log_roll_after_turn;if(item.logDir)p.log_direction=item.logDir;if(item.logRollAfterTurn!==undefined)p.log_roll_after_turn=item.logRollAfterTurn;
      delete p.ammo;delete p.max_ammo;delete p.facing;if(item.ammo!==undefined)p.ammo=item.ammo;if(item.maxAmmo!==undefined)p.max_ammo=item.maxAmmo;if(item.facing!==undefined)p.facing=item.facing;
      delete p.mana;delete p.max_mana;if(item.mana!==undefined)p.mana=item.mana;if(item.maxMana!==undefined)p.max_mana=item.maxMana;
      delete p.bear_retaliations_remaining;delete p.bear_move_locked_until_turn;
      if(item.bearRetaliationsRemaining!==undefined)p.bear_retaliations_remaining=item.bearRetaliationsRemaining;
      if(item.bearMoveLockedUntilTurn!==undefined)p.bear_move_locked_until_turn=item.bearMoveLockedUntilTurn;
      p.shielded=Boolean(item.shielded);
      p.owner=item.color;delete p.gold;if(item.gold!==undefined)p.gold=item.gold;
      delete p.herald_jump_lock_turn;delete p.herald_jump_locked;
      if(item.heraldJumpLockTurn!==undefined)p.herald_jump_lock_turn=item.heraldJumpLockTurn;
      if(item.heraldJumpUnlocked===false)p.herald_jump_locked=true;
      if(item.cardNoCaptureUntil!==undefined&&!p.statuses.some(s=>s.kind==='cannot_capture_until_owner_turn'))p.statuses.push({kind:'cannot_capture_until_owner_turn',owner:item.color,completed_turn:item.cardNoCaptureUntil});
      p.anchor=squares[0];p.footprint=squares;p.hp=item.hp??null;p.max_hp=item.maxHp??null;
    }
    if(previous.result && base.plain(ctx,`(state.oracleReason||'').includes('협정')`))previous.result.reason='herald_agreement';
    if(previous.result && base.plain(ctx,`(state.oracleReason||'').includes('매수')`))previous.result.reason='royal_purchase';
    delete previous.time_stopped;const stopped=base.plain(ctx,`['white','black'].filter(c=>state.skipTurn[c])`);if(stopped.length)previous.time_stopped=stopped;
    delete previous.delayed_spells;const hazards=base.plain(ctx,`state.delayedHazards.map(h=>({spell:h.type,anchor:h.cells[0],owner:h.owner,caster:h.casterId?Number(h.casterId):null}))`);if(hazards.length)previous.delayed_spells=hazards;
    delete previous.pending_bear_retaliations;
    const pendingBears=base.plain(ctx,`(state.pendingBearRetaliations||[]).map(e=>({color:e.color,square:e.square,attacker:Number(e.attackerId),attacker_color:e.attackerColor,captured_by:e.capturedBy,counter_destination:e.counterDestination,remaining:e.remaining,bear:e.bear}))`);
    if(pendingBears.length)previous.pending_bear_retaliations=pendingBears.map(e=>{
      const prior=(input.pending_bear_retaliations||[]).find(q=>q.bear.id===Number(e.bear.id))?.bear || input.pieces.find(q=>q.id===Number(e.bear.id));
      if(!prior)throw new Error('missing pending bear template');
      const bear=structuredClone(prior);bear.owner=e.bear.color;bear.kind=e.bear.type;bear.moved=Boolean(e.bear.moved);bear.shielded=Boolean(e.bear.shielded);
      delete bear.bear_move_locked_until_turn;bear.bear_retaliations_remaining=e.bear.bearRetaliationsRemaining??bear.bear_retaliations_remaining;
      return {...e,bear};
    });
    if(previous.result && base.plain(ctx,`(state.oracleReason||'').includes('장기전') && !(state.oracleReason||'').includes('동형반복')`))previous.result.reason='turn_limit_stars';
    const chain=base.plain(ctx,`state.board.flat().filter(p=>p?.checkerChainCapture).map(p=>Number(p.id))`);
    previous.turn.continuation=chain.length?{kind:'checker_capture',piece:chain[0]}:null;
    const modes=base.plain(ctx,`state.board.flat().filter(p=>p?.windmillMode).map(p=>[Number(p.id),p.windmillMode])`);
    for(const p of previous.pieces){delete p.windmill_mode;const mode=modes.find(([id])=>id===p.id);if(mode&&mode[1]!=='bishop')p.windmill_mode=mode[1];}
    return {state:previous,actions:actions(ctx),check:base.plain(ctx,`({white:findPiecesByPredicate(p=>p.color==='white'&&(isRoyalKing(p)||p.type==='vip')).some(p=>isSquareAttacked(p.row,p.col,'black')),black:findPiecesByPredicate(p=>p.color==='black'&&(isRoyalKing(p)||p.type==='vip')).some(p=>isSquareAttacked(p.row,p.col,'white'))})`)};
  }};
}
function actions(ctx){return base.plain(ctx,`(()=>{
 if(state.mode==='gameover')return [];
 if(state.pendingPromotion){const p=state.pendingPromotion;return p.choices.map(into=>({kind:'promote',piece:Number(state.board[p.row][p.col].id),into}));}
 const out=[],seen=new Set();forEachSquare((p,row,col)=>{
  if(!p||p.color!==state.turn||seen.has(p.id))return;seen.add(p.id);
  // Enumerate complete player spell choices; AI heuristics intentionally omit empty targets.
  if(p.type==='wizard')for(const id of ['lightning','shield','meteor','timeStop']){
    if((p.mana??0)<wizardSpellInfo(id).cost)continue;
    const targets=[];
    if(id==='timeStop')targets.push({row,col});
    else if(id==='shield'){const ids=new Set();forEachSquare((q,r,c)=>{if(q&&q.color===p.color&&!['wall','scarecrow'].includes(q.type)&&!ids.has(q.id)){ids.add(q.id);targets.push({row:r,col:c});}});}
    else for(let r=0;r<(id==='meteor'?7:8);r++)for(let c=0;c<(id==='meteor'?7:8);c++)targets.push({row:r,col:c});
    for(const target of targets)out.push({kind:'cast_spell',wizard:Number(p.id),spell:id==='timeStop'?'time_stop':id,target});
  }
  if(p.type==='shotgunKing'){
    if((p.ammo??0)<(p.maxAmmo??3))out.push({kind:'reload',piece:Number(p.id)});
    for(const mode of ['shotgun','snipe']){
      if((p.ammo??0)<(mode==='shotgun'?2:3))continue;
      state.shotgunAction=mode;
      for(const m of getLegalMoves(row,col)){
        const a=mode==='shotgun'?{kind:'shotgun_blast',piece:Number(p.id),direction:{dr:m.shotgunDirection[0],dc:m.shotgunDirection[1]}}:{kind:'shotgun_snipe',piece:Number(p.id),target:{row:m.row,col:m.col}};
        if(!out.some(x=>JSON.stringify(x)===JSON.stringify(a)))out.push(a);
      }
    }
    state.shotgunAction='move';
  }
  for(const m of getLegalMoves(row,col)){
   if(m.colossusBody)continue;
   if(m.setLogDirection){out.push({kind:'set_log_direction',piece:Number(p.id),direction:m.setLogDirection});continue;}
   if(m.merchantBuy){const a={kind:'purchase',merchant:Number(p.id),target:Number(state.board[m.row][m.col].id)};if(!out.some(x=>JSON.stringify(x)===JSON.stringify(a)))out.push(a);continue;}
   if(m.colossusAttack){const sectors=colossusAttackSectors(row,col,p.color);const index=sectors.findIndex(cells=>JSON.stringify(cells)===JSON.stringify(m.sectorCells));const a={kind:'attack_sector',piece:Number(p.id),sector:index};if(!out.some(x=>JSON.stringify(x)===JSON.stringify(a)))out.push(a);}
   else out.push({kind:'move',from:{row,col},to:{row:m.row,col:m.col},route:[]});
  }
 });return out;
})()`);}
module.exports={createSession};
