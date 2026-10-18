#!/usr/bin/env python3
"""Original armored rhinoceros; every dorsal plate seats inside its barrel."""
import argparse,json,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import ellipsoid,loft,curved_profiles,translate,subdivided_part,fitted_contour,fitted_patch,crease_normals
from bovine_sculpt import curved_tube,oval
from abra_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.11
GRAY=(.64,.67,.69);PLATE=(.70,.73,.74);SHADOW=(.47,.51,.54);INK=(.09,.12,.14);IVORY=(.93,.94,.89);RED=(.73,.23,.20)

def longitudinal(name,color,profiles,segments=40,steps=4):
 p=loft(name,color,curved_profiles([(z,rx,ry,-y) for z,y,rx,ry in profiles],steps),segments)
 for key in ('positions','normals'):
  p[key]=[v for i in range(0,len(p[key]),3) for v in (p[key][i],-p[key][i+2],p[key][i+1])]
 return p

def armor(name,color,z,width,height,base=.46,depth=.072):
 # A thick closed pentagonal plate: its lower half intersects the actual
 # barrel, rather than hanging above it as the older disconnected ridge did.
 cross=[(-width,base),(width,base),(width,base+.125),(0,height),(-width,base+.125)]
 v=[(x,y,zz) for zz in (z-depth,z+depth) for x,y in cross];n=len(cross)
 f=[tuple(reversed(range(n))),tuple(range(n,2*n))]
 f.extend((i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n))
 return crease_normals(subdivided_part(name,color,v,f,1,True),42)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=ellipsoid('Body / continuous heavy gray rhinoceros barrel',GRAY,(0,.425,-.14),(.365,.265,.47),48,28)
 head=longitudinal('Head / lowered broad armored wedge',PLATE,[(-.02,.51,.19,.16),(.10,.54,.31,.235),(.29,.53,.335,.23),(.45,.465,.295,.17),(.61,.395,.21,.105),(.665,.395,.035,.065)])
 p=[body,head]
 p.append(longitudinal('Jaw / rooted broad lower armored muzzle',SHADOW,[(.21,.335,.175,.065),(.45,.313,.255,.07),(.62,.325,.183,.063),(.649,.33,.025,.033)]))
 p.append(curved_tube('Horn / rooted swept stone nasal lance',PLATE,[(0,.477,.475),(0,.585,.551),(0,.710,.595),(0,.795,.615)],[.103,.082,.038,.0015],32))
 for j,(z,w,h) in enumerate(((-.46,.22,.84),(-.31,.31,.96),(-.15,.35,.99),(.012,.31,.94))):
  p.append(armor('Back armor / seated pentagonal dorsal plate '+str(j),PLATE,z,w,h))
 p.append(armor('Brow armor / seated central forehead crest',PLATE,.255,.245,.77,base=.51,depth=.072))
 hs=surface(head)
 for sign in (-1,1):
  p.append(curved_tube('Ear / rooted pointed stone pinna '+str(sign),GRAY,[(sign*.24,.646,.106),(sign*.302,.750,.050),(sign*.303,.832,-.005)],[.084,.054,.0015],24))
  x=sign*.239;y=.554
  xy=[(x+sign*dx,y+dy) for dx,dy in ((-.045,.023),(.035,.040),(.044,-.008),(-.032,-.024))]
  socket=crease_normals(fitted_contour('Eye socket / fitted angular black recess '+str(sign),INK,xy,hs,.006,subdivisions=3),35);p.append(socket)
  cx=sum(a for a,b in xy)/len(xy);cy=sum(b for a,b in xy)/len(xy)
  white=crease_normals(fitted_contour('Eye white / narrow watchful ivory '+str(sign),IVORY,[(cx+(a-cx)*.78,cy+(b-cy)*.70) for a,b in xy],surface(socket),.003,subdivisions=3),35);p.append(white)
  iris=crease_normals(fitted_patch('Iris / fitted red rhino gaze '+str(sign),RED,oval(cx-sign*.005,cy,.010,.016,20),surface(white),.002),35);p.append(iris)
  p.append(crease_normals(fitted_patch('Pupil / fitted dark rhino slit '+str(sign),INK,oval(cx-sign*.005,cy,.004,.011,20),surface(iris),.0015),35))
  p.append(crease_normals(fitted_patch('Nostril / fitted dark nasal opening '+str(sign),INK,oval(sign*.107,.434,.021,.009,28),hs,.004),35))
 for sign in (-1,1):
  for front in (False,True):
   z=.13 if front else -.37;name=str(sign)+' '+('front' if front else 'rear')
   leg=translate(loft('Leg / stocky rooted armored pillar '+name,GRAY,curved_profiles(((.035,.065,.07,z+.02),(.12,.095,.10,z+.015),(.245,.105,.113,z),(.365,.101,.11,z-.014),(.46,.065,.07,z-.02)),4),32),sign*.255,0,0);p.append(leg)
   p.append(translate(loft('Leg armor / rooted tapered knee cuff '+name,PLATE,[(.17,.093,.101,z+.005),(.23,.11,.119,z),(.285,.099,.106,z-.01)],32),sign*.255,0,0))
   foot=ellipsoid('Foot / broad grounded gray rhino pad '+name,GRAY,(sign*.255,.065,z+.038),(.120,.065,.139),32,20);p.append(foot);fs=surface(foot)
   for j,dx in enumerate((-.064,0,.064)):
    p.append(crease_normals(fitted_patch('Claw / supported blunt ivory toe '+name+' '+str(j),IVORY,oval(sign*.255+dx,.048,.025,.028,28),fs,.006),35))
 p.append(curved_tube('Tail / short rooted rear stone point',GRAY,[(0,.43,-.50),(0,.44,-.615),(.015,.47,-.725)],[.074,.05,.002],28))
 return {'name':'rhyhorn','version':1,'coordinate_system':COORDINATES,'primitives':p}
def sculpture(normalize=True):
 m=deepcopy(raw_sculpture());p=m['primitives']
 if not normalize:return m
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:
  q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return m

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,required=True);args=a.parse_args();args.out.mkdir(parents=True,exist_ok=True)
 from rhino_glb import export_rhyhorn
 blob=export_rhyhorn();(args.out/'rhyhorn.glb').write_bytes(blob);print(len(blob),'bytes')
if __name__=='__main__':main()
