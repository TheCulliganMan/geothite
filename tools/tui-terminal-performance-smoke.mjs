// Large-window real PTY benchmark. This is NOT a Ghostty GUI measurement.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {resolve} from 'node:path';
import {writeFile} from 'node:fs/promises';
import {StringDecoder} from 'node:string_decoder';
import {PtyScreen} from './tui-pty-screen.mjs';
const pause=ms=>new Promise(r=>setTimeout(r,ms));
const child=spawn('python3',['-u','-c',`
import os,pty,fcntl,termios,struct,subprocess,select,sys,signal
sys.path.insert(0,'tools')
from tui_shared_memory_consumer import SharedMemoryConsumer
receiver=SharedMemoryConsumer()
master,slave=pty.openpty()
fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',60,160,2560,1920))
def attach():
    os.setsid()
    fcntl.ioctl(slave,termios.TIOCSCTTY,0)
game=subprocess.Popen(sys.argv[1:],stdin=slave,stdout=slave,stderr=slave,preexec_fn=attach)
signal.signal(signal.SIGTERM,lambda signum,frame:sys.exit(0))
os.close(slave)
try:
    while game.poll() is None:
        for fd in select.select([master,0],[],[],.1)[0]:
            try:data=os.read(fd,65536)
            except OSError:sys.exit(0)
            if not data:sys.exit(0)
            if fd==master:receiver.feed(data)
            os.write(1 if fd==master else master,data)
finally:
    if game.poll() is None:os.killpg(game.pid,signal.SIGTERM)
    os.close(master)
    game.wait()
`,resolve(process.env.TUI_NATIVE_BIN??'target/release/geothite'),'play',resolve(process.env.TUI_NATIVE_PACK??'content-packs/realtime-clock.browser.crystalpack')],{env:{...process.env,GEOTHITE_TUI_GRAPHICS:'kitty'},stdio:['pipe','pipe','pipe','pipe']});
const decoder=new StringDecoder('utf8'),screen=new PtyScreen(160,60);
let bytes=0,raw='',errors='';const frames=[];
child.stdout.on('data',chunk=>{
  bytes+=chunk.length;const text=decoder.write(chunk);raw+=text;screen.feed(text);
  if(text.includes('m=0;'))frames.push(performance.now());
});
child.stderr.on('data',chunk=>errors+=chunk);
let reports='';
child.stdio[3].on('data',chunk=>{
  reports+=chunk;
  for(;;){const end=reports.indexOf('\n');if(end<0)break;JSON.parse(reports.slice(0,end));reports=reports.slice(end+1);frames.push(performance.now());}
});
async function waitFor(condition) {
  const start=performance.now();
  while(!condition()&&performance.now()-start<10000&&child.exitCode===null)await pause(5);
  assert(condition(),`Native input/paint timeout: ${errors}\n${screen.text()}`);
}
try {
  await waitFor(()=>frames.length>0);
  await pause(500);const begin=performance.now(),beforeBytes=bytes,beforeFrames=frames.length,beforeRaw=raw.length;
  await pause(2500);
  const duration=(performance.now()-begin)/1000;
  const ambient=raw.slice(beforeRaw).replace(/\x1b_G[\s\S]*?\x1b\\/g,'');
  const latency=[];
  for(let i=0;i<12;i++) {
    const start=performance.now();child.stdin.write('\r');
    await waitFor(()=>screen.text().includes('SAVE'));
    latency.push(performance.now()-start);
    child.stdin.write('x');await waitFor(()=>!screen.text().includes('SAVE'));
    await pause(15);
  }
  const result={fps:(frames.length-beforeFrames)/((performance.now()-begin)/1000),idleFps:frames.slice(beforeFrames).filter(t=>t<begin+duration*1000).length/duration,wireMBps:(bytes-beforeBytes)/((performance.now()-begin)/1000)/1e6,ambientGlyphs:(ambient.match(/[\u2801-\u28ff]/gu)??[]).length,startLatencyMs:latency,maxLatencyMs:Math.max(...latency)};
  console.log(JSON.stringify(result,null,2));
  if(process.env.TUI_PERF_OUTPUT)await writeFile(resolve(process.env.TUI_PERF_OUTPUT),JSON.stringify(result,null,2));
  if(process.env.TUI_PERF_ASSERT==='1') {
    assert(result.idleFps>24,'Large-window animation must keep up without dropping resolution');
    assert.equal(result.ambientGlyphs,0,'No hidden animated Braille underneath the PNG');
    assert(result.wireMBps<.05,'Local graphics must not flood the PTY with pixel data');
    assert(result.maxLatencyMs<150,'Start must not wait behind cosmetic rendering');
  }
  assert.equal(errors,'');
} finally {
  child.stdin.write('\x03');await pause(250);child.stdin.end();child.kill('SIGTERM');
  if(child.exitCode===null&&child.signalCode===null)await new Promise(r=>child.once('close',r));
}
