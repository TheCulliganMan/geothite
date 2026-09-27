// Regression from an unmodified, real pre-handoff save. This is not a fresh-start
// policy evaluation: the buttons below deliberately isolate the engine boundary.
import {chromium} from 'playwright';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
const checkpoint=process.env.FLYGON_RESUME;
if(!checkpoint)throw Error('Set FLYGON_RESUME to a real save in Elm’s lab, after receiving a starter');
const url=process.env.FLYGON_GAME_URL||'http://127.0.0.1:33127/?multiplayer=off&flygon=1';
const output=process.env.FLYGON_EVIDENCE_DIR||'target/potion-regression';
await fs.mkdir(output,{recursive:true});
const storage=JSON.parse(await fs.readFile(`${checkpoint}/browser-state.json`,'utf8'));
const manifest=JSON.parse(await fs.readFile(`${checkpoint}/manifest.json`,'utf8'));
if(!storage.origins.some(o=>o.localStorage.some(v=>!v.name.endsWith('.bak')&&createHash('sha256').update(Buffer.from(v.value,'base64')).digest('hex')===manifest.gameSha256)))throw Error('Mismatched save fixture');
for(const origin of storage.origins)origin.origin=new URL(url).origin;
const browser=await chromium.launch({headless:true,args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader',...(process.env.FLYGON_CDP_PORT?[`--remote-debugging-port=${process.env.FLYGON_CDP_PORT}`]:[])]});
try{
 const context=await browser.newContext({storageState:storage});const page=await context.newPage();
 page.on('pageerror',e=>console.error('PAGE_ERROR',e.message));
 await page.goto(url);await page.waitForFunction(()=>globalThis.__flygonGameBridge,null,{timeout:180000});
 // Use the game's visible audio control to unlock the browser audio clock.
 await page.locator('#mute').click();
 const trace=[];
 async function command(button){
  let o=await page.evaluate(button=>__flygonGameBridge.execute(button?{kind:'press',button,frames:8}:{kind:'observe'}),button);
  for(let i=0;i<30&&o.flow_state?.animating;i++){
   await page.waitForTimeout(100);o=await page.evaluate(()=>__flygonGameBridge.execute({kind:'observe'}));
  }
  trace.push({button,observation:o});return o;
 }
 let o=await command();
 for(let i=0;i<30&&o.status.screen!=='overworld';i++){
  o=await command(o.status.screen==='title'&&o.observe.text.includes('CONTINUE')?(/^>\s*CONTINUE/m.test(o.observe.text)?'a':'up'):'start');
 }
 if(o.map_info.name!=='ElmsLab'||!o.status.party.length||![4,5].includes(o.map_info.player.x)||o.map_info.player.y>7)throw Error('Fixture must be in the lab’s central aisle before the aide trigger');
 for(let i=0;i<16&&!o.observe.visible_dialogue;i++)o=await command('down');
 for(let i=0;i<60&&(o.observe.visible_dialogue||o.reward_state.scenes.ElmsLab==='SCENE_ELMSLAB_AIDE_GIVES_POTION');i++){
  await page.waitForTimeout(400);o=await command(i%2?'b':'a');
 }
 const receiptSeen=trace.some(t=>/received[\s\S]*POTION/i.test(t.observation.observe.visible_dialogue||''));
 if(!o.observe.visible_dialogue)for(let i=0;i<12&&o.map_info.name==='ElmsLab';i++)o=await command('down');
 await fs.writeFile(`${output}/trace.json`,JSON.stringify(trace,null,2));
 try{await page.screenshot({path:`${output}/final.png`,timeout:5000});}catch(e){console.error('SCREENSHOT_SKIPPED',e.message);}
 const result={fixtureGameSha256:manifest.gameSha256,url,receiptSeen,exitedLab:o.map_info.name==='NewBarkTown',lastMap:o.map_info.name,dialogue:o.observe.visible_dialogue,debug:o.debug_text};
 await fs.writeFile(`${output}/result.json`,JSON.stringify(result,null,2));console.log(JSON.stringify(result));
 if(!result.receiptSeen||!result.exitedLab)process.exitCode=1;
 if(process.env.FLYGON_KEEP_OPEN)await new Promise(resolve=>{process.on('SIGINT',resolve);process.on('SIGTERM',resolve);});
}finally{await browser.close();}
