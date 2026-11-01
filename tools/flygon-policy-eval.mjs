// Cold-start gameplay evidence. All Game Boy inputs come from the neural worker.
// Run on the homelab; use a private origin and fresh browser context per trial.
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
const budget = Number(process.env.FLYGON_EVAL_DECISIONS || 200);
if (!Number.isInteger(budget) || budget < 20 || budget > 2000) throw Error('Invalid decision budget');
const output = process.env.FLYGON_EVIDENCE_DIR || 'target/flygon-policy-eval';
await fs.mkdir(output, { recursive: true });
const browser = await chromium.launch({headless:true,args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']});
try {
  const page = await browser.newPage({viewport:{width:1440,height:1000}});
  const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  await page.addInitScript(({worker,budget})=>{
    globalThis.__policyEvidence={decisions:0,trace:[],observations:[]};
    const OriginalWorker=globalThis.Worker;
    globalThis.Worker=class extends OriginalWorker {
      constructor(url,...rest){
        super(worker && String(url).endsWith('/flygon-worker.js') ? worker : url,...rest);
        this.addEventListener('message',({data})=>{
          const r=data.result;if(!r)return;
          const evidence=globalThis.__policyEvidence;
          if(r.action){
            evidence.decisions++;
            evidence.trace.push({action:r.action,sensory:r.sensory?.encoding,
              neuralNorm:r.sensory?.readout_activity?.reduce((n,c)=>n+c.signal*c.signal,0)});
          }
          if(r.event)evidence.trace.push({event:r.event,breadcrumb:r.breadcrumb});
          // Pause at the same decision budget, without supplying game inputs.
          if(r.event && evidence.decisions>=budget && document.querySelector('#run')?.textContent==='Pause')
            document.querySelector('#run').click();
        });
      }
      postMessage(data,...rest){
        if(data.observation && data.kind==='step') {
          const v=data.observation;
          globalThis.__policyEvidence.observations.push({screen:v.status?.screen,map:v.map_info?.name,
            player:v.map_info?.player,party:v.status?.party,flags:v.reward_state?.event_flags,engineFlags:v.reward_state?.engine_flags,menus:v.observe?.menus,text:v.observe?.text,dialogue:v.observe?.visible_dialogue});
        }
        return super.postMessage(data,...rest);
      }
    };
  },{worker:process.env.FLYGON_EVAL_WORKER || null,budget});
  await page.goto(process.env.FLYGON_URL || 'http://127.0.0.1:33217/flygon');
  await page.waitForFunction(()=>!document.querySelector('#run').disabled && document.querySelector('#game').contentWindow.__flygonGameBridge,null,{timeout:180000});
  await page.click('#run');
  const progress = setInterval(async()=>{
    try {
      const sample=await page.evaluate(()=>({decisions:globalThis.__policyEvidence.decisions,last:globalThis.__policyEvidence.observations.at(-1),phase:document.querySelector('#phase').textContent}));
      console.log(JSON.stringify({progress:sample.decisions,screen:sample.last?.screen,map:sample.last?.map,player:sample.last?.player,phase:sample.phase}));
      await fs.writeFile(`${output}/progress.json`,JSON.stringify(sample));
    } catch {}
  },20000);
  try { await page.waitForFunction(()=>globalThis.__policyEvidence.decisions>=2000 || document.querySelector('#run').textContent!=='Pause',null,{timeout:900000}); } finally {clearInterval(progress);}
  await page.waitForTimeout(1000);
  const result=await page.evaluate(async()=>({
    ...globalThis.__policyEvidence,
    observation:await document.querySelector('#game').contentWindow.__flygonGameBridge.execute({kind:'observe'}),
    phase:document.querySelector('#phase').textContent,
  }));
  result.errors=errors;result.budget=budget;
  await page.screenshot({path:`${output}/gameplay.png`});
  if(process.env.FLYGON_EVAL_CHECKPOINT==='1') {
    await page.locator('.advanced > details > summary').click();
    const pendingDownload=page.waitForEvent('download');
    await page.click('#save-brain');
    await (await pendingDownload).saveAs(`${output}/brain.json`);
  }
  await fs.writeFile(`${output}/gameplay.json`,JSON.stringify(result,null,2));
  const events=result.trace.filter(r=>r.event).map(r=>r.event);
  console.log(JSON.stringify({decisions:result.decisions,map:result.observation.map_info.name,
    party:result.observation.status.party,flags:result.observation.reward_state.event_flags,
    positive:events.filter(e=>e.outcome>0).length,negative:events.filter(e=>e.outcome<0).length,
    phase:result.phase,errors}));
  if(errors.length || result.decisions<budget || result.phase.includes('ERROR')) throw Error('Gameplay evaluation failed');
} finally {await browser.close();}
