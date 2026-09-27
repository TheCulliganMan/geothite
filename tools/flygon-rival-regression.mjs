// Engine regression using normal buttons and a real pre-rival save. Deliberately
// chooses LEER to exercise the authored loss branch; never a policy evaluation.
import {chromium} from 'playwright';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
const fixture=process.env.FLYGON_RESUME;
if(!fixture)throw Error('FLYGON_RESUME must contain a real pre-rival paired save');
const url=process.env.FLYGON_GAME_URL||'http://127.0.0.1:33131/?multiplayer=off&flygon=1';
const output=process.env.FLYGON_EVIDENCE_DIR||'target/rival-engine-regression';
const manifest=JSON.parse(await fs.readFile(`${fixture}/manifest.json`));
const storage=JSON.parse(await fs.readFile(`${fixture}/browser-state.json`));
const game=await fs.readFile(`${fixture}/game.crystalsave`);
if(createHash('sha256').update(game).digest('hex')!==manifest.gameSha256)throw Error('Fixture checksum mismatch');
if(!storage.origins.some(o=>o.localStorage.some(v=>!v.name.endsWith('.bak')&&createHash('sha256').update(Buffer.from(v.value,'base64')).digest('hex')===manifest.gameSha256)))throw Error('Browser save differs from exported fixture');
for(const o of storage.origins)o.origin=new URL(url).origin;
await fs.mkdir(output,{recursive:true});
const browser=await chromium.launch({headless:true,args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
try{
 const page=await (await browser.newContext({storageState:storage})).newPage();
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(url);await page.waitForFunction(()=>globalThis.__flygonGameBridge,null,{timeout:180000});
 await page.locator('#mute').click();
 const trace=[];
 async function command(button){
  let o=await page.evaluate(button=>__flygonGameBridge.execute(button?{kind:'press',button,frames:8}:{kind:'observe'}),button);
  for(let i=0;i<60&&o.flow_state?.animating&&!o.observe?.menus?.length&&!o.observe?.visible_dialogue&&!o.observe?.battle_message;i++){
   await page.waitForTimeout(50);o=await page.evaluate(()=>__flygonGameBridge.execute({kind:'observe'}));
  }
  trace.push({button,observation:o});
  if(trace.length%20===0){
   await fs.writeFile(`${output}/trace.json`,JSON.stringify(trace,null,2));
   console.log(JSON.stringify({commands:trace.length,map:o.map_info?.name,screen:o.status?.screen,scene:o.reward_state?.scenes?.CherrygroveCity}));
  }
  return o;
 }
 let o=await command();
 for(let i=0;i<30&&o.status.screen!=='overworld';i++)o=await command(o.status.screen==='title'&&o.observe.text.includes('CONTINUE')?(/^>\s*CONTINUE/m.test(o.observe.text)?'a':'up'):'start');
 if(o.map_info.name!=='CherrygroveCity'||o.status.party.length!==1||!o.status.party[0].moves.some(m=>m.name==='LEER'))throw Error('Need the single-starter pre-rival Cherrygrove fixture with LEER');
 let battles=0,inBattle=false,sawCanLose=false,sawFaint=false,finished=false;
 for(let i=0;i<240;i++){
  if(o.status.screen==='battle'){
   if(!inBattle)battles++;inBattle=true;
   sawCanLose ||= o.observe.battle.includes('BATTLETYPE_CANLOSE');
  }else inBattle=false;
  sawFaint ||= o.status.party.every(p=>p.hp===0);
  finished ||= battles>0&&o.reward_state.scenes?.CherrygroveCity==='SCENE_CHERRYGROVECITY_NOOP';
  if(finished&&o.map_info.name==='Route29')break;
  if(battles>1)break;
  let button='a';
  if(o.status.screen==='battle'){
   const entries=o.observe.menus?.[0]?.entries||[];
   const labels=entries.map(s=>s.trim().replace(/^>/,'').trim());
   const selected=entries.findIndex(s=>s.trim().startsWith('>'));
   if(o.observe.battle_message)button='a';
   else if(labels.includes('FIGHT'))button=selected===0?'a':selected%2===1?'left':'up';
   else if(labels.includes('LEER')){const target=labels.indexOf('LEER');button=selected===target?'a':selected>target?'up':'down';}
   else if(entries.length)button='b';
  }else if(o.observe.battle_message||o.observe.visible_dialogue||o.observe.menus?.length)button='a';
  else if(!o.flow_state?.animating){
   const p=o.map_info.player;
   button=p.y<7?'down':p.y>7?'up':'right';
  }else {await page.waitForTimeout(100);button=null;}
  o=await command(button);
 }
 const artifacts=await page.evaluate(async()=>{
  const urls=performance.getEntriesByType('resource').map(r=>r.name).filter(n=>/crystal-bevy.*\.wasm$|\.crystalpack$/.test(n));
  return Promise.all([...new Set(urls)].map(async url=>({url,sha256:Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',await (await fetch(url)).arrayBuffer())),v=>v.toString(16).padStart(2,'0')).join('')})));
 });
 const result={fixtureGameSha256:manifest.gameSha256,artifacts,battles,sawCanLose,sawFaint,sceneFinished:finished,exitedToRoute29:o.map_info.name==='Route29',lastMap:o.map_info.name,errors,scope:'restored-save engine loss-path regression; not neural gameplay'};
 await fs.writeFile(`${output}/trace.json`,JSON.stringify(trace,null,2));await fs.writeFile(`${output}/result.json`,JSON.stringify(result,null,2));
 await page.screenshot({path:`${output}/final.png`,timeout:5000}).catch(e=>console.error('SCREENSHOT_SKIPPED',e.message));
 console.log(JSON.stringify(result));
 if(!sawCanLose||!sawFaint||!finished||!result.exitedToRoute29||errors.length)process.exitCode=1;
}finally{await browser.close();}
