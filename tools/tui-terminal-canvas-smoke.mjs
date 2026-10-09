// Actual executable/PTY graphics bytes, not a fake game or terminal emulator.
// This proves payload/input/cleanup behavior; it does not claim a GUI screenshot.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdir, writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
const binary=resolve(process.env.TUI_NATIVE_BIN ?? 'target/release/geothite');
const pack=resolve(process.env.TUI_NATIVE_PACK ?? 'content-packs/realtime-clock.browser.crystalpack');
const fixture=resolve(process.env.TUI_NATIVE_FIXTURE ?? 'target/tui-smoke/lowlevel.crystalsave');
const output=resolve(process.env.TUI_TERMINAL_ART_ROOT ?? 'target/tui-terminal-art');
await mkdir(output,{recursive:true});
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function check(reduced) {
  const child=spawn('python3',['-u','-c',`
import os, pty, fcntl, termios, struct, subprocess, select, sys, signal
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
            os.write(1 if fd==master else master,data)
finally:
    if game.poll() is None:os.killpg(game.pid,signal.SIGTERM)
    os.close(master)
    game.wait()
`,binary,'play',pack,'--load',fixture],{env:{...process.env,GEOTHITE_TUI_GRAPHICS:'kitty',GEOTHITE_REDUCED_MOTION:reduced?'1':'0'},stdio:['pipe','pipe','pipe']});
  let raw='',errors='',encoded='',current=null,commands=[];
  const frames=[];let pending='';
  child.stdout.on('data',bytes=>{
    const text=bytes.toString();raw+=text;pending+=text;
    for(;;) {
      const start=pending.indexOf('\x1b_G');if(start<0) {pending=pending.slice(-3);break;}
      const end=pending.indexOf('\x1b\\',start);if(end<0) {pending=pending.slice(start);break;}
      const command=pending.slice(start+3,end);pending=pending.slice(end+2);commands.push(command);
      const separator=command.indexOf(';');const header=separator<0?command:command.slice(0,separator),payload=separator<0?'':command.slice(separator+1);
      const fields=Object.fromEntries(header.split(',').map(s=>s.split('=')));
      if(fields.a==='T') {current=fields;encoded='';assert.equal(fields.q,'2');assert.equal(fields.C,'1');assert.equal(fields.f,'100');}
      if(fields.m!==undefined) {
        assert(payload.length<=4096);encoded+=payload;
        if(fields.m==='0') {
          const png=Buffer.from(encoded,'base64');assert.equal(png.subarray(0,8).toString('hex'),'89504e470d0a1a0a');
          const width=png.readUInt32BE(16),height=png.readUInt32BE(20);
          assert(width>100&&height>40&&width<=2048&&height<=2048);
          frames.push({png,fields:current,hash:createHash('sha256').update(png).digest('hex')});
        }
      }
    }
  });
  child.stderr.on('data',bytes=>errors+=bytes);
  const visible=()=>raw.replace(/\x1b_G[^]*?\x1b\\/g,'').replace(/\x1b\[[0-?]*[ -/]*[@-~]/g,'');
  async function waitFor(condition,message) {for(let i=0;i<300&&!condition();i++){if(child.exitCode!==null)break;await pause(50);}assert(condition(),`${message}: ${errors}\n${visible().slice(-1200)}`);}
  async function key(value) {child.stdin.write(value);await pause(180);}
  try {
    await waitFor(()=>frames.length>0,'Real native dot canvas');
    assert(raw.startsWith('\x1b[?1049h'),'No logs before fullscreen');
    assert(visible().includes('Route29'));
    await writeFile(resolve(output,reduced?'pty-canvas-reduced.png':'pty-canvas.png'),frames[0].png);
    await pause(700);
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
    child.kill('SIGUSR1');
    await waitFor(()=>frames.some(f=>f.png.readUInt32BE(16)>1000),'Resize repositions and scales the shared scene');
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
      for(let i=0;i<256&&!raw.slice(entry).includes('Overworld');i++)await key('a');
      await waitFor(()=>frames.length>battleFrames,'Victory restores the overworld canvas');
      await key('\x1b[A');await key('\x1b[A');await key('\r');
      assert(visible().slice(visible().indexOf('WildBattle')).includes('SAVE'),'Post-battle Start remains playable');
    }
    assert.equal(errors,'');
    const exitOffset=raw.length;
    await key('\x03');await waitFor(()=>child.exitCode!==null,'Clean keyboard exit');
    const exitBytes=raw.slice(exitOffset);
    assert(exitBytes.includes('a=d,d=I,i='),'Exit deletes owned images');
    assert(exitBytes.indexOf('a=d,d=I,i=')<exitBytes.indexOf('\x1b[?1049l'),'Image cleanup precedes leaving fullscreen');
    console.log(`Native canvas (${reduced?'reduced motion':'animated'}): real PNG frames, quiet/chunked protocol, resize, V/exit cleanup and production keyboard Start${reduced?'':', battle/victory'} passed.`);
  } finally {
    child.stdin.write('\x03');await pause(250);child.stdin.end();child.kill('SIGTERM');
    if(child.exitCode===null&&child.signalCode===null)await new Promise(r=>child.once('close',r));
  }
}
await check(false);await check(true);
