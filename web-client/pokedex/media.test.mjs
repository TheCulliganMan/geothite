import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { JSDOM } from 'jsdom';

const root = process.env.POKEDEX_WEB_RELEASE;
assert.ok(root, 'Set POKEDEX_WEB_RELEASE to the exported release');
const catalog = JSON.parse(fs.readFileSync(path.join(root, 'pokedex.json')));
const accepted = JSON.parse(fs.readFileSync('planning/minnesota/pokemon/overworld/20261027-v1/accepted.json'));
const acceptedIds = new Set(accepted.species.map(species => species.species_id));
const script = fs.readFileSync('web-client/pokedex/app.js', 'utf8');
function page(file = 'index.html') {
  const dom = new JSDOM(fs.readFileSync(path.join(root, file), 'utf8'), { url: `https://example.test/pokedex/${file}`, runScripts: 'outside-only', pretendToBeVisual: true });
  const { window } = dom;
  const listeners = [];
  const media = { matches: false, addEventListener(event, listener) { if (event === 'change') listeners.push(listener); } };
  window.matchMedia = () => media;
  const observers = [];
  window.IntersectionObserver = class {
    constructor(callback) { this.callback = callback; this.targets = []; observers.push(this); }
    observe(target) { this.targets.push(target); }
  };
  const players = [];
  window.Audio = class extends window.EventTarget {
    constructor() { super(); this.src = ''; this.plays = 0; this.pauses = 0; players.push(this); }
    play() { this.plays++; return this.result ? this.result() : Promise.resolve(); }
    pause() { this.pauses++; }
  };
  window.eval(script);
  return { dom, document: window.document, player: players[0], observers, reduce(matches) { media.matches = matches; listeners.forEach(listener => listener({ matches })); } };
}
const index = page();
const { document: doc, player } = index;
const cards = [...doc.querySelectorAll('.pokemon-card[data-cut=false]')];
assert.equal(cards.length, catalog.cry_species_count);
const expectedMediaCount = process.env.POKEDEX_WEB_BASELINE
 ? JSON.parse(fs.readFileSync(path.join(process.env.POKEDEX_WEB_BASELINE, 'pokedex.json'))).cry_species_count
 : JSON.parse(fs.readFileSync('planning/minnesota/pokemon/roster-curation.json')).active_species_count;
assert.equal(catalog.cry_species_count, expectedMediaCount);
assert.equal(doc.querySelectorAll('[data-cry]').length, cards.length);
assert.equal(doc.querySelectorAll('.pokemon-card[data-cut=true] [data-cry]').length, 0);
const walkers = [...doc.querySelectorAll('[data-walker]')];
assert.equal(walkers.length, catalog.walking_thumbnail_count);
assert.ok(walkers.every(walker => acceptedIds.has(walker.closest('[data-id]').dataset.id)), 'only reviewed species receive animated thumbnails');
assert.equal(player.plays, 0, 'no cry autoplay');
assert.equal(player.src, '', 'no audio request until a gesture');
assert.equal(player.preload, 'none');
for (const card of cards) {
  assert.ok(card.querySelector('[data-cry]').dataset.cry.endsWith(`/${card.dataset.id}.wav`));
  assert.equal(card.querySelector('a [data-cry]'), null, 'cry controls must not navigate away');
}
for (const walker of walkers) {
  const image = walker.querySelector('img');
  for (const url of [image.dataset.walkNormal, image.dataset.walkShiny]) {
    const png = fs.readFileSync(path.join(root, 'thumbnails', path.basename(url)));
    assert.deepEqual([png.readUInt32BE(16), png.readUInt32BE(20)], [64, 16]);
  }
  assert.equal(walker.hasAttribute('data-sprite-running'), false, 'offscreen thumbnails rest');
}
const observer = index.observers.find(observer => observer.targets.includes(walkers[0]));
observer.callback([{ target: walkers[0], isIntersecting: true }]);
assert.ok(walkers[0].hasAttribute('data-sprite-running'));
observer.callback([{ target: walkers[0], isIntersecting: false }]);
assert.equal(walkers[0].hasAttribute('data-sprite-running'), false);
observer.callback([{ target: walkers[0], isIntersecting: true }]);
const toggle = doc.querySelector('[data-walking-toggle]');
toggle.click();
assert.equal(walkers[0].hasAttribute('data-sprite-running'), false);
assert.equal(index.dom.window.localStorage.getItem('geothite.pokedex.walking.v1'), 'off');
toggle.click();
assert.ok(walkers[0].hasAttribute('data-sprite-running'));
index.reduce(true);
assert.equal(walkers[0].hasAttribute('data-sprite-running'), false);
assert.ok(toggle.disabled);
index.reduce(false);
assert.ok(walkers[0].hasAttribute('data-sprite-running'));
const [first, second] = doc.querySelectorAll('[data-cry]');
first.click();
await new Promise(setImmediate);
assert.equal(first.getAttribute('aria-pressed'), 'true');
assert.ok(player.src.endsWith('/MN_001.wav'));
second.click();
await new Promise(setImmediate);
assert.equal(first.getAttribute('aria-pressed'), 'false');
assert.equal(second.getAttribute('aria-pressed'), 'true');
assert.equal(player.plays, 2);
assert.ok(player.pauses >= 2, 'one active cry at a time');
second.click();
assert.equal(second.getAttribute('aria-pressed'), 'false');
first.click();
player.dispatchEvent(new index.dom.window.Event('ended'));
assert.equal(first.getAttribute('aria-pressed'), 'false');
player.result = () => Promise.reject(new Error('blocked'));
first.click();
await new Promise(setImmediate);
assert.match(doc.querySelector('[role=status]').textContent, /Could not play/);
assert.equal(first.getAttribute('aria-pressed'), 'false');
let rejectPrevious;
player.result = () => new Promise((_, reject) => { rejectPrevious = reject; });
first.click();
player.result = null;
second.click();
rejectPrevious(new Error('superseded'));
await new Promise(setImmediate);
assert.equal(second.getAttribute('aria-pressed'), 'true', 'stale audio rejection must not stop a newer cry');
index.dom.window.dispatchEvent(new index.dom.window.Event('pagehide'));
assert.equal(second.getAttribute('aria-pressed'), 'false');
const entry = page('species/birchit.html');
const walkingImage = entry.document.querySelector('[data-walk-normal]');
assert.ok(walkingImage);
entry.document.querySelector('#shiny-toggle').click();
assert.ok(walkingImage.src.endsWith('-shiny.png'));
entry.document.querySelector('#shiny-toggle').click();
assert.ok(walkingImage.src.endsWith('-normal.png'));
assert.equal(entry.document.querySelectorAll('[data-cry]').length, 1);
console.log(`PASS: ${cards.length} cries, ${walkers.length} native walking thumbnails, colors, visibility, reduced motion and exclusive audio playback`);
index.dom.window.close();
entry.dom.window.close();
