import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
const dir=resolve(process.env.TUI_COMPARE_ROOT ?? 'target/tui-art-eval');
const before=JSON.parse(await readFile(resolve(dir,'before/results.json')));
const after=JSON.parse(await readFile(resolve(dir,'after/results.json')));
let desktop=0, phones=0;
for(const a of after) {
  assert.deepEqual(a.errors,[]);
  const m=a.metrics;
  assert.equal(m.view, a.scene === 'room-text' ? 'text' : 'painted', 'Painted is the default on BOTH platforms');
  assert(m.x>=-1&&m.y>=-1&&m.x+m.width<=m.viewport[0]+1&&m.y+m.height<=m.viewport[1]+1,`${a.name}/${a.scene}: frame fits viewport`);
  if(a.name.startsWith('desktop')) {
    desktop++;
    assert.equal(a.metrics.controls.length,0,'Desktop has no Game Boy buttons');
  }else{
    assert(m.x>=-1&&m.y>=-1&&m.x+m.width<=m.viewport[0]+1&&m.y+m.height<=m.viewport[1]+1,`${a.name}/${a.scene}: frame fits viewport`);
    assert.equal(m.controls.length,8,'Eight mobile Game Boy controls');
    for(const c of m.controls) {assert(c.width>=44&&c.height>=44,'44px thumb target');assert(c.x>=0&&c.y>=0&&c.x+c.width<=m.viewport[0]+1&&c.y+c.height<=m.viewport[1]+1,'Every button on screen');}
    phones++;
  }
}
assert(desktop >= 12, 'Both desktop sizes must exercise painted room/menu/dialogue/route/battle');
const order = ['iphone', 'iphone-se', 'iphone-large', 'phone-landscape', 'desktop', 'desktop-small'];
const scenes = ['battle', 'route', 'room', 'room-text', 'menu', 'dialogue', 'dialogue-page2'];
const rows=[...after].sort((a,b)=>order.indexOf(a.name)-order.indexOf(b.name)||scenes.indexOf(a.scene)-scenes.indexOf(b.scene)).map(a=>`<section><h2>${a.name} · ${a.scene}</h2><div>${before.some(b=>b.name===a.name&&b.scene===a.scene)?`<figure><figcaption>Before</figcaption><img src="before/${a.name}-${a.scene}.png"></figure>`:''}<figure><figcaption>${a.metrics.view === 'text' ? 'Selectable text view' : 'Painted default'}</figcaption><img src="after/${a.name}-${a.scene}.png"></figure></div></section>`).join('');
await writeFile(resolve(dir,'index.html'),`<!doctype html><html lang="en"><meta charset="utf-8"><title>Geothite ASCII visual eval</title><style>body{background:#080e17;color:#e8e6da;font:16px system-ui;margin:32px}section{margin-bottom:48px}section>div{display:flex;gap:24px;align-items:flex-start}figure{margin:0;max-width:48%}img{max-width:100%;height:auto}figcaption{margin:12px 0;color:#f6c271}</style><h1>Geothite · visual comparisons</h1><p>${desktop} desktop painted/text layouts fit without Game Boy controls. ${phones} mobile scenes fit with eight 44px+ targets. Artwork quality is separately inspected; green metrics alone are not an art evaluation.</p>${rows}</html>`);
console.log(`${desktop} desktop painted/text layouts passed; ${phones} phone frame/button fit evals passed.`);
