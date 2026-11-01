#!/usr/bin/env python3
"""Original twin-flame Magmar with orange cuffs, open bill and a yellow tail."""
import argparse,json,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,part
from bovine_sculpt import oval,curved_tube,reverse_front,smooth
from raptor_bird_sculpt import sheet
from sculpt_geometry import fitted_adaptive_contour,fitted_triangle_contour
from cubone_sculpt import skin_shading
from moltres_sculpt import flowing
from skin_glb import COORDINATES
from goldfish_sculpt import surface
from lugia_sculpt import paddle
from chikorita_sculpt import add,sub,mul,dot,cross,unit
ORANGE=(.94,.45,.23);GOLD=(.98,.85,.38);FIRE=(.98,.70,.28)
CREAM=(.97,.88,.66);IVORY=(.98,.98,.92);BLUE=(.10,.20,.30)
INK=(.075,.07,.09);PINK=(.83,.47,.55);HEIGHT=1.30
BODY_ROWS=((.170,.110,.130,-.010),(.245,.222,.200,-.010),(.380,.272,.228,-.010),(.540,.250,.210,-.010),(.680,.185,.164,-.005),(.790,.121,.100,.008),(.855,.069,.061,.012))

def arm_path(s):
 return [(s*.185,.781,-.008),(s*.316,.711,.026),(s*.386,.614,.111),(s*.407,.578,.173)]

def paint(name,color,xy,parent,th=.006,sub=3):
 return fitted_adaptive_contour(name,color,xy,None,th,surface_at=surface(parent),initial_subdivisions=1)
def both_faces(p,parent,xy,color,name,th=.008,sub=3):
 for face in (-1,1):
  actual=parent if face==1 else reverse_front(parent)
  q=fitted_triangle_contour(name+' '+str(face),color,xy if face==1 else [(-x,y) for x,y in xy],actual,th)
  p.append(q if face==1 else reverse_front(q))
def fitted_shading(parts):
 def find(prefix,side=None):return next(p for p in parts if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+side)))
 for i,p in enumerate(parts):
  n=p['part'];side=n.split()[-1];parent=None;back=False
  for child,owner in (('Belly /','Body /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /')):
   if n.startswith(child):parent=find(owner,side if owner not in ('Body /','Head /') else None)
  if n.startswith('Crown glow /'):parent=find('Crown flame /',n.split()[-2]);back=side=='-1'
  if n.startswith('Tail glow /'):parent=find('Tail flame /');back=side=='-1'
  if n.startswith('Belly /'):continue
  if parent:parts[i]=reverse_front(skin_shading(reverse_front(p),reverse_front(parent))) if back else skin_shading(p,parent)
 return parts

def belly_wrap(body):
 # Clip the actual torso triangles, so the flame boundary cannot cut through
 # a differently triangulated surface. Both shells retain the torso facets.
 source=[tuple(body['positions'][i:i+3]) for i in range(0,len(body['positions']),3)]
 points=[];faces=[];lookup={}
 def field(q):
  a=math.atan2(q[2]+.010,q[0]);top=.490+.170*max(0.,math.sin(9*a+.6))**4+.025*math.sin(5*a)
  return q[1]-top
 def vertex(q):
  key=tuple(round(v,8) for v in q)
  if key not in lookup:lookup[key]=len(points);points.append(key)
  return lookup[key]
 for k in range(0,len(body['indices']),3):
  tri=[source[i] for i in body['indices'][k:k+3]];polygon=[]
  for a,b in zip(tri,tri[1:]+tri[:1]):
   fa,fb=field(a),field(b)
   if fa<=0:polygon.append(a)
   if (fa<=0)!=(fb<=0):
    t=fa/(fa-fb);polygon.append(tuple(a[j]+t*(b[j]-a[j]) for j in range(3)))
  ids=[vertex(q) for q in polygon]
  for j in range(1,len(ids)-1):
   face=(ids[0],ids[j],ids[j+1])
   if len(set(face))==3:faces.append(face)
 count=len(points);lo=min(q[1] for q in points);vv=[]
 for inside in (False,True):
  scale=.985 if inside else 1.026
  for x,y,z in points:vv.append((x*scale,y+(.001 if inside else -.001) if abs(y-lo)<1e-7 else y,-.010+(z+.010)*scale))
 ff=list(faces)+[tuple(i+count for i in reversed(face)) for face in faces];edges={}
 for face in faces:
  for a,b in zip(face,face[1:]+face[:1]):edges.setdefault(tuple(sorted((a,b))),[]).append((a,b))
 for uses in edges.values():
  if len(uses)==1:
   a,b=uses[0];ff.append((b,a,a+count,b+count))
 return crease_normals(part('Belly / wrapped yellow rising-flame spitfire field',GOLD,vv,ff,True),45)

