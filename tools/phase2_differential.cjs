'use strict';
const {createSession}=require('./phase2_oracle.cjs');
const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const {spawn}=require('node:child_process'),{createInterface}=require('node:readline');
const ROOT=path.resolve(__dirname,'..');
const binary=path.join(ROOT,'target/debug/examples/oracle_bridge');
const rust=spawn(binary,[],{stdio:['pipe','pipe','inherit']});
const lines=createInterface({input:rust.stdout})[Symbol.asyncIterator]();
async function request(state,action){rust.stdin.write(JSON.stringify({state,action})+'\n');const line=await lines.next();assert(!line.done,'Rust bridge exited');const result=JSON.parse(line.value);assert(!result.error,result.error);return result;}
const stable=v=>Array.isArray(v)?v.map(stable):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,stable(v[k])])):v;
const key=v=>JSON.stringify(stable(v));
function compare(a,b,field='$'){
  if(key(a)===key(b))return null;
  if(a&&b&&typeof a==='object'&&typeof b==='object')for(const k of new Set([...Object.keys(a),...Object.keys(b)])){const d=compare(a[k],b[k],field+'.'+k);if(d)return d;}
  return {field,js:a,rust:b};
}
function normalize(result){return {...result,actions:[...new Map(result.actions.map(a=>[key(a),a])).values()].sort((a,b)=>key(a).localeCompare(key(b)))};}
let checks=0,decisions=0,current,session;
const scenarios=[];
async function check(state,action=null){
  current={...current,input:state,action};
  const js=normalize(session.step(action)),rs=normalize(await request(state,action));
  const diff=compare(js,rs);
  if(diff){current={...current,diff,js,rust:rs};throw new Error(JSON.stringify(diff));}
  checks++;if(action)decisions++;return rs.state;
}
const move=(r,c,tr,tc)=>({kind:'move',from:{row:r,col:c},to:{row:tr,col:tc},route:[]});
function setup(base,entries,side='white'){
 const s=structuredClone(base);s.pieces=entries.map(([owner,kind,row,col,moved=false],i)=>({id:i+1,owner,kind,anchor:{row,col},footprint:[{row,col}],origin:{row,col},moved,shielded:false,hp:null,max_hp:null,statuses:[]}));
 s.board.cells=Array(64).fill(null);for(const p of s.pieces)s.board.cells[p.anchor.row*8+p.anchor.col]=p.id;
 s.ids.next_piece=s.pieces.length+1;s.turn.side=side;s.history.position_counts=[];return s;
}
async function sequence(name,s,actions){scenarios.push(name);current={name,initial_state:s,history:[]};session=createSession(s);await check(s);for(const action of actions){s=await check(s,action);current.history.push(action);}return s;}
(async()=>{
const replayIndex=process.argv.indexOf('--replay');
if(replayIndex>=0){
 const failure=JSON.parse(fs.readFileSync(process.argv[replayIndex+1],'utf8'));
 const history=failure.initial_state?[...failure.history,...failure.action?[failure.action]:[]]:failure.action?[failure.action]:[];
 await sequence('replay-'+failure.name,failure.initial_state??failure.input,history);
 console.log('Replay passed');return;
}
const base=(await request(null,null)).state;
await sequence('initial-e4',base,[move(6,4,4,4)]);
await sequence('en-passant',base,[move(6,4,4,4),move(1,0,2,0),move(4,4,3,4),move(1,3,3,3),move(3,4,2,3)]);
await sequence('en-passant-expiry',base,[move(6,4,4,4),move(1,0,2,0),move(4,4,3,4),move(1,3,3,3),move(7,6,5,5),move(2,0,3,0)]);
for(const side of ['white','black']){
 const row=side==='white'?7:0, enemy=side==='white'?'black':'white';
 for(const dest of [2,6])await sequence(side+'-castle-'+dest,setup(base,[[side,'king',row,4],[side,'rook',row,0],[side,'rook',row,7],[enemy,'king',7-row,4]],side),[move(row,4,row,dest)]);
 for(const col of [4,5,6,3,2])await sequence(side+'-castle-attacked-'+col,setup(base,[[side,'king',row,4],[side,'rook',row,0],[side,'rook',row,7],[enemy,'rook',7-row,col]],side),[]);
 for(const kind of ['queen','rook','bishop','knight']){
  const r=side==='white'?1:6,t=side==='white'?0:7;
  await sequence(side+'-promotion-'+kind,setup(base,[[side,'pawn',r,0],[side,'king',row,4],[enemy,'king',7-row,7]],side),[move(r,0,t,0),{kind:'promote',piece:1,into:kind}]);
 }
}
await sequence('expose-king',setup(base,[['white','king',7,4],['white','rook',6,4],['black','rook',0,4],['black','king',0,0]]),[move(6,4,6,5)]);
await sequence('king-into-attack',setup(base,[['white','king',7,4],['black','rook',0,5],['black','king',0,0]]),[move(7,4,7,5)]);
await sequence('royal-capture',setup(base,[['white','rook',1,4],['white','king',7,4],['black','king',0,4]]),[move(1,4,0,4)]);
await sequence('no-action-loss',setup(base,[['white','king',7,4],['black','pawn',7,0]]),[move(7,4,6,4)]);
await sequence('pawn-back-rank-double',setup(base,[['white','pawn',7,0],['white','king',7,4],['black','king',0,4]]),[move(7,0,5,0)]);
await sequence('threefold',base,Array.from({length:2},()=>[move(7,6,5,5),move(0,6,2,5),move(5,5,7,6),move(2,5,0,6)]).flat());
for(const enabled of [false,true]){
 const s=structuredClone(base);s.turn.completed={white:45,black:44};s.turn.side='black';s.turn.full_move=45;s.turn.move_count=89;s.config.deathmatch_enabled=enabled;
 await sequence('turn-45-'+enabled,s,[move(0,6,2,5)]);
}
for(const progress of [false,true])for(const side of ['white','black']){
 const s=structuredClone(base);s.turn.side=side;s.deathmatch={started_at_turn:45,half_turns_since_progress:18,interval_half_turns:20,progress_this_turn:progress};
 await sequence('deathmatch-'+side+'-'+progress,s,[side==='white'?move(7,6,5,5):move(0,6,2,5)]);
 await sequence('deathmatch-pawn-'+side+'-'+progress,s,[side==='white'?move(6,0,5,0):move(1,0,2,0)]);
}
// Sparse boards exercise all six movement geometries and occupied destinations.
for(const kind of ['pawn','knight','bishop','rook','queen','king']) {
 for(const side of ['white','black']) {
  const enemy=side==='white'?'black':'white';
  await sequence('sparse-'+side+'-'+kind,setup(base,[[side,kind,4,4],[side,'pawn',4,6],[side,'pawn',3,3],[enemy,'pawn',2,4],[enemy,'pawn',5,5],[enemy,'king',0,0]],side),[]);
  const fresh=setup(base,[[side,kind,4,4],[enemy,'rook',3,3],[enemy,'rook',5,5],[enemy,'king',0,0]],side);
  fresh.pieces[0].statuses=[{kind:'cannot_capture_until_owner_turn',owner:side,completed_turn:1}];
  await sequence('fresh-'+side+'-'+kind,fresh,[]);
 }
}
for(const kind of ['pawn','rook','king']) {
 const s=setup(base,[['white',kind,1,4],['black','king',0,4],['black','rook',0,3]]);
 s.deathmatch={started_at_turn:45,half_turns_since_progress:18,interval_half_turns:20,progress_this_turn:false};
 await sequence('deathmatch-capture-'+kind,s,[kind==='pawn'?move(1,4,0,3):move(1,4,0,4)]);
}
const count=Number(process.env.PHASE2_SEEDS??100), max=Number(process.env.PHASE2_DECISIONS??200);
assert(Number.isSafeInteger(count)&&count>=0,'PHASE2_SEEDS must be a nonnegative integer');
assert(Number.isSafeInteger(max)&&max>0,'PHASE2_DECISIONS must be a positive integer');
for(let seed=1;seed<=count;seed++){
 let random=seed>>>0;const next=()=>{random^=random<<13;random^=random>>>17;random^=random<<5;return random>>>0;};
 let s=structuredClone(base);current={name:'random',seed,initial_state:s,history:[]};session=createSession(s);await check(s);
 for(let n=0;n<max&&!s.result;n++){
   const actions=(await request(s,null)).actions;assert(actions.length>0);
   const a=actions[next()%actions.length];s=await check(s,a);current.history.push(a);
 }
 if(seed%10===0)console.log(`Seeds ${seed}/${count}; ${decisions} transitions checked`);
}
const report={checks,decisions,seeds:count,max_decisions:max,named_scenarios:scenarios};
fs.writeFileSync(path.join(ROOT,'analysis/phase2_differential_report.json'),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({checks,decisions,seeds:count,max_decisions:max,named_scenarios:scenarios.length}));
})().catch(error=>{fs.writeFileSync(path.join(ROOT,'analysis/phase2_failure.json'),JSON.stringify({...current,error:error.stack},null,2)+'\n');console.error(error);process.exitCode=1;}).finally(()=>rust.stdin.end());
