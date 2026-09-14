// Exercise the production sampling loop with deterministic game/save responses.
// Browser startup is excluded; no browser or neural process is launched here.
import {readFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
import test from 'node:test';
const source=await readFile(new URL('./flygon-runtime-watch.mjs',import.meta.url),'utf8');
const loop=source.slice(source.indexOf(' const samples=[], maps=new Set();'),source.indexOf('\n await saveBoundary(page,output).catch'));
assert(loop.includes('for(let i=0;'));
const execute=new (Object.getPrototypeOf(async function(){}).constructor)('page','fs','process','saveBoundary','output','screenshotTimeout','console',loop);
async function run({safeAt=3,finishSafe=true,limit=2,screenshotFails=false,interruptAt=0}={}) {
 let samples=0,saves=0,screenshots=0;const files=new Map(),signals=new Map();
 const page={
  waitForTimeout:async()=>{},
  evaluate:async(_fn,control)=>{
   if(typeof control==='boolean')return;
   samples++;assert(samples<1000,'Sampling loop failed to stop');
   if(samples===interruptAt)signals.get('SIGTERM')();
   return {trace:[{event:{outcome:0}}],phase:'RUNNING',observation:{status:{screen:samples<safeAt?'battle':'overworld'},map_info:{name:'VioletGym',player:{x:5,y:10}}}};
  },
  screenshot:async()=>{screenshots++;if(screenshotFails)throw Error('renderer timeout');},
 };
 const fs={writeFile:async(p,data)=>files.set(p,data),rename:async(a,b)=>{files.set(b,files.get(a));files.delete(a);}};
 const process={env:{FLYGON_WATCH_SAMPLES:String(limit),FLYGON_FINISH_SAFE_BOUNDARY:finishSafe?'1':'0'},on:(signal,fn)=>signals.set(signal,fn)};
 await execute(page,fs,process,async()=>{saves++;return samples>=safeAt;},'/evidence',100,{log(){},error(){}});
 return {samples,saves,screenshots,files};
}
test('scheduled rotation keeps the live battle until a paired save succeeds',async()=>{
 const result=await run({safeAt:5});assert.equal(result.samples,5);assert(result.saves>=4);
});
test('screenshot failure does not force a restart or skip safe completion',async()=>{
 const result=await run({safeAt:4,screenshotFails:true});assert.equal(result.samples,4);assert(result.screenshots>0);
});
test('bounded evaluation retains its observation limit',async()=>{
 assert.equal((await run({finishSafe:false,safeAt:100})).samples,2);

});
test('long unsavable battles archive traces without losing observations',async()=>{
 const {samples,files}=await run({safeAt:725});assert.equal(samples,725);
 const archive=JSON.parse(files.get('/evidence/runtime-segment-000000000.json'));
 const current=JSON.parse(files.get('/evidence/runtime.json'));
 assert.equal(archive.length,360);assert.equal(current.length,365);assert.equal(archive.length+current.length,samples);
});

test('shutdown during battle waits for a paired save even beyond the sample limit',async()=>{
 for(const finishSafe of [true,false]){
  const result=await run({finishSafe,safeAt:6,interruptAt:2,limit:2});
  assert.equal(result.samples,6);assert(result.saves>=5);
 }
});

test('safe-boundary hook pauses only after feedback on an idle overworld',()=>{
 const init=source.split(' await page.addInitScript(()=>{')[1].split('\n });')[0];
 const install=new Function('globalThis','document','performance',init);
 const state={Worker:class {addEventListener(_name,fn){this.listener=fn;}postMessage(){}}};
 let clicks=0;const run={textContent:'Pause',click(){clicks++;this.textContent='Run brain';}};
 install(state,{querySelector:()=>run},{now:()=>0});
 const worker=new state.Worker('flygon-worker.js');
 const idle={status:{screen:'overworld'},flow_state:{animating:false},observe:{menus:[],visible_dialogue:null}};
 state.__flygonPauseAtSafeBoundary=true;
 for(const observation of [ {...idle,status:{screen:'battle'}},{...idle,flow_state:{animating:true}},{...idle,observe:{menus:[{kind:'party'}]}},{...idle,observe:{visible_dialogue:'Hello'}} ]){
  state.__flygonLatestObservation=observation;worker.listener({data:{result:{event:{}}}});
 }
 assert.equal(clicks,0);
 state.__flygonLatestObservation=idle;worker.listener({data:{result:{action:{}}}});assert.equal(clicks,0);
 worker.listener({data:{result:{event:{}}}});assert.equal(clicks,1);assert.equal(state.__flygonBoundaryPaused,true);
 worker.listener({data:{result:{event:{}}}});assert.equal(clicks,1);
});
