#!/usr/bin/env python3
"""Original compact bone carrier with recessed skull orbits and gripped club."""
import argparse,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,part
from bovine_sculpt import curved_tube,oval
from functools import lru_cache
import copy
from sculpt_geometry import fitted_adaptive_contour
def paint(name,color,outline,parent,thickness=.012,subdivisions=3):
 return fitted_adaptive_contour(name,color,outline,parent,thickness,initial_subdivisions=1)
from skin_glb import COORDINATES
HEIGHT=1.00
TAN=(.76,.57,.33);CREAM=(.97,.90,.71);BONE=(.96,.96,.92);WHITE=(.995,.99,.96);INK=(.095,.08,.105)

def recessed(p,holes):
 vertices=[]
 for i in range(0,len(p['positions']),3):
  x,y,z=p['positions'][i:i+3]
  if z>0:
   for cx,cy,rx,ry,depth in holes:
    q=((x-cx)/rx)**2+((y-cy)/ry)**2
    if q<1:z-=depth*(1-q)**2
  vertices.append((x,y,z))
 return part(p['part'],p['base_color'][:3],vertices,[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)],True)

def skin_shading(layer,parent):
 # Match cap shading to the actual underlying mesh's interpolated normals.
 vertices=[tuple(parent['positions'][i:i+3]) for i in range(0,len(parent['positions']),3)]
 normals=[tuple(parent['normals'][i:i+3]) for i in range(0,len(parent['normals']),3)]
 bins={};cell=.025
 for i in range(0,len(parent['indices']),3):
  ids=parent['indices'][i:i+3];a,b,c=[vertices[j] for j in ids];den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
  if abs(den)<1e-12:continue
  item=(ids,a,b,c,den)
  for ix in range(math.floor(min(a[0],b[0],c[0])/cell),math.floor(max(a[0],b[0],c[0])/cell)+1):
   for iy in range(math.floor(min(a[1],b[1],c[1])/cell),math.floor(max(a[1],b[1],c[1])/cell)+1):bins.setdefault((ix,iy),[]).append(item)
 cache={};out=dict(layer);out['positions']=[];out['normals']=[];out['indices']=[];lookup={}
 points=[tuple(layer['positions'][i:i+3]) for i in range(0,len(layer['positions']),3)]
 for k in range(0,len(layer['indices']),3):
  tri=layer['indices'][k:k+3];a,b,c=[points[j] for j in tri];u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)]
  face=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);length=math.sqrt(sum(q*q for q in face));cap=abs(face[2])>length*.1
  for old in tri:
   x,y,z=points[old];key=(x,y)
   if cap:
    if key not in cache:
     hits=[]
     for ids,a,b,c,den in bins.get((math.floor(x/cell),math.floor(y/cell)),[]):
      u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den;w=1-u-v
      if min(u,v,w)>=-1e-6:
       n=[sum(normals[j][axis]*weight for j,weight in zip(ids,(u,v,w))) for axis in range(3)];hits.append((u*a[2]+v*b[2]+w*c[2],n))
     if not hits:raise ValueError(('shading outside parent',layer['part'],x,y))
     _,n=max(hits,key=lambda hit:hit[0]);length=math.sqrt(sum(v*v for v in n));cache[key]=[v/length for v in n]
    n=tuple(v*(1 if face[2]>0 else -1) for v in cache[key])
    # At a steep recess edge, constrain the smooth field to the cap's
    # actual outward hemisphere rather than shading across its fold.
    face_length=math.sqrt(sum(v*v for v in face));unit_face=tuple(v/face_length for v in face);alignment=sum(v*w for v,w in zip(n,unit_face))
    if alignment<.15:
     n=tuple(v+(.15-alignment)*w for v,w in zip(n,unit_face));length=math.sqrt(sum(v*v for v in n));n=tuple(v/length for v in n)
   else:n=tuple(layer['normals'][old*3:old*3+3])
   key=(x,y,z)+n
   if key not in lookup:
    lookup[key]=len(out['positions'])//3;out['positions'].extend((x,y,z));out['normals'].extend(n)
   out['indices'].append(lookup[key])
 return out

