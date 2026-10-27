import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';
const root = process.env.POKEDEX_WEB_RELEASE || 'target/minnesota-pokedex-web/pokedex/current';
const curation = JSON.parse(fs.readFileSync('planning/minnesota/pokemon/roster-curation.json', 'utf8'));
const expectedCount = curation.active_species_count;
const expectedFamilies = curation.active_family_count;
const sourceRoster = JSON.parse(fs.readFileSync('planning/minnesota/pokemon/roster.json', 'utf8'));
const sourceFamilies = new Map(sourceRoster.families.map(family => [family.id, family]));
const retiredFamilies = new Set(curation.retired_families.map(family => family.id));
const expansion = JSON.parse(fs.readFileSync('planning/minnesota/pokemon/expansion/20261027-eleven-generation.json', 'utf8'));
const sourceOrder = JSON.parse(fs.readFileSync('planning/minnesota/pokemon/pokedex-order.json', 'utf8')).species;
const activeIds = new Set(sourceRoster.families.filter(family => !retiredFamilies.has(family.id)).flatMap(family => family.members.map(member => member.id)));
const sourceProfiles = new Map(JSON.parse(fs.readFileSync('planning/minnesota/pokemon/species-profiles.json', 'utf8')).species.map(profile => [profile.id, profile]));
assert.equal(curation.source_species_count, sourceProfiles.size);
assert.equal(curation.source_species_count, sourceRoster.families.reduce((count, family) => count + family.members.length, 0));
assert.equal(expectedCount, activeIds.size);
assert.equal(expectedFamilies, sourceFamilies.size - retiredFamilies.size);
const spriteManifests = new Map(JSON.parse(fs.readFileSync('planning/minnesota/pokemon/sprite-generation.json', 'utf8')).families.map(family => [family.id, family.manifest]));
const script = fs.readFileSync('web-client/pokedex/app.js', 'utf8');
function page(file, query = '') {
  const dom = new JSDOM(fs.readFileSync(path.join(root, file), 'utf8'), { url: `https://example.test/pokedex/${file}${query}`, runScripts: 'outside-only' });
  dom.window.mediaChanges = new Map();
  dom.window.matchMedia = query => ({ matches: false, addEventListener(event, callback) { if (event === 'change') dom.window.mediaChanges.set(query, callback); } });
  dom.window.eval(script);
  return dom;
}
const dom = page('index.html');
const doc = dom.window.document;
const click = s => doc.querySelector(s).click();
const count = () => [...doc.querySelectorAll('.pokemon-card')].filter(n => !n.hidden).length;
assert.equal(count(), expectedCount);
click('[data-type="WATER"]'); click('[data-type="ICE"]');
assert.ok(count() >= 3);
for (const card of doc.querySelectorAll('.pokemon-card:not([hidden])')) assert.ok(card.dataset.types.includes('WATER') && card.dataset.types.includes('ICE'));
click('#clear-filters');
doc.querySelector('#rarity-filter').value = 'rare'; doc.querySelector('#rarity-filter').dispatchEvent(new dom.window.Event('change'));
assert.equal(count(), 8);
click('#clear-filters');
doc.querySelector('#rarity-filter').value = 'pseudo'; doc.querySelector('#rarity-filter').dispatchEvent(new dom.window.Event('change'));
assert.equal(count(), 3);
click('#clear-filters'); click('[data-view="families"]');
assert.equal([...doc.querySelectorAll('.family-card')].filter(n => !n.hidden).length, expectedFamilies);
click('[data-view="species"]'); click('[data-save="MN_266"]'); click('#notebook-filter');
assert.equal(count(), 1); assert.ok(!doc.querySelector('[data-id="MN_266"]').hidden);
assert.deepEqual(JSON.parse(dom.window.localStorage.getItem('geothite.pokedex.notebook.v1')), ['MN_266']);
const numerical = page('index.html', '?sort=total');
assert.deepEqual([...numerical.window.document.querySelectorAll('.pokemon-card[data-cut="false"]')].map(n => Number(n.dataset.number)), Array.from({length:expectedCount}, (_, i) => i + 1));
assert.equal(numerical.window.document.querySelector('#sort-filter').value, 'number');
const orderedCards = [...numerical.window.document.querySelectorAll('.pokemon-card[data-cut="false"]')];
assert.deepEqual(orderedCards.map(card => card.dataset.id), sourceOrder.filter(species => activeIds.has(species.id)).map(species => species.id));
assert.ok(orderedCards.slice(0,-11).every(card => card.dataset.rarity === 'ordinary'));
assert.deepEqual(orderedCards.slice(-11).map(card => card.dataset.rarity), ['pseudo','pseudo','pseudo',...Array(6).fill('legendary'),'mythical','mythical']);
assert.deepEqual(orderedCards.slice(-11).map(card => card.dataset.id), ['MN_151','MN_152','MN_153','MN_095','MN_096','MN_097','MN_098','MN_099','MN_248','MN_100','MN_250']);
assert.equal(orderedCards.at(-8).dataset.id, 'MN_095');
const walleye = ['MN_251','MN_252','MN_266'].map(id => Number(numerical.window.document.querySelector(`[data-id="${id}"]`).dataset.number));
assert.deepEqual(walleye.map(n => n - walleye[0]), [0,1,2]);
const filtered = page('index.html', '?q=tipulord');
assert.equal([...filtered.window.document.querySelectorAll('.pokemon-card')].filter(n => !n.hidden).length, 1);
const detail = page('species/tipulord.html'); const d = detail.window.document;
d.querySelector('[data-facing="back"]').click(); d.querySelector('#shiny-toggle').click();
assert.match(d.querySelector('#specimen-portrait').src, /mn-266-back-shiny.png$/);
d.querySelector('[data-tab="egg"]').click(); assert.equal(d.querySelector('#moves-egg').hidden, false); assert.equal(d.querySelector('#moves-level').hidden, true);
d.querySelector('[data-tab="egg"]').dispatchEvent(new detail.window.KeyboardEvent('keydown', { key: 'Home', bubbles: true }));
assert.equal(d.querySelector('#tab-level').getAttribute('aria-selected'), 'true');
for (const file of fs.readdirSync(path.join(root, 'species'))) {
 const content = fs.readFileSync(path.join(root, 'species', file), 'utf8');
 assert.ok(!content.includes('{{')); assert.ok(content.includes('class="move-table"')); assert.ok(!content.includes('&quot;water&quot;')); assert.ok(!content.includes('>"water"<'));
 for (const match of content.matchAll(/(?:src|href)="\/pokedex\/releases\/[^/]+\/([^"#?]+)"/g)) assert.ok(fs.existsSync(path.join(root, match[1])), `${file}: missing ${match[1]}`);
}
assert.equal(fs.readdirSync(path.join(root, 'species')).length, curation.source_species_count);
assert.equal(fs.readdirSync(path.join(root, 'sprites')).length, curation.source_species_count * 4);
for (const family of curation.retired_families) {
 for (const card of doc.querySelectorAll(`[data-family="${family.id}"]`)) {
  assert.equal(card.dataset.cut, 'true');
  assert.equal(card.hidden, true, `${family.id} must be hidden by default`);
 }
}
assert.ok(doc.querySelector('.family-card[data-family="MN_F050"]'), 'LINE050 Taconite must remain');
assert.ok(doc.querySelector('.family-card[data-family="MN_F092"]'), 'first fossil choice must remain');
assert.ok(doc.querySelector('.family-card[data-family="MN_F093"]'), 'second fossil choice must remain');
const actualTypes = {};
for (const card of orderedCards) for (const type of card.dataset.types.split(' ')) actualTypes[type] = (actualTypes[type] || 0) + 1;
assert.deepEqual(Object.fromEntries(Object.entries(actualTypes).sort()), curation.type_counts_after);
assert.ok(actualTypes.FAIRY > 0, 'the balanced roster must contain Fairy species');
for (const [family, ids] of [
  ['MN_F041', ['MN_267','MN_268','MN_092']],
  ['MN_F060', ['MN_131','MN_132']],
  ['MN_F063', ['MN_137','MN_138','MN_270']],
]) {
 const line = doc.querySelector(`.family-card[data-family="${family}"]`);
 assert.match(line.textContent, new RegExp(`${ids.length} STAGES`));
 assert.deepEqual([...doc.querySelectorAll(`.pokemon-card[data-family="${family}"]`)].map(card => card.dataset.id), ids);
}
const catalog = JSON.parse(fs.readFileSync(path.join(root, 'pokedex.json'), 'utf8'));
await import('./cut-proposals.test.mjs');
assert.equal(catalog.moves.length, 19);
for (const ids of [['MN_267','MN_268','MN_092'], ['MN_131','MN_132'], ['MN_137','MN_138','MN_270']]) {
 assert.equal(new Set(ids.map(id => JSON.stringify(catalog.species.find(s => s.identity.id === id).normal_rgb555))).size, ids.length);
}
const speciesById = new Map(catalog.species.map(species => [species.identity.id, species]));
// Restoring LINE087 adds the original complete family without displacing another line.
const noteIds = ['MN_186','MN_187','MN_188'];
assert.equal(retiredFamilies.has('MN_F087'), false);
assert.match(doc.querySelector('.family-card[data-family="MN_F087"]').textContent, /LINE 087/);
assert.deepEqual(sourceFamilies.get('MN_F087').members.map(member => member.sheet_count), [1,3,7]);
const noteNumbers = noteIds.map(id => Number(doc.querySelector(`.pokemon-card[data-id="${id}"]`).dataset.number));
assert.deepEqual(noteNumbers.map(number => number - noteNumbers[0]), [0,1,2]);
for (const [index,id] of noteIds.entries()) {
 const species = speciesById.get(id);
 assert.equal(species.status, 'active');
 assert.equal(species.family, 'MN_F087');
 assert.deepEqual(species.identity.types, ['PSYCHIC']);
 assert.deepEqual(species.identity.name, ['Jotlet','Memofold','Recallisk'][index]);
 assert.equal(Object.values(species.profile.base_stats).reduce((sum, stat) => sum + stat, 0), [285,390,500][index]);
 assert.deepEqual(species.profile.base_stats, sourceProfiles.get(id).base_stats);
 assert.deepEqual(species.profile.evolution, index < 2 ? {method:'LEVEL',level:[24,44][index]} : null);
 assert.equal(catalog.cut_species.some(species => species.identity.id === id), false);
 const entry = page(`species/${species.identity.name.toLowerCase()}.html`).window.document;
 assert.equal(entry.querySelector('.cut-notice'), null);
 assert.equal(entry.querySelector('.entry-intro .type-badge').textContent, 'Psychic');
}

function expectedPackedProfile(id) {
 const profile = structuredClone(sourceProfiles.get(id));
 for (const move of catalog.moves) for (const learner of move.learners) {
  if (learner.species_id === id && !profile.level_moves.some(([, learned]) => learned === move.id)) profile.level_moves.push([learner.level, move.id]);
 }
 profile.level_moves.sort((a, b) => a[0] - b[0]);
 return profile;
}
// The approved eleven families replace complete old lines without growing the active dex.
assert.equal(expansion.status, 'integrated-native-reviewed');
assert.equal(expansion.species_count, 23);
assert.equal(expansion.replacement_slot_count, 23);
assert.equal(expansion.families.length, 11);
const newFamilies = [
 ['MN_F134', ['Wadebud','Rakehorn','Marshrack']],
 ['MN_F135', ['Bristub','Latchquill']],
 ['MN_F136', ['Densy','Dozebear','Denumbra']],
 ['MN_F137', ['Ticklet','Solstort']],
 ['MN_F138', ['Peckit','Notchbill','Gougecrest']],
 ['MN_F139', ['Crookit','Crookveil']],
 ['MN_F140', ['Gillpup','Mudmender']],
 ['MN_F141', ['Velvawn','Shedstag']],
 ['MN_F142', ['Dewbit','Glisnare']],
 ['MN_F143', ['Rondive']],
 ['MN_F144', ['Driftile']],
];
for (const [familyId, names] of newFamilies) {
 const sourceFamily = sourceFamilies.get(familyId);
 const registered = expansion.families.find(family => family.id === familyId);
 assert.ok(sourceFamily && registered, `${familyId} needs canonical and expansion records`);
 assert.deepEqual(sourceFamily.members.map(member => member.name), names);
 assert.deepEqual(registered.species_ids, sourceFamily.members.map(member => member.id));
 assert.equal(retiredFamilies.has(familyId), false);
 const cards = orderedCards.filter(card => card.dataset.family === familyId);
 assert.deepEqual(cards.map(card => card.dataset.id), registered.species_ids);
 assert.ok(cards.every(card => card.dataset.rarity === 'ordinary' && card.dataset.cut === 'false'));
 assert.equal(new Set(registered.species_ids.map(id => JSON.stringify(speciesById.get(id).normal_rgb555))).size, names.length, `${familyId} needs individual stage palettes`);
 for (const [stage, id] of registered.species_ids.entries()) {
  const species = speciesById.get(id);
  assert.equal(species.identity.name, names[stage]);
  assert.deepEqual(species.identity.types, sourceFamily.members[stage].types);
  assert.equal(species.identity.evolves_to, registered.species_ids[stage + 1] || null);
  assert.equal(species.starter_role, null);
  assert.deepEqual(species.profile, expectedPackedProfile(id), `${names[stage]} must ship its full authored profile and packed extension learnsets`);
  const entry = page(`species/${names[stage].toLowerCase()}.html`).window.document;
  assert.equal(entry.querySelector('h1').textContent, names[stage]);
  assert.equal(entry.querySelector('.cut-notice'), null);
  assert.equal(entry.querySelectorAll('#moves-level tbody tr').length, species.profile.level_moves.length);
  assert.equal(entry.querySelectorAll('#moves-machine tbody tr').length, species.profile.tmhm_moves.length);
  const stem = id.toLowerCase().replace('_', '-');
  entry.querySelector('[data-facing="back"]').click();
  entry.querySelector('#shiny-toggle').click();
  assert.ok(entry.querySelector('#specimen-portrait').src.endsWith(`/sprites/${stem}-back-shiny.png`));
  for (const view of ['front','back']) {
   const normal = fs.readFileSync(path.join(root, 'sprites', `${stem}-${view}.png`));
   const shiny = fs.readFileSync(path.join(root, 'sprites', `${stem}-${view}-shiny.png`));
   assert.notDeepEqual(normal, shiny, `${id} ${view} needs a distinct shiny palette`);
   for (const image of [normal,shiny]) {
    assert.equal(image.readUInt32BE(16), view === 'front' ? 56 : 48);
    assert.equal(image.readUInt32BE(20), view === 'front' ? 56 : 48);
   }
  }
 }
}
const cutById = new Map(catalog.cut_species.map(species => [species.identity.id, species]));
let replacedSpecies = 0;
for (const replacement of expansion.replacement_plan) {
 assert.equal(retiredFamilies.has(replacement.old_family), true);
 for (const member of sourceFamilies.get(replacement.old_family).members) {
  replacedSpecies += 1;
  assert.equal(speciesById.has(member.id), false);
  assert.equal(cutById.get(member.id)?.family, replacement.old_family);
  assert.equal(doc.querySelector(`.pokemon-card[data-id="${member.id}"]`).hidden, true);
 }
}
assert.equal(replacedSpecies, 23);
for (const family of ['MN_F050','MN_F062','MN_F126','MN_F130','MN_F133']) assert.equal(retiredFamilies.has(family), false, `${family} is explicitly preserved`);
const runestone = speciesById.get('MN_277');
assert.ok(runestone, 'the new runestone solo must be exported');
assert.equal(runestone.identity.name, 'Kensigard');
assert.equal(runestone.family, 'MN_F126');
assert.deepEqual(runestone.identity.types, ['GROUND']);
assert.equal(runestone.identity.stage, 1);
assert.equal(runestone.identity.evolves_to, null);
assert.equal(runestone.starter_role, null);
assert.deepEqual(runestone.normal_rgb555, [32767,17080,13641,0]);
assert.notDeepEqual(runestone.shiny_rgb555, runestone.normal_rgb555);
assert.deepEqual(runestone.profile, expectedPackedProfile('MN_277'));
assert.equal(runestone.profile.classification, 'Runestone');
assert.deepEqual(runestone.profile.base_stats, {hp:90,attack:105,defense:110,speed:35,special_attack:40,special_defense:95});
assert.equal(Object.values(runestone.profile.base_stats).reduce((sum, value) => sum + value, 0), 475);
assert.equal(runestone.profile.evolution, null);
assert.equal(runestone.profile.gender_ratio, 255);
assert.deepEqual(runestone.profile.egg_groups, ['EGG_MINERAL','EGG_MINERAL']);
assert.equal(runestone.profile.growth_rate, 'GROWTH_MEDIUM_FAST');
assert.equal(runestone.profile.catch_rate, 60);
assert.ok(runestone.profile.level_moves.some(([level, move]) => level === 1 && move === 'MUD_SLAP'));
assert.ok(runestone.profile.level_moves.some(([level, move]) => level === 44 && move === 'EARTHQUAKE'));
assert.deepEqual(runestone.profile.egg_moves, []);
const runestoneCard = doc.querySelector('.pokemon-card[data-id="MN_277"]');
assert.equal(runestoneCard.dataset.rarity, 'ordinary');
assert.equal(runestoneCard.querySelector('.starter-label'), null);
const runestoneIndex = orderedCards.findIndex(card => card.dataset.id === 'MN_277');
assert.ok(runestoneIndex >= 0 && runestoneIndex < expectedCount - 12);
assert.equal(orderedCards[runestoneIndex + 1].dataset.id, 'MN_013', 'runestone opens lake country before Riceling');
assert.match(doc.querySelector('.family-card[data-family="MN_F126"]').textContent, /1 STAGE/);
assert.equal(doc.querySelectorAll('.pokemon-card[data-family="MN_F126"]').length, 1);
const runestoneSearch = page('index.html', '?q=kensigard').window.document;
assert.deepEqual([...runestoneSearch.querySelectorAll('.pokemon-card:not([hidden])')].map(card => card.dataset.id), ['MN_277']);
assert.equal(page('index.html', '?type=GROUND').window.document.querySelector('[data-id="MN_277"]').hidden, false);
assert.equal(page('index.html', '?type=ROCK').window.document.querySelector('[data-id="MN_277"]').hidden, true);
const runestoneEntry = page('species/kensigard.html').window.document;
assert.equal(runestoneEntry.querySelector('h1').textContent, 'Kensigard');
assert.deepEqual([...runestoneEntry.querySelectorAll('.entry-types .type-badge')].map(badge => badge.textContent.toUpperCase()), ['GROUND']);
assert.equal(runestoneEntry.querySelectorAll('#moves-level tbody tr').length, runestone.profile.level_moves.length);
assert.equal(runestoneEntry.querySelectorAll('#moves-machine tbody tr').length, runestone.profile.tmhm_moves.length);
for (const id of ['MN_277']) for (const view of ['front','back']) {
 const stem = id.toLowerCase().replace('_', '-');
 const normal = fs.readFileSync(path.join(root, 'sprites', `${stem}-${view}.png`));
 const shiny = fs.readFileSync(path.join(root, 'sprites', `${stem}-${view}-shiny.png`));
 assert.notDeepEqual(normal, shiny);
 for (const sprite of [normal,shiny]) {
  assert.equal(sprite.readUInt32BE(16), view === 'front' ? 56 : 48);
  assert.equal(sprite.readUInt32BE(20), view === 'front' ? 56 : 48);
 }
}
runestoneEntry.querySelector('[data-facing="back"]').click();
assert.match(runestoneEntry.querySelector('#specimen-portrait').src, /mn-277-back.png$/);
runestoneEntry.querySelector('#shiny-toggle').click();
assert.match(runestoneEntry.querySelector('#specimen-portrait').src, /mn-277-back-shiny.png$/);
runestoneEntry.querySelector('[data-facing="front"]').click();
assert.match(runestoneEntry.querySelector('#specimen-portrait').src, /mn-277-front-shiny.png$/);
// Stable IDs belong to the replacement line; stale names/pages must not survive export.
for (const [id, name, types] of [
 ['MN_163','Mitwist',['GHOST','FAIRY']],
 ['MN_164','Mispair',['GHOST','FAIRY']],
 ['MN_165','Shawlter',['FAIRY','FLYING']],
]) {
 const species = speciesById.get(id);
 assert.equal(species.identity.name, name);
 assert.equal(species.family, 'MN_F076');
 assert.deepEqual(species.identity.types, types);
 const entry = page(`species/${name.toLowerCase()}.html`).window.document;
 assert.equal(entry.querySelector('h1').textContent, name);
 assert.deepEqual([...entry.querySelectorAll('.entry-types .type-badge')].map(badge => badge.textContent.toUpperCase()), types);
}
assert.deepEqual([...doc.querySelectorAll('.pokemon-card[data-family="MN_F076"]')].map(card => card.dataset.name), ['Mitwist','Mispair','Shawlter']);
for (const [id, classification, total, evolution] of [
 ['MN_163','Lost Mitten',300,{method:'LEVEL',level:22}],
 ['MN_164','Lost Pair',405,{method:'LEVEL',level:40}],
 ['MN_165','Shawl Set',515,null],
]) {
 const profile = speciesById.get(id).profile;
 assert.equal(profile.classification, classification);
 assert.equal(Object.values(profile.base_stats).reduce((sum, value) => sum + value, 0), total);
 assert.deepEqual(profile.base_stats, sourceProfiles.get(id).base_stats);
 assert.deepEqual(profile.evolution, evolution);
 assert.deepEqual(profile.dex_pages, sourceProfiles.get(id).dex_pages);
 assert.equal(profile.tmhm_moves.includes('FLY'), id === 'MN_165', 'only the shawl stage gains flight');
}
for (const oldName of ['Swaylet','Seichamp','Surgewrit','Hemhowl','Gustitch']) {
 assert.equal(catalog.species.some(species => species.identity.name === oldName), false);
 assert.equal(fs.existsSync(path.join(root, 'species', `${oldName.toLowerCase()}.html`)), false);
 const staleSearch = page('index.html', `?q=${oldName.toLowerCase()}`).window.document;
 assert.equal(staleSearch.querySelectorAll('.pokemon-card:not([hidden])').length, 0);
}
const fairyFlyers = page('index.html', '?type=FAIRY,FLYING').window.document;
for (const id of ['MN_163','MN_164']) assert.equal(fairyFlyers.querySelector(`[data-id="${id}"]`).hidden, true);
assert.equal(fairyFlyers.querySelector('[data-id="MN_165"]').hidden, false);
const ghostFairies = page('index.html', '?type=GHOST,FAIRY').window.document;
for (const id of ['MN_163','MN_164']) assert.equal(ghostFairies.querySelector(`[data-id="${id}"]`).hidden, false);
assert.equal(ghostFairies.querySelector('[data-id="MN_165"]').hidden, true);
const familySizes = new Map();
for (const species of catalog.species) familySizes.set(species.family, (familySizes.get(species.family) || 0) + 1);
const evolutionMix = [1,2,3].map(size => [...familySizes.values()].filter(count => count === size).length);
assert.deepEqual(evolutionMix, [curation.evolution_mix.single_stage_species,curation.evolution_mix.two_stage_families,curation.evolution_mix.three_stage_families]);
const supportedTypes = new Set(['NORMAL','FIRE','WATER','ELECTRIC','GRASS','ICE','FIGHTING','POISON','GROUND','FLYING','PSYCHIC','BUG','ROCK','GHOST','DRAGON','DARK','STEEL','FAIRY']);
for (const species of catalog.species) for (const type of species.identity.types) assert.ok(supportedTypes.has(type), `${species.identity.name}: unsupported type ${type}`);
assert.equal(doc.querySelectorAll('.type-filter').length, 18);
const fairyFilter = doc.querySelector('[data-type="FAIRY"]');
assert.equal(fairyFilter.getAttribute('aria-label'), 'Filter by Fairy type');
const fairyPage = page('index.html', '?type=FAIRY');
const fairyDoc = fairyPage.window.document;
assert.equal(fairyDoc.querySelector('[data-type="FAIRY"]').getAttribute('aria-pressed'), 'true');
assert.deepEqual([...fairyDoc.querySelectorAll('.pokemon-card:not([hidden])')].map(card => card.dataset.id), catalog.species.filter(species => species.identity.types.includes('FAIRY')).map(species => species.identity.id));
for (const card of fairyDoc.querySelectorAll('.pokemon-card:not([hidden])')) assert.ok(card.querySelector('[aria-label="Fairy type"]'));
fairyDoc.querySelector('[data-type="GRASS"]').click();
assert.ok([...fairyDoc.querySelectorAll('.pokemon-card:not([hidden])')].every(card => card.dataset.types.includes('FAIRY') && card.dataset.types.includes('GRASS')));
fairyDoc.querySelector('#clear-filters').click();
assert.equal(fairyDoc.querySelector('[data-type="FAIRY"]').getAttribute('aria-pressed'), 'false');
assert.equal(fairyDoc.querySelectorAll('.pokemon-card:not([hidden])').length, expectedCount);
for (const [id, name, classification] of [['MN_213','Chislet','Butter Chip'],['MN_214','Buttermuse','Dairy Royal']]) {
 const species = speciesById.get(id);
 assert.equal(species.identity.name, name);
 assert.deepEqual(species.identity.types, sourceFamilies.get('MN_F098').members.find(member => member.id === id).types);
 assert.equal(species.profile.classification, classification);
 assert.deepEqual(species.profile.base_stats, sourceProfiles.get(id).base_stats);
 const entry = page(`species/${name.toLowerCase()}.html`).window.document;
 assert.ok(entry.querySelector('.entry-intro .eyebrow').textContent.includes(classification));
 assert.deepEqual([...entry.querySelectorAll('.entry-types .type-badge')].map(badge => badge.textContent.toUpperCase()), species.identity.types);
 for (const move of ['Charm','Encore','Swift','Heal Bell','Moonlight']) {
  const row = [...entry.querySelectorAll('#moves-level tbody tr')].find(row => row.cells[1].textContent === move);
  assert.ok(row, `${name} must render its actual ${move} learnset row`);
  assert.equal(row.cells[2].textContent, ['Charm','Moonlight'].includes(move) ? 'Fairy' : 'Normal', `${move} must use the packed Fairy overlay type`);
 }
}
const butterStats = speciesById.get('MN_214').profile.base_stats;
assert.equal(Object.values(butterStats).reduce((sum, value) => sum + value, 0), 495);
assert.equal(butterStats.attack, 65);
assert.equal(butterStats.special_attack, 100);
// Match the actual selected native assets, catching stale high-detail previews.
const nativeViews = new Set();
for (const family of familySizes.keys()) {
 assert.ok(spriteManifests.has(family), `${family} needs a registered sprite manifest`);
 const manifest = JSON.parse(fs.readFileSync(path.join('planning/minnesota/pokemon', spriteManifests.get(family)), 'utf8'));
 for (const frame of manifest.sprites) {
  if (!speciesById.has(frame.species_id)) continue;
  const key = `${frame.species_id}:${frame.view}`;
  assert.equal(nativeViews.has(key), false, `duplicate reviewed native view ${key}`);
  nativeViews.add(key);
  const stem = frame.species_id.toLowerCase().replace('_', '-');
  const shipped = fs.readFileSync(path.join(root, 'sprites', `${stem}-${frame.view}.png`));
  const native = fs.readFileSync(`target/minnesota-pokemon/native/${family}/${frame.id}.${frame.view}.png`);
  assert.deepEqual(shipped, native, `${frame.species_id} ${frame.view} must use the reviewed native art`);
  const size = frame.view === 'front' ? 56 : 48;
  assert.equal(shipped.readUInt32BE(16), size);
  assert.equal(shipped.readUInt32BE(20), size);
  const metadata = JSON.parse(fs.readFileSync(`target/minnesota-pokemon/native/${family}/${frame.id}.${frame.view}.json`, 'utf8'));
  assert.equal(metadata.sampling || 'area_vote', frame.sampling || 'area_vote');
  assert.equal(metadata.exterior_contour || 'source_backed', frame.exterior_contour || 'source_backed');
  assert.deepEqual(metadata.palette_rgb555, speciesById.get(frame.species_id).normal_rgb555);
 }
}
assert.equal(nativeViews.size, expectedCount * 2, 'every active Pokemon needs both reviewed native views');
for (const id of ['MN_269','MN_271','MN_272','MN_273']) assert.equal(speciesById.has(id), false, `${id} is archived or unregistered`);
assert.equal(fs.existsSync(path.join(root, 'species', 'oreviathan.html')), false);
assert.equal(doc.querySelector('.family-card[data-family="MN_F124"]'), null);
const starters = [
 ['grass', 'MN_F001', ['MN_001','MN_002','MN_003']],
 ['fire', 'MN_F125', ['MN_274','MN_275','MN_276']],
 ['water', 'MN_F004', ['MN_010','MN_011','MN_012']],
];
assert.deepEqual(orderedCards.slice(0,9).map(card => card.dataset.id), starters.flatMap(([, , ids]) => ids));
assert.equal(doc.querySelectorAll('.pokemon-card .starter-label').length, 9);
assert.equal(doc.querySelectorAll('.family-card .starter-label').length, 3);
// The old furnace family stays active, with stable identities and no starter badge.
for (const [id, name] of [['MN_004','Cindore'],['MN_005','Pelletor'],['MN_006','Mesablast']]) {
 const species = speciesById.get(id);
 assert.equal(species.identity.name, name);
 assert.equal(species.family, 'MN_F002');
 assert.equal(species.starter_role, null);
 assert.equal(doc.querySelector(`.pokemon-card[data-id="${id}"] .starter-label`), null);
 assert.ok(fs.existsSync(path.join(root, 'species', `${name.toLowerCase()}.html`)));
}
assert.equal(doc.querySelector('.family-card[data-family="MN_F002"] .starter-label'), null);
assert.deepEqual(['MN_274','MN_275','MN_276'].map(id => speciesById.get(id).identity.types), [['FIRE'],['FIRE'],['FIRE','ICE']]);
const fireStarters = page('index.html', '?q=fire%20starter').window.document;
// Search also covers themes, so ordinary species may mention fire starters.
for (const id of ['MN_274','MN_275','MN_276']) assert.equal(fireStarters.querySelector(`[data-id="${id}"]`).hidden, false);
assert.deepEqual([...fireStarters.querySelectorAll('.pokemon-card:not([hidden]) .starter-label')].map(label => label.closest('.pokemon-card').dataset.id), ['MN_274','MN_275','MN_276']);
const waterStarters = page('index.html', '?q=water%20starter').window.document;
assert.deepEqual([...waterStarters.querySelectorAll('.pokemon-card:not([hidden])')].map(card => card.dataset.id), ['MN_010','MN_011','MN_012']);
for (const [role, family, ids] of starters) {
 assert.equal(doc.querySelector(`.family-card[data-family="${family}"] .starter-label`).dataset.starterRole, role);
 assert.equal(new Set(ids.map(id => JSON.stringify(speciesById.get(id).normal_rgb555))).size, 3, `${family} needs independent stage palettes`);
 for (const [stage, id] of ids.entries()) {
  const species = speciesById.get(id);
  const profile = species.profile;
  assert.equal(species.starter_role, role);
  assert.deepEqual(profile.base_stats, sourceProfiles.get(id).base_stats);
  assert.equal(Object.values(profile.base_stats).reduce((sum, value) => sum + value, 0), [318,405,530][stage]);
  assert.deepEqual(profile.evolution, stage < 2 ? {method:'LEVEL',level:[16,36][stage]} : null);
  assert.equal(profile.catch_rate, 45);
  assert.equal(profile.gender_ratio, 31);
  assert.equal(profile.hatch_cycles, 20);
  assert.equal(profile.growth_rate, 'GROWTH_MEDIUM_SLOW');
  assert.equal(doc.querySelector(`.pokemon-card[data-id="${id}"] .starter-label`).dataset.starterRole, role);
  const entry = page(`species/${species.identity.name.toLowerCase()}.html`).window.document;
  assert.equal(entry.querySelector('.entry-intro .starter-label').dataset.starterRole, role);
  const portrait = entry.querySelector('#specimen-portrait');
  const stem = id.toLowerCase().replace('_', '-');
  assert.ok(portrait.src.endsWith(`/sprites/${stem}-front.png`));
  entry.querySelector('[data-facing="back"]').click();
  assert.ok(portrait.src.endsWith(`/sprites/${stem}-back.png`));
  entry.querySelector('#shiny-toggle').click();
  assert.ok(portrait.src.endsWith(`/sprites/${stem}-back-shiny.png`));
  entry.querySelector('[data-facing="front"]').click();
  assert.ok(portrait.src.endsWith(`/sprites/${stem}-front-shiny.png`));
  assert.equal(entry.querySelector('#shiny-toggle').getAttribute('aria-pressed'), 'true');
  assert.notDeepEqual(fs.readFileSync(path.join(root, 'sprites', `${stem}-front.png`)), fs.readFileSync(path.join(root, 'sprites', `${stem}-front-shiny.png`)));
 }
}
for (const [id, total, level] of [['MN_132',520,null],['MN_138',430,44],['MN_270',540,null]]) {
 const profile = speciesById.get(id).profile;
 assert.equal(Object.values(profile.base_stats).reduce((sum, value) => sum + value, 0), total);
 assert.deepEqual(profile.evolution, level ? {method:'LEVEL',level} : null);
}
assert.deepEqual(speciesById.get('MN_131').profile.evolution, {method:'LEVEL',level:30});
assert.deepEqual(speciesById.get('MN_137').profile.evolution, {method:'LEVEL',level:26});
function learns(id, move, level) {
 assert.ok(speciesById.get(id).profile.level_moves.some(entry => entry[0] === level && entry[1] === move), `${id} must learn ${move} at ${level}`);
}
for (const [id, aquaLevel] of [['MN_010',9],['MN_011',13],['MN_012',13]]) { learns(id,'AQUA_JET',aquaLevel); learns(id,'WATER_PULSE',18); }
for (const id of ['MN_287','MN_288']) {
 learns(id,'AQUA_JET',9);
 assert.equal(sourceProfiles.get(id).egg_moves.includes('AQUA_JET'), false, 'imported otter moves belong to the extension learner pipeline');
 assert.ok(catalog.moves.find(move => move.id === 'AQUA_JET').learners.some(learner => learner.species_id === id && learner.level === 9));
}
for (const [id, level] of [['MN_001',9],['MN_002',16],['MN_003',1]]) learns(id,'MEGA_DRAIN',level);
learns('MN_006','IRON_HEAD',36);
learns('MN_299','IRON_HEAD',36);
learns('MN_276','ICE_BEAM',36);
learns('MN_276','BLIZZARD',56);
learns('MN_012','VITAL_THROW',36);
learns('MN_012','DRAIN_PUNCH',38);
learns('MN_003','SLIPPER_WALTZ',36);
for (const [id, moves] of [
 ['MN_213',[['CHARM',13],['ENCORE',22],['SWIFT',27],['HEAL_BELL',37],['MOONLIGHT',46]]],
 ['MN_214',[['CHARM',15],['ENCORE',24],['SWIFT',30],['HEAL_BELL',38],['MOONLIGHT',47]]],
]) for (const [move, level] of moves) learns(id, move, level);
for (const [id, category, power, effect] of [
 ['AQUA_JET','physical',40,'PRIORITY_HIT'],
 ['WATER_PULSE','special',60,'CONFUSE_HIT'],
 ['DRAIN_PUNCH','physical',75,'LEECH_HIT'],
 ['IRON_HEAD','physical',80,'FLINCH_HIT'],
 ['SLIPPER_WALTZ','special',75,'CONFUSE_HIT'],
]) {
 const move = catalog.moves.find(move => move.id === id);
 assert.equal(move.category, category);
 assert.equal(move.battle.power, power);
 assert.equal(move.battle.effect, effect);
}
const movePage = page('moves.html'); const md = movePage.window.document;
assert.equal(md.querySelectorAll('.signature-card').length, 5);
assert.equal(md.querySelectorAll('.imported-animation-card').length, 14);
assert.equal(md.querySelectorAll('.move-animation-card').length, catalog.moves.length);
assert.equal(md.querySelectorAll('.move-index a').length, catalog.moves.length);
for (const link of md.querySelectorAll('.move-index a')) assert.ok(md.querySelector(link.getAttribute('href')));
assert.equal(md.querySelectorAll('.imported-moves tbody tr').length, 14);
for (const id of ['FAIRY_WIND','DISARMING_VOICE','DAZZLING_GLEAM','PLAY_ROUGH','MOONBLAST']) {
 const move = catalog.moves.find(move => move.id === id);
 assert.ok(move, `${id} needs a real imported move definition`);
 assert.equal(move.kind, 'imported');
 assert.equal(move.battle.type, 'FAIRY');
 assert.ok(move.battle.source_index >= 265 && move.battle.source_index <= 269);
}
assert.equal(md.querySelectorAll('.imported-moves [aria-label="Fairy type"]').length, 5);
const poisonJab = catalog.moves.find(move => move.id === 'POISON_JAB');
assert.equal(poisonJab.battle.source_index, 270);
assert.equal(poisonJab.battle.type, 'POISON');
assert.equal(poisonJab.category, 'physical');
assert.deepEqual([poisonJab.battle.power,poisonJab.battle.accuracy,poisonJab.battle.pp,poisonJab.battle.effect_chance], [80,100,20,30]);
for (const preview of md.querySelectorAll('[data-animation-base]')) {
 const img = preview.querySelector('img');
 assert.match(img.src, /player-normal-still.png$/);
 preview.querySelector('[data-animation-play]').click(); assert.match(img.src, /player-normal.png$/);
 preview.querySelector('[data-animation-side]').click(); assert.match(img.src, /enemy-normal.png$/);
 preview.querySelector('[data-animation-shiny]').click(); assert.match(img.src, /enemy-shiny.png$/);
 assert.match(preview.querySelector('.animation-download').href, /enemy-shiny.png$/);
 assert.ok(preview.querySelector('[data-animation-play]').getAttribute('aria-label').includes(preview.dataset.moveName));
 preview.querySelector('[data-animation-play]').click(); assert.match(img.src, /enemy-shiny-still.png$/);
}
assert.equal(fs.readdirSync(path.join(root, 'animations')).length, 152);
for (const move of catalog.moves) {
 const id = move.id.toLowerCase();
 const card = md.getElementById(id);
 assert.ok(card, `${move.id} needs a playable animation card`);
 assert.equal(card.dataset.moveId, move.id);
 assert.match(card.textContent, new RegExp(move.display_name));
 assert.match(card.textContent, new RegExp(move.category, 'i'));
 for (const side of ['player','enemy']) for (const palette of ['normal','shiny']) {
  const animation = fs.readFileSync(path.join(root, 'animations', `${id}-${side}-${palette}.png`));
  assert.ok(animation.includes(Buffer.from('acTL')), `${move.id} ${side}/${palette} must be animated`);
  assert.deepEqual([animation.readUInt32BE(16), animation.readUInt32BE(20)], [160,144]);
  const still = fs.readFileSync(path.join(root, 'animations', `${id}-${side}-${palette}-still.png`));
  assert.deepEqual([still.readUInt32BE(16), still.readUInt32BE(20)], [160,144]);
  if (process.env.POKEDEX_ANIMATION_SOURCE) {
   const source = path.join(process.env.POKEDEX_ANIMATION_SOURCE, id);
   const variant = `${side}-${palette}`;
   const metadataPath = path.join(source, `${variant}-keyframes.json`);
   const metadata = fs.existsSync(metadataPath) ? JSON.parse(fs.readFileSync(metadataPath, 'utf8')) : null;
   const frame = metadata ? metadata.contact_indices[3] : 0;
   if (metadata) { assert.equal(metadata.move_id, move.id); assert.equal(metadata.variant, variant); assert.ok(frame < metadata.frames); }
   assert.deepEqual(still, fs.readFileSync(path.join(source, variant, `frame-${String(frame).padStart(3,'0')}.png`)), `${move.id} idle still must be the exact native contact frame`);
  }
 }
}
// Large move collections never autoplay or run multiple APNGs at once.
const previews = [...md.querySelectorAll('[data-animation-base]')];
previews[0].querySelector('[data-animation-play]').click();
previews[1].querySelector('[data-animation-play]').click();
assert.equal(previews[0].querySelector('[data-animation-play]').getAttribute('aria-pressed'), 'false');
assert.equal(previews[1].querySelector('[data-animation-play]').getAttribute('aria-pressed'), 'true');
movePage.window.mediaChanges.get('(prefers-reduced-motion: reduce)')({ matches: true });
assert.equal(previews[1].querySelector('[data-animation-play]').getAttribute('aria-pressed'), 'false');
previews[1].querySelector('[data-animation-play]').click();
previews[1].dispatchEvent(new movePage.window.Event('animation-offscreen'));
assert.equal(previews[1].querySelector('[data-animation-play]').getAttribute('aria-pressed'), 'false');
for (const species of [...catalog.species, ...catalog.cut_species]) {
 const html = fs.readFileSync(path.join(root, `species/${species.identity.name.toLowerCase()}.html`), 'utf8');
 const speciesDom = new JSDOM(html);
 const entry = speciesDom.window.document;
 const learned = [
  ...species.profile.level_moves.map(([level,id]) => [`Lv. ${level}`,id]),
  ...species.profile.tmhm_moves.map(id => ['Machine',id]),
  ...species.profile.egg_moves.map(id => ['Inherited',id]),
 ];
 const expected = catalog.moves.filter(move => learned.some(([,id]) => id === move.id));
 assert.deepEqual([...entry.querySelectorAll('.move-animation-card')].map(card => card.dataset.moveId), expected.map(move => move.id), `${species.identity.name} only shows genuinely learned moves`);
 for (const move of expected) {
  const card = entry.getElementById(move.id.toLowerCase());
  for (const [method,id] of learned) if (id === move.id) assert.ok(card.querySelector('.move-learning').textContent.includes(method));
 }
 speciesDom.window.close();
}
for (const move of catalog.moves) for (const learner of move.learners) {
 const species = catalog.species.find(s => s.identity.id === learner.species_id);
 if (species) assert.ok(species.profile.level_moves.some(([level, id]) => id === move.id && level === learner.level));
}
for (const [name, move] of [['taconarch','magnetic_rail'],['mosquitan','bog_siphon'],['tipulord','tip_up_strike'],['cyprielle','slipper_waltz'],['musketunge','pike_drill']]) {
 const entry = page(`species/${name}.html`).window.document;
 assert.ok(entry.querySelector(`#${move}`));
 if (name === 'cyprielle') for (const name of ['Confusion','Psybeam','Psychic']) {
  const row = [...entry.querySelectorAll('.move-table tbody tr')].find(row => row.cells[1]?.textContent === name);
  assert.ok(row, `${name} must remain in Cyprielle's real learnset`);
  assert.equal(row.querySelector('.type-badge').textContent, 'Psychic');
  assert.equal(row.querySelector('.type-badge').getAttribute('aria-label'), 'Psychic type');
 }
 assert.match(entry.querySelector('#moves-level').textContent, new RegExp(move.split('_').map(s => s[0].toUpperCase()+s.slice(1)).join(' ')));
}
console.log(`PASS: ${expectedCount} sequential entries, ${expectedFamilies} families; exact LINE cuts and type counts; Taconite and fossils preserved; rare species last; search/notebook/sprite/move controls`);
console.log('PASS: three equal starter progressions and visible roles; freighter cut, twine growth and individual palettes; 19 packed move learnsets; all nineteen playable native move previews with side/shiny controls and genuine per-species learning methods');
console.log('PASS: wool-ghost replacements and Ghost/Fairy/Flying filters; butter royalty and packed Fairy/native move rows; every active family matches reviewed native art and conversion metadata; starter front/back/shiny controls');
console.log('PASS: Kensigard is an ordinary pure-Ground solo with its own exact palette, valid authored profile/learnsets, lake-country ordering and distinct normal/shiny front/back sprites');
console.log('PASS: all eleven approved families, 23 new identities/full profiles and shiny views; exactly 23 replaced species remain labeled cut proposals; Taconite, Kensigard, windvane, otter and oak families preserved');

console.log('PASS: eighteen labeled type filters, Fairy intersection/reset, five packed Fairy imports, physical Poison Jab counter and Fairy status-move rows');

console.log('PASS: restored LINE087 retains stable IDs/names, pure Psychic typing, one/three/seven sheets, approved 285/390/500 budgets, level24/44 evolution and adjacent ordinary-family order');
