import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const root = process.env.POKEDEX_WEB_RELEASE;
const baseline = process.env.POKEDEX_WEB_BASELINE;
assert.ok(root && baseline, 'Set the new release and its preserved baseline');
const catalog = JSON.parse(fs.readFileSync(path.join(root, 'pokedex.json')));
const before = JSON.parse(fs.readFileSync(path.join(baseline, 'pokedex.json')));
const byName = new Map(catalog.species.map(entry => [entry.identity.name.toUpperCase(), entry]));
const progression = process.env.POKEDEX_EXPECT_PROGRESSION === '1'
  ? JSON.parse(fs.readFileSync('planning/minnesota/pokemon/acquisition-v01/encounter-progression-v01/progression.json')) : null;
const journey = new Map((progression?.groups || []).flatMap(group => group.maps.map(map => [map, group])));
assert.equal(catalog.species.length, 225);
assert.equal(catalog.acquisition_source.available_species, 225);
assert.deepEqual(catalog.acquisition_source.pending_species, []);
for (const key of Object.keys(before).filter(key => !['species', 'acquisition_source'].includes(key))) assert.deepEqual(catalog[key], before[key], key);
for (const entry of catalog.species) {
  const { acquisition, ...original } = entry;
  const { acquisition: previousAcquisition, ...previous } = before.species.find(row => row.identity.id === entry.identity.id);
  assert.deepEqual(original, previous);
  const page = fs.readFileSync(path.join(root, 'species', `${entry.identity.name.toLowerCase()}.html`), 'utf8');
  const dom = new JSDOM(page);
  const panel = dom.window.document.querySelector('.locations-panel');
  assert.ok(panel, `${entry.identity.name} needs a location panel`);
  for (const method of panel.querySelectorAll('.location-method')) assert.ok(!/All day ·/.test(method.textContent), 'all-day availability is stated once');
  assert.equal(dom.window.document.querySelectorAll('#locations-title').length, 1);
  const moves = dom.window.document.querySelector('.move-animation-entry, .move-panel');
  assert.ok(panel.compareDocumentPosition(moves) & dom.window.Node.DOCUMENT_POSITION_FOLLOWING,
    'locations precede the move previews and learnsets');
  assert.equal(Boolean(panel.querySelector('.location-pending')), acquisition.status === 'pending');
  assert.equal(dom.window.document.querySelector('.planned-location'), null);
  assert.ok(dom.window.document.querySelector('link[href$="/locations.css"]'));
  dom.window.close();
}
assert.deepEqual(fs.readdirSync(path.join(root, 'species')), fs.readdirSync(path.join(baseline, 'species')),
  'the existing species directory contains only the preserved pages');
