#!/usr/bin/env python3
"""Original long rock serpent with an arched chain and a wedge-shaped face."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,loft,curved_profiles,crease_normals,translate
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint
from murkrow_sculpt import bill
from skin_glb import COORDINATES
HEIGHT=1.72
# Verified Crystal normal/front/back RGB555 stone (17,15,20), shadow (9,6,11).
STONE=(140/255,123/255,165/255);DARK=(74/255,49/255,90/255)
LIGHT=tuple(v*.85+.15 for v in STONE);IVORY=(1.,1.,1.);INK=(0.,0.,0.)
CHAIN=((.17,.028,.20,.027),(.105,.047,.21,.049),(0,.066,.21,.070),(-.12,.109,.16,.088),(-.22,.135,.055,.110),(-.32,.168,-.015,.130),(-.37,.232,-.20,.148),(-.27,.359,-.35,.169),(-.045,.533,-.42,.197),(.17,.748,-.38,.220),(.28,.980,-.26,.240),(.22,1.210,-.11,.249),(-.025,1.330,-.005,.251))

def rock(j,c):
 x,y,z,r=c;color=STONE
 base=ellipsoid('Rock segment / original irregular Onix boulder '+str(j),color,(x,y,z),(r,r*.94,r*1.02),12,7);vv=[]
 for i in range(0,len(base['positions']),3):
  px,py,pz=base['positions'][i:i+3];a=math.atan2(pz-z,px-x);q=(py-y)/(r*.94);bump=1+.075*math.sin(3*a+j*.71)*max(0,1-q*q)+.035*math.cos(5*a-j)*max(0,1-q*q)
  vv.append((x+(px-x)*bump,py,z+(pz-z)*bump))
 return part(base['part'],color,vv,[tuple(base['indices'][i:i+3]) for i in range(0,len(base['indices']),3)],False)

def anatomy():
 p=[rock(j,c) for j,c in enumerate(CHAIN)]
 # Thin buried connectors provide volume roots without exposed straight rods.
 for j,(a,b) in enumerate(zip(CHAIN,CHAIN[1:])):
  p.append(curved_tube('Joint / buried dark Onix rock connection '+str(j),DARK,[a[:3],b[:3]],[min(a[3],b[3])*.42]*2,20))
 head=translate(crease_normals(loft('Head / angular wedge-shaped grey Onix face',STONE,curved_profiles(((.898,.055,.109,.124),(.967,.139,.173,.124),(1.090,.222,.208,.075),(1.251,.216,.157,.016),(1.347,.089,.079,-.013)),2),12),15),-.306,0,0);p.append(head)
 for s in (-1,1):
  xy=[(-.306+s*x,y) for x,y in ((.032,1.167),(.154,1.191),(.140,1.112),(.066,1.104))]
  socket=paint('Eye socket / fitted angular dark Onix recess '+str(s),INK,xy,head,.006,3);p.append(socket);cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted narrow ivory Onix gaze '+str(s),IVORY,[(cx+.88*(x-cx),cy+.80*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  p.append(paint('Pupil / fitted small dark Onix stare '+str(s),INK,oval(-.306+s*.101,1.145,.005,.010,24),eye,.003,2))
 nose=bill('Nose ridge / rooted pointed grey Onix face keel',LIGHT,[(.202,.067,.981,1.176),(.291,.040,.947,1.122),(.333,.006,.934,.946)])
 nose=translate(nose,-.306,0,0);p.append(nose)
 jaw=bill('Jaw / rooted dark angular open Onix lower jaw',DARK,[(.122,.149,.914,.973),(.215,.145,.872,.906),(.294,.079,.887,.911)])
 p.append(translate(jaw,-.306,0,0))
 p.append(ellipsoid('Mouth back / rooted recessed dark Onix chamber',INK,(-.306,.967,.169),(.119,.061,.036),32,22))
 p.append(curved_tube('Horn / rooted long narrow grey Onix forehead spike',LIGHT,[(-.306,1.272,.077),(-.297,1.486,.085),(-.289,1.698,.099)],[.044,.026,.002],12))
 return p

def body_reference_span():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);points=[q[:3] for q in CHAIN]+[(-.306,1.12,.10)]
 return sum(math.dist(a,b) for a,b in zip(points,points[1:]))*HEIGHT/(hi-lo)

def sculpture():
 p=anatomy()
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'big_onix','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from onix_glb import export_onix
 (args.output/'onix.glb').write_bytes(export_onix())
if __name__=='__main__':main()
