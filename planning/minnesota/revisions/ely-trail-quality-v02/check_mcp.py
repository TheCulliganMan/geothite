#!/usr/bin/env python3
"""Replay the reached clinic save via production MCP and retain actual checkpoints."""
import argparse,hashlib,json,shutil,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from check_minneapolis_mcp import Client


def main():
 ap=argparse.ArgumentParser();ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--save',type=Path,required=True);ap.add_argument('--proof',type=Path,required=True);ap.add_argument('--output',type=Path,required=True);ap.add_argument('--client',type=Path,default=ROOT/'target/northstar-story/wasm-dev/geothite');a=ap.parse_args()
 a.output.mkdir(parents=True,exist_ok=False);live=a.output/'live.crystalsave'
 plan=json.load(open(a.proof));assert plan['pack_sha256']==hashlib.sha256(a.pack.read_bytes()).hexdigest()
 old=json.load(open(ROOT/'planning/minnesota/revisions/ely-backroute-v01/source-v08/input-plan.json'))
 def keep(name):shutil.copy2(live,a.output/(name+'.crystalsave'))
 with tempfile.TemporaryFile()as stderr:
  p=subprocess.Popen([str(a.client.resolve()),'mcp',str(a.pack.resolve()),'--load',str(a.save.resolve()),'--save',str(live.resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,text=True,bufsize=1)
  try:
   c=Client(p);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='ely-pine-knolls',version='2')))
   c.at('MnR11C00R02',33,34);c.move('up');c.at('MnSplitRockPokecenter1F',3,5)
   c.press('start');assert 'PACK'in c.observe()['content'][0]['text'];c.press('b')
   for _ in range(6):
    if c.status()['map']!='MnSplitRockPokecenter1F':break
    c.move('down')
   c.at('MnR11C00R02',33,34)
   for s in plan['dry_return']['steps']:
    c.at('MnR11C00R02',*s['from_core'])
    if s['jump']:keep('ledge-lip')
    c.move(s['button']);c.at('MnR11C00R02',*s['to_core'])
    if s['jump']:
     keep('ledge-landing');c.move('right');c.at('MnR11C00R02',*s['to_core'])
   c.move('left');c.at('MnElyShorePass',13,10)
   points=old['pass_west_to_east'][::-1]
   for f,t in zip(points,points[1:]):
    dx,dy=t[0]-f[0],t[1]-f[1];button={(1,0):'right',(-1,0):'left',(0,1):'down',(0,-1):'up'}[dx,dy];c.move(button);c.at('MnElyShorePass',*t)
   c.move('left');c.at('MnEly',39,14);keep('ely-pine-gate')
   c.press('start');assert 'PACK'in c.observe()['content'][0]['text'];c.press('b')
   result=dict(result='PASS',pack_sha256=plan['pack_sha256'],clinic_entry_exit_start=True,dry_return_inputs=len(plan['dry_return']['steps']),native_ledge=True,reverse_blocked=True,shore_pass_ely_seams=True,checkpoint_scope='Actual native controller autosaves reached from the explicit local party/position fixture; no earned campaign claim.')
  finally:
   p.stdin.close();assert p.wait(timeout=30)==0
  stderr.seek(0);assert stderr.read()==b'';result['stderr_byte_empty']=True
 (a.output/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
