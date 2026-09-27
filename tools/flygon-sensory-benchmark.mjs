// Compare actual graph responses to recorded observations; no gameplay claims.
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawn} from 'node:child_process';
const [data,configFile,traceFile,out='target/sensory-benchmark']=process.argv.slice(2);
if(!traceFile)throw Error('Usage: node tools/flygon-sensory-benchmark.mjs DATA CONFIG RUNTIME_JSON [OUTPUT]');
await fs.mkdir(out,{recursive:true});
const config=JSON.parse(await fs.readFile(configFile));
const samples=JSON.parse(await fs.readFile(traceFile));
const observations=new Map();
for(const {observation:o} of samples){
 if(!o)continue;
 const key=JSON.stringify([o.status?.screen,o.map_info?.name,!!o.observe?.visible_dialogue,!!o.observe?.menus?.length]);
 if(!observations.has(key))observations.set(key,o);
}
const selected=[...observations].slice(0,12);
await fs.writeFile(path.join(out,'observations.json'),JSON.stringify(selected));
const results=[];
const widths=JSON.parse(process.env.FLYGON_BENCH_WIDTHS||'[256,1024,4096,14080]');
const gains=JSON.parse(process.env.FLYGON_BENCH_GAINS||'[0.02,0.05,0.1]');
const windows=JSON.parse(process.env.FLYGON_BENCH_WINDOWS||'[150]');
for(const width of widths)for(const gain of gains)for(const window of windows){
 const c=structuredClone(config);c.learning=false;c.contact_gain_mv=gain;c.operant.sensory_neurons=width;c.operant.sensory_window_ms=window;
 const file=path.join(out,`config-${width}-${gain}-${window}.json`);await fs.writeFile(file,JSON.stringify(c));
 const operations=selected.flatMap(([,observation])=>[{op:'reset'},{op:'operant_decide',observation}]);
 const child=spawn('target/web-release/flygon-lab',[data,file]);let stdout='',stderr='';
 child.stdout.on('data',c=>stdout+=c);child.stderr.on('data',c=>stderr+=c);child.stdin.end(operations.map(v=>JSON.stringify(v)).join('\n')+'\n');
 const code=await new Promise((resolve,reject)=>{child.on('error',reject);child.on('close',resolve);});
 if(code)throw Error(stderr);
 await fs.writeFile(path.join(out,`responses-${width}-${gain}-${window}.jsonl`),stdout);
 const lines=stdout.trim().split('\n').map(JSON.parse).slice(1).filter(r=>r.result?.sensory);
 if(lines.length!==selected.length)throw Error(`Incomplete experiment ${width}/${gain}`);
 const row={width,gain,window,observations:lines.map((r,i)=>({context:selected[i][0],wall_ms:r.wall_ms,inputs:r.result.sensory.wiring.input_indices.length,driven:r.result.sensory.currents.filter(v=>v[1]>0).length,readoutSpikes:r.result.sensory.readout_activity.reduce((n,v)=>n+v.spikes,0),readoutSignals:r.result.sensory.readout_activity.map(v=>v.signal),telemetry:r.result.telemetry}))};
 results.push(row);await fs.writeFile(path.join(out,'results.json'),JSON.stringify(results,null,2));
 console.log(JSON.stringify({width,gain,window,count:lines.length,mean_ms:row.observations.reduce((n,v)=>n+v.wall_ms,0)/lines.length,readoutSpikes:row.observations.map(v=>v.readoutSpikes)}));
}
