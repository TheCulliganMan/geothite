#!/usr/bin/env python3
"""Read-only stateful graph and original-art review of the published shafts."""
import argparse,json,time
from collections import deque
from pathlib import Path
import cbor2
from PIL import Image,ImageDraw
from render_reference import SourceArt
ROOT=Path(__file__).resolve().parents[3]
SOURCE=ROOT/'planning/minnesota/lairs/iron-workings'
def review(path):
 p=cbor2.loads(Path(path).read_bytes()[22:]);n=json.loads((SOURCE/'network.json').read_text())
 maps={name:p['data']['maps'][name] for name in n['maps']+['MnIronDrillApproach','MnIronPumpGallery']}
 ports={};arrivals=[]
 for l in n['links']:
  for a,b in [(l['a'],l['b']),(l['b'],l['a'])]:
   m=maps[a['map']];dest=maps[b['map']]
   for at in [a['active'],a['arrival']]:
    ws=[w for w in m['events']['warps'] if [w['x'],w['y']]==at];assert len(ws)==1,(l['id'],at,ws)
    w=ws[0];assert w['target_map_constant']==dest['attributes']['map_constant']
    landing=dest['events']['warps'][w['target_warp_id']-1];assert [landing['x'],landing['y']]==b['arrival'],(l['id'],a,w,landing)
   ports[(a['map'],*a['active'])]=(b['map'],*b['arrival']);arrivals.append((a['map'],*a['arrival']))
 def collision(name,x,y,flags):
  m=maps[name];w=m['attributes']['width'];h=m['attributes']['height'];t=m['attributes']['tileset_name']
  if not(0<=x<2*w and 0<=y<2*h):return 'WALL'
  block=m['blocks'][y//2*w+x//2]
  for con in n['controls']:
   if con['id'] in flags and con['map']==name and con['door']==[x//2,y//2]:block=7 if t=='facility' else 13
  if len(flags)==2 and n['gate']['map']==name and n['gate']['door']==[x//2,y//2]:block=13
  return p['data']['tilesets'][t]['collision'][f'{block:02x}'][y%2*2+x%2]
 def reach(start,flags,links=True):
  q=deque([start]);seen={start}
  while q:
   name,x,y=q.popleft()
   for xx,yy in [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]:
    if collision(name,xx,yy,flags) not in ['FLOOR','LADDER']:continue
    at=(name,xx,yy)
    if at in ports:
     if not links:continue
     at=ports[at]
    if at not in seen:seen.add(at);q.append(at)
  return seen
 start=('MnIronHaulage',4,23);target=('MnIronPumpGallery',4,19)
 for flags in [set(),{'HAUL'},{'VENT'},{'HAUL','VENT'}]:
  seen=reach(start,flags)
  assert (target in seen)==(len(flags)==2),('shutter dependency',flags)
  for con in n['controls']:
   x,y=con['tile'];assert (con['map'],x,y+1) in seen,('unreachable control',con['id'],flags)
  for at in arrivals:
   assert collision(*at,flags)=='FLOOR',('unsafe arrival',at)
   if at in seen:assert start in reach(at,flags),('return trap',at,flags)
 for con in n['controls']:
  x,y=con['door'];name=con['map']
  assert (name,x*2+2,y*2) not in reach((name,x*2-1,y*2),set(),False),('crosscut bypass',name)
  assert (name,x*2+2,y*2) in reach((name,x*2-1,y*2),{con['id']},False),('crosscut still closed',name)
 # Full graph permits return by interlevel loops even while local crosscuts are
 # shut; verify local compartment separation separately, without ladder edges.
 out=ROOT/'planning/minnesota/renders/story-events-v2'/f'iron-workings-{time.time_ns()}';out.mkdir(parents=True)
 art=SourceArt(p);sheet=Image.new('RGB',(1320,1000),'#191c22');d=ImageDraw.Draw(sheet)
 for i,name in enumerate(n['maps']):
  m=maps[name];w=m['attributes']['width'];h=m['attributes']['height'];im=art.render(m['attributes']['tileset_name'],w,h,m['blocks'],'day');di=ImageDraw.Draw(im)
  for l in n['links']:
   for ep in [l['a'],l['b']]:
    if ep['map']==name:
     x,y=ep['active'];di.text((x*16+2,y*16-10),l['id'],fill='#ff5050')
  im.save(out/(name+'.png'));x=(i%2)*650;y=(i//2)*480;sheet.paste(im,(x,y+22));d.text((x+5,y+3),name,fill='white')
 sheet.save(out/'whole-complex.png');print('PASS: eight reciprocal ladder pairs, both independent circuits required, reachable controls, safe returns.');print(out/'whole-complex.png')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('pack');review(a.parse_args().pack)
