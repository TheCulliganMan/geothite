// Paired, atomic checkpoint generations. Uses the running game's own save API.
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
export async function saveBoundary(page, output) {
 const resume=(await page.locator('#run').textContent())==='Pause';
 if(resume)await page.click('#run');
 try {
  await page.waitForFunction(()=>!globalThis.__flygonActionPending,null,{timeout:15000});
  const frame=page.frames().find(f=>f!==page.mainFrame()&&f.url().includes('flygon=1'));
  if(!frame)throw Error('Missing game frame');
  const saved=await frame.evaluate(async()=>{
   const o=await globalThis.__flygonGameBridge.execute({kind:'observe'});
   if(o.status.screen!=='overworld'||o.flow_state?.animating||o.observe?.visible_dialogue||o.observe?.battle_message||o.observe?.menus?.length)
    return null;
   const moduleUrl=performance.getEntriesByType('resource').map(r=>r.name).find(n=>/\/crystal-bevy(?:-[a-f0-9]+)?\.js(?:\?|$)/.test(n));
   if(!moduleUrl)throw Error('Cannot identify the live game module');
   const resources=performance.getEntriesByType('resource').map(r=>r.name);
   const pinnedUrls=resources.filter(n=>/\/crystal-bevy(?:-[a-f0-9]+)?(?:_bg)?\.wasm(?:\?|$)/.test(n)||/\.crystalpack(?:\?|$)/.test(n));
   const artifacts=await Promise.all([...new Set(pinnedUrls)].map(async url=>{
    const response=await fetch(url);if(!response.ok)throw Error(`Cannot fingerprint ${url}`);
    const bytes=await response.arrayBuffer();
    const sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
    return {url,sha256,bytes:bytes.byteLength};
   }));
   if(!artifacts.some(a=>a.url.includes('.wasm')))throw Error('Cannot identify the live game binary');
   const wasm=await import(moduleUrl);
   async function request(action){
    wasm.crystal_save_manager_request(action,new Uint8Array());
    const end=performance.now()+10000;
    while(performance.now()<end){
     const r=JSON.parse(wasm.crystal_save_manager_poll()).result;
     if(r){if(r.error)throw Error(r.error);return r;}
     await new Promise(r=>setTimeout(r,25));
    }
    throw Error('Game save timed out');
   }
   wasm.crystal_save_manager_open();
   try {
    const status=await request('status');if(!status.can_save)return null;
    await request('save');await request('export');
    return {bytes:Array.from(wasm.crystal_save_manager_take_bytes()),moduleUrl,artifacts,observation:o,
      storage:{origin:location.origin,localStorage:Object.keys(localStorage).filter(name=>name.startsWith('crystal.save.')).map(name=>({name,value:localStorage.getItem(name)}))}};
   }finally{wasm.crystal_save_manager_close();}
  });
  if(!saved)return false;
  const brain=await page.evaluate(()=>new Promise((resolve,reject)=>{
   const w=globalThis.__flygonNeuralWorker,id=crypto.randomUUID();
   const timer=setTimeout(()=>{w.removeEventListener('message',listener);reject(Error('Brain save timed out'));},15000);
   const listener=({data})=>{if(data.id!==id)return;clearTimeout(timer);w.removeEventListener('message',listener);data.error?reject(Error(data.error)):resolve(data.result.checkpoint);};
   w.addEventListener('message',listener);w.postMessage({id,kind:'checkpoint'});
  }));
  const neuralWorker=page.workers().find(w=>w.url().includes('flygon-worker'));
  if(!neuralWorker)throw Error('Cannot identify the live neural worker');
  const neuralArtifacts=await neuralWorker.evaluate(async()=>{
   const urls=performance.getEntriesByType('resource').map(r=>r.name).filter(u=>/\/crystal_flygon-[a-f0-9]+\.wasm$/.test(u));
   return await Promise.all([...new Set(urls)].map(async url=>{
    const response=await fetch(url);if(!response.ok)throw Error('Cannot fingerprint neural binary');
    const bytes=await response.arrayBuffer();
    return {url,sha256:Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('')};
   }));
  });
  if(neuralArtifacts.length!==1)throw Error('Missing unique neural binary pin');
  const generation=`checkpoint-${Date.now()}`;
  const dir=`${output}/${generation}`;await fs.mkdir(dir,{recursive:true});
  const bytes=Buffer.from(saved.bytes);
  await fs.writeFile(`${dir}/game.crystalsave`,bytes);await fs.writeFile(`${dir}/brain.json`,brain);
  await fs.writeFile(`${dir}/browser-state.json`,JSON.stringify({cookies:[],origins:[saved.storage]}));
  const manifest={generation,gameModule:saved.moduleUrl,artifacts:saved.artifacts,neuralArtifacts,gameSha256:createHash('sha256').update(bytes).digest('hex'),brainSha256:createHash('sha256').update(brain).digest('hex'),observation:saved.observation};
  await fs.writeFile(`${dir}/manifest.json`,JSON.stringify(manifest,null,2));
  await fs.writeFile(`${output}/latest-checkpoint.next`,generation);await fs.rename(`${output}/latest-checkpoint.next`,`${output}/latest-checkpoint`);
  console.log(JSON.stringify({checkpoint:generation,map:saved.observation.map_info.name,gameBytes:bytes.length}));
  return true;
 }finally{if(resume && (await page.locator('#run').textContent())!=='Pause')await page.click('#run');}
}
