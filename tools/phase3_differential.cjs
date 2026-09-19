'use strict';
const {createSession}=require('./phase3_oracle.cjs');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const {spawn}=require('node:child_process'),{createInterface}=require('node:readline');
const ROOT=path.resolve(__dirname,'..');
const rust=spawn(path.join(ROOT,'target/debug/examples/oracle_bridge'),[],{stdio:['pipe','pipe','inherit']});
const lines=createInterface({input:rust.stdout})[Symbol.asyncIterator]();
async function request(state,action){rust.stdin.write(JSON.stringify({state,action})+'\n');const line=await lines.next();assert(!line.done);const result=JSON.parse(line.value);assert(!result.error,result.error);return result;}
const stable=v=>Array.isArray(v)?v.map(stable):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,stable(v[k])])):v;
const key=v=>JSON.stringify(stable(v));
const normalize=v=>({...v,actions:[...new Map(v.actions.map(a=>[key(a),a])).values()].sort((a,b)=>key(a).localeCompare(key(b)))});
function firstDifference(a,b,field='$'){
 if(key(a)===key(b))return null;
 if(a&&b&&typeof a==='object'&&typeof b==='object')for(const k of new Set([...Object.keys(a),...Object.keys(b)])){const d=firstDifference(a[k],b[k],field+'.'+k);if(d)return d;}
 return {field,js:a,rust:b};
}
const spawnCorrections=new WeakMap();
let checks=0,transitions=0,current;
async function check(session,state,action=null){
 current={...current,input:state,action};
 const js=normalize(session.step(action)),rs=normalize(await request(state,action));
 // Explicit specification correction: JS recruiter omits the common spawn lock.
 let corrections=spawnCorrections.get(session);if(!corrections){corrections=new Map();spawnCorrections.set(session,corrections);}
 if(action?.kind==='move'&&state.pieces.some(p=>p.kind==='recruiter'&&key(p.anchor)===key(action.from))){
   const spawned=js.state.pieces.find(p=>p.id===state.ids.next_piece);
   assert(spawned&&spawned.kind==='pawn');assert.deepEqual(spawned.statuses,[]);
   corrections.set(spawned.id,[{kind:'cannot_capture_until_owner_turn',owner:state.turn.side,completed_turn:state.turn.completed[state.turn.side]+1}]);
 }
 for(const p of js.state.pieces)if(corrections.has(p.id)&&p.statuses.length===0)p.statuses=corrections.get(p.id);
 current={...current,js,rust:rs,diff:firstDifference(js,rs)};assert.deepEqual(stable(rs),stable(js));checks++;if(action)transitions++;return rs;
}
function setup(base,kind,row,col,side,occupied,fresh,mode){
 const s=structuredClone(base),enemy=side==='white'?'black':'white';
 const entries=[[side,kind,row,col],[side,'king',7,7],[enemy,'king',0,7]];
 if(occupied)entries.push([side,'pawn',3,3],[enemy,'pawn',5,3],[side,'pawn',4,5],[enemy,'rook',2,5],[enemy,'knight',6,6]);
 const used=new Set();s.pieces=entries.filter(([, ,r,c])=>{const k=r+','+c;if(used.has(k))return false;used.add(k);return true}).map(([owner,kind,row,col],i)=>({id:i+1,owner,kind,anchor:{row,col},footprint:[{row,col}],origin:{row,col},moved:false,shielded:false,hp:null,max_hp:null,statuses:[]}));
 if(['colossus','bigRook','bigBishop'].includes(kind)){
  const p=s.pieces[0];p.footprint=[{row,col},{row,col:col+1},{row:row+1,col},{row:row+1,col:col+1}];p.hp=p.max_hp=kind==='colossus'?3:2;
  s.pieces=s.pieces.filter(q=>q.id===p.id||!p.footprint.some(sq=>sq.row===q.anchor.row&&sq.col===q.anchor.col));
 }
 if(fresh)s.pieces[0].statuses=[{kind:'cannot_capture_until_owner_turn',owner:side,completed_turn:1}];
 if(mode&&kind==='windmill')s.pieces[0].windmill_mode=mode;
 if(kind==='shotgunKing'){s.pieces[0].hp=s.pieces[0].max_hp=4;s.pieces[0].ammo=mode??0;s.pieces[0].max_ammo=3;s.pieces[0].facing=side==='white'?'up':'down';}
 if(kind==='wizard'&&mode!==null){s.pieces[0].mana=mode;s.pieces[0].max_mana=5;}
 if(kind==='merchant'&&mode!==null)s.pieces[0].gold=mode;
 if(['bear','hedgehog'].includes(kind)){s.pieces[0].bear_retaliations_remaining=mode;s.pieces[0].bear_move_locked_until_turn=mode===1?1:undefined;}
 if(kind==='herald'&&mode){s.pieces[0].herald_jump_locked=true;if(mode!=='legacy')s.pieces[0].herald_jump_lock_turn=mode==='fresh'?0:1;}
 s.board.cells=Array(64).fill(null);for(const p of s.pieces)p.footprint.forEach(sq=>s.board.cells[sq.row*8+sq.col]=p.id);
 for(const p of s.pieces)if(p.kind==='shotgunKing'&&p.hp===null)Object.assign(p,{hp:4,max_hp:4,ammo:0,max_ammo:3,facing:p.owner==='white'?'up':'down'});s.ids.next_piece=Math.max(...s.pieces.map(p=>p.id))+1;s.turn.side=side;s.history.position_counts=[];return s;
}
(async()=>{
 const base=(await request(null,null)).state;
 const replay=process.argv.indexOf('--replay');
 if(replay>=0){const f=JSON.parse(fs.readFileSync(process.argv[replay+1],'utf8'));current={name:'replay'};await check(createSession(f.input),f.input,f.action);console.log('Replay passed');return;}

 const kinds=process.argv.includes('--sequences-only')?[]:(process.env.PHASE3_KINDS?.split(',') || ['man','ferz','alfil','camel','eagle','pegasus','fanatic','primeMinister','royalKnight','missionary','jester','bat','vip','bear','hedgehog','campfire','lobster','slime','paladin','amazon','knightmaster','windmill','assassin','guard','cannon','grasshopper','hook','cardinal','protestant','checker','checkerKing','squire','standardBearer','colossus','bigRook','bigBishop','berserker','princess','clockwork','herald','recruiter','merchant','wizard','log','shotgunKing']);
 for(const kind of kinds)for(const side of ['white','black'])for(const [row,col] of [[4,4],[0,0],[7,0]])for(const occupied of [false,true])for(const fresh of [false,true])for(const mode of kind==='windmill'?[null,'bishop','rook']:kind==='herald'?[null,'legacy','fresh','expired']:kind==='merchant'?[null,1,2,3,9,20]:kind==='wizard'?[null,1,2,3,5]:kind==='shotgunKing'?[0,1,2,3]:['bear','hedgehog'].includes(kind)?[0,1,2]:[null]){
  if(['colossus','bigRook','bigBishop'].includes(kind)&&(row>=7||col>=7))continue;
  const state=setup(base,kind,row,col,side,occupied,fresh,mode);
  current={name:[kind,side,row,col,occupied,fresh,mode].join('-'),initial_state:state,history:[]};
  const initial=await check(createSession(state),state);
  for(const action of initial.actions.filter(a=>(a.from?.row===row && a.from?.col===col)||a.kind==='attack_sector'||a.kind==='purchase'||a.kind==='cast_spell'||a.kind==='set_log_direction'||['reload','shotgun_blast','shotgun_snipe'].includes(a.kind))){
    const result=await check(createSession(state),state,action);
    // Every resulting state must replay without query mutations.
    assert.deepEqual(stable((await request(result.state,null)).state),stable(result.state));
  }
 }
 // Multi-action sequences retain JS state across turns and pending decisions.
 const move=(r,c,tr,tc)=>({kind:'move',from:{row:r,col:c},to:{row:tr,col:tc},route:[]});
 function custom(entries){const s=setup(base,entries[0][1],entries[0][2],entries[0][3],'white',false,false,null);s.pieces=entries.map(([owner,kind,row,col],i)=>({id:i+1,owner,kind,anchor:{row,col},footprint:[{row,col}],origin:{row,col},moved:false,shielded:false,hp:null,max_hp:null,statuses:[]}));for(const p of s.pieces)if(['colossus','bigRook','bigBishop'].includes(p.kind)){const {row,col}=p.anchor;p.footprint=[{row,col},{row,col:col+1},{row:row+1,col},{row:row+1,col:col+1}];p.hp=p.max_hp=p.kind==='colossus'?3:2;}for(const p of s.pieces){if(p.kind==='shotgunKing'&&p.hp===null)Object.assign(p,{hp:4,max_hp:4,ammo:0,max_ammo:3,facing:p.owner==='white'?'up':'down'});if(['bear','hedgehog'].includes(p.kind))p.bear_retaliations_remaining=2;}s.ids.next_piece=Math.max(...s.pieces.map(p=>p.id))+1;s.board.cells=Array(64).fill(null);for(const p of s.pieces)p.footprint.forEach(sq=>s.board.cells[sq.row*8+sq.col]=p.id);return s;}
 const sequences=[
  ['prime-minister-blocked',[['white','primeMinister',4,4],['white','pawn',3,3],['black','pawn',3,4],['white','pawn',3,5],['black','rook',2,4],['black','king',0,7]],[]],
  ['prime-minister-detour',[['white','primeMinister',4,4],['white','pawn',3,3],['black','pawn',3,4],['black','rook',2,4],['black','king',0,7]],[move(4,4,2,4)]],
  ['capture-royal-knight',[['white','rook',4,0],['black','royalKnight',4,4]],[move(4,0,4,4)]],
  ['assassin-royal-knight',[['white','assassin',4,0],['black','royalKnight',4,4]],[move(4,0,4,4)]],
  ['herald-royal-knight',[['white','herald',4,0],['black','royalKnight',3,3]],[move(4,0,4,3)]],
  ['missionary-converts',[['white','missionary',4,4],['black','guard',3,3],['black','king',0,7]],[move(4,4,3,3)]],
  ['missionary-royal',[['white','missionary',4,4],['black','royalKnight',3,3]],[move(4,4,3,3)]],
  ['jester-captures-merchant',[['white','jester',4,0],['black','merchant',4,4]],[move(4,0,4,4)]],
  ['royal-captures-jester',[['white','royalKnight',4,4],['black','jester',2,3]],[move(4,4,2,3)]],
  ['capture-vip',[['white','rook',4,0],['black','vip',4,4]],[move(4,0,4,4)]],
  ['bear-counter',[['white','rook',4,0],['black','bear',4,4],['black','king',0,7]],[move(4,0,4,4)]],
  ['bear-counter-royal',[['white','king',4,0],['black','bear',4,1]],[move(4,0,4,1)]],
  ['bear-counter-checker',[['white','checker',4,4],['black','bear',3,3],['black','king',0,7]],[move(4,4,2,2)]],
  ['bear-counter-large',[['white','bigRook',4,4],['black','bear',3,4],['black','king',0,7]],[move(4,4,3,4)]],
  ['bear-counter-sector',[['white','colossus',4,4],['black','bear',2,6],['black','king',0,0]],[{kind:'attack_sector',piece:1,sector:0}]],
  ['hedgehog-counter',[['white','rook',4,0],['black','hedgehog',4,4],['black','king',0,7]],[move(4,0,4,4)]],
  ['campfire-protection',[['white','rook',4,0],['black','pawn',4,4],['black','campfire',3,4],['black','king',0,7]],[]],
  ['campfire-royal-unprotected',[['white','rook',4,0],['black','king',4,4],['black','campfire',3,4]],[move(4,0,4,4)]],
  ['paladin-radiance-dark',[['white','rook',5,2],['black','paladin',4,4],['black','king',0,7]],[]],
  ['paladin-radiance-light',[['white','rook',5,1],['black','paladin',4,4],['black','king',0,7]],[move(5,1,5,7)]],
  ['capture-merchant',[['white','rook',4,0],['black','merchant',4,4]],[move(4,0,4,4)]],
  ['herald-merchant',[['white','herald',4,0],['black','merchant',3,3]],[move(4,0,4,3)]],
  ['recruiter-spawn-replay',[['white','recruiter',4,4],['black','king',0,7]],[move(4,4,3,4),move(0,7,1,7),move(3,4,2,4)]],
  ['herald-agreement',[['white','herald',4,0],['black','king',3,3]],[move(4,0,4,3)]],
  ['king-enters-herald',[['white','king',5,3],['black','herald',3,3]],[move(5,3,4,3)]],
  ['herald-actor-priority',[['white','herald',3,3],['black','king',2,3],['black','herald',6,6],['white','king',7,7],['white','pawn',6,0]],[move(6,0,5,0)]],
  ['wall-cannon-screen',[['white','cannon',4,0],['neutral','wall',4,2],['black','pawn',4,5],['black','king',0,7]],[move(4,0,4,5)]],
  ['checker-global-force',[['white','checker',6,0],['black','pawn',5,1],['white','checkerKing',4,7],['white','pawn',6,6]],[move(6,0,4,2)]],
  ['hp-lethal-stationary',[['white','rook',4,0],['black','bigRook',4,4],['black','king',0,7]],[move(4,0,4,4),move(0,7,0,6),move(4,0,4,4)]],
  ['big-friendly-king',[['white','bigRook',4,4],['white','king',3,4],['black','king',0,7]],[move(4,4,3,4)]],
  ['big-both-kings',[['white','bigBishop',4,4],['black','king',3,3],['white','king',3,4]],[move(4,4,3,3)]],
  ['checker-royal-jump',[['white','checker',4,4],['black','king',3,3]],[move(4,4,2,2)]],
  ['sector-multiple-hp',[['white','colossus',4,4],['black','bigRook',1,6],['black','king',0,0]],[{kind:'attack_sector',piece:1,sector:0}]],
  ['checker-chain',[['white','checker',6,0],['black','pawn',5,1],['black','pawn',3,3],['white','rook',7,7],['black','king',0,7]],[move(6,0,4,2),move(4,2,2,4)]],
  ['checker-crown-fresh',[['white','checker',2,0],['black','pawn',1,1],['black','pawn',1,3],['black','king',0,7]],[move(2,0,0,2),move(0,7,0,6),move(0,2,2,4)]],
  ['squire-capture',[['white','squire',4,4],['black','pawn',3,3],['black','king',0,7]],[move(4,4,3,3)]],
  ['squire-promotion',[['white','squire',1,4],['black','pawn',0,3],['black','king',0,7]],[move(1,4,0,3),{kind:'promote',piece:1,into:'rook'}]],
  ['windmill-toggle',[['white','windmill',4,4],['black','pawn',3,3],['black','king',0,7]],[move(4,4,3,3),move(0,7,0,6),move(3,3,3,4)]],
  ['knightmaster-pawn-capture',[['white','pawn',4,4],['white','knightmaster',4,3],['black','pawn',2,5],['black','king',0,7]],[move(4,4,2,5)]],
  ['bearer-pawn-capture',[['white','standardBearer',4,0],['white','pawn',4,4],['black','pawn',4,5],['black','king',0,7]],[move(4,4,4,5)]]
 ];
 for(const [name,entries,actions] of sequences){let state=custom(entries);current={name,initial_state:state,history:[]};const session=createSession(state);await check(session,state);for(const action of actions){state=(await check(session,state,action)).state;current.history.push(action);}}

 const cast=(spell,row,col)=>({kind:'cast_spell',wizard:1,spell,target:{row,col}});
 const wizardSequences=[
  ['wizard-lightning',1,[['white','wizard',4,4],['black','king',0,7]],[cast('lightning',0,6),move(0,7,0,6)]],
  ['wizard-shield',2,[['white','wizard',4,4],['white','pawn',6,0],['black','rook',6,7],['black','king',0,7]],[cast('shield',6,0),move(6,7,6,0)]],
  ['wizard-meteor-hp',3,[['white','wizard',7,0],['black','colossus',3,3],['black','king',0,7]],[cast('meteor',3,3),move(0,7,0,6)]],
  ['wizard-meteor-shield-hp',3,[['white','wizard',7,0],['black','colossus',3,3],['black','king',0,7]],[cast('meteor',3,3),move(0,7,0,6)]],
  ['wizard-time-stop',5,[['white','wizard',4,4],['white','pawn',6,0],['black','king',0,7]],[cast('time_stop',4,4),move(6,0,5,0),move(5,0,4,0)]],
  ['wizard-time-stop-delays-hazard',5,[['white','wizard',4,4],['white','pawn',6,0],['black','king',0,7]],[cast('time_stop',4,4),move(6,0,5,0),move(5,0,4,0)]],
  ['wizard-caster-death',1,[['white','wizard',4,4],['black','rook',4,0],['black','king',0,7]],[cast('lightning',0,7),move(4,0,4,4)]],
  ['wizard-two-royals',3,[['white','wizard',7,0],['white','king',3,3],['black','king',3,4],['black','pawn',1,7]],[cast('meteor',3,3),move(1,7,2,7)]],
  ['wizard-friendly-loss',1,[['white','wizard',7,0],['white','pawn',3,3],['black','king',0,7]],[cast('lightning',3,3),move(0,7,0,6)]],
  ['wizard-guard',1,[['white','wizard',7,0],['black','guard',3,3],['black','king',0,7]],[cast('lightning',3,3),move(0,7,0,6)]],
  ['wizard-fresh',1,[['white','wizard',7,0],['black','pawn',3,3],['black','king',0,7]],[cast('lightning',3,3),move(0,7,0,6)]],
  ['wizard-bear',1,[['white','wizard',7,0],['black','bear',3,3],['black','king',0,7]],[cast('lightning',3,3),move(0,7,0,6)]]
 ];
 for(const [name,mana,entries,actions] of wizardSequences){
  let state=custom(entries);state.pieces[0].mana=mana;state.pieces[0].max_mana=5;
  if(name==='wizard-meteor-shield-hp')state.pieces[1].shielded=true;
  if(name==='wizard-time-stop-delays-hazard')state.delayed_spells=[{spell:'lightning',anchor:{row:4,col:4},owner:'black',caster:null}];
  if(name==='wizard-fresh')state.pieces[0].statuses=[{kind:'cannot_capture_until_owner_turn',owner:'white',completed_turn:2}];
  current={name,initial_state:state,history:[]};const session=createSession(state);await check(session,state);
  for(const action of actions){state=(await check(session,state,action)).state;current.history.push(action);}
 }
 const manaSequences=[
  ['mana-direct',[['white','wizard',7,7],['white','pawn',4,4],['black','rook',4,0],['black','king',0,7]],[move(4,0,4,4)]],
  ['mana-checker',[['white','wizard',7,7],['white','pawn',3,3],['black','checker',2,2],['black','king',0,7]],[move(2,2,4,4)]],
  ['mana-large',[['white','wizard',7,7],['white','pawn',3,4],['white','pawn',3,5],['black','bigRook',4,4],['black','king',0,7]],[move(4,4,3,4)]],
  ['mana-hp',[['white','wizard',7,7],['white','bigRook',4,4],['black','rook',4,0],['black','king',0,7]],[move(4,0,4,4)]],
  ['mana-purchase',[['white','wizard',7,7],['white','pawn',3,3],['black','merchant',0,0],['black','king',0,7]],[{kind:'purchase',merchant:3,target:2}]]
 ];
 for(const [name,entries,actions] of manaSequences)for(const startMana of [0,5]){
  let state=custom(entries);state.turn.side='black';state.pieces[0].mana=startMana;state.pieces[0].max_mana=5;
  if(name==='mana-hp')state.pieces[1].hp=1;
  if(name==='mana-purchase')state.pieces[2].gold=2;
  current={name:name+'-'+startMana,initial_state:state,history:[]};const session=createSession(state);await check(session,state);
  for(const action of actions){state=(await check(session,state,action)).state;current.history.push(action);}
 }
 const dir=(piece,dr,dc)=>({kind:'set_log_direction',piece,direction:{dr,dc}});
 const logSequences=[
  ['log-ep-squire',[['white','log',6,3],['white','pawn',6,4],['black','squire',4,3],['black','king',0,7]],[move(6,4,4,4),move(4,3,5,4)]],
  ['log-ep-landing',[['white','log',6,3],['white','pawn',6,4],['black','pawn',4,3],['black','king',0,7]],[move(6,4,4,4),move(4,3,5,4)]],
  ['log-start',[['white','log',4,4],['white','pawn',6,0],['black','king',0,7]],[dir(1,-1,0),move(0,7,0,6),move(6,0,5,0)]],
  ['log-shield',[['white','log',4,4],['white','pawn',6,0],['black','pawn',3,4],['black','king',0,7]],[move(6,0,5,0)]],
  ['log-guard',[['white','log',4,4],['white','pawn',6,0],['black','guard',3,4],['black','king',0,7]],[move(6,0,5,0)]],
  ['log-capture',[['white','log',4,4],['white','pawn',6,0],['black','pawn',3,4],['black','king',0,7]],[move(6,0,5,0)]],
  ['log-royal',[['white','log',4,4],['white','pawn',6,0],['black','king',3,4]],[move(6,0,5,0)]],
  ['log-hp',[['white','log',4,4],['white','pawn',6,0],['black','bigRook',2,4],['black','king',0,7]],[move(6,0,5,0)]],
  ['log-hp-lethal',[['white','log',4,4],['white','pawn',6,0],['black','bigRook',2,4],['black','king',0,7]],[move(6,0,5,0)]],
  ['log-board-order',[['white','log',4,4],['white','log',3,4],['white','pawn',6,0],['black','king',0,7]],[move(6,0,5,0)]],
  ['log-time-stop',[['white','log',4,4],['white','pawn',6,0],['black','king',0,7]],[move(6,0,5,0),move(5,0,4,0)]],
  ['log-boundary',[['white','log',0,4],['white','pawn',6,0],['black','king',0,7]],[move(6,0,5,0)]],
  ['log-fresh',[['white','log',4,4],['white','pawn',6,0],['black','pawn',3,4],['black','king',0,7]],[move(6,0,5,0)]]
  ,['log-bear-pending',[['white','log',4,4],['white','pawn',6,0],['black','bear',3,4],['black','king',0,7]],[move(6,0,5,0),move(0,7,0,6),move(5,0,4,0)]]
 ];
 for(const [name,entries,actions] of logSequences){
  let state=custom(entries);if(name!=='log-start')for(const p of state.pieces)if(p.kind==='log')p.log_direction={dr:-1,dc:0};
  if(name.startsWith('log-ep-'))state.pieces[0].log_direction={dr:-1,dc:1};
  if(name==='log-shield')state.pieces[2].shielded=true;
  if(name==='log-hp-lethal')state.pieces[2].hp=1;
  if(name==='log-time-stop')state.time_stopped=['black'];
  if(name==='log-fresh')state.pieces[0].statuses=[{kind:'cannot_capture_until_owner_turn',owner:'white',completed_turn:1}];
  current={name,initial_state:state,history:[]};const session=createSession(state);await check(session,state);
  for(const action of actions){state=(await check(session,state,action)).state;current.history.push(action);}
 }
 const blast=(dr,dc)=>({kind:'shotgun_blast',piece:1,direction:{dr,dc}});
 const shotgunSequences=[
  ['shotgun-reload',[['white','shotgunKing',4,4],['black','king',0,7]],[{kind:'reload',piece:1},move(0,7,0,6),move(4,4,5,5)]],
  ['shotgun-friends',[['white','shotgunKing',4,4],['white','pawn',3,3],['black','pawn',3,4],['black','guard',3,5],['black','king',0,7]],[blast(-1,0)]],
  ['shotgun-hp',[['white','shotgunKing',4,4],['black','colossus',1,3],['black','king',0,7]],[blast(-1,0)]],
  ['shotgun-shield',[['white','shotgunKing',4,4],['black','pawn',4,7],['black','king',0,7]],[{kind:'shotgun_snipe',piece:1,target:{row:4,col:7}}]],
  ['shotgun-two-royals',[['white','shotgunKing',4,4],['white','king',3,3],['black','king',3,4]],[blast(-1,0)]],
  ['shotgun-lethal',[['white','rook',4,0],['black','shotgunKing',4,4]],[move(4,0,4,4)]],
  ['shotgun-herald',[['white','herald',4,0],['black','shotgunKing',3,3]],[move(4,0,4,3)]],
  ['shotgun-star-limit',[['white','shotgunKing',4,4],['black','king',0,7]],[move(4,4,5,4),move(0,7,0,6)]],
  ['shotgun-bear',[['white','shotgunKing',4,0],['black','bear',4,4],['black','king',0,7]],[{kind:'shotgun_snipe',piece:1,target:{row:4,col:4}}]]
 ];
 for(const [name,entries,actions] of shotgunSequences){
  let state=custom(entries);for(const p of state.pieces)if(p.kind==='shotgunKing')p.ammo=name==='shotgun-reload'?0:3;
  if(name==='shotgun-lethal')state.pieces[1].hp=1;
  if(name==='shotgun-shield')state.pieces[1].shielded=true;
  if(name==='shotgun-star-limit'){state.config.star_win_limit=1;state.config.deathmatch_enabled=false;}
  current={name,initial_state:state,history:[]};const session=createSession(state);await check(session,state);
  for(const action of actions){state=(await check(session,state,action)).state;current.history.push(action);}
 }
 const shieldSequences=[
  ['shield-rook',[['white','rook',4,0],['black','pawn',4,4],['black','king',0,7]],[move(4,0,4,4)]],
  ['shield-royal',[['white','rook',4,0],['black','king',4,4],['black','pawn',1,0]],[move(4,0,4,4)]],
  ['shield-hp',[['white','rook',4,0],['black','bigRook',4,4],['black','king',0,7]],[move(4,0,4,4)]],
  ['shield-windmill',[['white','windmill',4,4],['black','pawn',3,3],['black','king',0,7]],[move(4,4,3,3)]],
  ['shield-squire',[['white','squire',4,4],['black','pawn',3,3],['black','king',0,7]],[move(4,4,3,3)]],
  ['shield-checker-chain',[['white','checkerKing',4,4],['black','pawn',3,3],['black','king',0,7]],[move(4,4,2,2),move(2,2,4,4)]],
  ['shield-colossus-landing',[['white','colossus',4,4],['black','pawn',3,4],['black','king',0,7]],[move(4,4,3,4)]],
  ['shield-big-rook-landing',[['white','bigRook',4,4],['black','pawn',3,4],['black','king',0,7]],[move(4,4,3,4)]],
  ['shield-sector-skip',[['white','colossus',4,4],['black','pawn',2,6],['black','pawn',1,6],['black','king',0,0]],[{kind:'attack_sector',piece:1,sector:0}]],
  ['shield-ep',[['white','pawn',3,2],['black','pawn',3,3],['black','king',0,7]],[move(3,2,2,3)]]
 ];
 for(const [name,entries,actions] of shieldSequences){
  let state=custom(entries);state.pieces[1].shielded=true;
  if(name==='shield-ep'){state.pieces[1].moved=true;state.history.en_passant={pawn:2,target:{row:2,col:3},available_to:'white'};}
  current={name,initial_state:state,history:[]};const session=createSession(state);await check(session,state);
  for(const action of actions){state=(await check(session,state,action)).state;current.history.push(action);}
 }
 // Every supported purchase price, including missing encyclopedia entries and shared HP entities.
 const purchaseKinds=['pawn','knight','bishop','rook','queen','king','ferz','man','alfil','camel','eagle','pegasus','fanatic','primeMinister','royalKnight','missionary','jester','vip','bear','hedgehog','campfire','lobster','slime','paladin','amazon','knightmaster','windmill','assassin','guard','cannon','grasshopper','hook','cardinal','protestant','checker','checkerKing','squire','standardBearer','colossus','bigRook','bigBishop','berserker','princess','clockwork','herald','recruiter','merchant','wizard','log','shotgunKing'];
 for(const kind of purchaseKinds){
  let state=custom([['white','merchant',7,0],['black',kind,3,3],['black','king',0,7]]);state.pieces[0].gold=20;
  current={name:'purchase-'+kind,initial_state:state,history:[]};const session=createSession(state);const initial=await check(session,state);
  const action=initial.actions.find(a=>a.kind==='purchase'&&a.target===2);
  if(action){state=(await check(session,state,action)).state;if(!state.result){state=(await check(session,state,move(0,7,0,6))).state;}}
 }
 let ownedEp=custom([['white','pawn',3,2],['white','pawn',3,3],['black','log',2,3],['black','king',0,7]]);
 ownedEp.pieces[1].moved=true;ownedEp.history.en_passant={pawn:2,target:{row:2,col:3},available_to:'white'};
 current={name:'own-ep-pawn-with-occupied-landing',initial_state:ownedEp,history:[]};const ownedEpSession=createSession(ownedEp);await check(ownedEpSession,ownedEp);await check(ownedEpSession,ownedEp,move(3,2,2,3));
 for(const col of [4,5,6]){
  const state=custom([['white','king',7,4],['white','rook',7,7],['black','king',0,7]]);
  state.delayed_spells=[{spell:'lightning',anchor:{row:7,col},owner:'black',caster:null}];
  current={name:'castle-pending-spell-'+col,initial_state:state,history:[]};const session=createSession(state);const first=await check(session,state);
  const castle=first.actions.find(a=>a.kind==='move'&&a.from.col===4&&a.to.col===6);if(castle)await check(session,state,castle);
 }
 const castleCases=[];
 for(const side of ['white','black'])for(const kingSide of [false,true])for(const mode of ['empty','friend','enemy','large-friend','king-moved','rook-moved','hazard']){
  const enemy=side==='white'?'black':'white',row=side==='white'?7:0,rookRow=side==='white'?6:0,rookCol=kingSide?6:0,end=kingSide?6:2,landingCol=kingSide?4:3,extraRow=side==='white'?6:1;
  const entries=[[side,'king',row,4],[side,'bigRook',rookRow,rookCol],[enemy,'king',side==='white'?0:7,4],[side,'wizard',4,0]];
  if(['friend','enemy'].includes(mode))entries.push([mode==='friend'?side:enemy,'guard',extraRow,landingCol]);
  if(mode==='large-friend')entries.push([side,'colossus',side==='white'?5:1,landingCol]);
  let state=custom(entries);state.turn.side=side;state.pieces[3].mana=0;
  if(mode==='king-moved')state.pieces[0].moved=true;
  if(mode==='rook-moved')state.pieces[1].moved=true;
  if(mode==='hazard')state.delayed_spells=[{spell:'lightning',anchor:{row,col:4},owner:enemy,caster:null}];
  const name='big-castle-'+[side,kingSide,mode].join('-');castleCases.push(name);current={name,initial_state:state,history:[]};const session=createSession(state);const first=await check(session,state);
  const action=first.actions.find(a=>a.kind==='move'&&a.from.row===row&&a.from.col===4&&a.to.col===end);
  if(action)await check(session,state,action);
 }
 let boughtEp=custom([['white','merchant',7,0],['black','pawn',3,3],['black','king',0,7]]);
 boughtEp.pieces[0].gold=2;boughtEp.pieces[1].moved=true;boughtEp.history.en_passant={pawn:2,target:{row:2,col:3},available_to:'white'};
 current={name:'purchase-ep-pawn',initial_state:boughtEp,history:[]};const boughtEpSession=createSession(boughtEp);await check(boughtEpSession,boughtEp);boughtEp=(await check(boughtEpSession,boughtEp,{kind:'purchase',merchant:1,target:2})).state;await check(boughtEpSession,boughtEp,move(0,7,0,6));
 let epState=custom([['white','standardBearer',2,2],['black','pawn',3,3],['black','king',0,7]]);
 epState.pieces[1].moved=true;epState.history.en_passant={pawn:2,target:{row:2,col:3},available_to:'white'};
 current={name:'bearer-ep-overlap',initial_state:epState,history:[]};const epSession=createSession(epState);await check(epSession,epState);await check(epSession,epState,move(2,2,2,3));
 for(const kind of ['squire','standardBearer']) {
  const state=custom([['white',kind,4,0],['black','king',0,7]]);
  state.deathmatch={started_at_turn:45,half_turns_since_progress:4,interval_half_turns:20,progress_this_turn:false};
  current={name:kind+'-deathmatch',initial_state:state,history:[]};
  const session=createSession(state);await check(session,state);await check(session,state,move(4,0,3,0));
 }
 fs.writeFileSync(path.join(ROOT,'analysis/'+(process.argv.includes('--sequences-only')?'phase3_sequence_report.json':process.env.PHASE3_KINDS?'phase3_subset_report.json':'phase3_differential_report.json')),JSON.stringify({schema_version:base.schema_version,source_sha256:'0dbbad680c6e8e2abcdb6e49817ff486aa2f8ef799a8035bd356baef8a2731ea',scope:kinds,intentional_corrections:['recruiter_spawn_fresh_status'],sequences:[...sequences.map(x=>x[0]),...shieldSequences.map(x=>x[0]),...wizardSequences.map(x=>x[0]),...logSequences.map(x=>x[0]),...shotgunSequences.map(x=>x[0]),...castleCases,...manaSequences.flatMap(x=>[x[0]+'-0',x[0]+'-5']),'bearer-ep-overlap','castle-pending-spell-4','castle-pending-spell-5','castle-pending-spell-6','purchase-ep-pawn','own-ep-pawn-with-occupied-landing','squire-deathmatch','standardBearer-deathmatch',...purchaseKinds.map(k=>'purchase-'+k)],checks,transitions},null,2)+'\n');
 console.log(JSON.stringify({checks,transitions}));
})().catch(e=>{fs.writeFileSync(path.join(ROOT,'analysis/phase3_failure.json'),JSON.stringify({...current,error:e.stack},null,2));console.error(e);process.exitCode=1;}).finally(()=>{rust.stdin.end();});
