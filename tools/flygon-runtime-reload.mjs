// Reload the neural worker in an observed session without touching game state.
import { chromium } from 'playwright';
const browser=await chromium.connectOverCDP(process.env.FLYGON_CDP || 'http://127.0.0.1:9338');
const page=browser.contexts()[0].pages().find(p=>p.url().includes('/flygon'));
if(!page)throw Error('No Flygon observation session');
if(await page.locator('#run').textContent()==='Pause')await page.click('#run');
await page.waitForTimeout(1000);
await page.evaluate(async()=>{
 globalThis.__watchCommand=(kind,data={})=>new Promise((resolve,reject)=>{
  const worker=globalThis.__flygonNeuralWorker,id=`watch-${crypto.randomUUID()}`;
  const listener=({data:r})=>{if(r.id!==id)return;worker.removeEventListener('message',listener);r.error?reject(Error(r.error)):resolve(r.result);};
  worker.addEventListener('message',listener);worker.postMessage({id,kind,...data});
 });
 globalThis.__watchCheckpoint ??= (await globalThis.__watchCommand('checkpoint')).checkpoint;
 globalThis.__watchGameBefore=globalThis.__flygonLatestObservation;
 const previous=globalThis.__flygonNeuralWorker;
 previous.__watchReplacement?.terminate();
 const NativeWorker=Object.getPrototypeOf(Worker);
 const replacement=new NativeWorker(`./flygon-worker.js?reload=${Date.now()}`,{type:'module'});
 replacement.addEventListener('message',({data})=>previous.dispatchEvent(new MessageEvent('message',{data})));
 replacement.addEventListener('error',e=>previous.dispatchEvent(new ErrorEvent('error',{message:e.message})));
 previous.terminate();
 previous.__watchReplacement=replacement;
 previous.postMessage=(data,...rest)=>{if(data.observation)globalThis.__flygonLatestObservation=data.observation;return replacement.postMessage(data,...rest);};
});
const result=await page.evaluate(async()=>{
 const config=await fetch('./flygon-operant.json',{cache:'no-store'}).then(r=>r.json());
 // Await the new worker's load reply, not a stale enabled button in the UI.
 await globalThis.__watchCommand('load',{config,laboratory:false});
 const restored=await globalThis.__watchCommand('restore',{checkpoint:globalThis.__watchCheckpoint});
 await globalThis.__watchCommand('configure',{config});
 delete globalThis.__watchCheckpoint;
 return {gameBefore:{map:globalThis.__watchGameBefore.map_info.name,player:globalThis.__watchGameBefore.map_info.player},restored:restored.interface_id,plastic:restored.changed_edges};
});
console.log(JSON.stringify(result));
await page.click('#run');
await browser.close();
