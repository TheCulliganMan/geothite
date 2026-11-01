// Render the real Rust terminal buffers at fixed, normal-sized monospace cells.
// This is visual QA, not a browser frontend or a second painter/game engine.
import assert from 'node:assert/strict';
import {readFile, readdir, writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {chromium} from 'playwright';
const output=resolve(process.env.TUI_TERMINAL_ART_ROOT ?? 'target/tui-terminal-art');
const browser=await chromium.launch({headless:true});
try {
  const page=await browser.newPage({deviceScaleFactor:2});
  const results=[];
  for(const file of (await readdir(output)).filter(f=>f.endsWith('.json')).sort()) {
    const frame=JSON.parse(await readFile(resolve(output,file)));
    const {cols,rows,before,after}=frame;
    assert.equal(before.length,cols*rows); assert.equal(after.length,cols*rows);
    assert(after.some(c=>/[\u2801-\u28ff]/u.test(c[0])),'Actual terminal subcell dots');
    await page.setViewportSize({width:cols*10+32,height:rows*20*2+96});
    await page.setContent('<style>body{margin:16px;background:#030312;color:#e8e6da;font:14px Menlo,monospace}canvas{display:block}</style><div>Before: one dot per cell</div><canvas id="before"></canvas><div>After: shared dot field, eight subcells per cell</div><canvas id="after"></canvas>');
    await page.evaluate(async ({cols,rows,before,after})=>{
      await document.fonts.ready;
      for(const [id,cells] of [['before',before],['after',after]]) {
        const canvas=document.getElementById(id); canvas.width=cols*10; canvas.height=rows*20;
        const ctx=canvas.getContext('2d');ctx.font='16px Menlo,monospace';ctx.textBaseline='middle';
        cells.forEach(([glyph,fg,bg],i)=>{
          const x=(i%cols)*10,y=Math.floor(i/cols)*20;
          ctx.fillStyle=bg;ctx.fillRect(x,y,10,20);ctx.fillStyle=fg;ctx.fillText(glyph,x,y+10);
        });
      }
    },frame);
    const image=file.replace('.json','.png');await page.screenshot({path:resolve(output,image)});
    results.push({file:image,map:frame.map,tile:frame.tile,size:[cols,rows]});
  }
  await writeFile(resolve(output,'index.html'),`<!doctype html><meta charset="utf-8"><title>Terminal detail comparison</title><style>body{background:#030312;color:#e8e6da;font:16px system-ui}img{max-width:100%}</style>${results.map(r=>`<h2>${r.map} ${r.size.join('×')}</h2><img src="${r.file}">`).join('')}`);
  console.log(`${results.length} real terminal frames compared; cells, palette and camera shared with the browser renderer.`);
} finally {await browser.close();}
