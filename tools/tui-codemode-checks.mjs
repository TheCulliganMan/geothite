// Shared real-session assertions. execute accepts a JS function BODY, not eval.
import assert from 'node:assert/strict';

export async function checkCodeMode(execute, observe) {
  assert.deepEqual((await execute('return [typeof window,typeof fetch,typeof process,typeof require];')).value,
    ['undefined', 'undefined', 'undefined', 'undefined']);
  const before = await observe();
  const result = await execute(`
    await tools.press({button:'start'});
    const menu = (await codemode.observe()).menu.map(line=>line.text);
    await tools.press({button:'b'});
    return {menu};
  `);
  assert.equal(result.calls, 3);
  assert(result.value.menu.some(text => text.includes('PACK')));
  assert.deepEqual((await observe()).marker, before.marker);
  // Real taps, branches and multiple outputs, with no implicit idle frames.
  const moved = await execute(`
    let state = await tools.observe(); const before = state.marker;
    for (let i=0;i<4;i++) {
      state = await tools.press({button:'right'});
      if (JSON.stringify(state.marker)!==JSON.stringify(before)) break;
    }
    return {before,after:state.marker};
  `);
  assert.notDeepEqual(moved.value.before, moved.value.after);
  await execute(`
    let state = await tools.observe(); const before=state.marker;
    for(let i=0;i<4;i++) {
      state=await tools.press({button:'left'});
      if(JSON.stringify(state.marker)!==JSON.stringify(before)) break;
    }
    return state.status_line;
  `);
  assert.deepEqual((await observe()).marker, before.marker);
  await assert.rejects(execute('while(true){}'));
  await assert.rejects(execute('await new Promise(()=>{});'));
  await assert.rejects(execute("await tools.press({button:'teleport'});"));
  await assert.rejects(execute("await tools.move({direction:'right',steps:21});"));
  await assert.rejects(execute("await tools.press({button:'start'}); throw new Error('partial');"));
  assert((await observe()).menu.some(line=>line.text.includes('PACK')), 'Earlier applied inputs survive errors');
  await execute("await tools.press({button:'b'}); return null;");
  assert.equal((await execute('return typeof temporary;')).value, 'undefined');
}

// Only inspect visible HP/menu rows and press ordinary buttons. Current Wild
// Challenge fights can require healing even with a healthy level-five starter.
export function createStarterBattleInput() {
  let canHeal = true;
  return view => {
    const selected = Math.max(0, view.menu.findIndex(line => line.kind === 'selected'));
    const choose = target => target === selected ? 'a' : target > selected ? 'down' : 'up';
    const fight = view.menu.findIndex(line => /\bFIGHT\b/.test(line.text));
    if (fight >= 0) {
      const hp = view.info.map(line => line.text).join('\n').match(/L\s*\d+\s+(\d+)\/(\d+)/);
      const pack = view.menu.findIndex(line => /\bPACK\b/.test(line.text));
      return choose(canHeal && hp && Number(hp[1]) <= Number(hp[2]) / 2 && pack >= 0 ? pack : fight);
    }
    const potion = view.menu.findIndex(line => /\bPOTION\b.*x0*[1-9]/.test(line.text));
    if (potion >= 0) return choose(potion);
    if (view.menu.length && view.menu.every(line => /^\s*>?\s*(CANCEL|CLOSE)\s*$/.test(line.text))) {
      canHeal = false;
      return 'b';
    }
    const attack = view.menu.findIndex(line => /PP/.test(line.text) && /SCRATCH|TACKLE|EMBER|BITE|WATER GUN|RAGE/.test(line.text));
    return attack >= 0 ? choose(attack) : 'a';
  };
}

export const battleCode = String.raw`
  const nextButton=(${createStarterBattleInput.toString()})();
  let view=await tools.observe(); let fight=false,moves=false,reward=false,turns=0,usedPotion=false,healed=false;
  for(let i=0;i<100;i++) {
    if(view.status_line.startsWith('Overworld') && !view.dialogue.length) break;
    fight ||= view.menu.some(line=>line.text.includes('FIGHT'));
    const moveMenu=view.menu.some(line=>/PP|TACKLE|SCRATCH|EMBER|BITE|WATER GUN|RAGE/.test(line.text));
    moves ||= moveMenu; if(moveMenu) turns++;
    reward ||= view.dialogue.some(line=>/EXP|experience/i.test(line.text));
    healed ||= /recovered\s+\d+\s*HP/i.test(view.dialogue.map(line=>line.text).join(' '));
    const button=nextButton(view);
    usedPotion ||= button==='a' && view.menu.some(line=>line.kind==='selected' && /POTION/.test(line.text));
    view=await tools.press({button});
  }
  if(!view.status_line.startsWith('Overworld') || view.dialogue.length) throw new Error('Battle and result pages did not finish');
  const before=view.marker;
  view=await tools.move({direction:'up',steps:2});
  const after=view.marker;
  view=await tools.press({button:'start'});
  return {fight,moves,reward,turns,usedPotion,healed,before,after,menu:view.menu.map(line=>line.text)};
`;

export async function checkCodeBattle(execute) {
  const {value} = await execute(battleCode);
  assert(value.fight && value.moves && value.reward, JSON.stringify(value));
  assert(value.turns>=2, 'Real starter-level, multi-turn battle');
  assert(!value.usedPotion || value.healed, 'Selected Potion must produce real HP recovery text');
  assert.notDeepEqual(value.before,value.after,'Code mode restores post-battle movement');
  assert(value.menu.some(line=>line.includes('SAVE')), 'Production Start after code-mode battle');
  await execute("await tools.press({button:'b'}); return null;");
}
