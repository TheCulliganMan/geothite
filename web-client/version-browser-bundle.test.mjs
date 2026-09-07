import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, writeFile, readFile, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execFileSync } from 'node:child_process';
import { gunzipSync } from 'node:zlib';

test('published page selects an inseparable content-versioned JS and WASM pair', async () => {
 const dir=await mkdtemp(join(tmpdir(),'crystal-bundle-'));
 try {
  const publish=async bytes=>{
   await writeFile(join(dir,'index.html'),"const wasm = await import('./crystal-bevy.js');");
   await writeFile(join(dir,'crystal-bevy.js'),"const path = new URL('crystal-bevy_bg.wasm', import.meta.url);");
   await writeFile(join(dir,'crystal-bevy_bg.wasm'),bytes);
   execFileSync('sh',[new URL('../tools/version-browser-bundle.sh',import.meta.url).pathname,dir]);
   const page=await readFile(join(dir,'index.html'),'utf8');
   const name=page.match(/import\('\.\/(crystal-bevy-[a-f0-9]{64})\.js'\)/)?.[1];
   assert.ok(name,'page must use content-versioned glue');
   const js=await readFile(join(dir,name+'.js'),'utf8');
   assert.ok(js.includes(`new URL('${name}.wasm', import.meta.url)`));
   assert.deepEqual(await readFile(join(dir,name+'.wasm')),bytes);
   assert.deepEqual(gunzipSync(await readFile(join(dir,name+'.wasm.gz'))),bytes);
   assert.ok(!(await readdir(dir)).includes('crystal-bevy.js'));
   assert.ok(!(await readdir(dir)).includes('crystal-bevy_bg.wasm'));
   return name;
  };
  const first=await publish(Buffer.from('build-one'));
  const second=await publish(Buffer.from('build-two'));
  assert.notEqual(first,second,'a new WASM must never reuse a cached JS URL');
 }finally{await rm(dir,{recursive:true,force:true});}
});

test('Docker ships the content-versioning script and runs it after page assembly', async () => {
 const docker=await readFile(new URL('../Dockerfile',import.meta.url),'utf8');
 const ignore=await readFile(new URL('../Dockerfile.dockerignore',import.meta.url),'utf8');
 assert.match(ignore,/^!tools\/version-browser-bundle\.sh$/m);
 assert.match(docker,/COPY tools\/version-browser-bundle\.sh/);
 assert.ok(docker.indexOf('sh /source/version-browser-bundle.sh /out/web') > docker.indexOf('cp /source/web-client/index.html'));
});
