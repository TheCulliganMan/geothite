import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import vm from 'node:vm';

// Run the actual report script with a document/canvas stub. No browser,
// navigation, network, or external dependencies are needed to check its data UI.
function dashboard(overrides = {}) {
  const elements = new Map();
  class Element {
    constructor() {
      this.children = [];
      this.value = '0';
      this.clientWidth = 1000;
      this.textContent = '';
      this.style = {};
      this.classList = { toggle() {} };
    }
    append(...items) { this.children.push(...items); }
    replaceChildren(...items) { this.children = items; }
    addEventListener() {}
    getContext() {
      return { scale() {}, fillText() {}, stroke() {}, beginPath() {},
        moveTo() {}, lineTo() {}, setLineDash() {}, fillRect() {},
        measureText: value => ({ width: String(value).length * 6 }) };
    }
  }
  const el = id => {
    if (!elements.has(id)) elements.set(id, new Element());
    return elements.get(id);
  };
  el('window').value = 'all';
  const frame = (id, height) => ({ id, start_ms: id * 16, interval_ms: 16,
    main_cpu_ms: 2, map: 'field', tile: [1, 2], mode: 'Normal',
    terrain_pending: false, warp_phase: 'None', visible_ready: true,
    motion: { active: true, camera_eye: [0, height, 0], camera_rotation: [0, 0, 0, 1] } });
  const run = { success: true, presentation_continuity: { passed: false }, issues: [], directory: 'run', cpu_operations: [],
    checkpoints: [], passes: {}, gpu_supported: false, gpu_samples: [],
    frames: [frame(0, 1), frame(1, null)], recommendations: [], measured_frames: 2,
    interval_ms: { p50: 16, p95: 16, p99: 16 },
    main_cpu_ms: { p95: 2, maximum: 2 },
    world_residency: { available: true, unavailable_ms: 16, unavailable_frames: 1, unknown_intervals: 0 },
    stationary_camera: { observed_pairs: 1, max_vertical_cells: .125 } };
  const data = { provenance: { hardware: { system: 'test', cpu_count: 1 } },
    scenarios: [{ name: 'field', runs: [run] }] };
  Object.assign(run, overrides);
  const html = readFileSync(new URL('./performance_audit.html', import.meta.url), 'utf8');
  const script = html.split('<script>')[1].split('</script>')[0].replace('__DATA__', JSON.stringify(data));
  const context = vm.createContext({ document: { getElementById: el, createElement: () => new Element() },
    window: { addEventListener() {} }, devicePixelRatio: 1 });
  vm.runInContext(script, context, { timeout: 1000 });
  return { elements, context };
}

const text = element => String(element.textContent) + element.children.map(text).join('');

test('report cards retain unavailable time and the camera height correction', () => {
  const { elements } = dashboard();
  const cards = elements.get('cards').children.map(text);
  assert.equal(cards.length, 7);
  assert.match(cards.find(card => card.startsWith('3D unavailable')), /16\.00 ms/);
  assert.match(cards.find(card => card.startsWith('Camera Y drift')), /0\.1250 cells/);
});

test('a poisoned camera is unavailable even when the renderer active flag is true', () => {
  const { elements, context } = dashboard();
  const states = elements.get('state-plot').segments.filter(segment => segment.lane === 0);
  assert.deepEqual(Array.from(states, segment => segment.label), ['Active', 'Unavailable']);
  assert.equal(context.worldState({}), 'Unknown');
  assert.equal(context.worldState({ motion: { active: true } }), 'Unknown');
  assert.equal(Math.max(...elements.get('state-plot').segments.map(segment => segment.lane)), 4);
});

test('gameplay success does not hide a failed renderer-continuity gate', () => {
  const { elements } = dashboard();
  assert.equal(elements.get('status').textContent, '3D CONTINUITY FAILED');
  assert.equal(elements.get('status').className, 'tag bad');
});

test('one peak above 45 ms remains a failure when p95 and gameplay pass', () => {
  const { elements } = dashboard({ presentation_continuity: { passed: true },
    frame_budget: { passed: false, limit_ms: 45 }, frame_ceiling_exceedances: 1,
    interval_ms: { maximum: 45.01, p50: 16, p95: 16, p99: 16 } });
  assert.equal(elements.get('status').textContent, 'FRAME CEILING FAILED');
  assert.equal(elements.get('status').className, 'tag bad');
  const cards = elements.get('cards').children.map(text);
  assert.match(cards[0], /Peak interval45\.01 ms/);
  assert.match(cards[0], /ceiling 45 ms.*1 exceedances/);
});
