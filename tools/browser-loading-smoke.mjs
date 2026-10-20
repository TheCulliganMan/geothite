// Check real loading, console diagnostics, asset requests, and in-page view swaps.
import { chromium, firefox, webkit, devices } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const [base, output] = process.argv.slice(2);
assert.ok(base && output, 'usage: node tools/browser-loading-smoke.mjs BASE_URL REPORT_JSON');
const reports = [];
try {
 for (const [name, engine, options, path] of [
  ['chromium-online-tui', chromium, {viewport:{width:1280,height:900}}, '/?view=tui'],
  ['webkit-online-3d', webkit, devices['iPhone 13'], '/?view=3d'],
  ['firefox-online-2d', firefox, {viewport:{width:1280,height:900}}, '/?view=2d'],
  ['chromium-tui-alias', chromium, {viewport:{width:1280,height:900}}, '/tui/'],
 ].filter(([name]) => !process.env.BROWSER_SMOKE_CASE || name.includes(process.env.BROWSER_SMOKE_CASE))) {
  const browser = await engine.launch({headless:true, args:engine===chromium
   ? process.platform==='darwin'?['--use-angle=metal','--mute-audio']:['--use-angle=swiftshader','--enable-unsafe-swiftshader','--mute-audio'] : undefined});
  const result = {name, path, console:[], browserAdvisories:[], errors:[], requests:[], swaps:[]}; reports.push(result);
  let collecting=true;
  try {
   const context=await browser.newContext(options);const page=await context.newPage();
   page.on('console',message=>{
    if(!collecting)return;
    const entry={level:message.type(),text:message.text()};
    if(['error','warning'].includes(entry.level)||/\bERROR\b/.test(entry.text)) {
     result.console.push(entry);
     // Firefox reports these upstream WebGL driver advisories for wgpu's
     // adapter inspection and vertex-index-only fullscreen triangles. Retain
     // them in the report; they are not failed requests or rendering errors.
     if(name.startsWith('firefox') && entry.level==='warning' &&
       /WEBGL_debug_renderer_info is deprecated in Firefox|WebGL warning: drawArraysInstanced: Drawing without vertex attrib 0 array enabled/.test(entry.text)) result.browserAdvisories.push(entry);
     console.log(JSON.stringify({name,...entry}));
    }
   });
   page.on('pageerror',error=>{if(collecting)result.errors.push(error.stack);});
   page.on('response',response=>{if(collecting&&response.status()>=400)result.requests.push({status:response.status(),url:response.url()});});
   page.on('requestfailed',request=>{if(collecting)result.requests.push({url:request.url(),failure:request.failure()});});
   const response=await page.goto(new URL(path,base).href);
   assert.equal(response.status(),200);
   await page.waitForFunction(()=>document.querySelector('#loading')?.hidden||document.querySelector('#startup-error')?.open,null,{timeout:300000});
   assert.equal(await page.locator('#startup-error').evaluate(e=>e.open?e.textContent:null),null);
   await page.waitForFunction(() => document.querySelector('.chat-status')?.dataset.connected === 'true', null, {timeout:60000});
   result.connected = true;
   await page.waitForTimeout(10000);
   const href=page.url();
   for(const mode of ['2d','3d','tui']) {
    await page.locator(`#view-navigation [data-view="${mode}"]`).click();
    await page.waitForTimeout(3000);
    assert.equal(await page.locator('#view-navigation [aria-current]').getAttribute('data-view'),mode);
    assert.equal(new URL(page.url()).pathname,new URL(href).pathname);
    result.swaps.push(mode);
   }
   await page.waitForFunction(()=>Number(document.querySelector('#tui-surface').dataset.frame)>=0,null,{timeout:30000});
   result.navigations=await page.evaluate(()=>performance.getEntriesByType('navigation').length);
   assert.equal(result.navigations,1);
   await page.screenshot({path:output.replace(/\.json$/,`-${name}.png`)});
   assert.deepEqual(result.errors,[],`${name}: JavaScript exceptions`);
   assert.deepEqual(result.requests,[],`${name}: failed asset/API requests`);
   assert.deepEqual(result.console.filter(entry=>!result.browserAdvisories.includes(entry)),[],`${name}: unexpected browser/renderer diagnostics`);
   console.log(JSON.stringify({name,passed:true,swaps:result.swaps}));
  } finally {collecting=false;await browser.close();}
 }
} finally {await fs.writeFile(output,JSON.stringify(reports,null,2)+'\n');}
