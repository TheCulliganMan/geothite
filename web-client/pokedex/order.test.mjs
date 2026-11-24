import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const root = process.env.POKEDEX_WEB_RELEASE;
const baseline = process.env.POKEDEX_WEB_BASELINE;
assert.ok(root, 'Set POKEDEX_WEB_RELEASE to the exported release');
const load = (directory, file) => fs.readFileSync(path.join(directory, file), 'utf8');
const read = file => JSON.parse(fs.readFileSync(file, 'utf8'));
const source = 'planning/minnesota/pokemon';
const roster = read(`${source}/roster.json`);
const curation = read(`${source}/roster-curation.json`);
const order = read(`${source}/pokedex-order.json`);
const families = new Map(roster.families.map(family => [family.id, family]));
const retired = new Set(curation.retired_families.map(family => family.id));
const orderedFamilies = order.sections.flatMap(section => section.families.map(id => families.get(id)));
const sourceIds = orderedFamilies.flatMap(family => family.members.map(member => member.id));
assert.deepEqual(order.species, sourceIds.map((id, index) => ({ id, number: index + 1 })));
const activeFamilies = orderedFamilies.filter(family => !retired.has(family.id));
const members = activeFamilies.flatMap(family => family.members);
const ids = members.map(member => member.id);
const numbers = new Map(ids.map((id, index) => [id, index + 1]));
const catalog = read(path.join(root, 'pokedex.json'));
const pad = number => String(number).padStart(3, '0');
const speciesPath = member => `/pokedex/species/${member.name.toLowerCase()}.html`;
const normalize = text => text.replace(/\/pokedex\/releases\/[A-Za-z0-9_-]+\//g, '/pokedex/releases/RELEASE/');
assert.equal(ids.length, 225);
assert.equal(activeFamilies.length, 99);
assert.deepEqual(catalog.species.map(species => species.identity.id), ids);
assert.deepEqual(catalog.species.map(species => species.identity.regional_number), ids.map(id => numbers.get(id)));
assert.deepEqual(ids.slice(-8), ['MN_151', 'MN_152', 'MN_153', 'MN_095', 'MN_096', 'MN_319', 'MN_098', 'MN_099']);

const html = load(root, 'index.html');
const dom = new JSDOM(html, { url: 'https://example.test/pokedex/?sort=total', runScripts: 'outside-only' });
const document = dom.window.document;
const all = selector => [...document.querySelectorAll(selector)];
const activeCards = () => all('.pokemon-card[data-cut="false"]');
const visibleIds = () => all('.pokemon-card:not([hidden])').map(card => card.dataset.id);
const familyCards = () => all('.family-card[data-cut="false"]');
function assertIndexOrder() {
  assert.deepEqual(activeCards().map(card => card.dataset.id), ids);
  assert.deepEqual(activeCards().map(card => Number(card.dataset.number)), ids.map(id => numbers.get(id)));
  assert.deepEqual(familyCards().map(card => card.dataset.family), activeFamilies.map(family => family.id));
  assert.deepEqual(familyCards().map(card => Number(card.dataset.number)), activeFamilies.map(family => numbers.get(family.members[0].id)));
}
assertIndexOrder(); // The initial HTML must be correct before browser sorting runs.
for (const [index, card] of activeCards().entries()) {
  assert.equal(card.querySelector('.card-top span').textContent, `#${pad(index + 1)}`);
  assert.deepEqual([...card.dataset.search.matchAll(/(?:^|\s)#(\d{3})(?=\s|$)/g)].map(match => match[1]), [pad(index + 1)]);
  assert.equal(card.querySelector('.card-link').getAttribute('href'), speciesPath(members[index]));
}
assert.ok(activeCards().slice(0, -8).every(card => card.dataset.rarity === 'ordinary'));
assert.deepEqual(activeCards().slice(-8).map(card => card.dataset.rarity), [...Array(3).fill('pseudo'), ...Array(5).fill('legendary')]);
dom.window.matchMedia = () => ({ matches: false, addEventListener() {} });
dom.window.eval(load(root, 'app.js'));
assertIndexOrder();
assert.deepEqual(visibleIds(), ids);
assert.equal(document.querySelector('#sort-filter').value, 'number');
const rarity = document.querySelector('#rarity-filter');
for (const [value, expected] of [['rare', ids.slice(-5)], ['pseudo', ids.slice(-8, -5)]]) {
  rarity.value = value;
  rarity.dispatchEvent(new dom.window.Event('change'));
  assert.deepEqual(visibleIds(), expected, `${value} filter follows current curation`);
}
document.querySelector('#clear-filters').click();
document.querySelector('[data-view="families"]').click();
assert.deepEqual(all('.family-card:not([hidden])').map(card => card.dataset.family), activeFamilies.map(family => family.id));
assertIndexOrder();
document.querySelector('[data-view="species"]').click();
for (const [index, id] of ids.entries()) {
  dom.window.history.replaceState(null, '', `/pokedex/?q=%23${pad(index + 1)}`);
  dom.window.dispatchEvent(new dom.window.PopStateEvent('popstate'));
  assert.deepEqual(visibleIds(), [id], `number search #${pad(index + 1)}`);
}
document.querySelector('#dex-search').value = '#222';
document.querySelector('#dex-search').dispatchEvent(new dom.window.Event('input'));
await new Promise(resolve => setTimeout(resolve, 120));
assert.deepEqual(visibleIds(), ['MN_096'], 'typed number search uses the updated label');
document.querySelector('#clear-filters').click();
assertIndexOrder();
assert.deepEqual(visibleIds(), ids);
const cut = document.querySelector('#cut-filter');
cut.checked = true;
cut.dispatchEvent(new dom.window.Event('change'));
assert.deepEqual(visibleIds().slice(0, ids.length), ids, 'archive toggle keeps active entries first');
assert.equal(visibleIds().length, ids.length + catalog.cut_species.length);
dom.window.close();

function withoutPageOrder(text) {
  return normalize(text.replace(/REGIONAL NO\. \d{3}/g, 'REGIONAL NO. NUMBER')
    .replace(/POKÉDEX \/ \d{3}/g, 'POKÉDEX / NUMBER')
    .replace(/<nav class="entry-pager"[\s\S]*?<\/nav>/g, 'PAGER'));
}
for (const [index, member] of members.entries()) {
  const file = `species/${member.name.toLowerCase()}.html`;
  const text = load(root, file);
  const page = new JSDOM(text);
  const doc = page.window.document;
  assert.equal(doc.querySelector('.breadcrumb span').textContent, `REGIONAL NO. ${pad(index + 1)}`, file);
  assert.equal(doc.querySelector('.portrait-meta span').textContent, `POKÉDEX / ${pad(index + 1)}`, file);
  const pager = doc.querySelector('.entry-pager');
  assert.equal(pager.children.length, 3, `${file}: both pager boundaries retained`);
  assert.equal(pager.children[1].getAttribute('href'), '/pokedex/#guide');
  for (const [child, offset] of [[pager.children[0], -1], [pager.children[2], 1]]) {
    const neighbor = members[index + offset];
    if (!neighbor) {
      assert.equal(child.tagName, 'SPAN', `${file}: no neighbor past the index boundary`);
      assert.equal(child.textContent, '');
    } else {
      assert.equal(child.getAttribute('href'), speciesPath(neighbor), file);
      assert.equal(child.querySelector('strong').textContent, neighbor.name, file);
      assert.equal(child.querySelector('span').textContent, `NO. ${pad(index + offset + 1)}`, file);
    }
  }
  for (const match of text.matchAll(/(?:src|href)="\/pokedex\/releases\/[^/]+\/([^"#?]+)"/g)) {
    assert.ok(fs.existsSync(path.join(root, match[1])), `${file}: missing ${match[1]}`);
  }
  if (baseline) assert.equal(withoutPageOrder(text), withoutPageOrder(load(baseline, file)), `${file}: preserve all other content`);
  page.window.close();
}

if (baseline) {
  const previous = read(path.join(baseline, 'pokedex.json'));
  const normalized = value => JSON.parse(normalize(JSON.stringify(value)));
  const { species: beforeSpecies, ...beforeCatalog } = previous;
  const { species: afterSpecies, ...afterCatalog } = catalog;
  assert.deepEqual(normalized(afterCatalog), normalized(beforeCatalog), 'preserve archive, media, pack and acquisition metadata');
  const beforeById = new Map(beforeSpecies.map(species => [species.identity.id, species]));
  for (const entry of afterSpecies) {
    const withoutNumber = species => {
      const result = structuredClone(species);
      delete result.identity.regional_number;
      return normalized(result);
    };
    assert.deepEqual(withoutNumber(entry), withoutNumber(beforeById.get(entry.identity.id)), `${entry.identity.name}: preserve profile and media`);
  }
  const before = new JSDOM(load(baseline, 'index.html'));
  const after = new JSDOM(html);
  function normalizedCard(card) {
    const copy = card.cloneNode(true);
    if (copy.dataset.cut !== 'true') {
      copy.dataset.number = 'NUMBER';
      if (copy.matches('.pokemon-card')) {
        copy.querySelector('.card-top span').textContent = '#NUMBER';
        copy.dataset.search = copy.dataset.search.replace(/(^|\s)#\d{3}(?=\s|$)/g, '$1#NUMBER');
      }
    }
    return normalize(copy.outerHTML);
  }
  for (const selector of ['.pokemon-card', '.family-card']) {
    const key = selector === '.pokemon-card' ? 'id' : 'family';
    const oldCards = new Map([...before.window.document.querySelectorAll(selector)].map(card => [card.dataset[key], normalizedCard(card)]));
    const newCards = new Map([...after.window.document.querySelectorAll(selector)].map(card => [card.dataset[key], normalizedCard(card)]));
    assert.deepEqual(newCards, oldCards, `${selector}: preserve all content except regional numbers`);
  }
  for (const page of [before, after]) for (const id of ['species-grid', 'family-grid']) page.window.document.getElementById(id).textContent = 'CARDS';
  assert.equal(normalize(after.serialize()), normalize(before.serialize()), 'preserve index controls and unrelated content');
  before.window.close(); after.window.close();
  function files(directory, prefix = '') {
    return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory()
      ? files(path.join(directory, entry.name), `${prefix}${entry.name}/`) : `${prefix}${entry.name}`);
  }
  const filenames = files(baseline).sort();
  assert.deepEqual(files(root).sort(), filenames, 'retain every asset with no new extracted catalog');
  const activePages = new Set(members.map(member => `species/${member.name.toLowerCase()}.html`));
  for (const file of filenames) {
    if (['index.html', 'pokedex.json'].includes(file) || activePages.has(file)) continue;
    if (/\.(html|json|js|css|svg)$/.test(file)) {
      assert.equal(normalize(load(root, file)), normalize(load(baseline, file)), `unchanged ${file}`);
    } else {
      assert.deepEqual(fs.readFileSync(path.join(root, file)), fs.readFileSync(path.join(baseline, file)), `unchanged media ${file}`);
    }
  }
}
console.log(`PASS: ${ids.length} canonical entries, ${activeFamilies.length} families, all labels/searches/pagers, 5 legendary and 3 pseudo filters${baseline ? ', and exact retained content/media' : ''}`);
