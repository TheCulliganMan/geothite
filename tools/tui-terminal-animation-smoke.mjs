// Actual ordinary-terminal glyph updates, not PNG hashes or cursor traffic.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {resolve} from 'node:path';
import {StringDecoder} from 'node:string_decoder';
const binary=resolve(process.env.TUI_NATIVE_BIN ?? 'target/release/geothite');
const pack=resolve(process.env.TUI_NATIVE_PACK ?? 'content-packs/realtime-clock.browser.crystalpack');
const fixture=resolve(process.env.TUI_NATIVE_FIXTURE ?? 'target/tui-smoke/lowlevel.crystalsave');
const pause=ms=>new Promise(r=>setTimeout(r,ms));
async function check(reduced) {
  const child=spawn('python3',['-u','-c',`
import os,pty,fcntl,termios,struct,subprocess,select,sys,signal
master,slave=pty.openpty()
fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',24,80,0,0))
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
            os.write(1 if fd==master else master,data)
finally:
    if game.poll() is None:os.killpg(game.pid,signal.SIGTERM)
    os.close(master)
    game.wait()
`,binary,'play',pack,'--load',fixture],{env:{...process.env,GEOTHITE_TUI_GRAPHICS:'off',GEOTHITE_REDUCED_MOTION:reduced?'1':'0'},stdio:['pipe','pipe','pipe']});
  const decoder=new StringDecoder('utf8');let raw='',errors='';
  child.stdout.on('data',bytes=>raw+=decoder.write(bytes));
  child.stderr.on('data',bytes=>errors+=bytes);
  const dots=text=>(text.match(/[\u2801-\u28ff]/gu)??[]).length;
  try {
    for(let i=0;i<200&&!raw.includes('Route29');i++)await pause(50);
    assert(raw.includes('Route29'),`Real native scene: ${errors}`);
    await pause(250);assert(dots(raw)>100,'Actual high-resolution Braille scene');
    assert(!raw.includes('\x1b_G'),'Portable terminal needs no graphics protocol');
    let offset=raw.length;await pause(1000);
    const idle=dots(raw.slice(offset));
    const repeat=setInterval(()=>child.stdin.write('p'),15); // Unmapped, not a game tick.
    try {
      // Drain the last idle redraw before measuring sustained input. Counting
      // that single trailing frame let the old idle-only timer falsely pass.
      await pause(350);offset=raw.length;await pause(1500);
    } finally {clearInterval(repeat);}
    const busy=dots(raw.slice(offset));
    if(reduced) {
      assert.equal(idle,0,'Reduced-motion glyphs stay still when idle');
      assert.equal(busy,0,'Reduced-motion glyphs stay still under repeated input');
    } else {
      assert(idle>5,`Idle dither must visibly update glyphs, not just the cursor (${idle})`);
      assert(busy>5,`Repeated input must not starve visible dither (${busy})`);
    }
    const toggle=raw.length;child.stdin.write('v');await pause(350);
    assert(raw.slice(toggle).includes('GAME BOY'),'Keyboard still switches to text');
    offset=raw.length;await pause(350);assert.equal(dots(raw.slice(offset)),0,'Text view does not animate');
    assert.equal(errors,'');
    console.log(`Portable terminal ${reduced?'reduced motion':'animation'}: ${idle} idle/${busy} busy visible dot updates; V and quiet output passed.`);
  } finally {
    child.stdin.write('\x03');await pause(200);child.stdin.end();child.kill('SIGTERM');
    if(child.exitCode===null&&child.signalCode===null)await new Promise(r=>child.once('close',r));
  }
}
await check(false);await check(true);
