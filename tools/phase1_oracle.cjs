// Dependency-free, hash-pinned, explicitly allowlisted reference extraction.
// Run from any working directory. No browser app, mock DOM, or timer execution.
'use strict';
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const crypto = require('node:crypto');
const assert = require('node:assert/strict');
const ROOT = path.resolve(__dirname, '..');
const HASH = '0dbbad680c6e8e2abcdb6e49817ff486aa2f8ef799a8035bd356baef8a2731ea';
const source = fs.readFileSync(path.join(ROOT, 'origin_code/main-DsoigPgV.js'), 'utf8');
assert.equal(crypto.createHash('sha256').update(source).digest('hex'), HASH, 'Reference changed; review extraction');
const functions = ['piece', 'createInitialBoard', 'createEmptyBoard', 'placeCampaignColossus',
  'colossusCells', 'inBounds', 'squareName', 'fileLabels', 'boardRowCount', 'boardColCount',
  'normalizeGameStyle', 'isChaosGameStyle', 'isGrandGameStyle', 'gameDeckSlotCount', 'createEmptyDeckSlots'];
const constants = ['FILES', 'EXTENDED_FILES', 'NORMAL_DECK_SLOT_COUNT', 'CHAOS_DECK_SLOT_COUNT',
  'GRAND_DECK_SLOT_COUNT', 'GAME_STYLE_IDS', 'DEFAULT_GAME_STYLE'];
function extractFunction(name) {
  const start = source.indexOf(`\nfunction ${name}(`) + 1;
  assert(start > 0, name);
  const end = source.indexOf('\n}', start) + 2;
  assert(end > start, name);
  return source.slice(start, end);
}
function extractConstant(name) {
  const start = source.indexOf(`\nconst ${name} = `) + 1;
  assert(start > 0, name);
  return source.slice(start, source.indexOf(';', start) + 1);
}
function context() {
  const ctx = vm.createContext({}, {codeGeneration: {strings: false, wasm: false}});
  vm.runInContext(`
    let state = null;
    let selectedGameStyle = 'normal';
    let idDraws = 0;
    // Only the piece ID generator consumes this seam in the allowlisted paths.
    Math.random = () => (++idDraws) / 4096;
    const ids = new Map();
  ` + constants.map(extractConstant).join('\n') + '\n' + functions.map(extractFunction).join('\n') + `
    const referencePiece = piece;
    piece = (...args) => {
      const item = referencePiece(...args);
      if (ids.has(item.id)) throw new Error('duplicate reference ID');
      ids.set(item.id, ids.size + 1);
      return item;
    };
  `, ctx, {timeout: 1000});
  return ctx;
}
function plain(ctx, code) { return JSON.parse(vm.runInContext(`JSON.stringify(${code})`, ctx, {timeout: 1000})); }
function normalizeBoard(board, idPairs) {
  const ids = new Map(idPairs), entities = new Map();
  const known = new Set(['id','color','type','moved','shielded','origin','hp','maxHp','anchorRow','anchorCol']);
  const cells = board.flatMap((row, r) => row.map((p, c) => {
    if (!p) return null;
    for (const key of Object.keys(p)) assert(known.has(key), `unmapped piece field: ${key}`);
    const id = ids.get(p.id);
    assert(id, `unmapped ID ${p.id}`);
    let entity = entities.get(id);
    if (!entity) {
      const match = /^([a-h])([1-8])$/.exec(p.origin);
      assert(match, 'only this fixture uses 8x8 origin notation');
      entity = {id, owner:p.color, kind:p.type, anchor:{row:p.anchorRow ?? r,col:p.anchorCol ?? c},
        footprint:[], origin:{row:8-Number(match[2]),col:match[1].charCodeAt(0)-97},
        moved:p.moved, shielded:p.shielded, hp:p.hp ?? null, max_hp:p.maxHp ?? null, statuses:[]};
      entities.set(id, entity);
    }
    entity.footprint.push({row:r,col:c});
    return id;
  }));
  return {board:{rows:board.length,cols:board[0].length,cells},pieces:[...entities.values()].sort((a,b)=>a.id-b.id)};
}
function initial() {
  const ctx = context();
  vm.runInContext('const board = createInitialBoard();', ctx, {timeout:1000});
  const result = normalizeBoard(plain(ctx,'board'), plain(ctx,'[...ids]'));
  // Select only literal scalar/counter fields from resetGame, not its UI path.
  const reset = source.slice(source.indexOf('function resetGame('), source.indexOf('function resetGame(') + 16000);
  const fields = ['turn','turnsTaken','fullMove','moveCount','actionsRemaining','cardsUsedThisTurn','firstMoveCardsForced'];
  result.reset = {};
  for (const field of fields) {
    const match = new RegExp(`^    ${field}: (.+),?$`, 'm').exec(reset);
    assert(match, field);
    result.reset[field] = plain(ctx, `(${match[1].replace(/,$/, '')})`);
  }
  result.deck_slots = plain(ctx, 'createEmptyDeckSlots({gameStyle:"normal", campaign:null})');
  assert.equal(plain(ctx, 'idDraws'), 32);
  return result;
}
function large() {
  const ctx = context();
  vm.runInContext(`const board = createEmptyBoard(); state = {board}; placeCampaignColossus(board, 'white', 2, 3);`,ctx,{timeout:1000});
  assert.equal(vm.runInContext('board[2][3] === board[3][4]',ctx), true);
  return normalizeBoard(plain(ctx,'board'),plain(ctx,'[...ids]'));
}
function checkOrWrite(relative, object) {
  const file = path.join(ROOT, relative), expected = JSON.stringify(object,null,2)+'\n';
  if (process.argv.includes('--write')) fs.writeFileSync(file,expected);
  else assert.equal(fs.readFileSync(file,'utf8'),expected,`stale fixture: ${relative}`);
}
if (require.main === module) {
  checkOrWrite('engine/tests/fixtures/js_initial.json',initial());
  checkOrWrite('engine/tests/fixtures/js_colossus.json',large());
  assert.throws(() => normalizeBoard([[{id:'x', surprise:true}]], [['x',1]]), /unmapped piece field/);
  const imports = [...source.matchAll(/^import .*?["'](\.\/[^"']+)["'];$/gm)].map(m=>m[1]);
  checkOrWrite('analysis/phase1_oracle_manifest.json', {
    source_sha256:HASH, functions, constants, imports,
    missing_imports:imports.filter(p=>!fs.existsSync(path.join(ROOT,'origin_code',p))),
    scope:'initial board, literal reset counters, empty normal deck slots, shared colossus entity only',
    ui_event_sink:'none: selected paths have no UI calls; unexpected dependencies throw',
    timer_continuations:'none executed; resetGame/draft/action/turn continuations remain unsupported',
    randomness:'piece ID only, remapped by allocation order; no rules randomness consumed',
  });
  console.log('Phase 1 JS oracle: initial placement, reset counters, slots, colossus alias, unknown-field guard passed');
}
module.exports = {initial,large,normalizeBoard};
