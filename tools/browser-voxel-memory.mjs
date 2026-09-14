/** Real saved-game switches, camera movement, and walking with explicit memory budgets. */
import { chromium, webkit } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const [url, output, engine='webkit', fixture='new-bark'] = process.argv.slice(2);
assert.ok(url && output && ['webkit','chromium'].includes(engine));
const storage=JSON.parse(await fs.readFile(new URL(`./fixtures/${fixture}-saved-game.json`,import.meta.url),'utf8'));
for(const origin of storage.origins) origin.origin=new URL(url).origin;
const report={url,engine,fixture,samples:[],errors:[]};
const browser=await ({chromium,webkit}[engine]).launch({headless:false});
try {
 const context=await browser.newContext({viewport:{width:430,height:932},deviceScaleFactor:3,isMobile:true,hasTouch:true,storageState:storage});
 await context.addInitScript(()=>{
  window.voxelMemory={largestBuffer:0,bufferUploadBytes:0,contextLost:0};
  const proto=WebGL2RenderingContext.prototype;
  const bufferData=proto.bufferData;
  proto.bufferData=function(...args){
   const bytes=typeof args[1]==='number'?args[1]:(args[1]?.byteLength??0);
   voxelMemory.largestBuffer=Math.max(voxelMemory.largestBuffer,bytes);
   voxelMemory.bufferUploadBytes+=bytes;
   return bufferData.apply(this,args);
  };
  document.addEventListener('webglcontextlost',()=>voxelMemory.contextLost++,true);
 });
 const page=await context.newPage();
 page.on('crash',()=>report.errors.push('Browser page process crashed'));
 page.on('pageerror',e=>report.errors.push(e.message));
 await page.goto(url);
 await page.waitForFunction(()=>document.querySelector('#loading').hidden||document.querySelector('#startup-error').open,null,{timeout:180000});
 assert.equal(await page.locator('#startup-error').evaluate(e=>e.open?e.textContent:null),null);
 const initial=await page.evaluate(async()=>{
  const scripts=[...document.scripts].map(s=>s.textContent).join('\n');
  const path=scripts.match(/import\('(\.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)'\)/)?.[1];
  if(!path)throw new Error('Missing compiled bundle import');
  const wasm=await import(path);window.voxelRaw=await wasm.default();
  const {createGameBridge}=await import('./webmcp.js');window.voxelBridge=createGameBridge(wasm);
  const state=await voxelBridge.execute({kind:'observe'});
  return {screen:state.status.screen,map:state.map_info.name,player:state.map_info.player};
 });
 assert.equal(initial.screen,'overworld','fixture must load a genuine saved overworld');
 report.initial=initial;
 const sample=async label=>{
  const data=await page.evaluate(label=>({label,heapBytes:voxelRaw.memory.buffer.byteLength,...voxelMemory,enabled:document.querySelector('#view-toggle').getAttribute('aria-pressed')}),label);
  report.samples.push(data);console.log(JSON.stringify(data));
  // WASM memory keeps its high-water mark, so this also detects transient peaks.
  assert.ok(data.heapBytes<=512*1024*1024,`WASM heap exceeded 512 MiB: ${data.heapBytes}`);
  assert.ok(data.largestBuffer<=32*1024*1024,`single GPU buffer exceeded 32 MiB: ${data.largestBuffer}`);
  assert.equal(data.contextLost,0);
  assert.deepEqual(report.errors,[]);
 };
 assert.equal(initial.map,'NewBarkTown');
 await sample('2d-start');
 const settings=page.locator('#player-options > summary');
 await settings.click();
 for(let i=0;i<6;i++){
  await page.locator('#view-toggle').click();
  await page.waitForTimeout(2000);
  assert.equal(await page.locator('#view-toggle').getAttribute('aria-pressed'),String(i%2===0));
  await sample(`switch-${i+1}`);
 }
 await page.locator('#view-toggle').click();
 for(let i=0;i<8;i++)await page.locator('#rotate-right').click();
 await page.locator('#zoom-out').click();
 await settings.click();
 await page.locator('canvas').focus();
 const pad=await page.locator('#dpad').boundingBox();
 assert.ok(pad,'mobile direction pad must be visible');
 await page.mouse.move(pad.x+pad.width*(fixture==='route'?0.2:0.8),pad.y+pad.height/2);
 await page.mouse.down();await page.waitForTimeout(3000);await page.mouse.up();
 await page.setViewportSize({width:932,height:430});
 await page.waitForTimeout(2000);
 await sample('orbit-walk-landscape');
 report.final=await page.evaluate(async()=>{const s=await voxelBridge.execute({kind:'observe'});return {screen:s.status.screen,map:s.map_info.name,player:s.map_info.player};});
 assert.equal(report.final.screen,'overworld');
 if(fixture==='route')assert.equal(report.final.map,'Route29');
 else assert.notDeepEqual(report.final.player,report.initial.player,'the mobile pad must move the player');
 await page.screenshot({path:output.replace(/\.json$/,'.png')});
}catch(error){report.failure=String(error);throw error;}
finally{await fs.writeFile(output,JSON.stringify(report,null,2)+'\n');await browser.close();}
