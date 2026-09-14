// Loopback-only immutable training assets; serves the same origin as saved games.
import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
const port=Number(process.env.FLYGON_WEB_PORT||33126);
if(!Number.isInteger(port)||port<1024||port>65535)throw Error('Invalid training port');
const root=path.resolve(process.env.FLYGON_WEB_ROOT||'web');
const mime={'.html':'text/html','.js':'text/javascript','.css':'text/css','.json':'application/json','.wasm':'application/wasm','.png':'image/png','.svg':'image/svg+xml'};
http.createServer((req,res)=>{
 let pathname;try{pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);}catch{res.writeHead(400).end();return;}
 if(pathname==='/v1/clock'){res.writeHead(200,{'Content-Type':'application/json','Cache-Control':'no-store'}).end(JSON.stringify({unixMillis:Date.now(),timeZone:'UTC'}));return;}
 if(/^\/flygon-view-[a-f0-9]+\.json$/.test(pathname))pathname='/flygon-view.json';
 if(pathname==='/flygon')pathname='/flygon.html';if(pathname==='/')pathname='/index.html';
 const file=path.resolve(root,'.'+pathname);
 if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
 fs.stat(file,(error,stat)=>{
  if(error||!stat.isFile()){res.writeHead(404).end();return;}
  res.writeHead(200,{'Content-Type':mime[path.extname(file)]||'application/octet-stream','Content-Length':stat.size,'Cross-Origin-Opener-Policy':'same-origin','Cross-Origin-Embedder-Policy':'require-corp','Cache-Control':'no-store'});
  const stream=fs.createReadStream(file);stream.on('error',()=>res.destroy());stream.pipe(res);
 });
}).listen(port,'127.0.0.1');
