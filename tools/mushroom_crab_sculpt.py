#!/usr/bin/env python3
"""Original Paras and Parasect: four walking legs, tapered foreclaws and spotted fungal caps."""
import argparse,math
from pathlib import Path
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import part,ellipsoid,tube
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from goldfish_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('paras','parasect');HEIGHTS={'paras':.78,'parasect':1.14}
ORANGE=(.91,.49,.27);LIGHT=(.98,.61,.34);RUSSET=(.63,.34,.22)
PINK=(.77,.45,.59);GOLD=(.98,.82,.43);IVORY=(.95,.92,.81);UNDERSIDE=(.69,.57,.55)
WHITE=(.94,.97,1.);INK=(.10,.12,.14)
CAPS={'paras':((-1,(-.145,.445,-.065),(.146,.141),.115),(1,(.145,.470,-.105),(.157,.150),.124)),
 'parasect':((0,(0.,.360,-.055),(.485,.430),.440),)}

def cap_local(label,radii,height):
 n=64;v=[];rings=(.006,.10,.22,.37,.52,.67,.80,.90,1.)
 for r in rings:
  for j in range(n):
   a=math.tau*j/n;h=.018+height*(1.-r**1.9)**.65
   v.append((r*radii[0]*math.sin(a),r*radii[1]*math.cos(a),h))
 # The underside closes the umbrella at a shallow raised central navel.
 for r in reversed(rings):
  for j in range(n):
   a=math.tau*j/n;v.append((r*radii[0]*math.sin(a),r*radii[1]*math.cos(a),-.006+.020*r*r))
 f=[tuple(reversed(range(n))),tuple((len(rings)*2-1)*n+j for j in range(n))]
 for k in range(len(rings)*2-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return part('Mushroom cap / domed pink fungal umbrella '+str(label),PINK,v,f,True)

def horizontal(p,center):
 for field in ('positions','normals'):
  values=[]
  for i in range(0,len(p[field]),3):
   x,y,z=p[field][i:i+3];q=(x,z,-y)
   values.extend(q[j]+(center[j] if field=='positions' else 0.) for j in range(3))
  p[field]=values
 return p

def mushrooms(species):
 adult=species=='parasect';p=[]
 for label,center,radii,height in CAPS[species]:
  cap=cap_local(label,radii,height);skin=surface(cap);local=[cap]
  spots=([(-.63,-.40,.16,.12),(.0,-.67,.23,.10),(.63,-.39,.15,.13),(-.70,.14,.11,.17),(.19,.12,.20,.13),(.65,.25,.12,.16),(-.27,.62,.14,.10),(.28,.60,.10,.13),(-.16,-.25,.19,.12)] if adult else [(-.40,.32,.21,.15),(.30,.38,.18,.13),(.03,-.25,.15,.17),(-.59,-.32,.13,.15),(.60,-.25,.16,.16)])
  for j,(x,y,rx,ry) in enumerate(spots):
   local.append(paint(f'Mushroom spot / seated golden cap marking {label} {j}',GOLD,oval(x*radii[0],y*radii[1],rx*radii[0],ry*radii[1],20),cap,.0018 if not adult else .003,2))
  underside=ellipsoid('Mushroom underside / inset pale gill disk '+str(label),UNDERSIDE,(0.,0.,-.004),(radii[0]*.92,radii[1]*.92,.010),32,16);local.append(underside)
  for j in range(4 if not adult else 8):
   a=(j+.25)*math.tau/(4 if not adult else 8);points=[]
   for r in (.17,.35,.55,.77):
    x=r*radii[0]*math.sin(a);y=r*radii[1]*math.cos(a);z=-.004-.010*math.sqrt(1.-(r/.92)**2)-.0002;points.append((x,y,z))
   local.append(tube(f'Mushroom gill / fine radial underside seam {label} {j}',RUSSET,points,[.001]*4,6))
  p.extend(horizontal(q,center) for q in local)
  x,y,z=center;root=(x*.75,.340 if adult else .318,z)
  p.append(curved_tube('Mushroom stalk / rooted cream fungal stem '+str(label),IVORY,[root,(x,(y+.340)/2 if adult else y-.06,z),center],[.065 if adult else .022,.060 if adult else .021,.055 if adult else .020],20))
 return p

@lru_cache(maxsize=2)
def raw_sculpture(species):
 if species not in SPECIES:raise ValueError(species)
 adult=species=='parasect';body=ellipsoid('Body / low rounded orange insect carapace',RUSSET if not adult else ORANGE,(0.,.265,-.035),(.220,.139,.232),32,22)
 head=ellipsoid('Head / broad orange mushroom crab face',ORANGE,(0.,.247,.164),(.207,.132,.119),32,22);p=[body,head]
 nose=ellipsoid('Muzzle / rounded orange crab snout',LIGHT,(0.,.190,.274),(.145 if adult else .074,.047 if adult else .039,.051),28,18);p.append(nose)
 p.append(paint('Mouth / fitted dark lower snout seam',RUSSET,oval(0.,.173,.105 if adult else .049,.007,24),nose,.002,2))
 for side in (-1,1):
  eye=ellipsoid('Eye / blank pale parasitic gaze '+str(side) if adult else 'Eye / round white juvenile gaze '+str(side),WHITE,(side*(.092 if adult else .110),.309,.270),(.063 if adult else .067,.065 if adult else .077,.047),28,18);p.append(eye)
  if not adult:
   pupil=paint('Pupil / fitted large black insect pupil '+str(side),INK,oval(side*.110,.308,.045,.054,24),eye,.003,2);p.append(pupil)
   p.append(paint('Glint / fitted white insect reflection '+str(side),WHITE,oval(side*.110-.011,.334,.016,.018,18),pupil,.002,1))
   for j in range(2):
    p.append(curved_tube(f'Mandible / paired ivory mouth claws {side} {j}',IVORY,[(side*(.049+j*.004),.170-j*.004,.290),(side*(.060+j*.005),.153-j*.022,.323),(side*(.027+j*.007),.144-j*.023,.333)],[.015,.013,.003],12))
  # Four walking limbs plus two foreclaws are the six insect appendages.
  for j,z in enumerate((-.112,.037)):
   p.append(curved_tube(f'Leg / four braced russet walking limbs {side} {j}',RUSSET if not adult else ORANGE,[(side*.166,.245,z),(side*(.308+j*.020),.167,z-.010),(side*(.364+j*.022),.048,z+.066)],[.026,.023,.013],16))
   p.append(ellipsoid(f'Foot / four planted tapered insect toes {side} {j}',RUSSET if not adult else ORANGE,(side*(.364+j*.022),.025,z+.085),(.024,.025,.047),20,12))
  root=(side*.174,.263,.142);mid=(side*(.301 if adult else .283),.203,.279);tip=(side*(.356 if adult else .301),.044,.471 if adult else .405)
  p.append(paddle('Foreclaw / long tapered orange digging blade '+str(side),LIGHT,[root,mid,tip],[.052 if adult else .041,.095 if adult else .068,.0015],[.039 if adult else .033,.057 if adult else .044,.0015],segments=24,steps=6))
  if not adult:
   for j,(x,y) in enumerate(((.030,.346),(.060,.319),(.028,.290))):
    hs=surface(head);xx=side*x;p.append(ellipsoid(f'Forehead bump / six small orange sensory beads {side} {j}',LIGHT,(xx,y,hs(y,xx)+.003),(.010,.010,.008),14,10))
 p.extend(mushrooms(species))
 if not adult:
  # Thin fitted shell seams stay behind the juvenile face and twin fungi.
  for j,y in enumerate((.300,.348)):
   skin=surface(body);points=[]
   for k in range(15):
    x=-.17+.34*k/14;points.append((x,y,skin(y,x)+.0005))
   p.append(tube('Carapace seam / fitted juvenile shell band '+str(j),RUSSET,points,[.002]*len(points),6))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(species,normalize=True):
 m=deepcopy(raw_sculpture(species));parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',type=Path,required=True);args=parser.parse_args()
 from mushroom_crab_glb import export_mushroom_crab
 args.out.mkdir(parents=True,exist_ok=True)
 for species in SPECIES:(args.out/(species+'.glb')).write_bytes(export_mushroom_crab(species))