def spitfire_bill(name,color,rows):
 # A rounded duckbill profile, with extra curvature only at this close feature.
 low,_=smooth([(z,w,y0) for z,w,y0,y1 in rows],[1.]*len(rows),4)
 high,_=smooth([(y1,0.,0.) for z,w,y0,y1 in rows],[1.]*len(rows),4)
 n=24;v=[]
 for (z,w,y0),(y1,_,_) in zip(low,high):
  mid=(y0+y1)/2;half=(y1-y0)/2
  for i in range(n):
   a=math.tau*i/n;c=math.cos(a);s=math.sin(a)
   v.append((w*math.copysign(abs(c)**.8,c),mid+half*math.copysign(abs(s)**.8,s),z))
 f=[tuple(reversed(range(n))),tuple((len(low)-1)*n+i for i in range(n))]
 for k in range(len(low)-1):
  for i in range(n):a=k*n+i;b=k*n+(i+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part(name,color,v,f,True),65.)

def cuff(s,j):
 # Closed toothed orange volume around a supported forearm, not floating fins.
 n=48;vv=[];pp,rr=smooth(arm_path(s),[.060,.060,.052,.048],16);steps=len(pp)-1;fractions=(.31+j*.29,.36+j*.29,.42+j*.29)
 for k,t in enumerate(fractions):
  index=round(t*steps);center=pp[index];tangent=unit(sub(pp[min(index+1,steps)],pp[max(index-1,0)]));side=unit(cross(tangent,(0,0,1)));other=cross(tangent,side)
  for i in range(n):
   a=math.tau*i/n;r=(.078 if k==1 else .062)+(0.023 if k==1 and i%6 in (2,3) else 0)
   vv.append(add(center,add(mul(side,r*math.cos(a)),mul(other,r*math.sin(a)))))
 ff=[tuple(reversed(range(n))),tuple(2*n+i for i in range(n))]
 for k in range(2):
  for i in range(n):a=k*n+i;b=k*n+(i+1)%n;ff.append((a,b,b+n,a+n))
 return crease_normals(part(f'Cuff / rooted jagged orange spitfire wrap {s} {j}',ORANGE,vv,ff,True),35)

def flame_volume(name,color,outline,centre_z):
 # A forked silhouette with a rounded cross-section. Both sides swell into
 # volume; the perimeter keeps thickness instead of becoming a flat card.
 q=sheet(name,color,outline,lambda y,x:centre_z,.080,1)
 def distance(x,y):
  best=float('inf')
  for a,b in zip(outline,outline[1:]+outline[:1]):
   dx,dy=b[0]-a[0],b[1]-a[1];length=dx*dx+dy*dy;t=max(0.,min(1.,((x-a[0])*dx+(y-a[1])*dy)/length))
   best=min(best,math.hypot(x-a[0]-t*dx,y-a[1]-t*dy))
  return best
 for k in range(0,len(q['positions']),3):
  x,y,z=q['positions'][k:k+3];half=.012+min(.056,1.05*distance(x,y));q['positions'][k+2]=centre_z+math.copysign(half,z-centre_z)
 return crease_normals(q,75.)

