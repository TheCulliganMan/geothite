// Observe genuine neural decisions and their game outcomes in a private browser.
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
const output=process.env.FLYGON_EVIDENCE_DIR || 'target/flygon-runtime-watch';
await fs.mkdir(output,{recursive:true});
const browser=await chromium.launch({headless:true,args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader',...(process.env.FLYGON_CDP_PORT?[`--remote-debugging-port=${process.env.FLYGON_CDP_PORT}`]:[])]});
try {
 const page=await browser.newPage({viewport:{width:1440,height:1000}});
 page.on('pageerror',e=>console.error('PAGE_ERROR',e.message));
 await page.addInitScript(()=>{
  globalThis.__flygonRuntimeTrace=[];
  const OriginalWorker=globalThis.Worker;
  globalThis.Worker=class extends OriginalWorker {
   postMessage(data,...rest){if(data.observation){globalThis.__flygonLatestObservation=data.observation;globalThis.__flygonNeuralWorker=this;}return super.postMessage(data,...rest);}
   constructor(...args){super(...args);this.addEventListener('message',({data})=>{
    const r=data.result;if(!r || (!r.action && !r.event))return;
    const activity=r.sensory?.readout_activity || [];
    globalThis.__flygonRuntimeTrace.push({time:performance.now(),action:r.action,event:r.event,
     activityNorm:activity.reduce((sum,c)=>sum+c.signal*c.signal,0),
     outputSpikes:activity.reduce((sum,c)=>sum+c.spikes,0),
     danSpikes:r.training?.telemetry?.dan_spikes,changedEdges:r.training?.telemetry?.changed_edges,
     training:r.training?{population:r.training.population,pulse_ms:r.training.pulse_ms}:null,
     breadcrumb:r.breadcrumb,events:r.events});
   });}
  };
 });
 await page.goto(process.env.FLYGON_URL || 'http://127.0.0.1:3003/flygon');
 await page.waitForFunction(()=>!document.querySelector('#run').disabled && document.querySelector('#game').contentWindow.__flygonGameBridge,null,{timeout:180000});
 await page.check('#play');await page.click('#run');
 const samples=[];
 for(let i=0;i<Number(process.env.FLYGON_WATCH_SAMPLES || 6);i++){
  await page.waitForTimeout(10000);
  const sample=await page.evaluate(async()=>({
   trace:globalThis.__flygonRuntimeTrace.splice(0),
   observation:globalThis.__flygonLatestObservation,
   phase:document.querySelector('#phase').textContent
  }));
  samples.push(sample);
  await fs.writeFile(`${output}/runtime.json`,JSON.stringify(samples,null,2));
  console.log(JSON.stringify({sample:i,phase:sample.phase,screen:sample.observation.status.screen,map:sample.observation.map_info.name,player:sample.observation.map_info.player,
   decisions:sample.trace.filter(r=>r.action).map(r=>({button:r.action.button,p:r.action.readouts.map(a=>+a.probability.toFixed(4)),norm:r.activityNorm,spikes:r.outputSpikes})),
   outcomes:sample.trace.filter(r=>r.event).map(r=>({event:r.event,dan:r.danSpikes,plastic:r.changedEdges}))}));
  await page.screenshot({path:`${output}/latest.png`});
 }
 if(await page.locator('#run').textContent()==='Pause')await page.click('#run');
}finally{await browser.close();}
