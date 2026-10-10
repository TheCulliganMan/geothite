/** Real WASM/GL check for the ignored converted Kenney people in Goldenrod.
 * node tools/open-people-browser-smoke.mjs URL OUTPUT_DIRECTORY
 * Build with location-tester and supply open-people.json. Uses normal keyboard
 * and production GameBridge inputs. Captures require visual review; SwiftShader
 * is a software renderer, so this is not a hardware performance benchmark.
 */
import { chromium } from 'playwright';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
const [url, output] = process.argv.slice(2);
assert.ok(url && output, 'usage: open-people-browser-smoke.mjs URL OUTPUT_DIRECTORY');
assert.equal(new URL(url).searchParams.get('multiplayer'), 'off');
assert.equal(new URL(url).searchParams.get('preview'), 'goldenrod');
await fs.mkdir(output, {recursive:true});
const executablePath=process.env.CHROME_BIN || (process.platform==='darwin' ? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' : '/usr/bin/chromium');
const browser=await chromium.launch({executablePath,headless:true,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader']});
const report={renderer:'Chrome / ANGLE SwiftShader (software)',errors:[],warnings:[],responses:[]};
try {
 const page=await browser.newPage({viewport:{width:1280,height:1000}});
 page.on('pageerror',e=>report.errors.push(String(e)));
 page.on('console',m=>{if(m.type()==='error')report.errors.push(m.text());if(m.type()==='warning')report.warnings.push(m.text());});
 page.on('response',r=>{if(r.url().includes('open-people.json'))report.responses.push({status:r.status(),headers:r.headers()});});
 await page.goto(url,{waitUntil:'domcontentloaded',timeout:180000});
 await page.waitForFunction(()=>document.querySelector('#loading')?.hidden || document.querySelector('#startup-error')?.open,null,{timeout:180000});
 assert.equal(await page.locator('#startup-error').evaluate(e=>e.open?e.textContent:null),null);
 await page.evaluate(async()=>{const scripts=[...document.scripts].map(s=>s.textContent).join('\n');const bundle=scripts.match(/import\(['"](.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)['"]\)/)?.[1];if(!bundle)throw new Error('Missing production WASM import');const wasm=await import(bundle);await wasm.default();const {createGameBridge}=await import('./webmcp.js');globalThis.peopleQA=createGameBridge(wasm,{timeoutMs:120000});});
 const observe=()=>page.evaluate(()=>peopleQA.execute({kind:'observe'}));
 report.before=await observe();
 assert.equal(report.before.map_info.name,'GoldenrodCity');
 assert.equal(report.before.map_info.player.x,20);assert.equal(report.before.map_info.player.y,18);
 const toggle=page.locator('#view-toggle');
 if(await toggle.getAttribute('aria-pressed')!=='true') {await page.locator('#player-options > summary').click();await toggle.click();await page.locator('#player-options > summary').click();}
 assert.equal(await toggle.getAttribute('aria-pressed'),'true');report.modeledView=true;
 await page.waitForTimeout(10000);
 await page.locator('#crystal-canvas').screenshot({path:`${output}/goldenrod.png`});
 await page.locator('#crystal-canvas').focus();await page.keyboard.down('ArrowDown');await page.waitForTimeout(900);await page.keyboard.up('ArrowDown');await page.waitForTimeout(1000);
 report.after=await observe();assert.ok(report.after.map_info.player.y>18,'Actual DOM keyboard must move the production player');
 await page.evaluate(()=>peopleQA.execute({kind:'press',button:'start',frames:1}));
 report.menu=await observe();assert.ok(report.menu.observe.menus.some(m=>m.kind==='start'),'Production Start must open');
 assert.deepEqual(report.menu.map_info.player,report.after.map_info.player);
 await page.locator('#crystal-canvas').screenshot({path:`${output}/start.png`});
 await page.evaluate(()=>peopleQA.execute({kind:'press',button:'b',frames:1}));
 report.closed=await observe();assert.ok(!report.closed.observe.menus.some(m=>m.kind==='start'));
 assert.ok(report.responses.some(r=>r.status===200));
 assert.ok(!report.warnings.some(w=>/using authored people|invalid person|unsupported person/.test(w)));
 assert.deepEqual(report.errors,[]);
 report.status='passed';
 console.log(JSON.stringify({status:report.status,modeledView:report.modeledView,before:report.before.map_info.player,after:report.after.map_info.player,menus:report.menu.observe.menus,errors:report.errors}));
} catch(e) {report.status='failed';report.failure=String(e);throw e;}
finally {await fs.writeFile(`${output}/report.json`,JSON.stringify(report,null,2));await browser.close();}
