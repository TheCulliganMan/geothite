// Persistent single-worker training. Each session resumes an actual paired save.
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawn} from 'node:child_process';
const root=path.resolve(process.env.FLYGON_TRAIN_DIR||'target/flygon-training');
const seed=process.env.FLYGON_RESUME;
if(!seed)throw Error('FLYGON_RESUME must identify the initial paired checkpoint');
await fs.mkdir(root,{recursive:true});
let child,stopping=false;
for(const signal of ['SIGINT','SIGTERM'])process.on(signal,()=>{stopping=true;child?.kill('SIGINT');});
let resume=seed;
try{resume=(await fs.readFile(path.join(root,'resume'),'utf8')).trim();}catch(e){if(e.code!=='ENOENT')throw e;}
while(!stopping){
 const session=path.join(root,`session-${Date.now()}`);await fs.mkdir(session);
 await fs.writeFile(path.join(root,'current-session'),session);
 const log=await fs.open(path.join(session,'runner.log'),'a');
 child=spawn(process.execPath,[new URL('./flygon-runtime-watch.mjs',import.meta.url).pathname],{
  env:{...process.env,FLYGON_RESUME:resume,FLYGON_EVIDENCE_DIR:session,FLYGON_WATCH_SAMPLES:'360',FLYGON_FINISH_SAFE_BOUNDARY:'1'},stdio:['ignore',log.fd,log.fd]});
 let busy=false;
 const maintain=setInterval(async()=>{
  if(busy)return;busy=true;
  try{
   const latest=(await fs.readFile(path.join(session,'latest-checkpoint'),'utf8')).trim();
   resume=path.join(session,latest);
   await fs.writeFile(path.join(root,'resume.next'),resume);await fs.rename(path.join(root,'resume.next'),path.join(root,'resume'));
   const checkpoints=(await fs.readdir(session)).filter(n=>/^checkpoint-\d+$/.test(n)).sort();
   for(const name of checkpoints.slice(0,-12))if(name!==latest)await fs.rm(path.join(session,name),{recursive:true});
  }catch(e){if(e.code!=='ENOENT')console.error(e);}
  finally{busy=false;}
 },10000);
 const code=await new Promise((resolve,reject)=>{child.once('error',reject);child.once('exit',resolve);});
 clearInterval(maintain);await log.close();
 // Capture the final safe save even if it happened immediately before exit.
 try{resume=path.join(session,(await fs.readFile(path.join(session,'latest-checkpoint'),'utf8')).trim());await fs.writeFile(path.join(root,'resume.next'),resume);await fs.rename(path.join(root,'resume.next'),path.join(root,'resume'));}catch(e){if(e.code!=='ENOENT')throw e;}
 if(await fs.stat(path.join(session,'runtime.json')).then(()=>true,()=>false))await new Promise(resolve=>{const report=spawn(process.execPath,[new URL('./flygon-exploration-report.mjs',import.meta.url).pathname,path.join(session,'runtime.json')],{stdio:['ignore','pipe','inherit']});let output='';report.stdout.on('data',c=>output+=c);report.on('close',async()=>{if(output)await fs.writeFile(path.join(session,'report.json'),output);resolve();});});
 const sessions=(await fs.readdir(root)).filter(n=>/^session-\d+$/.test(n)).sort();
 for(const name of sessions.slice(0,-24))if(!resume.startsWith(path.join(root,name)+'/'))await fs.rm(path.join(root,name),{recursive:true});
 console.log(JSON.stringify({session,exitCode:code,resume,stopping}));
 if(!stopping)await new Promise(r=>setTimeout(r,30000));
}
