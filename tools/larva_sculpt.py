#!/usr/bin/env python3
"""Original connected Caterpie and Weedle with annulated bodies, fitted faces and characteristic tips."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,ellipsoid,crease_normals
from bovine_sculpt import curved_tube,smooth,oval
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from goldfish_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('caterpie','weedle');HEIGHTS={'caterpie':.95,'weedle':.95}
GREEN=(.45,.71,.36);CREAM=(.97,.87,.52);YELLOW=(.99,.87,.47);RED=(.93,.32,.20);MUZZLE=(.39,.65,.31)
TAN=(.72,.54,.34);PINK=(.81,.44,.60);IVORY=(.94,.94,.98);INK=(.06,.08,.08);WHITE=(.98,.99,.97)
SPINE=((0.,.190,-.785),(0.,.129,-.670),(0.,.119,-.540),(0.,.124,-.402),(0.,.143,-.255),(0.,.198,-.107),(0.,.282,.010),(0.,.435,.095))
RADII=(.026,.055,.072,.085,.098,.108,.116,.095)
PATH,BASE_RADII=smooth(SPINE,RADII,8)

def frame(index):
 a=PATH[max(0,index-1)];b=PATH[min(len(PATH)-1,index+1)];dy=b[1]-a[1];dz=b[2]-a[2];length=math.hypot(dy,dz)
 return (0.,-dz/length,dy/length)

def body_parts(species):
 n=32;grid=[]
 for k,(center,radius) in enumerate(zip(PATH,BASE_RADII)):
  t=k/(len(PATH)-1);r=radius*(.91+.09*math.cos(math.tau*t*7));ventral=frame(k)
  grid.append([tuple(center[q]+r*(math.sin(j*math.tau/n)*(1. if q==0 else 0.)+math.cos(j*math.tau/n)*ventral[q]) for q in range(3)) for j in range(n)])
 def build(name,color,angles,closed_half):
  count=len(angles);vertices=[row[j%n] for row in grid for j in angles];faces=[tuple(reversed(range(count))),tuple((len(grid)-1)*count+j for j in range(count))]
  for k in range(len(grid)-1):
   for j in range(count-1 if closed_half else count):
    a=k*count+j;b=k*count+(j+1)%count;faces.append((a,b,b+count,a+count))
   if closed_half:
    a=k*count;b=a+count-1;faces.append((a,a+count,b+count,b))
  return crease_normals(part(name,color,vertices,faces,True),40.)
 if species=='weedle':return [build('Body / continuous annulated tan larva',TAN,list(range(n)),False)]
 return [build('Body / continuous annulated green dorsal larva',GREEN,list(range(8,25)),True),build('Belly / continuous cream ventral larva',CREAM,list(range(24,41)),True)]

def side_surface(parts,side):
 transformed=[]
 for p in parts:
  transformed.append(dict(p,positions=[v for i in range(0,len(p['positions']),3) for v in (p['positions'][i+2],p['positions'][i+1],side*p['positions'][i])]))
 skins=[surface(p) for p in transformed]
 def at(y,z):
  hits=[]
  for skin in skins:
   try:hits.append(skin(y,z))
   except ValueError:pass
  if not hits:raise ValueError(('outside larva side',y,z))
  return max(hits)
 return at

def side_ring(body,side,j):
 skin=side_surface(body,side);normal=frame(j*8);center=tuple(SPINE[j][q]-normal[q]*RADII[j]*.48 for q in range(3));major=RADII[j]*.26;minor=.0025 if j>3 else .0018;n=28;m=8;v=[]
 for i in range(n):
  a=math.tau*i/n
  for k in range(m):
   b=math.tau*k/m;r=major+minor*math.sin(b);y=center[1]+r*math.cos(a);z=center[2]+r*math.sin(a);x=side*(skin(y,z)+.0005+minor*math.cos(b));v.append((x,y,z))
 faces=[]
 for i in range(n):
  for k in range(m):faces.append((i*m+k,i*m+(k+1)%m,((i+1)%n)*m+(k+1)%m,((i+1)%n)*m+k))
 ring=part(f'Side ring / five seated hollow yellow larva circles {side} {j}',YELLOW,v,faces,True)
 # Python versions differ in the sign retained by tiny summed normal terms.
 # Canonicalize only components that the final seven-place normal rounds to zero.
 ring['normals']=[0. if round(c,7)==0. else c for c in ring['normals']]
 return ring

@lru_cache(maxsize=2)
def raw_sculpture(species):
 if species not in SPECIES:raise ValueError(species)
 weedle=species=='weedle';p=body_parts(species);body=list(p)
 head=ellipsoid('Head / rounded tan stinging larva' if weedle else 'Head / large rounded green caterpillar',TAN if weedle else GREEN,(0.,.570,.132),(.177 if weedle else .187,.188 if weedle else .195,.165 if weedle else .172),36,26);p.append(head);hs=surface(head)
 for side in (-1,1):
  if weedle:
   x=side*.116;y=.619
   eye=ellipsoid('Eye / small glossy black larva bead '+str(side),INK,(x,y,hs(y,x)+.005),(.022,.024,.013),22,14);p.append(eye)
   p.append(paint('Glint / tiny white larva eye reflection '+str(side),WHITE,oval(x-side*.006,y+.008,.006,.007,16),eye,.0015,2))
  else:
   rim=ellipsoid('Eye rim / broad creamy yellow caterpillar ring '+str(side),YELLOW,(side*.118,.607,.244),(.064,.086,.035),28,20);p.append(rim)
   eye=ellipsoid('Eye / glossy black caterpillar lens '+str(side),INK,(side*.118,.607,.263),(.049,.067,.024),28,20);p.append(eye)
   p.append(paint('Glint / soft ivory caterpillar eye reflection '+str(side),WHITE,oval(side*.106,.635,.015,.019,20),eye,.0025,2))
   for j in range(2,7):p.append(side_ring(body,side,j))
  for j in range(1,8) if weedle else (5,6,7):
   center=SPINE[j];r=RADII[j];normal=frame(j*8);planted=weedle and j in (2,3,4)
   if planted:position=(side*r*.50,.034,center[2]);radii=(.030,.034,.034)
   else:
    position=tuple(center[q]+r*((side*.70 if q==0 else 0.)+normal[q]*.66) for q in range(3));radii=(.027,.027,.030) if weedle else (.032,.032,.035)
   p.append(ellipsoid(f'Proleg / '+('planted pink larva pad ' if planted else 'raised pink larva bead ' if weedle else 'raised cream caterpillar leg ')+f'{side} {j}',PINK if weedle else CREAM,position,radii,20,14))
 if weedle:
  nose=ellipsoid('Nose / broad rosy larva snout',PINK,(0.,.557,.296),(.089,.067,.043),30,22);p.append(nose)
  p.append(paint('Nose glint / fitted pale rosy reflection',(.97,.72,.82),oval(-.024,.579,.025,.014,22),nose,.002,2))
  p.append(paddle('Horn / tall tapered ivory head sting',IVORY,[(0.,.733,.105),(.005,.830,.075),(.014,.951,.035)],[.044,.031,.0015],[.040,.027,.0015],segments=24,steps=6))
  p.append(paddle('Tail sting / curved ivory posterior barb',IVORY,[(0.,.180,-.765),(0.,.255,-.835),(0.,.342,-.850)],[.023,.032,.0015],[.020,.024,.0015],segments=22,steps=6))
 else:
  p.append(ellipsoid('Muzzle / rounded upper green caterpillar snout',MUZZLE,(0.,.488,.287),(.099,.073,.061),28,20))
  lower=ellipsoid('Muzzle lower / rounded green caterpillar lip',GREEN,(0.,.472,.323),(.077,.052,.039),28,20);p.append(lower)
  p.append(curved_tube('Horn stem / broad rooted red Y antenna',RED,[(0.,.733,.202),(0.,.768,.228),(0.,.790,.220)],[.030,.033,.034],20))
  for side in (-1,1):
   p.append(curved_tube('Horn branch / rounded red forked antenna '+str(side),RED,[(0.,.786,.220),(side*.060,.815,.210),(side*.122,.857,.203)],[.032,.035,.030],20))
   p.append(ellipsoid('Horn tip / rounded red antenna end '+str(side),RED,(side*.122,.857,.203),(.031,.033,.031),22,16))
  p.append(paddle('Tail tip / soft pale yellow posterior point',CREAM,[(0.,.180,-.765),(0.,.240,-.840),(0.,.312,-.860)],[.012,.025,.004],[.012,.018,.003],segments=20,steps=5))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(species,normalize=True):
 m=deepcopy(raw_sculpture(species));parts=m['primitives']
 if not normalize:return m
 lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for p in parts:
  p['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return m

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',type=Path,required=True);args=parser.parse_args();args.out.mkdir(parents=True,exist_ok=True)
 from larva_glb import export_larva
 for species in SPECIES:
  (args.out/(species+'.glb')).write_bytes(export_larva(species));(args.out/(species+'.mesh.json')).unlink(missing_ok=True)
if __name__=='__main__':main()
