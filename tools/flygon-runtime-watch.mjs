// Observe genuine neural decisions and their game outcomes in a private browser.
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import { saveBoundary } from './flygon-runtime-state.mjs';
const output=process.env.FLYGON_EVIDENCE_DIR || 'target/flygon-runtime-watch';
const screenshotTimeout=Number(process.env.FLYGON_SCREENSHOT_TIMEOUT_MS||5000);
if(!Number.isFinite(screenshotTimeout)||screenshotTimeout<1||screenshotTimeout>10000)throw Error('Invalid screenshot timeout');
await fs.mkdir(output,{recursive:true});
const resumeDir=process.env.FLYGON_RESUME;
if(resumeDir && process.env.FLYGON_SEED!==undefined){
 const savedSeed=JSON.parse(await fs.readFile(`${resumeDir}/brain.json`,'utf8')).config.operant.action_seed;
 if(Number(process.env.FLYGON_SEED)!==savedSeed)throw Error(`Resume preserves checkpoint seed ${savedSeed}; FLYGON_SEED cannot override a learned checkpoint`);
}
// The runner owns shutdown: finish feedback and attempt a paired save before
// closing Chromium. Playwright's default signal handler closes it too early.
const browser=await chromium.launch({headless:true,handleSIGINT:false,handleSIGTERM:false,args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader',...(process.env.FLYGON_CDP_PORT?[`--remote-debugging-port=${process.env.FLYGON_CDP_PORT}`]:[])]});
try {
 const context=await browser.newContext({viewport:{width:1440,height:1000},...(resumeDir?{storageState:`${resumeDir}/browser-state.json`}:{})});
 const page=await context.newPage();
 page.on('pageerror',e=>console.error('PAGE_ERROR',e.message));
 page.on('requestfailed',r=>console.error('REQUEST_FAILED',r.url(),r.failure()?.errorText));
 page.on('response',r=>{if(r.status()>=400)console.error('HTTP_ERROR',r.status(),r.url());});
 await page.addInitScript(()=>{
  globalThis.__flygonRuntimeTrace=[];
  const OriginalWorker=globalThis.Worker;
  globalThis.Worker=class extends OriginalWorker {
   postMessage(data,...rest){if(data.kind==='load'&&this.__isFlygonNeural)globalThis.__flygonNeuralWorker=this;if(data.kind==='step')globalThis.__flygonActionPending=true;if(data.observation){globalThis.__flygonLatestObservation=data.observation;globalThis.__flygonNeuralWorker=this;}return super.postMessage(data,...rest);}
   constructor(...args){super(...args);this.__isFlygonNeural=String(args[0]).includes('flygon-worker');this.addEventListener('message',({data})=>{
    const r=data.result;if(r?.event || data.error)globalThis.__flygonActionPending=false;if(!r || (!r.action && !r.event))return;
    const o=globalThis.__flygonLatestObservation;
    if(r.event && globalThis.__flygonPauseAtSafeBoundary && o?.status?.screen==='overworld' && !o.flow_state?.animating && !o.observe?.visible_dialogue && !o.observe?.battle_message && !o.observe?.menus?.length){
     const run=document.querySelector('#run');
     if(run?.textContent==='Pause'){run.click();globalThis.__flygonBoundaryPaused=true;}
    }
    const activity=r.sensory?.readout_activity || [];
    globalThis.__flygonRuntimeTrace.push({time:performance.now(),action:r.action,event:r.event,
     activityNorm:activity.reduce((sum,c)=>sum+c.signal*c.signal,0),
     outputSpikes:activity.reduce((sum,c)=>sum+c.spikes,0),
     danSpikes:r.training?.telemetry?.dan_spikes,changedEdges:r.training?.telemetry?.changed_edges,
     training:r.training?{population:r.training.population,pulse_ms:r.training.pulse_ms}:null,
     sensory:r.sensory?.features,
     game:r.event?{map:globalThis.__flygonLatestObservation?.map_info?.name,player:globalThis.__flygonLatestObservation?.map_info?.player,screen:globalThis.__flygonLatestObservation?.status?.screen}:null,
     breadcrumb:r.breadcrumb,events:r.events});
   });}
  };
 });
 if(process.env.FLYGON_SEED || process.env.FLYGON_CONFIG_OVERRIDE){
  const override=process.env.FLYGON_CONFIG_OVERRIDE?JSON.parse(await fs.readFile(process.env.FLYGON_CONFIG_OVERRIDE,'utf8')):null;
  const seed=Number(process.env.FLYGON_SEED ?? override?.operant?.action_seed ?? 0);
  if(!Number.isSafeInteger(seed)||seed<0)throw Error('Invalid action seed');
  await page.route('**/flygon-operant.json',async route=>{const response=await route.fetch();const config=override || await response.json();config.operant.action_seed=seed;await route.fulfill({response,json:config});});
 }
 const trainingUrl=new URL(process.env.FLYGON_URL || 'http://127.0.0.1:3003/flygon');
 if(process.env.FLYGON_RENDER_BRAIN!=='1')trainingUrl.searchParams.set('headless','1');
 await page.goto(trainingUrl.href);
 await page.waitForFunction(()=>!document.querySelector('#run').disabled && document.querySelector('#game').contentWindow.__flygonGameBridge,null,{timeout:180000});
 if(resumeDir){
  const manifest=JSON.parse(await fs.readFile(`${resumeDir}/manifest.json`,'utf8'));
  const brain=await fs.readFile(`${resumeDir}/brain.json`,'utf8');
  const game=await fs.readFile(`${resumeDir}/game.crystalsave`);
  const storage=JSON.parse(await fs.readFile(`${resumeDir}/browser-state.json`,'utf8'));
  if(!storage.origins?.some(o=>o.localStorage?.some(v=>v.name.startsWith('crystal.save.')&&!v.name.endsWith('.bak')&&createHash('sha256').update(Buffer.from(v.value,'base64')).digest('hex')===manifest.gameSha256)))throw Error('Browser save does not match exported game checkpoint');
  if(createHash('sha256').update(brain).digest('hex')!==manifest.brainSha256||createHash('sha256').update(game).digest('hex')!==manifest.gameSha256)throw Error('Checkpoint pair checksum mismatch');
  await page.evaluate(async({brain,manifest})=>{
   const f=document.querySelector('#game').contentWindow;
   const live=f.performance.getEntriesByType('resource').map(r=>r.name).find(n=>/\/crystal-bevy(?:-[a-f0-9]+)?\.js(?:\?|$)/.test(n));
   if(new URL(live).pathname!==new URL(manifest.gameModule).pathname)throw Error('Pinned game binary differs from checkpoint');
   for(const artifact of manifest.artifacts||[]){
    const response=await fetch(new URL(new URL(artifact.url).pathname,live));
    if(!response.ok)throw Error('Pinned game artifact unavailable');
    const sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',await response.arrayBuffer())),b=>b.toString(16).padStart(2,'0')).join('');
    if(sha256!==artifact.sha256)throw Error('Pinned game artifact checksum mismatch');
   }
   const bridge=f.__flygonGameBridge;
   let o=await bridge.execute({kind:'observe'});
   // Normal Continue-menu inputs only; these restore the saved run, not a fresh-start trial.
   for(let i=0;i<30&&o.status.screen!=='overworld';i++){
    if(o.status.screen==='title'&&o.observe.text.includes('CONTINUE')){
     if(!o.observe.text.split('\n').some(line=>/^>\s*CONTINUE/.test(line.trim()))) {o=await bridge.execute({kind:'press',button:'up',frames:8});continue;}
     o=await bridge.execute({kind:'press',button:'a',frames:8});
    }else o=await bridge.execute({kind:'press',button:'start',frames:8});
   }
   if(o.status.screen!=='overworld'||o.map_info.name!==manifest.observation.map_info.name||o.map_info.player.x!==manifest.observation.map_info.player.x||o.map_info.player.y!==manifest.observation.map_info.player.y)throw Error('Continue did not restore checkpoint location');
   const worker=globalThis.__flygonNeuralWorker;
   if(!worker)throw Error('Missing neural worker');
   await new Promise((resolve,reject)=>{
    const id=crypto.randomUUID(),timer=setTimeout(()=>reject(Error('Restore timed out')),15000);
    const listener=({data})=>{if(data.id!==id)return;clearTimeout(timer);worker.removeEventListener('message',listener);data.error?reject(Error(data.error)):resolve();};
    worker.addEventListener('message',listener);worker.postMessage({id,kind:'restore',checkpoint:brain});
   });
  },{brain,manifest});
  console.log(JSON.stringify({resumed:resumeDir,gameModule:manifest.gameModule}));
 }
 await page.check('#play');
 if(!(await page.isChecked('#learning')) || !(await page.isChecked('#auto-reward')))throw Error('Training is not enabled');
 await page.click('#run');
 const samples=[], maps=new Set();
 const sampleLimit=Number(process.env.FLYGON_WATCH_SAMPLES || 360);
 const finishSafe=process.env.FLYGON_FINISH_SAFE_BOUNDARY==='1';
 let archivedSamples=0;
 const stopMap=process.env.FLYGON_STOP_MAP;
 let stopMapObserved=false;
 let lastOutcome=Date.now();
 let shutdownRequested=false;let stopping=false;process.on('SIGINT',()=>{shutdownRequested=true;});process.on('SIGTERM',()=>{shutdownRequested=true;});
 for(let i=0;finishSafe || shutdownRequested || i<sampleLimit;i++){
  await page.waitForTimeout(10000);
  const sample=await page.evaluate(async()=>({
   trace:globalThis.__flygonRuntimeTrace.splice(0),
   observation:globalThis.__flygonLatestObservation,
   phase:document.querySelector('#phase').textContent
  }));
  if(sample.trace.some(r=>r.event))lastOutcome=Date.now();
  if(Date.now()-lastOutcome>300000)throw Error('No action outcomes for five minutes');
  samples.push(sample);
  if(stopMap && sample.observation?.map_info?.name===stopMap)stopMapObserved=true;
  // Long battles must retain their live brain without unbounded trace memory.
  // Preserve older evidence in numbered segments before trimming the live window.
  if(finishSafe && samples.length>720){
   const segment=samples.slice(0,360);
   await fs.writeFile(`${output}/runtime-segment-${String(archivedSamples).padStart(9,'0')}.json`,JSON.stringify(segment));
   samples.splice(0,360);archivedSamples+=360;
  }
  await fs.writeFile(`${output}/runtime.next.json`,JSON.stringify(samples,null,2));
  await fs.rename(`${output}/runtime.next.json`,`${output}/runtime.json`);
  console.log(JSON.stringify({sample:i,phase:sample.phase,screen:sample.observation.status.screen,map:sample.observation.map_info.name,player:sample.observation.map_info.player,
   decisions:sample.trace.filter(r=>r.action).map(r=>({button:r.action.button,p:r.action.readouts.map(a=>+a.probability.toFixed(4)),norm:r.activityNorm,spikes:r.outputSpikes})),
   outcomes:sample.trace.filter(r=>r.event).map(r=>({event:r.event,dan:r.danSpikes,plastic:r.changedEdges}))}));
  if(shutdownRequested || stopMapObserved || (finishSafe && i+1>=sampleLimit)) {
   await page.evaluate(requested=>{globalThis.__flygonPauseAtSafeBoundary=requested;},true);
  }
  if(i%6===0 || !maps.has(sample.observation?.map_info?.name) || shutdownRequested || stopMapObserved || (finishSafe && i+1>=sampleLimit)) {
   const saved=await saveBoundary(page,output).catch(e=>{console.error('SAVE_DEFERRED',e.message);return false;});
   if(!saved)await page.evaluate(()=>{
    if(globalThis.__flygonBoundaryPaused){
     globalThis.__flygonBoundaryPaused=false;
     const run=document.querySelector('#run');if(run?.textContent==='Run brain')run.click();
    }
   },false);
   if(shutdownRequested && saved){console.log(JSON.stringify({shutdown:true,pairedSave:true,samples:i+1,archivedSamples}));stopping=true;}
   if(finishSafe && i+1>=sampleLimit && saved){console.log(JSON.stringify({sessionRotation:true,pairedSave:true,samples:i+1,archivedSamples}));stopping=true;}
   if(stopMapObserved && saved){console.log(JSON.stringify({stopMap,observed:true,pairedSave:true}));stopping=true;}
  }
  if(stopping)break;
  // A renderer stall must not destroy an otherwise live learning session.
  await page.screenshot({path:`${output}/latest.png`,timeout:screenshotTimeout}).catch(e=>console.error('SCREENSHOT_SKIPPED',e.message));
  const map=sample.observation.map_info.name;
  if(!maps.has(map)) {maps.add(map);await page.screenshot({path:`${output}/arrival-${map.replace(/[^a-zA-Z0-9_-]/g,'_')}.png`,timeout:screenshotTimeout}).catch(e=>console.error('SCREENSHOT_SKIPPED',e.message));}
 }
 await saveBoundary(page,output).catch(e=>console.error('SAVE_DEFERRED',e.message));
 if(await page.locator('#run').textContent()==='Pause')await page.click('#run');
}finally{await browser.close();}
