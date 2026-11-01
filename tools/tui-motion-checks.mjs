// Actual canvas pixels at normal rendering scale. Hash inequality is not a
// perceptibility check; empty paper/captions must not inflate this measurement.
import assert from 'node:assert/strict';
import {mkdir,writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';

export async function checkVisibleInkMotion(page, captureOutput=true) {
  const before = await page.evaluate(() => window.geothiteTui.observe());
  await page.emulateMedia({reducedMotion:'no-preference'});
  await page.waitForTimeout(200); // Allow preference-driven painter replacement.
  // Record EVERY actual dot-layer compose, not two snapshots seconds apart.
  // That old check accepted a long hold followed by a sudden change.
  const cadence = await page.evaluate(async () => {
    const prototype=CanvasRenderingContext2D.prototype;
    const fillRect=prototype.fillRect;
    const frames=[];
    let previous;
    prototype.fillRect=function(x,y,w,h) {
      const result=fillRect.call(this,x,y,w,h);
      if(this.canvas.id!=='ascii' && x===0 && y===0 && w===this.canvas.width && h===this.canvas.height) {
        const ctx=this;
        queueMicrotask(()=>{
          const pixels=ctx.getImageData(0,0,w,h).data;
          let sum=0;
          if(previous) for(let i=0;i<pixels.length;i+=4) sum+=Math.max(
            Math.abs(pixels[i]-previous[i]),Math.abs(pixels[i+1]-previous[i+1]),Math.abs(pixels[i+2]-previous[i+2]));
          frames.push({time:performance.now(),change:previous?sum/(pixels.length/4):null});
          previous=pixels;
        });
      }
      return result;
    };
    try { await new Promise(resolve=>setTimeout(resolve,8500)); }
    finally {prototype.fillRect=fillRect;}
    const steps=frames.slice(1).map((f,i)=>({gap:f.time-frames[i].time,change:f.change}));
    return {count:frames.length,minChange:Math.min(...steps.map(s=>s.change)),
      maxChange:Math.max(...steps.map(s=>s.change)),maxGap:Math.max(...steps.map(s=>s.gap))};
  });
  assert(cadence.count>140,`Slow motion still needs a smooth visual cadence: ${JSON.stringify(cadence)}`);
  assert(cadence.minChange>0.02,`Every painted frame must drift, with no peak plateau: ${JSON.stringify(cadence)}`);
  assert(cadence.maxChange<8,`No sudden catch-up frame: ${JSON.stringify(cadence)}`);
  assert(cadence.maxGap<250,`No long visual hold: ${JSON.stringify(cadence)}`);
  const measure = () => page.evaluate(async () => {
    const canvas=[...document.querySelectorAll('canvas')].find(c=>c.id!=='ascii');
    if(!canvas) throw new Error('Missing real Rust dot layer');
    const ctx=canvas.getContext('2d');
    const first=ctx.getImageData(0,0,canvas.width,canvas.height).data;
    await new Promise(resolve=>setTimeout(resolve,2000));
    // Draws replace the offscreen canvas. Never sample a detached old painter.
    const current=[...document.querySelectorAll('canvas')].find(c=>c.id!=='ascii');
    if(!current||current.width!==canvas.width||current.height!==canvas.height) throw new Error('Scene resized during motion measurement');
    const last=current.getContext('2d').getImageData(0,0,current.width,current.height).data;
    let sum=0,strong=0;
    for(let i=0;i<first.length;i+=4) {
      const delta=Math.max(...[0,1,2].map(c=>Math.abs(first[i+c]-last[i+c])));
      sum+=delta;if(delta>=24)strong++;
    }
    return {mean:sum/(first.length/4),strong:strong/(first.length/4)};
  });
  const idle=await measure();
  assert(idle.mean>6&&idle.strong>.08,`Idle motion must be clearly visible, not tiny hash changes: ${JSON.stringify(idle)}`);
  const measuring=measure();
  // R redraws presentation only. Faster redraws must not postpone the deadline.
  const start=performance.now();
  while(performance.now()-start<2050) {
    await page.keyboard.press('r');
    await page.waitForTimeout(15);
  }
  const busy=await measuring;
  assert(busy.mean>6&&busy.strong>.08,`Redraw/input must not freeze motion: ${JSON.stringify(busy)}`);
  if(captureOutput&&process.env.TUI_CAPTURE_ANIMATION==='1') {
    const output=resolve(process.env.TUI_MOTION_OUTPUT??'target/tui-motion-review/web');
    await mkdir(output,{recursive:true});
    const start=performance.now(),timing=[];
    for(let index=0;index<40;index++) {
      timing.push({index,ms:performance.now()-start});
      await page.screenshot({path:resolve(output,`${String(index).padStart(4,'0')}.png`)});
      await page.waitForTimeout(100);
    }
    await writeFile(resolve(output,'timing.json'),JSON.stringify(timing,null,2));
  }
  assert.deepEqual(await page.evaluate(()=>window.geothiteTui.observe()),before,'Art clocks cannot change gameplay or tool snapshots');
  await page.emulateMedia({reducedMotion:'reduce'});
  await page.waitForTimeout(200);
  assert.deepEqual(await measure(),{mean:0,strong:0},'Reduced motion stays still while retaining state updates');
  console.log(`Continuous ink: ${cadence.count} frames, ${cadence.minChange.toFixed(2)}–${cadence.maxChange.toFixed(2)} pixel change/frame, max gap ${cadence.maxGap.toFixed(0)}ms. Idle ${idle.mean.toFixed(1)}, busy ${busy.mean.toFixed(1)}; reduced motion still.`);
}
