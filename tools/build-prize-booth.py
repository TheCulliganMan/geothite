#!/usr/bin/env python3
"""Original closed Prize Room booth geometry; no game artwork is exported."""
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
PAINT={'blue_enamel':[.24,.24,.78,1.],'green_enamel':[.12,.32,.012,1.],'gold_enamel':[.76,.50,.14,1.],'brass':[.52,.34,.08,1.],'timber':[.24,.12,.035,1.],'paper':[.93,.92,.82,1.],'display':[.035,.055,.12,1.]}
def document():
 parts=[]
 def box(name,b,material):
  x0,x1,y0,y1,z0,z1=b
  assert x1>x0 and y1>y0 and z1>z0
  faces=[([(x1,y0,z0),(x1,y1,z0),(x1,y1,z1),(x1,y0,z1)],(1,0,0)),([(x0,y0,z1),(x0,y1,z1),(x0,y1,z0),(x0,y0,z0)],(-1,0,0)),([(x0,y1,z0),(x0,y1,z1),(x1,y1,z1),(x1,y1,z0)],(0,1,0)),([(x0,y0,z1),(x0,y0,z0),(x1,y0,z0),(x1,y0,z1)],(0,-1,0)),([(x0,y0,z1),(x1,y0,z1),(x1,y1,z1),(x0,y1,z1)],(0,0,1)),([(x1,y0,z0),(x0,y0,z0),(x0,y1,z0),(x1,y1,z0)],(0,0,-1))]
  positions=[];normals=[];indices=[]
  for points,normal in faces:
   base=len(positions)//3;positions.extend(v for point in points for v in point);normals.extend(normal*4);indices.extend(base+i for i in [0,1,2,0,2,3])
  parts.append(dict(name=name,positions=positions,normals=normals,indices=indices,base_color=PAINT[material],material=material))
 box('Closed green counter carcass',(0,96,0,8,0,7.9),'green_enamel')
 box('Golden continuous service top',(0,96,8,10,0,8),'gold_enamel')
 box('Blue lower wall course',(0,96,10,15,0,2.5),'blue_enamel')
 box('Blue upper wall course',(0,96,25,32,0,2.5),'blue_enamel')
 for a,b in [(0,32),(48,64),(80,96)]:box('Blue pier between service openings',(a,b,15,25,0,2.5),'blue_enamel')
 box('Timber wall coping',(0,96,31.2,32,0,2.8),'timber')
 for x in [32,64]:
  box('Opaque service display backing',(x,x+16,15,25,2.4,2.7),'display')
  for a,b in [(x,x+1),(x+15,x+16)]:box('Service window vertical frame',(a,b,15,25,2.5,3.2),'brass')
  for a,b in [(15,16),(24,25)]:box('Service window horizontal frame',(x,x+16,a,b,2.5,3.2),'brass')
  box('Service ledge under opening',(x-1,x+17,14.1,14.8,2.4,5.2),'gold_enamel')
 box('Sign closed paper backing',(1,15,16,31,2.5,2.8),'paper')
 for a,b in [(0.5,1.5),(14.5,15.5)]:box('Sign vertical timber frame',(a,b,15.5,31.5,2.5,3.1),'timber')
 for a,b in [(15.5,16.5),(30.5,31.5)]:box('Sign horizontal timber frame',(.5,15.5,a,b,2.5,3.1),'timber')
 for x in range(6,96,8):box('Source-inspired dotted counter decoration',(x-.5,x+.5,3,4,7.9,7.98),'gold_enamel')
 return dict(name='prize_booth',coordinate_system='right-handed Y-up',origin=[0,0,0],bounds=dict(min=[0,0,0],max=[96,32,8]),triangle_count=sum(len(p['indices'])//3 for p in parts),primitives=parts)
if __name__=='__main__':
 p=ROOT/'crates/crystal-voxel-view/models/interiors/prize_booth.mesh.json';p.write_text(json.dumps(document(),separators=(',',':'))+'\n');print(p)
