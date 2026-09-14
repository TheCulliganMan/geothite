// Explicit, offline paired-checkpoint migration to one verified game binary.
// Never changes the supervisor's resume pointer or the source checkpoint.
import {chromium} from 'playwright';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import {saveBoundary} from './flygon-runtime-state.mjs';
const [source,url,expectedWasmSha,output,...flags]=process.argv.slice(2);
assert(flags.every(f=>f==='--upgrade-readout'),'Unknown migration option');
const upgradeReadout=flags.includes('--upgrade-readout');
if(!source||!url||!/^[a-f0-9]{64}$/.test(expectedWasmSha||'')||!output)throw Error('Usage: node tools/flygon-migrate-checkpoint.mjs SOURCE NEW_FLYGON_URL EXPECTED_GAME_WASM_SHA NEW_OUTPUT');
await fs.mkdir(output); // Refuse overwriting earlier migration evidence.
const manifest=JSON.parse(await fs.readFile(`${source}/manifest.json`));
const brain=await fs.readFile(`${source}/brain.json`,'utf8');
const game=await fs.readFile(`${source}/game.crystalsave`);
const storage=JSON.parse(await fs.readFile(`${source}/browser-state.json`));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
assert.equal(hash(game),manifest.gameSha256);assert.equal(hash(brain),manifest.brainSha256);
assert(storage.origins.some(o=>o.localStorage.some(v=>v.name.startsWith('crystal.save.')&&!v.name.endsWith('.bak')&&hash(Buffer.from(v.value,'base64'))===manifest.gameSha256)),'paired browser save');
for(const origin of storage.origins)origin.origin=new URL(url).origin;
const browser=await chromium.launch({headless:true,args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
try{
 const page=await (await browser.newContext({storageState:storage})).newPage();
 const trainingConfig=JSON.parse(brain).config;
 if(upgradeReadout){trainingConfig.exponential_current_integration=true;if(trainingConfig.operant)trainingConfig.operant.visible_menu_inputs=true;}
 await fs.writeFile(`${output}/training-config.json`,JSON.stringify(trainingConfig,null,2));
 await page.route('**/flygon-operant.json',route=>route.fulfill({json:trainingConfig}));
 await page.addInitScript(()=>{
  const Original=Worker;
  globalThis.Worker=class extends Original{
   constructor(...args){super(...args);if(String(args[0]).includes('flygon-worker'))globalThis.__flygonNeuralWorker=this;}
  };
 });
 await page.goto(url);
 await page.waitForFunction(()=>!document.querySelector('#run').disabled&&document.querySelector('#game').contentWindow.__flygonGameBridge,null,{timeout:180000});
 const migration=await page.evaluate(async({brain,manifest,expectedWasmSha,upgradeReadout})=>{
  const f=document.querySelector('#game').contentWindow;
  const urls=f.performance.getEntriesByType('resource').map(r=>r.name);
  const wasmUrl=urls.find(n=>/\/crystal-bevy.*\.wasm$/.test(n));
  if(!wasmUrl)throw Error('Missing target engine');
  const digest=async url=>{
   const response=await fetch(url);if(!response.ok)throw Error(`Cannot fetch ${url}`);
   return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',await response.arrayBuffer())),v=>v.toString(16).padStart(2,'0')).join('');
  };
  if(await digest(wasmUrl)!==expectedWasmSha)throw Error('Target engine checksum differs');
  const sourcePacks=manifest.artifacts.filter(a=>new URL(a.url).pathname.endsWith('.crystalpack'));
  if(!sourcePacks.length)throw Error('Source has no pinned content pack');
  for(const pack of sourcePacks){
   const live=urls.find(u=>new URL(u).pathname===new URL(pack.url).pathname);
   if(!live||await digest(live)!==pack.sha256)throw Error('Content pack changed during engine migration');
  }
  let o=await f.__flygonGameBridge.execute({kind:'observe'});
  for(let i=0;i<30&&o.status.screen!=='overworld';i++){
   const isContinue=o.status.screen==='title'&&o.observe.text.includes('CONTINUE');
   const selected=o.observe.text.split('\n').some(line=>/^>\s*CONTINUE/.test(line.trim()));
   o=await f.__flygonGameBridge.execute({kind:'press',button:isContinue?(selected?'a':'up'):'start',frames:8});
  }
  const old=manifest.observation;
  if(o.status.screen!=='overworld'||o.map_info.name!==old.map_info.name||o.map_info.player.x!==old.map_info.player.x||o.map_info.player.y!==old.map_info.player.y)throw Error('Continue did not restore the actual location');
  if(JSON.stringify(o.status.party)!==JSON.stringify(old.status.party))throw Error('Restored party differs');
  return await new Promise((resolve,reject)=>{
   const worker=globalThis.__flygonNeuralWorker,id=crypto.randomUUID();
   const timer=setTimeout(()=>reject(Error('Brain restore timed out')),15000);
   const listener=({data})=>{if(data.id!==id)return;clearTimeout(timer);worker.removeEventListener('message',listener);data.error?reject(Error(data.error)):resolve(data.result?.migration);};
   worker.addEventListener('message',listener);worker.postMessage({id,kind:upgradeReadout?'upgrade_readout':'restore',checkpoint:brain});
  });
 },{brain,manifest,expectedWasmSha,upgradeReadout});
 if(!await saveBoundary(page,output))throw Error('Restored game is not at a safe save boundary');
 const generation=(await fs.readFile(`${output}/latest-checkpoint`,'utf8')).trim();
 const migrated=JSON.parse(await fs.readFile(`${output}/${generation}/brain.json`));
 // The normal worker restore records an intervention and clears transient
 // action/history state. Verify those exact changes, retaining learned values.
 const expected=JSON.parse(brain);
 const reason='restored_checkpoint';
 if(upgradeReadout){
  assert.equal(migration?.policy_reset,true);
  assert.equal(migration?.synaptic_weights_preserved,true);
  expected.interface_id=migration.target_interface;
  expected.config=trainingConfig;
  expected.circuit.heads={};
  expected.circuit.weights.fill(0);expected.circuit.value_weights.fill(0);
  expected.circuit.last_value=0;expected.circuit.last_advantage=0;expected.circuit.last_entropy=0;expected.circuit.context='';
  // Dynamics are intentionally reset for the new integration mode. Persistent
  // weights, ledgers, counters and seed are compared below against the source.
  for(const key of ['voltage','current','adaptation','refractory','external','trace','last_spike','counts','queue','tick','last_window_steps','pulse_ticks','reward_ticks','delivered_dan_spikes','active'])delete expected[key];
 }
 expected.campaign.assisted=true;
 expected.campaign.interventions[reason]=(expected.campaign.interventions[reason]||0)+1;
 expected.campaign.history=[...expected.campaign.history,{kind:'intervention',reason,tick:upgradeReadout?0:expected.tick,config:expected.config}].slice(-64);
 expected.circuit.sensory_history={last:'',movement:'',places:[],repeated:0};
 expected.circuit.exploration_pressure=0;expected.circuit.pending=null;expected.circuit.rollout=[];
 assert.deepEqual(migrated.weights,expected.weights,'synaptic weights unchanged');
 // New default fields are allowed; tolerate only floating-point JSON rounding.
 function preserved(old,current,path='brain'){
  if(old&&typeof old==='object'){
   if(Array.isArray(old))assert.equal(current.length,old.length,path);
   for(const [key,value] of Object.entries(old))preserved(value,current?.[key],`${path}.${key}`);
  }else if(typeof old==='number'&&typeof current==='number'){
   assert(Math.abs(current-old)<=4*Number.EPSILON*Math.max(1,Math.abs(old)),path);
  }else assert.equal(current,old,path);
 }
 preserved(expected,migrated);
 await fs.writeFile(`${output}/migration.json`,JSON.stringify({source,sourceGameSha256:manifest.gameSha256,sourceBrainSha256:manifest.brainSha256,targetWasmSha256:expectedWasmSha,trainingConfig:`${output}/training-config.json`,checkpoint:`${output}/${generation}`,learnedValuesPreserved:!upgradeReadout,synapticWeightsPreserved:true,policyReset:upgradeReadout,migrationVerified:true,restoreEffects:upgradeReadout?migration:'One recorded restore; pending actions, rollout, exploration pressure and transient sensory history reset by normal worker restore.'},null,2));
 console.log(JSON.stringify({checkpoint:`${output}/${generation}`,learnedValuesPreserved:!upgradeReadout,synapticWeightsPreserved:true,policyReset:upgradeReadout,migrationVerified:true}));
}finally{await browser.close();}
