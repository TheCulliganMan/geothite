#!/usr/bin/env python3
"""Three distinct rounded mole capsules emerging through one grounded soil bed."""
import math
from sculpt_geometry import loft,curved_profiles,ellipsoid,part,translate
from raptor_bird_sculpt import paint
from bovine_sculpt import oval
from diglett_sculpt import BROWN,RED,INK,WHITE
from skin_glb import COORDINATES
HEIGHT=.98
# Rear tall, left middle, foreground short: each has its own continuous body.
MOLES=((.145,-.080,.755,.158,.148),(-.170,.005,.565,.166,.155),(.075,.210,.405,.151,.140))
def sculpture(*,normalize=True):
 p=[]
 for j,(x,z,h,rx,rz) in enumerate(MOLES):
  dome_height=rx*1.15;shoulder=h-dome_height
  profiles=curved_profiles(((.006,rx*.91,rz*.91,0.),(.10,rx,rz,0.),(shoulder,rx,rz,0.)),3)
  for i in range(1,13):
   a=(math.pi/2-.010)*i/12;profiles.append((shoulder+dome_height*math.sin(a),rx*math.cos(a),rz*math.cos(a),0.))
  body=loft(f'Body / continuous emerging capsule {j}',BROWN,profiles,24);local=[body]
  ey=h-.155;eye_width=.0115;eye_height=.027
  for side in (-1,1):
   eye=paint(f'Eye / fitted black gaze {side} {j}',INK,oval(side*rx*.33,ey,eye_width,eye_height,16),body,.005,2);local.append(eye)
   local.append(paint(f'Eye glint / fitted white highlight {side} {j}',WHITE,oval(side*rx*.33-.0025,ey+.009,.003,.005,12),eye,.002,2))
  local.append(ellipsoid(f'Nose / rooted broad red oval {j}',RED,(0.,h-.237,rz+.012),(.060,.031,.043),20,12))
  local.append(paint(f'Forehead glint / fitted cap highlight {j}',WHITE,oval(-.031,h-.065,.012,.009,16),body,.004,2))
  p.extend(translate(q,x,0.,z) for q in local)
 for j in range(12):
  a=math.tau*j/12;x=.344*math.cos(a);z=.071+.322*math.sin(a)
  rx=.063+.016*(j%3);rz=.060+.009*((j//3)%4);ry=.041+.005*(j%2)
  name=f'Soil / separate disturbed perimeter clod {j}';q=ellipsoid(name,RED,(x,ry,z),(rx,ry,rz),10,6)
  vs=[tuple(q['positions'][i:i+3]) for i in range(0,len(q['positions']),3)];fs=[tuple(q['indices'][i:i+3]) for i in range(0,len(q['indices']),3)]
  p.append(part(name,RED,vs,fs,False))
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return dict(name='dugtrio',version=1,coordinate_system=COORDINATES,primitives=p)
def body_reference_span():
 ps=[p for p in sculpture()['primitives'] if p['part'].startswith('Body /')];return max(v for p in ps for v in p['positions'][1::3])-min(v for p in ps for v in p['positions'][1::3])
if __name__=='__main__':
 import argparse
 from pathlib import Path
 from dugtrio_glb import export_dugtrio
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_dugtrio())
