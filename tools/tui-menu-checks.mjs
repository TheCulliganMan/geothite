// Shared actual-input checks for native MCP and browser keyboard/WebMCP.
import assert from 'node:assert/strict';
export async function checkMenus(observe, drivePress, capture = async () => {}) {
  const text = s => [s.status_line, ...s.menu, ...s.prompt, ...s.dialogue, ...s.info].map(x => x.text ?? x).join('\n');
  const press = async button => {
    await drivePress(button);
    if (process.env.TUI_MENU_TRACE) console.log(button, text(await observe()));
  };
  const selected = s => s.menu.find(line => line.kind === 'selected')?.text ?? '';
  const hp = text(await observe()).match(/CYNDAQUIL\s+L\s*5\s+(\d+)\/(\d+)/);
  assert(hp, 'Menu fixture contains the real damaged starter');
  async function choose(pattern) {
    for (let i = 0; i < 24; i++) {
      const view = await observe();
      if (pattern.test(selected(view))) return press('a');
      await press('down');
    }
    assert.fail(`Cannot select ${pattern}: ${text(await observe())}`);
  }
  await press('start');
  await choose(/\bPACK\b/);
  assert.match(text(await observe()), /POTION.*x02/);
  for (let i = 0; i < 3; i++) await press('a');
  const healed = text(await observe()).match(/CYNDAQUIL\s+L\s*5\s+(\d+)\/(\d+)/);
  assert.equal(Number(healed?.[1]), Math.min(Number(hp[1]) + 20, Number(hp[2])), 'Potion changes actual party HP');
  for (let i = 0; i < 6; i++) await press('b');
  await press('start'); await choose(/\bPACK\b/);
  assert.match(text(await observe()), /POTION.*x01/, 'Exactly one Potion consumed');
  await capture('pack');
  await press('right');
  assert.match(text(await observe()), /BALL.*x03/i);
  await press('right');
  assert.match(text(await observe()), /ITEMFINDER/);
  await press('a'); await press('a');
  assert.match(text(await observe()), /respond|nope|items/i, 'Itemfinder finishes its source audio/notice path');
  for (let i = 0; i < 6; i++) await press('b');
  await press('start'); await choose(/\bPACK\b/);
  // Pack preserves its last pocket. Rotate to the TM/HM surface.
  for (let i = 0; i < 4 && !/DYNAMICPUNCH|TM01|TM 01/.test(text(await observe())); i++) await press('right');
  assert.match(text(await observe()), /DYNAMICPUNCH|TM01|TM 01/);
  await press('a');
  assert.match(text(await observe()), /USE|BOOT|DYNAMICPUNCH|TM01/);
  for (let i = 0; i < 6; i++) await press('b');

  await press('start'); await choose(/DEX/);
  await choose(/CYNDAQUIL/);
  const page1 = text(await observe());
  assert.match(page1, /No\.155/);
  await press('a');
  assert.notEqual(text(await observe()), page1, 'Dex PAGE changes authored entry text');
  await press('right'); await press('a');
  assert.match(text(await observe()), /NEST|JOHTO/);
  await press('b'); await press('right'); await press('a'); // CRY
  await press('right');
  assert.match(selected(await observe()), /PRINT/, 'Cry cannot lock subsequent input');
  await press('a'); assert.match(text(await observe()), /Printer Error/);
  await press('b'); await press('b');
  await press('select');
  assert.match(text(await observe()), /NEW|OLD|ABC/);
  await press('b');
  await press('start'); await press('right'); // NORMAL -> FIRE
  assert.match(selected(await observe()), /FIRE/);
  await press('down'); await press('down'); await press('a');
  assert((await observe()).menu.some(line => /CYNDAQUIL/.test(line.text)), 'Caught-species search returns the actual listing');
  assert(!(await observe()).menu.some(line => /TYPE1/.test(line.text)), 'Search animation must finish');
  await press('b'); await press('left'); // Reset NORMAL -> STEEL; no caught Steel species.
  await press('down'); await press('down'); await press('a');
  assert.match(selected(await observe()), /TYPE1 STEEL/, 'Empty search returns to the source search menu');
  await press('down'); assert.match(selected(await observe()), /TYPE2/, 'Not-found hold releases input');
  await press('b'); await press('b');
  await capture('pokedex');

  await choose(/GEAR/);
  assert.match(text(await observe()), /CLOCK.*\n.*DAY|CLOCK/s);
  await press('right');
  const map = text(await observe());
  assert.match(map, /MAP JOHTO.*NEW BARK/is);
  await press('up'); assert.notEqual(text(await observe()), map, 'Map location selection changes');
  await capture('pokegear-map');
  await press('right');
  assert.match(text(await observe()), /MOM/);
  await press('a'); await press('a');
  assert(!/CALL\s*\n.*CANCEL/.test(text(await observe())), 'CALL enters authored dialogue after both rings');
  await capture('pokegear-phone');
  let finished = false;
  for (let i = 0; i < 100; i++) {
    await press('a');
    if (/MOM/.test(selected(await observe()))) { finished = true; break; }
  }
  assert(finished, 'Phone dialogue and timed hangup return to contact list');
  await press('right');
  assert.match(text(await observe()), /RADIO.*No signal/s);
  for (let i = 0; i < 8; i++) await press('up');
  const radio = text(await observe());
  assert.match(radio, /RADIO 4\.50/);
  assert(!/No signal/.test(radio), 'A real station is tuned');
  await capture('pokegear-radio');
  await press('left'); await press('b');
  assert.match(text(await observe()), /SAVE/);
  await press('b');
  const before = (await observe()).marker;
  await press('right'); await press('right');
  assert.notDeepEqual((await observe()).marker, before, 'Movement works after all menu/audio surfaces');
  await press('start'); assert.match(text(await observe()), /SAVE/);
}
