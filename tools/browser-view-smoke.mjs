// Real renderer swaps: one WASM engine, live input, menus and authored dialogue.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { chromium, devices } from 'playwright';
const [base, output] = process.argv.slice(2);
assert.ok(base && output, 'usage: node tools/browser-view-smoke.mjs BASE_URL REPORT_JSON');
const browser=await chromium.launch({headless:true,args:process.platform==='darwin'?['--use-angle=metal','--mute-audio']:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader','--mute-audio']});
const report={base,errors:[],console:[],failedRequests:[],swaps:[]};
const inspectPage=page=>{
 let collecting=true;
 page.on('pageerror',e=>{if(collecting)report.errors.push(e.message);});
 page.on('console',message=>{if(collecting&&(['error','warning'].includes(message.type())||/\bERROR\b/.test(message.text())))report.console.push({level:message.type(),text:message.text()});});
 page.on('response',response=>{if(collecting&&response.status()>=400)report.failedRequests.push({status:response.status(),url:response.url()});});
 page.on('requestfailed',request=>{if(collecting)report.failedRequests.push({url:request.url(),failure:request.failure()});});
 return ()=>{collecting=false;};
};
try {
 const context=await browser.newContext({viewport:{width:1280,height:900},reducedMotion:'reduce'});
 const page=await context.newPage();const stopInspection=inspectPage(page);
 await page.goto(new URL('/tui/?multiplayer=off',base).href);
 await page.waitForFunction(()=>document.querySelector('#loading').hidden||document.querySelector('#startup-error').open,null,{timeout:240000});
 assert.equal(await page.locator('#startup-error').evaluate(e=>e.open?e.textContent:null),null);
 assert.equal(await page.locator('#view-navigation [aria-current]').textContent(),'TUI');
 await page.evaluate(async()=>{
  const wasm=await import(new URL(__crystalRendererBundle.module,document.baseURI));
  window.viewTestEngine=await wasm.default(); window.viewTestWasm=wasm;
  const {createGameBridge}=await import('/webmcp.js');window.viewTestBridge=createGameBridge(wasm,{timeoutMs:60000});
  window.viewTestMarker=crypto.randomUUID();
 });
 const marker=await page.evaluate(()=>viewTestMarker);
 const observe=()=>page.evaluate(()=>viewTestBridge.execute({kind:'observe'}));
 let state=await observe(); console.log("Started",JSON.stringify(state.status));
 const press=async button=>{
  state=await page.evaluate(({button,frames})=>viewTestBridge.execute({kind:'press',button,frames}),{button,frames:state.status.screen==='naming'?1:8});
  for(let i=0;state.flow_state?.animating&&!state.observe.visible_dialogue&&!state.observe.menus?.length&&i<180;i++){
   await page.waitForTimeout(50);state=await observe();
  }
  console.log("Input",button,JSON.stringify({status:state.status,position:state.map_info.player,map:state.map_info.name,menus:state.observe.menus,text:state.observe.text,flow:state.flow_state}));
  return state;
 };
 const stable=s=>({name:s.status.player_name,screen:s.status.screen,position:s.map_info.player,map:s.map_info.name,menus:s.observe.menus,dialogue:s.observe.visible_dialogue,battle:s.observe.battle});
 const swap=async(mode,{preserve=true}={})=>{
  const before=await observe();
  const choice=page.locator(`#view-navigation [data-view="${mode}"]`);
  if(mode==='tui'){await choice.focus();await page.keyboard.press('Enter');}
  else await choice.click();
  assert.equal(new URL(page.url()).pathname,'/tui/');
  assert.equal(await page.evaluate(()=>viewTestMarker),marker);
  assert.ok(await page.evaluate(async()=>viewTestEngine===await viewTestWasm.default()),'view swap restarted WASM');
  assert.equal(await page.locator('#view-navigation [aria-current]').textContent(),mode==='tui'?'TUI':mode.toUpperCase());
  const after=await observe();
  if(preserve)assert.deepEqual(stable(after),stable(before));
  report.swaps.push({mode,before:stable(before),after:stable(after)});
  state=after;
 };
 await page.locator('#mute').click();
 await page.waitForFunction(()=>document.querySelector('#mute').dataset.audible==='true',null,{timeout:120000});
 for(let i=0;state.status.screen==='intro'&&i<30;i++)await press('a');
 for(let i=0;state.status.screen==='title'&&!/NEW GAME/.test(state.observe.text)&&i<30;i++)await press('start');
 for(let i=0;state.status.screen!=='overworld'&&i<240;i++){
  const menu=state.observe.menus?.find(m=>m.kind==='name_choices');
  if(menu){const target=menu.options.findIndex(x=>/CHRIS/.test(x));assert.ok(target>=0);await press(menu.selected===target?'a':menu.selected<target?'down':'up');}
  else await press('a');
 }
 assert.equal(state.map_info.name,'PlayersHouse2F');assert.equal(state.status.screen,'overworld');
 report.initial=stable(state);
 await swap('2d');await page.screenshot({path:output.replace(/\.json$/,'-2d.png')});
 await swap('3d');await page.waitForTimeout(3000);await page.screenshot({path:output.replace(/\.json$/,'-3d.png')});
 await swap('tui');
 await page.waitForFunction(()=>document.querySelector('#tui-screen').textContent.includes('PlayersHouse2F'),null,{timeout:60000});
 await page.screenshot({path:output.replace(/\.json$/,'-tui.png')});
 await page.locator('#tui-style-toggle').click();await page.locator('#tui-style-toggle').click();
 for(let i=0;!state.observe.menus?.length&&i<4;i++)await press('start');
 assert.ok(state.observe.menus?.length);
 await swap('2d');await swap('tui');
 await page.waitForFunction(()=>document.querySelector('#tui-screen').textContent.includes('PACK'),null,{timeout:60000});
 await press('down');const selected=stable(state);await swap('3d');assert.deepEqual(stable(state),selected);
 await swap('tui');await press('b');
 for(let i=0;state.map_info.player.x<7&&i<64;i++)await press('right');
 assert.equal(state.map_info.player.x,7);
 for(let i=0;state.map_info.name==='PlayersHouse2F'&&i<64;i++)await press('up');
 assert.equal(state.map_info.name,'PlayersHouse1F');
 for(let i=0;!state.observe.visible_dialogue&&i<64;i++)await press('down');
 assert.ok(state.observe.visible_dialogue,'Mom must intercept through the shared production controller');
 await page.waitForTimeout(3000);state=await observe();
 const motherPosition=state.map_info.player;
 await press('right');assert.deepEqual(state.map_info.player,motherPosition);
 await swap('2d');await swap('tui');
 assert.ok(state.observe.visible_dialogue);report.mom=stable(state);
 const first=state.observe.visible_dialogue;await press('a');
 assert.notEqual(state.observe.visible_dialogue,first,'A must advance the actual dialogue');
 assert.deepEqual(report.errors,[]);
 assert.deepEqual(report.console,[]);
 assert.deepEqual(report.failedRequests,[]);
 report.navigationCount=await page.evaluate(()=>performance.getEntriesByType('navigation').length);
 assert.equal(report.navigationCount,1);
 stopInspection();await context.close();
 const phone=await browser.newContext({...devices['iPhone 13'],reducedMotion:'reduce'});
 const mobile=await phone.newPage();const stopMobileInspection=inspectPage(mobile);
 await mobile.goto(new URL('/?view=tui&multiplayer=off',base).href);
 await mobile.waitForFunction(()=>document.querySelector('#loading').hidden||document.querySelector('#startup-error').open,null,{timeout:240000});
 assert.equal(await mobile.locator('#startup-error').evaluate(e=>e.open?e.textContent:null),null);
 assert.ok(await mobile.locator('#tui-surface').isVisible());
 assert.equal(await mobile.evaluate(()=>document.elementFromPoint(innerWidth/2,innerHeight/3)?.id),'crystal-canvas','TUI presentation must preserve the production touch input target');
 for(const mode of ['2d','3d','tui'])assert.ok(await mobile.locator(`#view-navigation [data-view="${mode}"]`).isVisible());
 await mobile.screenshot({path:output.replace(/\.json$/,'-phone.png')});
 assert.deepEqual(report.errors,[]);assert.deepEqual(report.console,[]);assert.deepEqual(report.failedRequests,[]);
 stopMobileInspection();await phone.close();
 console.log(JSON.stringify({base,swaps:report.swaps.length,navigations:report.navigationCount,errors:report.errors}));
}finally{await fs.writeFile(output,JSON.stringify(report,null,2));await browser.close();}