for (const [name, map, area, levels] of [
  ['OATLET', 'MnR02C00R00', 'Harvest Oats', progression ? [9, 11] : [8]],
  ['CINDERFIN', 'MnR11C00R03', 'Smokehouse', progression ? [41, 43, 44] : [40]],
  ['WICKIT', 'MnR11C00R02', 'Lighthouse', progression ? [41, 43, 44] : [41]],
]) {
  const source = byName.get(name).acquisition.direct_sources.find(source => source.map === map && source.area.includes(area));
  assert.ok(source);
  assert.deepEqual(source.levels, levels);
  assert.deepEqual(source.times, ['day', 'morning', 'night']);
}
assert.ok(byName.get('BURRBUD').acquisition.direct_sources.some(source => source.method === 'Headbutt'));
assert.ok(byName.get('BURROOT').acquisition.evolutions.some(edge => edge.from === 'BURRBUD'));
for (const name of ['TRILUME', 'SIPHLET']) {
  const sources = byName.get(name).acquisition.direct_sources;
  assert.ok(sources.some(source => source.note.includes('Care Badge') && source.note.includes('free party slot')));
  assert.ok(sources.some(source => source.map === 'MnRochesterFossilLab' && source.place === 'Rochester Fossil Lab'));
}
if (process.env.POKEDEX_EXPECT_EARLY_RESEARCH === '1') {
  for (const name of ['POUCHID', 'KINDLIT', 'BIRCHIT']) {
    const note = byName.get(name).acquisition.direct_sources.find(source => source.note.startsWith('Free level-5 starter partner'))?.note;
    assert.ok(note?.includes('first parcel') && note.includes('two starters you did not choose'));
    assert.ok(!note.includes('Champion'));
  }
  for (const name of ['TRILUME', 'SIPHLET']) {
    const note = byName.get(name).acquisition.direct_sources.find(source => source.map === 'MnRochesterFossilLab')?.note;
    assert.ok(note?.includes('lab assistant gives you the other specimen for free'));
    assert.ok(!note.includes('Champion'));
  }
}
for (const name of ['trilume', 'panoplite', 'siphlet', 'obelurk']) {
  const page = fs.readFileSync(path.join(root, 'species', `${name}.html`), 'utf8');
  assert.ok(page.includes('Rochester Fossil Lab'));
  assert.ok(!page.includes('proposed Rochester revival courtyard') && !page.includes('it is not runtime registered'));
}
for (const name of ['VERDIGRIM', 'KETTLURK', 'PEPIVERN', 'BABEHEMOTH', 'BUNJANT', 'ICEMYTH', 'KENSIGARD', 'FLARKINDLE', 'STRANDRAK']) {
  assert.notEqual(byName.get(name).acquisition.status, 'pending');
  assert.ok(byName.get(name).acquisition.direct_sources.some(source => source.note.length > 40), `${name} needs discovery directions`);
}
for (const name of ['CURDLET', 'SQUEAKURD', 'DOUGHLIN', 'CRUMCONE', 'PAILBARON', 'TILTAWISP']) {
  assert.ok(byName.get(name).acquisition.direct_sources.length || byName.get(name).acquisition.evolutions.length, `${name} is obtainable at the fair or by evolving its fair family`);
  assert.ok(byName.get(name).acquisition.direct_sources.every(source => source.map === 'MnStateFairgrounds'), `${name} is exclusive to the State Fair`);
}
for (const name of ['PINIT', 'PITCHWICK', 'PYROPINE']) {
  assert.ok(!byName.get(name).acquisition.direct_sources.some(source => source.map === 'MnR02C00R00'), `${name} is not generic prairie farm filler`);
}
assert.ok(byName.get('PINIT').acquisition.direct_sources.some(source => source.map === 'MnTallTaleTrail'));
assert.ok(byName.get('PINIT').acquisition.direct_sources.some(source => source.map === 'MnR01C00R00' && source.levels.includes(4)), 'southern opening woodland keeps its low-level pine cone');
assert.ok(byName.get('PIPULSE').acquisition.direct_sources.every(source => source.map === 'MnRochesterPatientHouse'));
assert.ok(byName.get('PIPULSE').acquisition.direct_sources.some(source => source.note.includes('Free level-8') && source.note.includes('first parcel')));
assert.ok(!catalog.species.some(entry => entry.acquisition.direct_sources.some(source => source.map === 'MnR02C00R00' && ['Surf', 'Old Rod', 'Good Rod', 'Super Rod'].includes(source.method))), 'sealed farm drainage is not an acquisition source');
assert.ok(byName.get('LEECHIMP').acquisition.direct_sources.some(source => source.map === 'MnLeechLake' && source.area.includes('Pine Island')));
assert.ok(!byName.get('LEECHIMP').acquisition.direct_sources.some(source => source.map === 'MnLeechLake' && source.method === 'Surf'), 'zero-rate fallback is not a habitat');
const habitats = JSON.parse(fs.readFileSync('planning/minnesota/pokemon/acquisition-v01/water-caves-v01/habitats.json'));
const ecology = JSON.parse(fs.readFileSync('planning/minnesota/pokemon/acquisition-v01/biome-ecology-v01/ecology.json'));
for (const habitat of habitats.maps) {
  for (const [field, method] of [['walking', 'Cave'], ['water', 'Surf']]) {
    if (!habitat[field]) continue;
    for (const [time, slots] of Object.entries(habitat[field].table)) {
      for (const [index, slot] of slots.entries()) {
        const correction = ecology[field === 'walking' ? 'grass' : 'surf'].find(site => site.map === habitat.map)
          ?.edits.find(edit => edit.time === time && edit.slot === index);
        const resident = correction?.after || slot.species;
        const relocation = ecology.habitat_review.relocations.find(edit => edit.map === habitat.map && edit.zone === null && edit.before === resident);
        const local = relocation?.after || resident;
        const group = journey.get(habitat.map);
        const species = field === 'walking' ? group?.promotions?.[local] || local : local;
        const level = group ? group[field === 'walking' ? 'grass' : 'surf'][index] : correction?.level || slot.level;
        const sources = byName.get(species).acquisition.direct_sources;
        assert.ok(sources.some(source => source.map === habitat.map && source.method === method
          && source.levels.includes(level) && source.times.includes(time)), `${habitat.map} ${time} ${species}`);
      }
    }
  }
}
if (progression) {
  const lateTown = catalog.species.flatMap(entry => entry.acquisition.direct_sources).filter(source => source.map === 'MnGrandMarais' && source.method === 'Grass');
  assert.ok(lateTown.length > 4);
  assert.ok(lateTown.every(source => source.levels.every(level => level >= 39 && level <= 43)));
  assert.ok(lateTown.every(source => source.note.includes('all eight badges')));
  assert.ok(byName.get('GOUGECREST').acquisition.direct_sources.some(source => source.map === 'MnGrandMarais'));
  assert.ok(!byName.get('PAILBARON').acquisition.direct_sources.length);
  assert.ok(byName.get('PAILBARON').acquisition.evolution_sources.some(source => source.map === 'MnStateFairgrounds' && source.note.includes('level 38')));
  assert.ok(byName.get('PAILBARON').acquisition.evolutions.some(edge => edge.from === 'CRUMCONE' && edge.level === 38));
  assert.ok(byName.get('CURDLET').acquisition.direct_sources.every(source => source.levels.every(level => level >= 21)));
  for (const [name, place] of [['pinit', 'Route 1'], ['pouchid', 'Winona']]) {
    const page = new JSDOM(fs.readFileSync(path.join(root, 'species', `${name}.html`), 'utf8'));
    assert.ok(page.window.document.querySelector('.location-list li strong').textContent.includes(place), 'field guide starts with the earliest acquisition');
    page.window.close();
  }
}
for (const map of ['MnBemidji', 'MnDuluth', 'MnMoorhead']) {
  const source = byName.get('KEYPOUT').acquisition.direct_sources.find(source => source.map === map && source.method === 'Surf');
  assert.deepEqual(source.times, ['night'], `${map} night-only burbot`);
}
assert.ok(byName.get('SCUTLET').acquisition.direct_sources.some(source => source.map === 'MnMoorhead'
  && source.method === 'Surf' && source.place === 'Moorhead · Red River' && source.note.includes('signed brick opening')));
assert.ok(byName.get('FENCICLE').acquisition.direct_sources.some(source => source.place === 'Voyageurs Ice Cavern · B1F'));
for (const entry of catalog.species) {
  assert.ok(!entry.acquisition.direct_sources.some(source => ['MnR12Cave1F', 'MnR13Cave1F'].includes(source.map) && source.method === 'Cave'),
    'refuge floors remain peaceful');
}
for (const folder of ['sprites', 'thumbnails', 'animations']) {
  const files = fs.readdirSync(path.join(baseline, folder));
  assert.deepEqual(fs.readdirSync(path.join(root, folder)), files);
  for (const file of files) assert.deepEqual(fs.readFileSync(path.join(root, folder, file)), fs.readFileSync(path.join(baseline, folder, file)));
}
const index = new JSDOM(fs.readFileSync(path.join(root, 'index.html'), 'utf8'));
const oatletId = byName.get('OATLET').identity.id;
assert.match(index.window.document.querySelector(`[data-id="${oatletId}"]`).dataset.locationSearch, /Harvest Oats/);
console.log('PASS: all 225 acquisition routes, specialist habitats, free starters/clinic companion, both fossils, rare discoveries and preserved media.');
