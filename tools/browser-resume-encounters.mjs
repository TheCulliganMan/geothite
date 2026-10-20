// Import an external disposable Route29 save, then prove browser reload encounters.
import {chromium} from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const [base, save, output] = process.argv.slice(2);
assert.ok(base&&save&&output,'usage: node tools/browser-resume-encounters.mjs BASE_URL SAVE_FILE REPORT_JSON');
const browser=await chromium.launch({headless:true,args:process.platform==='darwin'?['--use-angle=metal','--mute-audio']:['--use-angle=swiftshader','--enable-unsafe-swiftshader','--mute-audio']});
const report={base,errors:[],console:[],steps:[]};
let page;
try {
 const context=await browser.newContext({viewport:{width:1280,height:900}});
 page=await context.newPage();
 page.on('pageerror',e=>report.errors.push(e.message));
 page.on('console',m=>{if(['error','warning'].includes(m.type())||/\bERROR\b/.test(m.text()))report.console.push(m.text());});
 const ready=async()=>{
  await page.waitForFunction(()=>document.querySelector('#loading')?.hidden||document.querySelector('#startup-error')?.open,null,{timeout:300000});
  assert.equal(await page.locator('#startup-error').evaluate(e=>e.open?e.textContent:null),null);
 };
 await page.goto(new URL('/?view=2d',base).href);await ready();console.log('Initial browser ready');
 await page.locator('#player-options > summary').click();await page.locator('#save-games').click();
 await page.waitForFunction(()=>!document.querySelector('#save-games-dialog input[type=file]').disabled);
 await page.locator('#save-games-dialog input[type=file]').setInputFiles(save);
 await page.locator('#save-games-dialog [data-confirmation]').waitFor({state:'visible'});
 await Promise.all([page.waitForEvent('load'),page.locator('#save-games-dialog [data-confirm]').click()]);
 await ready();console.log('Import reload ready');
 await page.reload();await ready();console.log('Browser reload ready'); // actual automatic BevyShellStart::LoadSave path
 await page.evaluate(async()=>{
  const wasm=await import(new URL(__crystalRendererBundle.module,document.baseURI));
  const {createGameBridge}=await import('/webmcp.js');window.resumeBridge=createGameBridge(wasm,{timeoutMs:60000});
 });
 const observe=()=>page.evaluate(()=>resumeBridge.execute({kind:'observe'}));
 let state=await observe();report.resumed=state;
 assert.equal(state.map_info.name,'Route29');assert.ok(state.status.party.length,'starter survives browser reload');
 const start=state.map_info.player;
 const forward=start.facing.toLowerCase();const reverse={up:'down',down:'up',left:'right',right:'left'}[forward];
 const keys={up:'ArrowUp',down:'ArrowDown',left:'ArrowLeft',right:'ArrowRight'};
 for(let i=0;state.status.screen!=='battle'&&i<160;i++){
  const atStart=state.map_info.player.x===start.x&&state.map_info.player.y===start.y;
  const key=keys[atStart?forward:reverse];
  await page.locator('#crystal-canvas').focus();
  await page.keyboard.down(key);await page.waitForTimeout(150);await page.keyboard.up(key);await page.waitForTimeout(100);
  state=await observe();
  report.steps.push({position:state.map_info.player,screen:state.status.screen,events:state.recent_events});
  if(i%10===0)console.log(JSON.stringify({step:i,position:state.map_info.player,screen:state.status.screen}));
 }
 assert.equal(state.status.screen,'battle','resumed Route29 grass must start a real wild battle');
 await page.waitForTimeout(5000);state=await observe();
 report.battle=state;assert.ok(state.observe.battle);
 console.log(JSON.stringify({presentation:state.observe.rendered_text,menus:state.observe.menus,message:state.observe.battle_message}));
 assert.deepEqual(report.errors,[]);assert.deepEqual(report.console,[]);
 await page.screenshot({path:output.replace(/\.json$/,'-battle.png')});
 console.log(JSON.stringify({base,resumed:report.resumed.map_info.name,steps:report.steps.length,battle:true}));
}catch(error){report.failure=String(error);report.visible=await page?.locator('body').innerText().catch(()=>null);if(page)await page.screenshot({path:output.replace(/\.json$/,'-failure.png')}).catch(()=>{});throw error;}finally{await fs.writeFile(output,JSON.stringify(report,null,2)+'\n');await browser.close();}
