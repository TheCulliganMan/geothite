#!/usr/bin/env python3
"""Read-only native-art, chamber-access and scripted movement preflight."""
import argparse,json,time
from collections import deque
from pathlib import Path
import cbor2
from PIL import Image,ImageDraw
from render_reference import SourceArt
ROOT=Path(__file__).resolve().parents[3]
def review(path):
 p=cbor2.loads(Path(path).read_bytes()[22:]);data=p['data'];art=SourceArt(p)
 for base,variant in [('house','mn_arc_harbor'),('mn_northstar_relay','mn_arc_shelter'),('mn_arc_shelter','mn_arc_shelter_gate'),('facility','mn_arc_intake')]:
  if variant in data['tilesets']:
   for group in ['morn','day','nite','indoor']:
    original,private=art.blocks(base,group),art.blocks(variant,group)
    for index,tile in enumerate(original):
     assert tile.tobytes()==private[index].tobytes(),('changed native block pixels',base,variant,group,index)
   print('PASS original native block pixels:',base,variant)
 source=json.loads((ROOT/'planning/minnesota/lairs/rival-arc/intake.json').read_text())
 guards=json.loads((ROOT/'planning/minnesota/story-arc-v3.json').read_text())['grunts']
 assert len(guards)==24
 stations={}
 for guard in guards:
  room=data['maps'][guard['map']];x,y=guard['at'];ww=room['attributes']['width'];col=data['tilesets'][room['attributes']['tileset_name']]['collision']
  assert col[f'{room["blocks"][y//2*ww+x//2]:02x}'][y%2*2+x%2]=='FLOOR',('guard in wall',guard['id'])
  actor=next(o for o in room['objects'] if o['object_identifier']==guard['id'])
  assert actor['object_type']=='OBJECTTYPE_TRAINER' and actor['radius']==guard['sight']
  assert 2<=len(data['trainers']['trainers'][guard['id']]['party'])<=3
  stations.setdefault(guard['map'],set()).add((x,y))
 print('PASS: 24 native sight grunts, floor stations and two/three-member parties.')
 w,h=source['dimensions_metatiles'];blocks=[int(t,16) for row in source['rows'] for t in row.split()]
 water=next(int(k,16) for k,v in data['tilesets']['cave']['collision'].items() if v==['WATER']*4)
 def collision(x,y,opened=False):
  if not(0<=x<w*2 and 0<=y<h*2):return 'WALL'
  block=blocks[y//2*w+x//2]
  if opened and (x//2,y//2)==(10,4):block=7
  return (data['tilesets']['cave']['collision'][f'{water:02x}'] if block==0x40 else data['tilesets']['facility']['collision'][f'{block:02x}'])[y%2*2+x%2]
 def reach(start,opened):
  q=deque([start]);seen={start}
  while q:
   x,y=q.popleft()
   for at in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
    if at not in seen and collision(*at,opened) in ['FLOOR','LADDER']:
     seen.add(at);q.append(at)
  return seen
 assert (23,7) not in reach((4,19),False),'closed chamber bypass'
 assert (23,7) in reach((4,19),True),'open gate does not reach chamber'
 assert (17,12) in reach((4,19),False),'drive approach unreachable'
 def steps(at,sequence,allow_water=False):
  x,y=at;visited=[]
  for dx,dy,count in sequence:
   for _ in range(count):
    x+=dx;y+=dy;assert (x,y) not in stations['MnBoundaryIntake'],('guard blocks acted scene',(x,y));assert collision(x,y,True) in (['FLOOR','WATER'] if allow_water else ['FLOOR']),('script wall',at,(x,y));visited.append((x,y))
  return (x,y),visited
 assert steps((17,12),[(1,0,1)])[0]==(18,12)
 assert steps((18,12),[(-1,0,5),(0,1,3)])[0]==(13,15)
 for x in range(2,20):
  assert steps((x,17),[(1,0,19-x),(0,-1,8),(-1,0,1)])[0]==(18,9)
 end,_=steps((23,7),[(0,1,2),(-1,0,4),(0,1,4),(1,0,3)],True);assert end==(22,13) and collision(*end)=='WATER'
 assert steps((9,5),[(0,1,10)])[0]==(9,15)
 assert steps((11,9),[(0,1,6)])[0]==(11,15)
 assert steps((15,9),[(0,1,8),(-1,0,6)])[0]==(9,17)
 for name,points in [('MnRochesterPokecenter1F',[(9,10),(7,7)]),('MnStPaulUnionDepot',[(22,17)]),('MnStCloud',[(24,5),(25,5),(25,4),(24,7),(25,9),(24,9),(29,12)]),('MnStCloudStation',[(15,16)]),('MnBoundaryWatersSurveyShelter',[(12,7)])]:
  m=data['maps'][name];ww=m['attributes']['width'];col=data['tilesets'][m['attributes']['tileset_name']]['collision']
  for x,y in points:
   assert col[f'{m["blocks"][y//2*ww+x//2]:02x}'][y%2*2+x%2]=='FLOOR',('actor/movement anchor',name,x,y)
 tiles=art.blocks('facility','day')+[art.blocks('cave','day')[water]]
 im=Image.new('RGB',(w*32,h*32));dr=ImageDraw.Draw(im)
 for y in range(h):
  for x in range(w):im.paste(tiles[blocks[y*w+x]],(x*32,y*32))
 for label,(x,y) in [('REED',(15,9)),('KETTLURK',(23,7)),('IVY',(9,5)),('FINN',(11,9)),('DRIVE',(17,11)),('OUTFLOW',(22,13)),('STAIR',(5,18))]:
  dr.ellipse((x*16+4,y*16+4,x*16+12,y*16+12),fill='#f54242');dr.text((x*16,y*16-10),label,fill='#bd1212')
 out=ROOT/'planning/minnesota/renders/story-arc-v3'/f'intake-preflight-{time.time_ns()}';out.mkdir(parents=True);im.save(out/'whole-intake.png')
 print('PASS: sealed/open intake, separate drive access, all creature/crew paths and chapter actor anchors.');print(out/'whole-intake.png')
if __name__=='__main__':
 a=argparse.ArgumentParser();a.add_argument('pack');review(a.parse_args().pack)
