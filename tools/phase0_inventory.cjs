// Extract only reviewed, data-only declarations; never execute the browser bundle.
const fs = require('node:fs');
const vm = require('node:vm');
const crypto = require('node:crypto');
const source = fs.readFileSync('origin_code/main-DsoigPgV.js', 'utf8');
const lines = source.split('\n');
const context = vm.createContext({});
const used = new Map();
function declaration(name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const m = new RegExp('^const ' + escaped + ' = ', 'm').exec(source);
  if (!m) throw new Error('Missing data declaration: ' + name);
  const end = name === 'CARD_CATEGORY_BY_ID' ? source.indexOf('}, {});', m.index) + 7 : source.indexOf(';\n', m.index) + 1;
  if (end < 1) throw new Error('Missing declaration end');
  return {code: source.slice(m.index, end), line: source.slice(0, m.index).split('\n').length};
}
// Explicit allowlist: dependency failures must be reviewed, not auto-executed.
const names = ['PREVIOUS_INTERNAL_BALANCE_UPDATES','PRE_PORTAL_BALANCE_UPDATES','PRE_ZUGZWANG_BALANCE_UPDATES','PRE_DEMOCRACY_BALANCE_UPDATES','PRE_CANNON_BALANCE_UPDATES','PRE_GHOST_CANNON_BALANCE_UPDATES','PRE_SELECTED_MIRACLE_BALANCE_UPDATES','PRE_THREE_BUFF_BALANCE_UPDATES','PRE_CHAOS_OPENING_BALANCE_UPDATES','PRE_VANGUARD_DIAGONAL_BALANCE_UPDATES','PRE_OVERTAKE_BALANCE_UPDATES','PRE_HIGHLANDER_BALANCE_UPDATES','PRE_THIEF_JUMP_BALANCE_UPDATES','INTERNAL_BALANCE_UPDATES','CARD_DISPLAY_STAR_OVERRIDES','make','SEPTEMBER_CARD_DEFINITIONS','SEPTEMBER_RULE_DEFINITION','SEPTEMBER_ALL_DEFINITIONS','card$l','INTERNAL_EIGHT_CARDS','card$k','INTERNAL_FIVE_CARDS','INTERNAL_THREE_CARDS','CARD_DEFS','CARD_CATEGORY_GROUPS','CARD_CATEGORY_BY_ID','PASSIVE_CARD_IDS','CARD_PRESENTATION_NAME_OVERRIDES_V19','LEGACY_CARD_ID_ALIASES','TYPE_LABELS'];
vm.runInContext('function internalBalanceCard(c) { return {...c, ...INTERNAL_BALANCE_UPDATES[c.id]}; }', context);
for (const name of names) {
  const d = declaration(name); vm.runInContext(d.code, context, {timeout: 1000}); used.set(name,d.line);
}
// Type label assignments are all literal data between TYPE_LABELS and CARD_DEFS.
vm.runInContext(source.slice(source.indexOf('TYPE_LABELS.scarecrow ='), source.indexOf('const CARD_DEFS =')), context);
const data = vm.runInContext(`({cards: CARD_DEFS.map(c => {
 const x = internalBalanceCard({...c, phase:CARD_CATEGORY_BY_ID[c.id] || c.phase, stars:CARD_DISPLAY_STAR_OVERRIDES[c.id] ?? c.stars});
 if (x.phase === 'RULE') x.stars = null;
 return {...x, name:CARD_PRESENTATION_NAME_OVERRIDES_V19[c.id] || (c.id === 'iron-monarch' ? '친정' : c.id === 'cool-guy' ? '매너' : c.name), activation:x.phase === 'RULE' ? 'MATCH_RULE' : PASSIVE_CARD_IDS.has(c.id) ? 'PASSIVE' : 'ACTIVE'};
}), labels:TYPE_LABELS, aliases:LEGACY_CARD_ID_ALIASES})`, context);
// Later standalone presentation patches, separate from executable semantics.
const patches = {'stealth':'아군 비숍 하나를 지정합니다. 해당 비숍은 상대에게 보이지 않습니다.', 'traitor':'폰을 포함해 상대의 가장 약한 기물 하나를 내 편으로 전향시킵니다.', 'cool-guy':'한번 말을 잡은 기물은 다시 평범한 이동을 하기 전까진 더이상 기물을 잡을 수 없습니다.', 'hook':'퀸과 룩을 1개씩 선택합니다. 선택한 룩을 희생하고 퀸이 구행이 됩니다.', 'acceleration':'적용 뒤 세번째 흑 차례부터 모든 플레이어가 한 턴에 2번 행동합니다.', 'merchant-guild':'킹을 2칸 전진시킨 뒤 상인으로 변경합니다. 상인은 골드로 상대 기물을 매수합니다.'};
for (const c of data.cards) {
 if (patches[c.id]) c.text = patches[c.id];
 c.sourceLines = lines.flatMap((line,i) => (line.includes('id: "'+c.id+'"') || line.trim() === '"'+c.id+'",' || line.includes('("'+c.id+'",')) ? [i+1] : []).filter(n => n < 4170 || n >= 37766 && n < 39531);
}
const funcs = lines.flatMap((line, i) => { const m = /^function ([\w$]+)\(/.exec(line); return m ? [{name:m[1],line:i+1}] : []; });
const random = lines.flatMap((line,i) => line.includes('Math.random') ? [{line:i+1, owner:funcs.filter(f=>f.line<=i+1).at(-1)?.name || '<top-level>', code:line.trim()}] : []);
const stateFields = [...new Set([...source.matchAll(/\bstate\??\.([A-Za-z_$][\w$]*)/g)].map(m=>m[1]))].sort();
const result = {source:'origin_code/main-DsoigPgV.js', sha256:crypto.createHash('sha256').update(source).digest('hex'), lines:lines.length-1, declarations:Object.fromEntries(used), cards:data.cards, pieceLabels:data.labels, aliases:data.aliases, random, stateFields};
if (new Set(data.cards.map(c=>c.id)).size !== data.cards.length) throw new Error('Duplicate card IDs');
fs.writeFileSync('analysis/phase0_inventory.json', JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({cards:data.cards.length, phases:data.cards.reduce((o,c)=>(o[c.phase]=(o[c.phase]||0)+1,o),{}), pieceLabels:Object.keys(data.labels).length, randomSites:random.length, stateFields:stateFields.length,sha256:result.sha256},null,2));
