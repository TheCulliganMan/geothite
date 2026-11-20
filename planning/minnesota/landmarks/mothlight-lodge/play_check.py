#!/usr/bin/env python3
"""Exercise a disposable Mothlight fixture through the production stdio client.

Usage: .local/minnesota-review-env/bin/python PATH/play_check.py PACK TARGET_SAVE [--full-bag]
Requires the release geothite client and mothlight_fixture example. The pack is
read only to plan walks; only real controller inputs change the supplied save.
"""
import json,subprocess,sys,collections,cbor2,os
from pathlib import Path
pack,save=sys.argv[1:3]
full = '--full-bag' in sys.argv[3:]
assert Path(save).resolve().is_relative_to(Path('target').resolve()), 'use an ignored disposable fixture under target/'
p=cbor2.loads(Path(pack).read_bytes()[22:]);maps=p['data']['maps'];tiles=p['data']['tilesets']
client=None;counter=0;transcript=[]
def start():
 global client
 client=subprocess.Popen([os.environ.get('MOTHLIGHT_CLIENT_BIN','target/release/geothite'),'mcp',str(Path(pack).resolve()),'--load',str(Path(save).resolve()),'--save',str(Path(save).resolve())],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
def rpc(name,**args):
 global counter
 if name=='save':name,args='execute',{'code':'return await tools.save();'}
 counter+=1;client.stdin.write(json.dumps(dict(jsonrpc='2.0',id=counter,method='tools/call',params=dict(name=name,arguments=args)))+'\n');client.stdin.flush();line=client.stdout.readline()
 assert line,client.stderr.read()
 r=json.loads(line);assert 'error' not in r,r
 r=r['result'];assert not r.get('isError'),r
 return r.get('structuredContent',r)
def state():return rpc('status')
def pos(s):return s['position']['x'],s['position']['y']
def move(direction):return rpc('move',direction=direction)
def press(button):return rpc('press',button=button)
def cross(direction,target):
 for _ in range(4):
  s=move(direction)
  if s['map']==target:return s
 raise AssertionError(('warp did not fire',target,s))
def kind(m,q):
 x,y=q;a=m['attributes']
 if x<0 or y<0 or x>=a['width']*2 or y>=a['height']*2:return 'WALL'
 b=m['blocks'][(y//2)*a['width']+x//2];return tiles[a['tileset_name']]['collision'][f'{b:02x}'][y%2*2+x%2]
def flee_wild():
 for _ in range(160):
  s=state();o=rpc('observe',detail='compact')
  if s['mode']=='overworld' and not s['dialogue'] and not s['menu']:return
  if any('RUN' in r['text'] for r in o.get('menu',[])):
   for _ in range(4):
    o=rpc('observe',detail='compact')
    if any('RUN' in r['text'] and r['kind']=='selected' for r in o['menu']):break
    press('down')
  press('a')
 raise AssertionError('wild encounter did not release input after RUN')
def walk(to):
 s=state();m=maps[s['map']];origin=pos(s);prev={origin:None};queue=collections.deque([origin]);objects={(o['x'],o['y']) for o in m['objects']};warps={(w['x'],w['y']) for w in m['events']['warps']}
 # Route around existing trainers for this destination-focused fixture.
 sightlines=set()
 for o in m['objects']:
  if o.get('object_type')!='OBJECTTYPE_TRAINER':continue
  direction=o.get('sightline_direction_override','')
  dx,dy={'UP':(0,-1),'DOWN':(0,1),'LEFT':(-1,0),'RIGHT':(1,0)}.get(direction,(0,0))
  for n in range(1,o.get('radius',0)+1):sightlines.add((o['x']+dx*n,o['y']+dy*n))
 while queue:
  q=queue.popleft()
  if q==to:break
  for dx,dy,d in [(1,0,'right'),(-1,0,'left'),(0,1,'down'),(0,-1,'up')]:
   z=q[0]+dx,q[1]+dy
   if z in prev or z in objects or z in sightlines or z in warps and z!=to:continue
   if kind(m,z) not in ['FLOOR','TALL_GRASS','DOOR','LADDER','WARP_CARPET_DOWN','WARP_CARPET_LEFT','WARP_CARPET_RIGHT']:continue
   prev[z]=(q,d);queue.append(z)
 assert to in prev,('no path',s['map'],origin,to)
 route=[];q=to
 while prev[q]:q,d=prev[q];route.append(d)
 for d in route[::-1]:
  s=move(d)
  if s['mode']=='wildbattle':
   flee_wild();return walk(to)
 assert pos(s)==to or s['map']!=m['id'],('walk end',to,s)
 return s
def talk(at):
 s=state();x,y=pos(s);dx,dy=at[0]-x,at[1]-y;direction={(1,0):'right',(-1,0):'left',(0,1):'down',(0,-1):'up'}[(dx,dy)]
 press(direction);s=press('a');pages=[];words=[]
 if at==(8,16) and not Path(save).with_name(Path(save).stem+'-host-dialogue.crystalsave').exists():Path(save).with_name(Path(save).stem+'-host-dialogue.crystalsave').write_bytes(Path(save).read_bytes())
 for _ in range(100):
  o=rpc('observe',detail='compact');pages.append(json.dumps(o));words.extend(r['text'] for r in o.get('dialogue',[]))
  if not s.get('dialogue') and not s.get('menu'):break
  s=press('a')
 else:raise AssertionError('unreleased text')
 transcript.extend(pages);return ' '.join(words)
def inspect():return json.loads(subprocess.check_output([os.environ.get('MOTHLIGHT_FIXTURE_BIN','target/release/examples/mothlight_fixture'),pack,save,'--inspect'],text=True))
if __name__ == '__main__':
 start();assert inspect()['hp']==1;print('INITIAL',json.dumps(state()),flush=True)
 s=walk((30,10));s=move('up');assert s['map']=='MnMothlightLodge',s;print('ENTERED',json.dumps(s),flush=True)
 walk((8,17));assert 'MARNIE' in talk((8,16))
 walk((10,10));assert 'Warm paws' in talk((10,9));health=inspect();assert health['hp']==health['max_hp'];print('HEALED',health,flush=True)
 walk((3,11));press('down');press('down');assert pos(state())==(3,11),'chair must block movement'
 walk((2,13));assert 'crumbs' in talk((3,13))
 walk((2,26));assert 'HIDE A CAKE' in talk((2,25))
 walk((5,26));assert 'KEEP THIS AWAY' in talk((5,25))
 walk((8,17));words=talk((8,16));assert 'handwriting' in words
 if full:
  assert 'PACK is full' in words,words
  failed=inspect();assert failed['found'] and not failed['rewarded'] and failed['rare_candy']==0,failed
  assert 'PACK is full' in talk((8,16))
  assert not inspect()['rewarded'];press('start');press('b');rpc('save')
  client.stdin.close();assert client.wait(timeout=15)==0;assert not client.stderr.read()
  print('PASS full Pack retains unclaimed reward and releases input',failed,flush=True);sys.exit(0)
 reward=inspect();assert reward['found'] and reward['rewarded'] and reward['rare_candy']==1;print('REWARD',reward,flush=True)
 assert 'another slice' in talk((8,16));assert inspect()['rare_candy']==1;print('REPEAT no duplicate',flush=True)
 walk((20,9));s=move('up');assert s['map']=='MnMothlightLoft',s
 walk((5,7));assert 'moths' in talk((5,6));rpc('save');client.stdin.close();assert client.wait(timeout=15)==0;assert not client.stderr.read();start();assert state()['map']=='MnMothlightLoft';assert inspect()['rare_candy']==1 and inspect()['rewarded']
 walk((13,8));s=cross('left','MnMothlightLodge')
 walk((8,17));assert 'another slice' in talk((8,16))
 s=press('start');assert s['menu'],s;press('b')
 walk((13,28));s=cross('down','MnMaraisCedarApproach')
 walk((26,23));s=move('down');assert s['map']=='MnGrandMarais',s
 s=press('start');assert s['menu'],s;press('b');rpc('save')
 Path(save).with_suffix('.transcript.json').write_text(json.dumps(transcript,indent=2));print('PASS entrance, conversations, healing, cake reward, repeat, loft, Continue, return to Grand Marais and Start',flush=True)
 client.stdin.close();assert client.wait(timeout=15)==0;assert not client.stderr.read()
