import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const root = process.env.POKEDEX_WEB_RELEASE;
assert.ok(root, 'Set POKEDEX_WEB_RELEASE');
const catalog = JSON.parse(fs.readFileSync(path.join(root, 'pokedex.json')));
const ids = ['MN_043', 'MN_044', 'MN_318'];
const line = ids.map(id => catalog.species.find(species => species.identity.id === id));
assert.deepEqual(line.map(species => species.identity.name), ['Jiglet', 'Trollure', 'Crappeller']);
assert.deepEqual(line.map(species => Object.values(species.profile.base_stats).reduce((a, b) => a + b)), [275, 410, 550]);
assert.deepEqual(line.map(species => species.profile.evolution?.level ?? null), [24, 40, null]);
assert.deepEqual(line.map(species => species.identity.evolves_to), ['MN_044', 'MN_318', null]);
assert.deepEqual(line.map(species => species.identity.regional_number - line[0].identity.regional_number), [0, 1, 2]);
assert.equal(line[2].profile.base_stats.attack, 120);
assert.equal(line[2].profile.base_stats.speed, 80);
assert.ok(line[2].profile.level_moves.some(([level, move]) => level === 44 && move === 'SWORDS_DANCE'));
const script = fs.readFileSync(path.join(root, 'app.js'), 'utf8');
for (const species of line) {
  const name = species.identity.name.toLowerCase();
  const dom = new JSDOM(fs.readFileSync(path.join(root, 'species', `${name}.html`)), {
    url: `https://example.test/pokedex/species/${name}.html`, runScripts: 'outside-only',
  });
  dom.window.matchMedia = () => ({ matches: false, addEventListener() {} });
  dom.window.eval(script);
  const doc = dom.window.document;
  assert.ok(doc.querySelector('[data-cry]').dataset.cry.endsWith(`/${species.identity.id}.wav`));
  assert.ok(doc.querySelector('[data-walker]'), `${name}: reviewed walking sprite`);
  doc.querySelector('[data-facing="back"]').click();
  doc.querySelector('#shiny-toggle').click();
  assert.ok(doc.querySelector('#specimen-portrait').src.endsWith('-back-shiny.png'));
  assert.ok(doc.querySelector('[data-walk-normal]').src.endsWith('-shiny.png'));
  dom.window.close();
}
console.log('PASS: adjacent three-stage lure family, earned 550-stat final power, Swords Dance, cries, walking sprites and back/shiny controls');
