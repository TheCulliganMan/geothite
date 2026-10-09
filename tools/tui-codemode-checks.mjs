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

export const battleCode = `
  let view=await tools.observe(); let fight=false,moves=false,reward=false,turns=0;
  for(let i=0;i<100;i++) {
    if(view.status_line.startsWith('Overworld')) break;
    fight ||= view.menu.some(line=>line.text.includes('FIGHT'));
    const moveMenu=view.menu.some(line=>/PP|TACKLE|SCRATCH|EMBER|BITE|WATER GUN|RAGE/.test(line.text));
    moves ||= moveMenu; if(moveMenu) turns++;
    reward ||= view.dialogue.some(line=>/fainted|EXP|experience/i.test(line.text));
    view=await tools.press({button:'a'});
  }
  if(!view.status_line.startsWith('Overworld')) throw new Error('Battle did not finish');
  const before=view.marker;
  view=await tools.move({direction:'up',steps:2});
  const after=view.marker;
  view=await tools.press({button:'start'});
  return {fight,moves,reward,turns,before,after,menu:view.menu.map(line=>line.text)};
`;

export async function checkCodeBattle(execute) {
  const {value} = await execute(battleCode);
  assert(value.fight && value.moves && value.reward, JSON.stringify(value));
  assert(value.turns>=2, 'Real starter-level, multi-turn battle');
  assert.notDeepEqual(value.before,value.after,'Code mode restores post-battle movement');
  assert(value.menu.some(line=>line.includes('SAVE')), 'Production Start after code-mode battle');
  await execute("await tools.press({button:'b'}); return null;");
}
