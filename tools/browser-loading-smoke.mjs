// Check real loading, console diagnostics, asset requests, and in-page view swaps.
import { chromium, webkit, devices } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const [base, output] = process.argv.slice(2);
assert.ok(base && output, 'usage: node tools/browser-loading-smoke.mjs BASE_URL REPORT_JSON');
const reports = [];
try {
 for (const [name, engine, options, path] of [
  ['chromium-online-tui', chromium, {viewport:{width:1280,height:900}}, '/?view=tui'],
  ['webkit-online-3d', webkit, devices['iPhone 13'], '/?view=3d'],
  ['chromium-tui-alias', chromium, {viewport:{width:1280,height:900}}, '/tui/'],
 ]) {
  const browser = await engine.launch({headless:true, args:engine===chromium
   ? process.platform==='darwin'?['--use-angle=metal','--mute-audio']:['--use-angle=swiftshader','--enable-unsafe-swiftshader','--mute-audio'] : undefined});
  const result = {name, path, console:[], errors:[], requests:[], swaps:[]}; reports.push(result);
  try {
   const context=await browser.newContext(options);const page=await context.newPage();
   page.on('console',message=>{
    const entry={level:message.type(),text:message.text()};
    if(['error','warning'].includes(entry.level)||/\bERROR\b/.test(entry.text)) {
     result.console.push(entry);console.log(JSON.stringify({name,...entry}));
    }
   });
   page.on('pageerror',error=>result.errors.push(error.stack));
   page.on('response',response=>{if(response.status()>=400)result.requests.push({status:response.status(),url:response.url()});});
   page.on('requestfailed',request=>result.requests.push({url:request.url(),failure:request.failure()}));
   const response=await page.goto(new URL(path,base).href);
   assert.equal(response.status(),200);
   await page.waitForFunction(()=>document.querySelector('#loading')?.hidden||document.querySelector('#startup-error')?.open,null,{timeout:300000});
   assert.equal(await page.locator('#startup-error').evaluate(e=>e.open?e.textContent:null),null);
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
   assert.deepEqual(result.console,[],`${name}: browser/renderer diagnostics`);
   console.log(JSON.stringify({name,passed:true,swaps:result.swaps}));
  } finally {await browser.close();}
 }
} finally {await fs.writeFile(output,JSON.stringify(reports,null,2)+'\n');}
