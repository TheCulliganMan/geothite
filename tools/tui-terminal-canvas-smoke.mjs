// Actual executable/PTY graphics bytes, not a fake game or terminal emulator.
// This proves payload/input/cleanup behavior; it does not claim a GUI screenshot.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdir, writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {StringDecoder} from 'node:string_decoder';
import {PtyScreen} from './tui-pty-screen.mjs';
const binary=resolve(process.env.TUI_NATIVE_BIN ?? 'target/release/geothite');
const pack=resolve(process.env.TUI_NATIVE_PACK ?? 'content-packs/realtime-clock.browser.crystalpack');
const fixture=resolve(process.env.TUI_NATIVE_FIXTURE ?? 'target/tui-smoke/lowlevel.crystalsave');
const output=resolve(process.env.TUI_TERMINAL_ART_ROOT ?? 'target/tui-terminal-art');
// Optional visual review of the exact PNGs emitted by the executable, not
// just distinct hashes. This is a transport capture, not a Ghostty screenshot.
const captureAnimation=process.env.TUI_CAPTURE_ANIMATION==='1';
await mkdir(output,{recursive:true});
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function check(reduced) {
  const child=spawn('python3',['-u','-c',`
import os, pty, fcntl, termios, struct, subprocess, select, sys, signal
sys.path.insert(0,'tools')
from tui_shared_memory_consumer import SharedMemoryConsumer
receiver=SharedMemoryConsumer()
master,slave=pty.openpty()
fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',24,80,960,576))
def attach():
    os.setsid()
    fcntl.ioctl(slave,termios.TIOCSCTTY,0)
game=subprocess.Popen(sys.argv[1:],stdin=slave,stdout=slave,stderr=slave,preexec_fn=attach)
signal.signal(signal.SIGTERM,lambda signum,frame:sys.exit(0))
def resize(signum,frame):
    fcntl.ioctl(master,termios.TIOCSWINSZ,struct.pack('HHHH',32,100,1200,768))
    os.killpg(game.pid,signal.SIGWINCH)
signal.signal(signal.SIGUSR1,resize)
os.close(slave)
try:
    while game.poll() is None:
        for fd in select.select([master,0],[],[],.1)[0]:
            try: data=os.read(fd,65536)
            except OSError:sys.exit(0)
            if not data:sys.exit(0)
            if fd==master:receiver.feed(data)
            os.write(1 if fd==master else master,data)
finally:
    if game.poll() is None:os.killpg(game.pid,signal.SIGTERM)
    os.close(master)
    game.wait()
`,binary,'play',pack,'--load',fixture],{env:{...process.env,GEOTHITE_TUI_GRAPHICS:'kitty',GEOTHITE_REDUCED_MOTION:reduced?'1':'0'},stdio:['pipe','pipe','pipe','pipe']});
  let raw='',errors='',encoded='',current=null,commands=[];
  const screen=new PtyScreen(),decoder=new StringDecoder('utf8');
  const frames=[];let pending='';
  child.stdout.on('data',bytes=>{
    const text=decoder.write(bytes);raw+=text;pending+=text;screen.feed(text);
    for(;;) {
      const start=pending.indexOf('\x1b_G');if(start<0) {pending=pending.slice(-3);break;}
      const end=pending.indexOf('\x1b\\',start);if(end<0) {pending=pending.slice(start);break;}
      const command=pending.slice(start+3,end);pending=pending.slice(end+2);commands.push(command);
      const separator=command.indexOf(';');const header=separator<0?command:command.slice(0,separator),payload=separator<0?'':command.slice(separator+1);
      const fields=Object.fromEntries(header.split(',').map(s=>s.split('=')));
      if(fields.a==='T') {current=fields;encoded='';assert.equal(fields.q,'2');assert.equal(fields.C,'1');assert(['24','100'].includes(fields.f));}
      if(fields.m!==undefined) {
        assert(payload.length<=4096);encoded+=payload;
        if(fields.m==='0') {
          const png=Buffer.from(encoded,'base64');assert.equal(png.subarray(0,8).toString('hex'),'89504e470d0a1a0a');
          const width=png.readUInt32BE(16),height=png.readUInt32BE(20);
          assert(width>100&&height>40&&width<=2048&&height<=2048);
          frames.push({png,width,height,fields:current,time:performance.now(),hash:createHash('sha256').update(png).digest('hex')});
        }
      }
    }
  });
  child.stderr.on('data',bytes=>errors+=bytes);
  let reports='';
  child.stdio[3].on('data',bytes=>{
    reports+=bytes;
    for(;;){
      const end=reports.indexOf('\n');if(end<0)break;
      const report=JSON.parse(reports.slice(0,end));reports=reports.slice(end+1);
      assert(report.width>100&&report.height>40&&report.width<=2048&&report.height<=2048);
      frames.push({...report,png:report.png?Buffer.from(report.png,'base64'):null,time:performance.now()});
    }
  });
  const visible=()=>screen.text();
  async function waitFor(condition,message) {for(let i=0;i<300&&!condition();i++){if(child.exitCode!==null)break;await pause(50);}assert(condition(),`${message}: ${errors}\n${visible().slice(-1200)}`);}
  async function key(value) {child.stdin.write(value);await pause(180);}
  try {
    await waitFor(()=>frames.length>0,'Real native dot canvas');
    assert(raw.startsWith('\x1b[?1049h'),'No logs before fullscreen');
    assert(visible().includes('Route29'));
    await writeFile(resolve(output,reduced?'pty-canvas-reduced.png':'pty-canvas.png'),frames[0].png);
    await pause(reduced?700:8500);
    if(!reduced) {
      const steps=frames.slice(1).map((f,i)=>f.time-frames[i].time);
      const maxGap=Math.max(...steps);
      assert(frames.length>140,`Actual PNG transport must sustain smooth motion, not 10fps steps: ${frames.length} frames`);
      assert(maxGap<250,`No hold then burst in the emitted terminal frames: ${maxGap.toFixed(0)}ms`);
      assert(frames.every((f,i)=>i===0||f.hash!==frames[i-1].hash),'Every delivered visual frame changes, not a peak plateau');
      console.log(`Continuous native circle frames: ${frames.length} over a full slow cycle, max gap ${maxGap.toFixed(0)}ms.`);
    }
    if(captureAnimation&&!reduced) {
      const directory=resolve(output,'animation');
      await mkdir(directory,{recursive:true});
      const captured=[...frames];
      for(const [index,frame] of captured.entries()) {
        await writeFile(resolve(directory,`${String(index).padStart(4,'0')}.png`),frame.png);
      }
      await writeFile(resolve(directory,'timing.json'),JSON.stringify(captured.map((f,index)=>({index,ms:f.time-captured[0].time,hash:f.hash})),null,2));
    }
    if(reduced) assert.equal(frames.length,1,'Reduced-motion sends one unchanged frame, not an animation');
    else assert(new Set(frames.map(f=>f.hash)).size>1,'The actual console circle sizes animate');
    const inputFrames=frames.length;
    const repeat=setInterval(()=>child.stdin.write('p'),15); // Unmapped key, no game action.
    try {await pause(1000);} finally {clearInterval(repeat);}
    if(reduced) assert.equal(frames.length,inputFrames,'Reduced motion also stays still under repeated input');
    else assert(new Set(frames.slice(inputFrames).map(f=>f.hash)).size>1,'Input faster than the idle timeout must not freeze dot animation');
    assert(new Set(frames.map(f=>f.fields.i)).size<=2,'Bounded image IDs');
    const offset=raw.length;
    await key('v');await waitFor(()=>visible().includes('GAME BOY'),'V still switches to text');
    assert(raw.slice(offset).includes('a=d,d=I,i='),'Text toggle removes only owned canvas images');
    const count=frames.length;await pause(300);assert.equal(frames.length,count,'Text view stops the canvas');
    await key('v');await waitFor(()=>frames.length>count,'V restores shared painted canvas');
    screen.resize(100,32);child.kill('SIGUSR1');
    await waitFor(()=>frames.some(f=>f.width>1000),'Resize repositions and scales the shared scene');
    await key('\r');await waitFor(()=>visible().includes('SAVE'),'Production Start still opens');
    await key('x');
    if(!reduced) {
      const entry=raw.length;
      for(let i=0;i<256&&!visible().includes('WildBattle');i++)await key(Math.floor(i/2)%2?'a':'d');
      await waitFor(()=>visible().includes('WildBattle'),'Graphics client enters a real battle');
      assert(raw.slice(entry).includes('a=d,d=I,i='),'Battle removes the overworld image before ASCII portraits');
      const battleFrames=frames.length;await pause(300);
      assert.equal(frames.length,battleFrames,'No overworld canvas over battle portraits');
      for(let i=0;i<4;i++)await key('x');
      await key('\x1b[A');await key('\x1b[D');
      // Core Overworld and a restored image can precede the last player-owned
      // victory/EXP page. Finish the visible dialogue, not just the core phase.
      for(let i=0;i<256&&(!visible().includes('Overworld')||visible().includes('DIALOGUE'));i++)await key('a');
      await waitFor(()=>visible().includes('Overworld')&&!visible().includes('DIALOGUE'),'Retained battle narration finishes');
      await waitFor(()=>frames.length>battleFrames,'Victory restores the overworld canvas');
      await key('\r');
      await waitFor(()=>visible().includes('SAVE'),'Post-battle Start remains playable');
      await key('x');
      const position=()=>visible().split('\n')[0].match(/\d+,\d+/)?.[0];
      const before=position();assert(before,'Actual player coordinates in terminal header');
      await key('\x1b[A');await key('\x1b[A');
      await waitFor(()=>position()!==before,'Directional input changes the actual post-battle position');
    }
    assert.equal(errors,'');
    const exitOffset=raw.length;
    await key('\x03');await waitFor(()=>child.exitCode!==null,'Clean keyboard exit');
    const exitBytes=raw.slice(exitOffset);
    assert(exitBytes.includes('a=d,d=I,i='),'Exit deletes owned images');
    assert(exitBytes.indexOf('a=d,d=I,i=')<exitBytes.indexOf('\x1b[?1049l'),'Image cleanup precedes leaving fullscreen');
    console.log(`Native canvas (${reduced?'reduced motion':'animated'}): real ${frames[0].fields.t==='s'?'shared RGB':'PNG'} frames, quiet/bounded protocol, resize, V/exit cleanup and production keyboard Start${reduced?'':', battle/victory'} passed.`);
  } finally {
    child.stdin.write('\x03');await pause(250);child.stdin.end();child.kill('SIGTERM');
    if(child.exitCode===null&&child.signalCode===null)await new Promise(r=>child.once('close',r));
  }
}
await check(false);await check(true);
