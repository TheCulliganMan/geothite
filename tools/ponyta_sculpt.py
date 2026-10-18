#!/usr/bin/env python3
"""Original slender cream pony with dark hooves and flowing two-sided fire."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from sculpt_geometry import ellipsoid,loft,curved_profiles,translate,part,crease_normals,fitted_adaptive_contour
from hoofed_sculpt import trunk
from bovine_sculpt import curved_tube,oval,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint,sheet
from moltres_sculpt import flowing
from cubone_sculpt import skin_shading
from goldfish_sculpt import zy,surface
from skin_glb import COORDINATES
HEIGHT=1.10
CREAM=(.98,.95,.78);INNER=(.61,.37,.33);INK=(.09,.075,.08);WHITE=(.99,.99,.95);HOOF=(.18,.26,.38)
FIRE=(.98,.45,.22);GOLD=(1.,.82,.29)

def flame(name,yz,x,core,th=.036,glow_offset=None):
 xy=flowing([(-z,y) for y,z in yz],4);base=sheet('Flame / '+name,FIRE,xy,lambda y,z:x,th,1);vertices=[]
 # Rounded fire volumes taper at the outline and billow across their faces.
 for i in range(0,len(base['positions']),3):
  u,v,w=base['positions'][i:i+3];distance=1e9
  for a,b in zip(xy,xy[1:]+xy[:1]):
   du,dv=b[0]-a[0],b[1]-a[1];dd=du*du+dv*dv;t=max(0,min(1,((u-a[0])*du+(v-a[1])*dv)/dd)) if dd else 0
   distance=min(distance,math.hypot(u-a[0]-t*du,v-a[1]-t*dv))
  vertices.append((u,v,x+(w-x)*(.18+.82*min(1,distance/.036))))
 base=crease_normals(part(base['part'],FIRE,vertices,[tuple(base['indices'][i:i+3]) for i in range(0,len(base['indices']),3)]),85);parts=[base]
 core_xy=flowing([(-z,y) for y,z in core],2)
 for face in (-1,1):
  parent=base if face==1 else reverse_front(base);xy=core_xy if face==1 else [(-x,y) for x,y in core_xy]
  patch=fitted_adaptive_contour('Flame glow / '+name+' '+str(face),GOLD,xy,parent,glow_offset if glow_offset is not None else (.015 if name=='mane' else .012),surface_at=surface(parent),initial_subdivisions=1)
  patch=skin_shading(patch,parent)
  parts.append(patch if face==1 else reverse_front(patch))

 return [zy(p) for p in parts]

@lru_cache(maxsize=1)
def anatomy():
 body=trunk('Body / compact horizontal cream Ponyta torso',CREAM,curved_profiles(((-.43,.03,.046,.675),(-.34,.168,.169,.678),(-.19,.207,.20,.68),(.05,.203,.209,.687),(.22,.169,.20,.698),(.34,.09,.139,.727),(.35,.025,.050,.75)),5))
 neck=loft('Neck / rooted arched cream Ponyta neck and chest',CREAM,curved_profiles(((.60,.08,.105,.21),(.73,.155,.177,.205),(.91,.117,.125,.236),(1.10,.086,.104,.273),(1.29,.097,.113,.283)),5),36)
 head=loft('Head / rooted youthful long cream Ponyta face',CREAM,curved_profiles(((1.11,.07,.114,.350),(1.18,.137,.175,.354),(1.29,.151,.176,.337),(1.38,.107,.132,.304),(1.416,.023,.025,.298)),4),32)
 muzzle=ellipsoid('Muzzle / rooted long modest cream Ponyta nose',CREAM,(0,1.204,.504),(.132,.083,.164),28,18);p=[body,neck,head,muzzle]
 for s in (-1,1):
  ear=paddle('Ear / rooted pointed cream Ponyta pinna '+str(s),CREAM,[(s*.098,1.351,.258),(s*.176,1.459,.243),(s*.229,1.503,.228)],[.052,.038,.0015],[.050,.026,.0015]);p.append(ear)
  p.append(paint('Ear inset / fitted dark pink Ponyta inner ear '+str(s),INNER,[(s*x,y) for x,y in ((.121,1.390),(.163,1.462),(.205,1.484),(.196,1.433),(.158,1.386))],ear,.005,2))
  eye=fitted_adaptive_contour('Eye / fitted broad dark Ponyta gaze '+str(s),INK,oval(s*.095,1.303,.039,.057,36),head,.007,surface_at=surface(head),initial_subdivisions=1);p.append(eye)
  p.append(paint('Eye glint / fitted white Ponyta gleam '+str(s),WHITE,oval(s*.090,1.326,.010,.014,24),eye,.003,1))
  p.append(paint('Nostril / fitted tiny Ponyta nostril '+str(s),INK,oval(s*.055,1.220,.004,.0055,20),muzzle,.003,1))
  for row,z in enumerate((.21,-.26)):
   if row==0:
    p.append(ellipsoid('Shoulder / rooted rounded cream Ponyta shoulder '+str(s),CREAM,(s*.10,.692,.190),(.102,.149,.129),24,16))
    points=[(s*.104,.715,z),(s*.158,.540,z+.02),(s*.167,.34,z+.052),(s*.173,.09,z+.078)];rr=[.081,.065,.041,.032]
   else:points=[(s*.10,.68,z),(s*.161,.52,z-.06),(s*.172,.35,z+.012),(s*.181,.20,z-.028),(s*.18,.085,z+.025)];rr=[.088,.078,.058,.039,.032]
   p.append(curved_tube(f'Leg / slender rooted cream Ponyta limb {s} {row}',CREAM,points,rr,24))
   x=points[-1][0];zz=points[-1][2]+.021
   hoof=translate(loft(f'Hoof / grounded dark blue Ponyta hoof {s} {row}',HOOF,[(0,.048,.065,zz),(.038,.060,.079,zz),(.078,.052,.064,zz-.004),(.111,.036,.040,zz-.009)],24),x,0,0);p.append(hoof)
   # Lower-leg fire grows from the real upper ankle rather than the floor.
   p.extend(flame(f'ankle {s} {row}',[(.28,z+.001),(.36,z+.087),(.445,z+.062),(.52,z+.11),(.51,z+.032),(.595,z+.021),(.565,z-.013),(.60,z-.068),(.505,z-.056),(.475,z-.108),(.427,z-.071),(.351,z-.076)],s*.162,[(.37,z-.025),(.40,z+.030),(.45,z+.037),(.495,z+.020),(.55,z+.005),(.522,z-.024),(.493,z-.036),(.45,z-.046),(.40,z-.05)],.033))
 p.append(paint('Mouth seam / fitted modest closed Ponyta smile',INK,[(-.061,1.164),(0,1.154),(.061,1.164),(.046,1.158),(0,1.149),(-.046,1.158)],muzzle,.003,1))
 p.extend(flame('forelock',[(1.349,.419),(1.436,.470),(1.518,.386),(1.535,.307),(1.61,.251),(1.614,.152),(1.696,.052),(1.616,.069),(1.591,-.014),(1.52,.055),(1.460,.167),(1.395,.279)],0,[(1.42,.34),(1.47,.37),(1.54,.25),(1.56,.19),(1.60,.105),(1.55,.12),(1.52,.18),(1.47,.25),(1.41,.31)],.061))
 p.extend(flame('mane',[(.770,.150),(.894,.178),(1.075,.184),(1.285,.19),(1.377,.218),(1.348,.115),(1.26,.033),(1.244,.084),(1.136,.010),(1.146,.063),(1.031,-.020),(1.002,.039),(.898,-.081),(.846,-.026),(.794,-.178),(.745,-.177)],0,[(.80,.13),(.92,.13),(1.12,.13),(1.29,.18),(1.25,.12),(1.16,.075),(1.09,.083),(.98,.05),(.90,.03),(.85,-.02),(.80,.02)],.043))
 stem=curved_tube('Tail stem / rooted short cream Ponyta tail',CREAM,[(0,.719,-.342),(0,.755,-.477),(0,.824,-.561)],[.070,.051,.031],24);p.append(stem)
 p.extend(flame('tail',[(.751,-.465),(.856,-.516),(.989,-.555),(1.068,-.637),(1.028,-.685),(1.110,-.756),(1.035,-.765),(1.059,-.88),(.974,-.823),(.936,-.960),(.890,-.868),(.833,-.859),(.805,-.75),(.728,-.682)],0,[(.80,-.59),(.87,-.58),(.94,-.64),(1.02,-.67),(.98,-.73),(.95,-.80),(.87,-.80),(.84,-.70),(.79,-.66)],.059))
 return p

@lru_cache(maxsize=2)
def sculpture(normalize=True):
 import copy
 p=copy.deepcopy(anatomy());lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo) if normalize else 1.
 for q in p:
  q['positions']=[round((v-(lo if normalize and i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'ponyta','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);from fire_horse_glb import export_ponyta; (args.output/'ponyta.glb').write_bytes(export_ponyta())
if __name__=='__main__':main()
