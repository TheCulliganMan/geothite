#!/usr/bin/env python3
"""Original athletic fire unicorn with a raised foreleg and open equine jaw."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,translate
from hoofed_sculpt import trunk
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from ponyta_sculpt import flame,CREAM,INNER,INK,WHITE,HOOF
from skin_glb import COORDINATES
HEIGHT=1.30
RED=(.77,.27,.24);PINK=(.85,.48,.51)

def anatomy():
 body=trunk('Body / long athletic cream Rapidash torso',CREAM,curved_profiles(((-.56,.034,.057,.854),(-.44,.197,.229,.863),(-.23,.24,.247,.873),(.04,.231,.259,.882),(.24,.187,.23,.913),(.37,.094,.16,.957),(.39,.029,.055,.991)),5))
 neck=loft('Neck / long arched cream Rapidash neck and chest',CREAM,curved_profiles(((.755,.081,.109,.242),(.935,.177,.197,.260),(1.15,.124,.139,.308),(1.39,.098,.114,.353),(1.585,.115,.135,.332),(1.716,.101,.116,.332)),5),40)
 head=ellipsoid('Head / rooted proportioned cream Rapidash skull',CREAM,(0,1.696,.382),(.166,.187,.210),40,26)
 muzzle=ellipsoid('Muzzle / rooted long cream Rapidash upper snout',CREAM,(0,1.670,.578),(.124,.081,.190),40,26)
 jaw=ellipsoid('Jaw / open rooted cream Rapidash lower jaw',CREAM,(0,1.530,.556),(.119,.042,.154),36,24)
 back=ellipsoid('Mouth back / recessed dark Rapidash oral chamber',INK,(0,1.591,.488),(.081,.085,.034),32,22)
 tongue=ellipsoid('Tongue / rooted pink Rapidash tongue',PINK,(0,1.567,.602),(.065,.012,.066),32,20);p=[body,neck,head,muzzle,jaw,back,tongue]
 for s in (-1,1):
  ear=paddle('Ear / rooted pointed cream Rapidash pinna '+str(s),CREAM,[(s*.102,1.79,.309),(s*.173,1.924,.275),(s*.211,1.987,.264)],[.057,.037,.0015],[.054,.028,.0015]);p.append(ear)
  p.append(paint('Ear inset / fitted dark pink Rapidash inner ear '+str(s),INNER,[(s*x,y) for x,y in ((.131,1.839),(.17,1.919),(.195,1.958),(.199,1.89),(.169,1.825))],ear,.005,3))
  xy=[(s*x,y) for x,y in ((.065,1.760),(.133,1.796),(.143,1.749),(.106,1.723),(.069,1.734))]
  socket=paint('Socket / fitted narrow dark Rapidash gaze '+str(s),INK,xy,head,.008,3);p.append(socket);cx=sum(x for x,y in xy)/5;cy=sum(y for x,y in xy)/5
  eye=paint('Eye / fitted narrow white Rapidash stare '+str(s),WHITE,[(cx+.85*(x-cx),cy+.78*(y-cy)) for x,y in xy],socket,.005,3);p.append(eye)
  iris=paint('Iris / fitted red Rapidash iris '+str(s),RED,oval(s*.109,1.756,.010,.018,24),eye,.003,2);p.append(iris)
  p.append(paint('Pupil / fitted dark Rapidash slit '+str(s),INK,oval(s*.11,1.756,.0035,.013,20),iris,.003,2))
  p.append(paint('Nostril / fitted small Rapidash nostril '+str(s),INK,oval(s*.047,1.697,.005,.007,20),muzzle,.003,2))
  for row,z in enumerate((.259,-.351)):
   raised=s==-1 and row==0
   if row==0:
    p.append(ellipsoid('Shoulder / rooted rounded cream Rapidash shoulder '+str(s),CREAM,(s*.118,.883,.237),(.116,.179,.145),36,24))
    points=[(s*.13,.923,z),(s*.232,.79,z+.11),(s*.266,.733,z+.281),(s*.248,.521,z+.228)] if raised else [(s*.13,.91,z),(s*.185,.677,z+.009),(s*.19,.383,z+.053),(s*.193,.085,z+.097)];rr=[.096,.078,.051,.037] if raised else [.096,.076,.047,.034]
   else:points=[(s*.124,.874,z),(s*.195,.667,z-.077),(s*.217,.443,z+.026),(s*.214,.240,z-.032),(s*.211,.085,z+.033)];rr=[.109,.094,.065,.043,.035]
   p.append(curved_tube(f'Leg / sculpted athletic cream Rapidash limb {s} {row}',CREAM,points,rr,28))
   x,y,zz=points[-1];zz+=.023;floor=y-.085
   p.append(translate(loft(f'Hoof / dark blue Rapidash hoof {s} {row}',HOOF,[(floor,.051,.071,zz),(floor+.037,.064,.085,zz),(floor+.082,.055,.068,zz-.005),(floor+.117,.039,.044,zz-.01)],24),x,0,0))
   # Each fire bank follows its actual supporting or raised lower limb.
   if raised:
    center_y=.685;center_z=.443;fx=-.255
   else:center_y=.48;center_z=z+.02;fx=s*.194
   yz=[(center_y+dy,center_z+dz) for dy,dz in ((-.18,.003),(-.09,.096),(-.025,.075),(.067,.151),(.045,.055),(.135,.056),(.104,.003),(.169,-.088),(.063,-.053),(.015,-.125),(-.050,-.079),(-.135,-.081))]
   core=[(center_y+dy,center_z+dz) for dy,dz in ((-.092,-.025),(-.055,.037),(0,.043),(.055,.021),(.09,.016),(.061,-.021),(.014,-.041),(-.044,-.050))]
   p.extend(flame(f'ankle {s} {row}',yz,fx,core,.037))
 horn=curved_tube('Horn / rooted long ivory Rapidash forehead horn',CREAM,[(0,1.809,.464),(0,1.951,.524),(0,2.103,.577)],[.045,.027,.0015],28);p.append(horn)
 p.extend(flame('crown',[(1.780,.311),(1.915,.335),(2.046,.209),(2.006,.14),(2.099,.063),(2.019,.067),(2.068,-.063),(1.963,-.045),(1.968,-.169),(1.872,-.091),(1.794,.090)],0,[(1.823,.229),(1.899,.235),(1.987,.176),(1.966,.107),(2.024,.072),(1.954,.071),(1.921,-.040),(1.866,.025),(1.832,.139)],.064))
 p.extend(flame('mane',[(.966,.125),(1.173,.202),(1.432,.237),(1.707,.229),(1.835,.270),(1.863,.103),(1.819,-.018),(1.736,-.073),(1.731,-.229),(1.645,-.178),(1.597,-.338),(1.516,-.234),(1.451,-.4),(1.40,-.283),(1.284,-.414),(1.27,-.307),(1.154,-.400),(1.14,-.235),(1.052,-.262),(.987,-.107)],0,[(1.033,.075),(1.242,.148),(1.494,.172),(1.766,.172),(1.795,.059),(1.719,-.05),(1.665,-.126),(1.595,-.112),(1.518,-.210),(1.454,-.190),(1.342,-.240),(1.292,-.207),(1.192,-.266),(1.17,-.149),(1.06,-.14)],.058))
 p.append(curved_tube('Tail stem / rooted cream Rapidash tail',CREAM,[(0,.919,-.459),(0,.982,-.613),(0,1.038,-.733)],[.081,.058,.039],28))
 p.extend(flame('tail',[(.964,-.575),(1.093,-.687),(1.218,-.826),(1.162,-.897),(1.252,-1.00),(1.179,-1.006),(1.224,-1.145),(1.11,-1.109),(1.129,-1.272),(1.019,-1.207),(.973,-1.35),(.903,-1.25),(.894,-1.144),(.802,-1.208),(.808,-1.072),(.741,-1.069),(.812,-.934),(.795,-.799),(.911,-.781)],0,[(.962,-.733),(1.06,-.735),(1.156,-.84),(1.107,-.91),(1.172,-1.01),(1.101,-1.05),(1.071,-1.147),(1.006,-1.127),(.95,-1.209),(.939,-1.099),(.862,-1.115),(.89,-1.004),(.853,-.947),(.89,-.84)],.062,glow_offset=.018))
 return p

def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'rapidash','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'rapidash.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
