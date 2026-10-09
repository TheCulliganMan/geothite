// Real authored first-floor scripts; reusable across WASM, stdio MCP and PTY.
import assert from 'node:assert/strict';

export async function checkHome(observe, press) {
  async function move(button, count) {
    for (let step = 0; step < count; step++) {
      const before = await observe();
      let moved = false;
      for (let attempt = 0; attempt < 4; attempt++) {
        await press(button);
        const after = await observe();
        moved = JSON.stringify(after.marker) !== JSON.stringify(before.marker)
          || after.viewport_title !== before.viewport_title;
        if (moved) break;
      }
      assert(moved, `Home movement blocked ${button}: ${(await observe()).status_line}`);
    }
  }
  async function finishDialogue() {
    for (let i = 0; i < 128 && !(await observe()).status_line.startsWith('Overworld'); i++) await press('a');
    assert.match((await observe()).status_line, /^Overworld/, 'Mom conversation must finish without idle frames');
  }
  async function start() {
    await press('start');
    assert((await observe()).menu.some(line => line.text.includes('PACK')), 'Home Start must expose the production menu');
    await press('b');
  }
  assert.match((await observe()).status_line, /PlayersHouse2F.*\(3, 3\)/);
  await move('right', 4); await move('up', 3); await move('down', 4);
  const mom = await observe();
  assert.match(mom.status_line, /Text.*PlayersHouse1F/);
  assert(mom.dialogue.some(line => line.text.includes('CHRIS')));
  await press('right');
  assert.deepEqual((await observe()).marker, mom.marker, 'Mom owns direction while speaking');
  await press('a');
  assert.notDeepEqual((await observe()).dialogue, mom.dialogue, 'A advances Mom pagination');
  await finishDialogue();
  for (let repeat = 0; repeat < 2; repeat++) {
    await start();
    await move('left', 1);
    await press('a');
    assert(!(await observe()).status_line.startsWith('Overworld'), 'Retalking to Mom opens actual dialogue');
    await finishDialogue();
    await move('right', 1);
  }
  await start();
  await move('down', 3); await move('left', 4);
  assert.match((await observe()).status_line, /PlayersHouse1F.*\(5, 7\)/, 'Immediately left of the doormat');
  await start();
  await move('left', 3); await start();
  await move('right', 7); await move('up', 3);
  assert.match((await observe()).status_line, /PlayersHouse1F.*\(9, 4\)/);
}
