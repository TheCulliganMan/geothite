#!/usr/bin/env python3
"""Production stdio Surf/island/declined trainer/reward/drop/return and Continue.

Only the initial native controller fixture supplies party, badges and position.
Every later state change uses public MCP input. Packs and input saves are read-only.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import traceback
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tools'))
from render_reference import load_pack
from check_mina_battle_mcp import LiveRecorded
from check_lake_country_mcp import escape_wild
from check_north_shore_story_mcp import choose,finish

cli=argparse.ArgumentParser(description=__doc__)
for name in ['pack','save','client','output']:cli.add_argument('--'+name,type=Path,required=True)
a=cli.parse_args();a.output.mkdir(parents=True,exist_ok=False);pack=load_pack(a.pack);calls=[];error=None;process=None;checkpoints=[]
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
frozen={str(path):sha(path)for path in [a.pack,a.save,a.client]};autosave=a.output/'actual-played.crystalsave'
def pos(c):
 s=c.status();assert s['map']=='MnLeechLake',s;return s['position']['x'],s['position']['y']
def step(c,direction,target):
 for _ in range(12):
  if c.status()['mode'] in {'battle', 'wildbattle'}:escape_wild(c)
  if pos(c)==target:return
  c.move(direction)
 assert pos(c)==target,(direction,target,c.status())
def go(c,target,allowed):
 m=pack['data']['maps']['MnLeechLake'];collision=pack['data']['tilesets']['mn_headwaters']['collision'];start=pos(c);todo=deque([start]);prior={start:None};bodies={(o['x'],o['y'])for o in m['objects']}
 while todo and target not in prior:
  x,y=todo.popleft()
  for q,d in [((x-1,y),'left'),((x+1,y),'right'),((x,y-1),'up'),((x,y+1),'down')]:
   if q in prior or q in bodies or not(0<=q[0]<64 and 0<=q[1]<48):continue
   b=m['blocks'][q[1]//2*32+q[0]//2]
   if collision[f'{b:02x}'][q[1]%2*2+q[0]%2]not in allowed:continue
   prior[q]=((x,y),d);todo.append(q)
 assert target in prior,(start,target,allowed)
 path=[];q=target
 while prior[q]is not None:p,d=prior[q];path.append((q,d));q=p
 for target,d in reversed(path):step(c,d,target)
def mount(c,direction,target):
 c.press(direction);c.press('a')
 for _ in range(64):
  if pos(c)==target:finish(c);return
  c.press('a')
 raise AssertionError('Surf mount did not reach the native launch')
def checkpoint(c,name):
 c.at('MnLeechLake',*pos(c));path=a.output/(name+'.crystalsave');assert not path.exists();path.write_bytes(autosave.read_bytes());checkpoints.append(dict(name=name,position=pos(c),save=str(path),sha256=sha(path)))
try:
 with(a.output/'stderr.log').open('x')as err:
  process=subprocess.Popen([str(a.client.resolve()),'mcp',str(a.pack),'--load',str(a.save),'--save',str(autosave)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1)
  c=LiveRecorded(process,calls);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='leech-island-native-trip',version='1')));c.at('MnLeechLake',31,29);initial=c.status()
  mount(c,'right',(32,29));go(c,(45,30),{'WATER'});step(c,'right',(46,30));checkpoint(c,'west-island-landing')
  go(c,(52,30),{'FLOOR','TALL_GRASS'});c.press('right');c.press('a');choose(c,False);finish(c);checkpoint(c,'naturalist-declined')
  go(c,(52,33),{'FLOOR','TALL_GRASS'});c.press('right');c.press('a');finish(c);checkpoint(c,'water-reward-claimed')
  step(c,'down',(52,34));step(c,'down',(52,36));checkpoint(c,'native-drop-landing')
  for _ in range(3):c.move('up');c.at('MnLeechLake',52,36)
  go(c,(46,30),{'FLOOR'});mount(c,'left',(45,30));go(c,(32,29),{'WATER'});step(c,'left',(31,29));checkpoint(c,'original-shore-return')
  assert c.status()['money']==initial['money']and c.status()['badges']==initial['badges']
  c.press('start');assert any('PACK'in row['text']for row in c.observe()['structuredContent']['menu']);c.press('b');process.stdin.close();assert process.wait(timeout=30)==0
  process=subprocess.Popen([str(a.client.resolve()),'mcp',str(a.pack),'--load',str(autosave),'--save',str(a.output/'continued.crystalsave')],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,text=True,bufsize=1)
  c=LiveRecorded(process,calls);c.rpc('initialize',dict(protocolVersion='2025-06-18',capabilities={},clientInfo=dict(name='leech-island-native-continue',version='1')));c.at('MnLeechLake',31,29);c.move('left');c.at('MnLeechLake',30,29);process.stdin.close();assert process.wait(timeout=30)==0
except BaseException as e:error=repr(e);traceback.print_exc()
finally:
 if process and process.poll()is None:process.terminate();process.wait(timeout=30)
 unchanged=all(sha(Path(p))==digest for p,digest in frozen.items());stderr=(a.output/'stderr.log').read_text();report=dict(status='PASS'if error is None and unchanged and not stderr else'FAIL',error=error,inputs=frozen,inputs_unchanged=unchanged,rpc_calls=len(calls),stderr_bytes=len(stderr.encode()),checkpoints=checkpoints,calls=calls,scope=__doc__)
 (a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(report['status'],error,len(calls),'RPC',flush=True)
 assert report['status']=='PASS',error
