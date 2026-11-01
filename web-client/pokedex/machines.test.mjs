import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const root = process.env.POKEDEX_WEB_RELEASE;
const baseline = process.env.POKEDEX_WEB_BASELINE;
assert.ok(root && baseline, 'Set POKEDEX_WEB_RELEASE and POKEDEX_WEB_BASELINE');
const load = (directory, file) => fs.readFileSync(path.join(directory, file), 'utf8');
const catalog = JSON.parse(load(root, 'pokedex.json'));
const previous = JSON.parse(load(baseline, 'pokedex.json'));
const normalize = text => text.replace(/\/pokedex\/releases\/[^/"\s]+/g, '/pokedex/releases/RELEASE');
const withoutPanel = text => normalize(text.replace(/<section class="move-panel"[\s\S]*?<\/section>/, 'MOVE_PANEL'));
assert.equal(catalog.species_count, previous.species_count);
assert.equal(catalog.machine_catalog.length, 57);
assert.deepEqual(catalog.machine_catalog.map(machine => machine.code), [
 ...Array.from({ length: 50 }, (_, i) => `TM${String(i + 1).padStart(2, '0')}`),
 ...Array.from({ length: 7 }, (_, i) => `HM${String(i + 1).padStart(2, '0')}`),
]);
const machineCodes = new Map(catalog.machine_catalog.map(machine => [machine.move_id, machine.code]));
assert.equal(machineCodes.size, 57);
assert.equal(machineCodes.get('SURF'), 'HM03');
assert.equal(machineCodes.get('FLASH'), 'HM05');
assert.equal(machineCodes.get('REST'), 'TM44');
assert.ok(!machineCodes.has('ICE_BEAM'));
assert.match(catalog.pack_content_hash, /^[a-f0-9]{64}$/);
for (const key of Object.keys(previous).filter(key => !['species', 'cut_species'].includes(key))) {
 assert.deepEqual(catalog[key], previous[key], `${key} must remain unchanged`);
}
const index = new JSDOM(load(root, 'index.html'));
const oldIndex = new JSDOM(load(baseline, 'index.html'));
let total = 0;
for (const group of ['species', 'cut_species']) {
 assert.equal(catalog[group].length, previous[group].length);
 for (const [position, species] of catalog[group].entries()) {
  const { machine_learnset, tutor_moves, ...unchanged } = species;
  assert.deepEqual(unchanged, previous[group][position], `${species.identity.name}: full profile, art metadata and roster unchanged`);
  const expected = catalog.machine_catalog.filter(machine => species.profile.tmhm_moves.includes(machine.move_id));
  assert.deepEqual(machine_learnset, expected);
  assert.deepEqual(tutor_moves, species.profile.tmhm_moves.filter(id => !machineCodes.has(id)));
  if (group === 'species') {
   assert.ok(expected.length > 0, `${species.identity.name} needs a TM/HM learnset`);
   assert.ok(tutor_moves.every(id => ['ICE_BEAM', 'THUNDERBOLT', 'FLAMETHROWER'].includes(id)));
   total += expected.length;
  }
  const file = `species/${species.identity.name.toLowerCase()}.html`;
  const html = load(root, file);
  assert.equal(withoutPanel(html), withoutPanel(load(baseline, file)), `${file}: preserve every other page section`);
  const entry = new JSDOM(html);
  const document = entry.window.document;
  assert.deepEqual([...document.querySelectorAll('#moves-machine tbody tr')].map(row => row.cells[0].textContent), expected.map(machine => machine.code));
  assert.equal(document.querySelector('#tab-machine span').textContent, String(expected.length));
  assert.equal(document.querySelectorAll('#moves-tutor tbody tr').length, tutor_moves.length);
  assert.equal(document.querySelector('#tab-tutor span').textContent, String(tutor_moves.length));
  if (!tutor_moves.length) assert.match(document.querySelector('#moves-tutor').textContent, /No moves listed/);
  const selector = `[data-id="${species.identity.id}"]`;
  assert.equal(index.window.document.querySelector(selector).dataset.search,
   `${expected.map(machine => machine.code).join(' ')} ${oldIndex.window.document.querySelector(selector).dataset.search}`);
  entry.window.close();
 }
}
function files(directory, prefix = '') {
 return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? files(path.join(directory, entry.name), `${prefix}${entry.name}/`) : `${prefix}${entry.name}`);
}
assert.deepEqual(files(root).sort(), files(baseline).sort(), 'no extra assets or missing files');
for (const file of files(baseline).filter(file => !file.endsWith('.html') && !['style.css', 'pokedex.json'].includes(file))) {
 assert.deepEqual(fs.readFileSync(path.join(root, file)), fs.readFileSync(path.join(baseline, file)), `unchanged ${file}`);
}
const script = load(root, 'app.js');
for (const code of ['HM03', 'TM44']) {
 const dom = new JSDOM(load(root, 'index.html'), { url: `https://example.test/pokedex/?q=${code}`, runScripts: 'outside-only' });
 dom.window.matchMedia = () => ({ matches: false, addEventListener() {} });
 dom.window.eval(script);
 assert.deepEqual([...dom.window.document.querySelectorAll('.pokemon-card:not([hidden])')].map(card => card.dataset.id), catalog.species.filter(species => species.machine_learnset.some(machine => machine.code === code)).map(species => species.identity.id));
 dom.window.close();
}
index.window.close(); oldIndex.window.close();
console.log(`PASS: ${catalog.species.length} active species, ${total} TM/HM compatibilities, ${catalog.cut_species.length} archived proposals; exact baseline profiles/media and numbered rows, tutors, counts and code search`);