@lru_cache(maxsize=1)
def _anatomy():
 body=loft('Body / compact rounded tan Cubone torso',TAN,curved_profiles(((.147,.071,.095,.001),(.251,.205,.197,.0),(.442,.278,.231,0),(.666,.249,.203,.005),(.803,.157,.132,.011),(.885,.052,.06,.019)),5),40)
 neck=ellipsoid('Neck / rooted tan Cubone neck',TAN,(0,.997,.029),(.121,.143,.117),24,16)
 skin=ellipsoid('Head / tan Cubone head inside skull',TAN,(0,1.153,.058),(.239,.246,.216),28,20)
 skull=recessed(ellipsoid('Skull / broad bone Cubone helmet with recessed orbits',BONE,(0,1.215,.070),(.315,.330,.285),48,32),[(s*.180,1.254,.077,.105,.095) for s in (-1,1)])
 snout=recessed(ellipsoid('Snout / broad tapered bone Cubone skull snout',BONE,(0,1.086,.282),(.236,.121,.210),64,40),[(s*.077,1.085,.033,.044,.062) for s in (-1,1)])
 p=[body,neck,skin,skull,snout]
 p.append(skin_shading(paint('Belly / fitted broad cream Cubone belly',CREAM,oval(0,.466,.189,.252,48),body,.009,3),body))
 for s in (-1,1):
  socket=skin_shading(paint('Orbit / fitted recessed tan Cubone socket '+str(s),TAN,oval(s*.180,1.254,.064,.084,24),skull,.004,4),skull);p.append(socket)
  eye=paint('Eye / fitted white Cubone eye inside orbit '+str(s),WHITE,[(s*x,y) for x,y in ((.162,1.239),(.169,1.274),(.190,1.297),(.208,1.261),(.204,1.220),(.173,1.222))],socket,.004,3);p.append(eye)
  pupil=paint('Pupil / fitted dark Cubone gaze '+str(s),INK,oval(s*.181,1.251,.013,.028,28),eye,.003,3);p.append(pupil)
  p.append(paint('Eye glint / fitted white Cubone point '+str(s),WHITE,oval(s*.176,1.267,.0035,.007,20),pupil,.002,2))
  p.append(paint('Nostril / fitted dark triangular Cubone skull opening '+str(s),INK,[(s*.063,1.068),(s*.058,1.092),(s*.078,1.110),(s*.098,1.079)],snout,.003,4))
  p.append(curved_tube('Horn / broad outward bone Cubone skull horn '+str(s),BONE,[(s*.198,1.445,.038),(s*.271,1.563,.021),(s*.364,1.643,-.047)],[.075,.055,.002],32))
  p.append(curved_tube('Cheek / rooted pointed bone Cubone helmet cheek '+str(s),BONE,[(s*.240,1.115,.195),(s*.250,1.035,.219),(s*.209,.979,.234)],[.056,.043,.002],28))
  p.append(ellipsoid('Shoulder / rooted rounded tan Cubone shoulder '+str(s),TAN,(s*.251,.693,.025),(.095,.117,.105),24,16))
  pts=[(-.264,.698,.054),(-.346,.527,.140),(-.311,.442,.267)] if s==-1 else [(.264,.698,.040),(.394,.581,.081),(.425,.480,.161)]
  p.append(curved_tube('Arm / bent tan Cubone forelimb '+str(s),TAN,pts,[.072,.067,.057],28))
  hand=pts[-1];p.append(ellipsoid('Hand / shaped rounded tan Cubone palm '+str(s),TAN,hand,(.074,.076,.071),24,16))
  for j in range(3):
   x=hand[0]+(j-1)*.034
   points=[(x,hand[1]-.011,hand[2]+.044),(x-.007 if s==-1 else x+.01,hand[1]-.052,hand[2]+.095),(x-.014 if s==-1 else x+.012,hand[1]-.046,hand[2]+.135)]
   p.append(curved_tube(f'Finger / rooted curved tan Cubone digit {s} {j}',TAN,points,[.021,.017,.005],24))
  p.append(ellipsoid('Haunch / rooted heavy tan Cubone thigh '+str(s),TAN,(s*.204,.243,.014),(.150,.210,.190),28,20))
  p.append(ellipsoid('Foot / grounded broad tan Cubone foot '+str(s),TAN,(s*.256,.066,.180),(.107,.066,.167),24,16))
  p.append(ellipsoid('Toe claw / rooted rounded white Cubone toe '+str(s),WHITE,(s*.265,.048,.314),(.037,.025,.053),24,16))
 p.append(curved_tube('Thumb / rooted tan Cubone bone-gripping thumb',TAN,[(-.350,.476,.274),(-.388,.466,.309),(-.376,.453,.338)],[.029,.024,.010],24))
 club=curved_tube('Bone shaft / cream Cubone club held by hand',BONE,[(-.320,.502,.315),(-.380,.284,.385),(-.460,.116,.444)],[.040,.037,.046],32);p.append(club)
 for end,(x,y,z) in enumerate(((-.320,.502,.315),(-.460,.116,.444))):
  for j,dx in enumerate((-.027,.027)):p.append(ellipsoid(f'Bone lobe / rooted bifurcated Cubone club end {end} {j}',BONE,(x+dx,y,z),(.048,.049,.052),24,16))
 p.append(curved_tube('Tail / rooted short pointed tan Cubone tail',TAN,[(0,.286,-.154),(.01,.231,-.346),(.072,.356,-.467)],[.100,.073,.002],32))
 # Juvenile proportions: a compact body below the oversized skull helmet.
 cranial=('Head /','Skull /','Snout /','Orbit /','Eye /','Pupil /','Eye glint /','Nostril /','Horn /','Cheek /')
 for q in p:
  head_part=q['part'].startswith(cranial)
  q['positions']=[v-.20 if head_part and i%3==1 else v*.8 if not head_part and i%3==1 else v for i,v in enumerate(q['positions'])]
  if not head_part:
   normals=[]
   for i in range(0,len(q['normals']),3):
    x,y,z=q['normals'][i:i+3];y/=.8;length=math.sqrt(x*x+y*y+z*z);normals.extend((x/length,y/length,z/length))
   q['normals']=normals
 return p

def anatomy():return copy.deepcopy(_anatomy())

def sculpture(normalize=True):
 p=anatomy()
 if not normalize:return {'name':'cubone','version':1,'coordinate_system':COORDINATES,'primitives':p}
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'cubone','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from bone_keeper_glb import export_bone_keeper
 (args.output/'cubone.glb').write_bytes(export_bone_keeper('cubone'))
if __name__=='__main__':main()
