/** Validate deployed WASM startup in Chromium and mobile WebKit, including stale glue. */
import { chromium, webkit, devices } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const [url, staleGlue, output] = process.argv.slice(2);
assert.ok(url && staleGlue && output, 'usage: node tools/browser-deployment-smoke.mjs URL OLD_GLUE_JS OUTPUT_JSON');
const oldGlue = await fs.readFile(staleGlue, 'utf8');
const results = [];
try {
 for (const [engine, type, options] of [
  ['chromium', chromium, { viewport: {width:1280,height:900} }],
  ['mobile-webkit', webkit, devices['iPhone 13']],
 ]) {
  const headless = process.env.BROWSER_HEADLESS === '1';
  const browser = await type.launch({headless,args:engine==='chromium'&&headless?['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']:undefined});
  try {
   for (const stale of [false,true]) {
    const context = await browser.newContext(options);
    const page = await context.newPage();
    const errors=[];page.on('pageerror',e=>errors.push(e.message));
    let staleRequests=0;
    if(stale) await page.route('**/crystal-bevy.js', route=>{
     staleRequests++;return route.fulfill({contentType:'text/javascript',body:oldGlue});
    });
    await page.goto(url);
    await page.waitForFunction(()=>document.querySelector('#loading').hidden || document.querySelector('#startup-error').open,null,{timeout:180000});
    const startup=await page.evaluate(()=>({error:document.querySelector('#startup-error').open?document.querySelector('#startup-error').textContent:null,loaded:document.querySelector('#loading').hidden,worker:typeof globalThis.__crystalPollMidi}));
    assert.equal(startup.error,null,`${engine} stale=${stale}: ${startup.error}`);
    assert.equal(startup.loaded,true);
    assert.equal(startup.worker,'function');
    const state=await page.evaluate(async()=>{
     const script=[...document.scripts].map(s=>s.textContent).join('\n');
     const path=script.match(/import\('(\.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)'\)/)?.[1];
     if(!path) throw new Error('Missing compiled bundle import');
     const wasm=await import(path);
     const {createGameBridge}=await import('./webmcp.js');
     const bridge=createGameBridge(wasm);
     let state=await bridge.execute({kind:'observe'});
     for(let i=0;i<4&&state.status.screen==='intro';i++) state=await bridge.execute({kind:'press',button:'a',frames:8});
     for(let i=0;i<6&&state.status.screen==='title'&&!state.observe.text.includes('NEW GAME');i++) state=await bridge.execute({kind:'press',button:'start',frames:8});
     return {screen:state.status.screen,text:state.observe.text,path};
    });
    assert.equal(state.screen,'title');
    assert.match(state.text,/NEW GAME/);
    assert.deepEqual(errors,[]);
    const screenshot=output.replace(/\.json$/,`-${engine}-${stale?'stale':'fresh'}.png`);
    await page.screenshot({path:screenshot});
    const result={engine,stale,staleRequests,startup,state,errors,screenshot};results.push(result);console.log(JSON.stringify(result));
    await context.close();
   }
  }finally{await browser.close()}
 }
}finally{await fs.writeFile(output,JSON.stringify({url,results},null,2)+'\n')}
