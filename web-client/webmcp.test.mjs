import test from 'node:test';
import assert from 'node:assert/strict';
import { createGameBridge, gameTools, playerVisibleObservation, registerGameTools } from './webmcp.js';

test('tools register with current Document API and unregister through abort', async () => {
  const registered = [];
  let canceled = 0;
  const doc = { modelContext: { async registerTool(tool, options) { registered.push({ tool, options }); } } };
  const result = await registerGameTools(doc, { cancel() { canceled++; } });
  assert.equal(result.supported, true);
  assert.equal(registered.length, 6);
  assert.ok(registered.every(({ options }) => options.signal instanceof AbortSignal));
  assert.ok(registered.some(({ tool }) => tool.name === 'pokemon_press'));
  result.dispose();
  assert.ok(registered.every(({ options }) => options.signal.aborted));
  assert.equal(canceled, 1);
});

test('unsupported browsers stay unsupported without installing a shim', async () => {
  const doc = {};
  assert.deepEqual(await registerGameTools(doc, {}), { supported: false, tools: [] });
  assert.equal(doc.modelContext, undefined);
});

test('partial registration failure aborts every registered tool', async () => {
  const signals = [];
  await assert.rejects(registerGameTools({ modelContext: { async registerTool(tool, { signal }) {
    signals.push(signal); if (signals.length === 2) throw new Error('denied');
  } } }, { cancel() {} }), /denied/);
  assert.ok(signals.every(signal => signal.aborted));
});

test('press validates bounds, rejects hidden actions, and returns actual outcome', async () => {
  const calls = [];
  const tools = gameTools({ async execute(command) { calls.push(command); return { status: { screen: 'battle' } }; } });
  const press = tools.find(tool => tool.name === 'pokemon_press');
  for (const input of [{ button: 'wait' }, { button: 'a', frames: 0 }, { button: 'a', frames: 61 }, { button: 'a', frames: 1.5 }, { button: 'a', warp: 'x' }]) {
    await assert.rejects(press.execute(input), TypeError);
  }
  assert.equal(calls.length, 0);
  assert.equal((await press.execute({ button: 'a' })).status.screen, 'battle');
  assert.deepEqual(calls, [{ kind: 'press', button: 'a', frames: 1 }]);
  assert.equal(press.annotations.readOnlyHint, false);
});

test('WebMCP exposes player-visible context without guide or engine state', () => {
  const visible = playerVisibleObservation({
    frame: 90,
    status: { screen: 'overworld', party: [{ nickname: 'CINDER', hp: 19, max_hp: 20 }] },
    reward_state: { event_flags: ['SECRET_GOAL'], engine_flags: ['HIDDEN'], scenes: { Lab: 'DONE' } },
    observe: {
      text: 'internal script LabScene:12',
      visible_dialogue: 'Hello there!',
      rendered_text: ['Hello there!'],
      menus: [{ kind: 'start', entries: ['>PACK', 'SAVE'] }],
      battle: '',
    },
    map_info: {
      name: 'NewBarkTown', dimensions: [20, 18],
      player: { x: 10, y: 7, facing: 'Down' },
      objects: [{ name: 'HIDDEN_SCRIPT_ID', x: 12, y: 6 }],
      players: [{ name: 'KRIS', x: 9, y: 8, facing: 'Left' }],
      terrain: { rows: [[{ permission: 119, terrain: 'Water' }]] },
    },
    flow_state: { animating: false, buttons: ['a'] },
    recent_events: { last_action: 'internal command', error: null },
  });
  assert.equal(visible.observe.visible_dialogue, 'Hello there!');
  assert.deepEqual(visible.map_info.visible_objects, [{ offset_x: 2, offset_y: -1 }]);
  assert.deepEqual(visible.map_info.visible_players, [{ name: 'KRIS', offset_x: -1, offset_y: 1, facing: 'Left' }]);
  const serialized = JSON.stringify(visible);
  for (const hidden of ['reward_state', 'event_flags', 'engine_flags', 'SECRET_GOAL', 'terrain', 'permission', 'dimensions', 'HIDDEN_SCRIPT_ID', 'internal script', 'recent_events']) {
    assert.equal(serialized.includes(hidden), false, `leaked ${hidden}`);
  }
});

test('bridge serializes simultaneous calls without entering WASM concurrently', async () => {
  let polls = 0;
  let requests = 0;
  const bridge = createGameBridge({ crystal_webmcp_request: () => ++requests,
    crystal_webmcp_poll: id => ++polls % 2 ? undefined : JSON.stringify({ frame: 40 + id }),
  }, { intervalMs: 1 });
  const first = bridge.execute({ kind: 'observe' });
  const second = bridge.execute({ kind: 'observe' });
  assert.deepEqual(await first, { frame: 41 });
  assert.deepEqual(await second, { frame: 42 });
  assert.equal(requests, 2);
});

test('abort and timeout cancel WASM input without replaying actions', async () => {
  let cancels = 0;
  let requests = 0;
  const wasm = { crystal_webmcp_request: () => ++requests, crystal_webmcp_poll() {}, crystal_webmcp_cancel() { cancels++; } };
  const bridge = createGameBridge(wasm, { timeoutMs: 5, intervalMs: 1 });
  const controller = new AbortController();
  const result = bridge.execute({ kind: 'press', button: 'right', frames: 60 }, { signal: controller.signal });
  controller.abort();
  await assert.rejects(result, { name: 'AbortError' });
  await assert.rejects(bridge.execute({ kind: 'observe' }), /timed out/);
  assert.equal(requests, 1);
  assert.ok(cancels > 0);
});

test('multiplayer tools use only the existing facing-player interactions', async () => {
  const calls = [];
  const tool = gameTools({ async execute(command) { calls.push(command); return { multiplayer: { session_active: false } }; } }).find(tool => tool.name === 'pokemon_multiplayer');
  await assert.rejects(tool.execute({ interaction: 'teleport' }), TypeError);
  await tool.execute({ interaction: 'trade' });
  assert.deepEqual(calls, [{ kind: 'multiplayer', interaction: 'trade' }]);
});
