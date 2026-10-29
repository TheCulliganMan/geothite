#!/usr/bin/env python3
"""Reach required grass through native inputs, spend Repel and observe a wild battle."""
import argparse,cbor2,hashlib,json,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from check_minneapolis_mcp import Client


def main():
 ap=argparse.ArgumentParser();ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--save',type=Path,required=True);ap.add_argument('--proof',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--client',type=Path,default=ROOT/'target/northstar-story/wasm-dev/geothite');a=ap.parse_args()
 a.output.mkdir(parents=True,exist_ok=False);raw=a.pack.read_bytes();proof=json.load(open(a.proof));assert proof['pack_sha256']==hashlib.sha256(raw).hexdigest()
 pack=cbor2.loads(raw[22:]);species={v['species']for group in pack['data']['wild_encounters']['MnR11C00R02']['grass'].values()for v in group}
 old=json.load(open(ROOT/'planning/minnesota/revisions/ely-backroute-v01/source-v08/input-plan.json'))
 with tempfile.TemporaryFile()as errors:
  p=subprocess.Popen([str(a.client.resolve()),'mcp',str(a.pack.resolve()),'--load',str(a.save.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=errors,text=True,bufsize=1)
  try:
   c=Client(p);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='ely-knoll-encounter',version='2')))
   c.at('MnEly',39,14);c.move('right');c.at('MnElyShorePass',0,14)
   for f,t in zip(old['pass_west_to_east'],old['pass_west_to_east'][1:]):c.move({(1,0):'right',(-1,0):'left',(0,1):'down',(0,-1):'up'}[t[0]-f[0],t[1]-f[1]]);c.at('MnElyShorePass',*t)
   c.move('right');c.at('MnR11C00R02',0,30)
   for s in proof['eastbound']['steps']:
    if s['terrain']=='TALL_GRASS':break
    c.move(s['button']);c.at('MnR11C00R02',*s['to_core'])
   c.at('MnR11C00R02',4,30)
   for i in range(512):
    c.move('up'if i%2==0 else'down',2)
    if c.status()['mode']=='wildbattle':
     c.at('MnR11C00R02',4,c.status()['position']['y']);assert c.status()['position']['y']in[28,29]
     text=c.observe()['content'][0]['text'];enemy=[n for n in species if n in text.upper()];assert enemy,text
     result=dict(result='PASS',pack_sha256=proof['pack_sha256'],native_required_grass_encounter=True,visible_enemy=enemy,walking_pairs_after_reaching_grass=i+1,scope='Existing Repel state is preserved; equal/higher scaled wilds can pass it. No RNG, species or battle state is forced.')
     (a.output/'visible-battle.txt').write_text(text);break
    while c.status()['dialogue'] and c.status()['mode']=='text':c.press('a')
   else:raise AssertionError('required grass never produced a native wild battle')
  finally:
   p.stdin.close();assert p.wait(timeout=30)==0
  errors.seek(0);assert errors.read()==b'';result['stderr_byte_empty']=True
 (a.output/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