def flames(p,lobes):
 for s in (-1,1):
  xy=flowing([(s*x,y) for x,y in ((.061,1.085),(.045,1.180),(.080,1.240),(.055,1.300),(.130,1.405),(.120,1.320),(.170,1.345),(.200,1.390),(.185,1.260),(.210,1.200),(.170,1.120))],3)
  fire=flame_volume('Crown flame / rooted flowing yellow spitfire plume '+str(s),GOLD,xy,.006);p.append(fire)
  xy=oval(s*.122,1.143,.028,.035,24)
  both_faces(p,fire,xy,FIRE,'Crown glow / fitted orange spitfire base '+str(s),.006,2)
 xy=flowing([(-.005,.569),(-.090,.660),(-.067,.728),(-.116,.801),(-.070,.818),(-.040,.927),(.005,.855),(.046,.955),(.070,.834),(.111,.860),(.084,.770),(.124,.724),(.062,.670),(.049,.572)],3)
 fire=flame_volume('Tail flame / rooted flowing orange spitfire plume',ORANGE,xy,-.554);p.append(fire)
 xy=flowing([(.009,.591),(-.041,.663),(-.020,.710),(.013,.772),(.014,.824),(.056,.758),(.063,.700),(.037,.648),(.034,.595)],3)
 both_faces(p,fire,xy,GOLD,'Tail glow / fitted yellow spitfire heart',.007,2)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=crease_normals(loft('Body / substantial orange spitfire pear',ORANGE,curved_profiles(BODY_ROWS,4),32),45)
 collar=loft('Collar / rooted dark blue spitfire neck wrap',BLUE,curved_profiles(((.811,.089,.079,.012),(.850,.100,.087,.012),(.895,.095,.081,.012)),3),28)
 head=ellipsoid('Head / broad orange spitfire face',ORANGE,(0,.983,.008),(.197,.153,.145),28,20);p=[body,collar,head];lobes=[]
 for s in (-1,1):
  lobe=ellipsoid('Forehead lobe / rooted broad orange spitfire brow '+str(s),ORANGE,(s*.094,1.078,.065),(.113,.109,.130),24,16);p.append(lobe);lobes.append(lobe)
  xy=[(s*x,y) for x,y in ((.019,1.001),(.142,1.032),(.142,.982),(.110,.954),(.048,.961))]
  socket=paint('Eye socket / fitted narrow dark spitfire recess '+str(s),INK,xy,head,.005,3);p.append(socket);cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted narrow ivory spitfire gaze '+str(s),IVORY,[(cx+.88*(x-cx),cy+.82*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  p.append(paint('Pupil / fitted small dark spitfire stare '+str(s),INK,oval(s*.084,.985,.005,.010,24),eye,.003,2))
 p.append(belly_wrap(body))
 upper=spitfire_bill('Bill / rooted golden spitfire upper beak',GOLD,[(.105,.140,.923,.968),(.190,.145,.917,.976),(.278,.091,.905,.941),(.296,.032,.909,.931)])
 lower=spitfire_bill('Bill / rooted open golden spitfire lower beak',GOLD,[(.105,.120,.862,.906),(.190,.111,.824,.855),(.260,.067,.851,.876)])
 p.extend((upper,lower,ellipsoid('Mouth back / rooted dark spitfire oral chamber',INK,(0,.904,.131),(.083,.055,.038),20,14),ellipsoid('Tongue / rooted pink spitfire mouth floor',PINK,(0,.856,.190),(.047,.014,.036),20,14)))
 for s in (-1,1):
  p.append(ellipsoid('Shoulder / rooted rounded yellow spitfire joint '+str(s),GOLD,(s*.190,.768,-.015),(.097,.105,.103),24,16))
  p.append(curved_tube('Arm / rooted orange spitfire forelimb '+str(s),ORANGE,arm_path(s),[.060,.065,.061,.051],20))
  for j in range(2):p.append(cuff(s,j))
  p.append(ellipsoid('Hand / broad orange spitfire claw junction '+str(s),ORANGE,(s*.419,.548,.215),(.076,.068,.072),20,16))
  for j,dx in enumerate((-.042,0.,.042)):
   x=s*(.419+dx);p.append(curved_tube(f'Hand claw / rooted ivory spitfire point {s} {j}',IVORY,[(x,.528,.255),(x,.494,.299),(x,.471,.312)],[.020,.014,.002],20))
  p.append(ellipsoid('Thigh / rooted round yellow spitfire haunch '+str(s),GOLD,(s*.218,.277,-.010),(.148,.137,.145),24,18))
  p.append(curved_tube('Shin band / rooted dark blue spitfire lower leg '+str(s),BLUE,[(s*.228,.250,.004),(s*.249,.153,.004),(s*.254,.070,.004)],[.058,.049,.044],20))
  p.append(ellipsoid('Foot / grounded broad orange spitfire support '+str(s),ORANGE,(s*.255,.045,.068),(.108,.045,.126),24,18))
  for j in range(2):
   x=s*(.220+.069*j);p.append(curved_tube(f'Toe claw / rooted ivory spitfire point {s} {j}',IVORY,[(x,.048,.154),(x,.028,.214),(x,.012,.239)],[.025,.017,.002],20))
 for j,(y,z,ty,tz) in enumerate(((.553,-.166,.605,-.298),(.738,-.127,.802,-.278),(.981,-.112,1.043,-.259))):
  p.append(curved_tube('Dorsal point / rooted orange spitfire spine '+str(j),ORANGE,[(0,y,z),(0,ty,tz)],[.054,.002],20))
 p.append(curved_tube('Tail / rooted long yellow spitfire balance',GOLD,[(0,.372,-.104),(-.058,.287,-.283),(-.067,.383,-.432),(.012,.645,-.556)],[.073,.062,.047,.025],20))
 flames(p,lobes)
 # Expose the blue collar between the broad head and shoulder line.
 for q in p:
  if q['part'].startswith(('Head /','Forehead lobe /','Eye socket /','Eye /','Pupil /','Bill /','Mouth back /','Tongue /','Crown flame /','Crown glow /')) or q['part'].startswith('Dorsal point /') and q['part'].endswith(' 2'):
   q['positions']=[v+(.035 if i%3==1 else 0) for i,v in enumerate(q['positions'])]
 return fitted_shading([crease_normals(q,75. if q['part'].startswith(('Crown flame /','Tail flame /')) else 45.) for q in p])

@lru_cache(maxsize=2)
def sculpture(normalize=True):
 p=deepcopy(raw_sculpture())
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo) if normalize else 1.
 for q in p:
  q['positions']=[round((v-(lo if normalize and i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'magmar','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'magmar.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
