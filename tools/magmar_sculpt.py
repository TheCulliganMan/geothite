#!/usr/bin/env python3
"""Original twin-flame Magmar with orange cuffs, open bill and a yellow tail."""
import argparse,json,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,part
from bovine_sculpt import oval,curved_tube,reverse_front,smooth
from raptor_bird_sculpt import sheet
from sculpt_geometry import fitted_adaptive_contour
from cubone_sculpt import skin_shading
from moltres_sculpt import flowing
from skin_glb import COORDINATES
from goldfish_sculpt import surface
ORANGE=(.94,.45,.23);GOLD=(.98,.85,.38);FIRE=(.98,.70,.28)
CREAM=(.97,.88,.66);IVORY=(.98,.98,.92);BLUE=(.10,.20,.30)
INK=(.075,.07,.09);PINK=(.83,.47,.55);HEIGHT=1.30

def paint(name,color,xy,parent,th=.006,sub=3):
 return fitted_adaptive_contour(name,color,xy,None,th,surface_at=surface(parent),initial_subdivisions=1)
def both_faces(p,parent,xy,color,name,th=.008,sub=3):
 for face in (-1,1):
  actual=parent if face==1 else reverse_front(parent)
  q=paint(name+' '+str(face),color,xy if face==1 else [(-x,y) for x,y in xy],actual,th,sub)
  p.append(q if face==1 else reverse_front(q))
def fitted_shading(parts):
 def find(prefix,side=None):return next(p for p in parts if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+side)))
 for i,p in enumerate(parts):
  n=p['part'];side=n.split()[-1];parent=None;back=False
  for child,owner in (('Belly /','Body /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /')):
   if n.startswith(child):parent=find(owner,side if owner not in ('Body /','Head /') else None)
  if n.startswith('Crown glow /'):parent=find('Crown flame /',n.split()[-2]);back=side=='-1'
  if n.startswith('Tail glow /'):parent=find('Tail flame /');back=side=='-1'
  if n.startswith('Belly /'):parts[i]=smooth_belly(p,parent);continue
  if parent:parts[i]=reverse_front(skin_shading(reverse_front(p),reverse_front(parent))) if back else skin_shading(p,parent)
 return parts

def smooth_belly(p,parent):
 at=surface(parent);out=dict(p,positions=[],normals=[],indices=[]);lookup={}
 for k in range(0,len(p['indices']),3):
  ids=p['indices'][k:k+3];vv=[p['positions'][3*i:3*i+3] for i in ids];offset=[z-at(y,x) for x,y,z in vv];side=1 if all(v>.004 for v in offset) else (-1 if all(v<-.001 for v in offset) else 0)
  a,b,c=vv;u=[b[i]-a[i] for i in range(3)];v=[c[i]-a[i] for i in range(3)];fn=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);fl=math.sqrt(sum(q*q for q in fn));fn=tuple(q/fl for q in fn)
  for i,(x,y,z) in zip(ids,vv):
   normal=p['normals'][3*i:3*i+3]
   if side:
    normal=(x/.239**2,(y-.572)/.290**2,(at(y,x)+.010)/.184**2);length=math.sqrt(sum(v*v for v in normal));normal=tuple(side*v/length for v in normal)
    correction=max(0,.01-sum(a*b for a,b in zip(normal,fn)));normal=tuple(a+correction*b for a,b in zip(normal,fn));length=math.sqrt(sum(q*q for q in normal));normal=tuple(q/length for q in normal)
   key=(x,y,z,*normal)
   if key not in lookup:lookup[key]=len(out['positions'])//3;out['positions'].extend((x,y,z));out['normals'].extend(normal)
   out['indices'].append(lookup[key])
 return out

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
 n=48;vv=[];xs=(.282+j*.083,.300+j*.083,.326+j*.083)
 for k,x in enumerate(xs):
  for i in range(n):
   a=math.tau*i/n;r=(.079 if k==1 else .062)+(0.025 if k==1 and i%6 in (2,3) else 0)
   vv.append((s*x,.790+r*math.cos(a),.015+r*math.sin(a)))
 ff=[tuple(reversed(range(n))),tuple(2*n+i for i in range(n))]
 for k in range(2):
  for i in range(n):a=k*n+i;b=k*n+(i+1)%n;ff.append((a,b,b+n,a+n))
 return crease_normals(part(f'Cuff / rooted jagged orange spitfire wrap {s} {j}',ORANGE,vv,ff,True),35)

def flames(p,lobes):
 for s in (-1,1):
  xy=flowing([(s*x,y) for x,y in ((.057,1.034),(.028,1.114),(.075,1.176),(.047,1.207),(.121,1.265),(.100,1.329),(.161,1.297),(.190,1.335),(.201,1.268),(.223,1.225),(.173,1.196),(.201,1.148),(.157,1.067))],5)
  fire=sheet('Crown flame / rooted flowing yellow spitfire plume '+str(s),GOLD,xy,lambda y,x:.016-.13*(y-1.1),.045,1);p.append(fire)
  xy=oval(s*.115,1.112,.021,.026,28)
  both_faces(p,fire,xy,FIRE,'Crown glow / fitted orange spitfire base '+str(s),.006,2)
 xy=flowing([(-.005,.603),(-.092,.658),(-.065,.726),(-.121,.796),(-.076,.813),(-.045,.910),(.006,.854),(.042,.939),(.072,.833),(.112,.856),(.087,.770),(.126,.724),(.063,.679),(.049,.610)],5)
 fire=sheet('Tail flame / rooted flowing orange spitfire plume',ORANGE,xy,lambda y,x:-.554-.1*(y-.650),.036,1);p.append(fire)
 xy=flowing([(.004,.625),(-.042,.683),(-.025,.743),(-.062,.773),(-.010,.845),(.005,.801),(.038,.873),(.050,.770),(.082,.744),(.037,.699),(.030,.631)],5)
 both_faces(p,fire,xy,GOLD,'Tail glow / fitted yellow spitfire heart',.007,2)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=ellipsoid('Body / substantial orange spitfire pear',ORANGE,(0,.572,-.010),(.239,.290,.184),32,20)
 collar=loft('Collar / rooted dark blue spitfire neck wrap',BLUE,curved_profiles(((.811,.089,.079,.012),(.850,.100,.087,.012),(.895,.095,.081,.012)),3),28)
 head=ellipsoid('Head / broad orange spitfire face',ORANGE,(0,.983,.008),(.197,.153,.145),32,20);p=[body,collar,head];lobes=[]
 for s in (-1,1):
  lobe=ellipsoid('Forehead lobe / rooted broad orange spitfire brow '+str(s),ORANGE,(s*.094,1.078,.065),(.113,.109,.130),24,16);p.append(lobe);lobes.append(lobe)
  xy=[(s*x,y) for x,y in ((.019,1.001),(.142,1.032),(.142,.982),(.110,.954),(.048,.961))]
  socket=paint('Eye socket / fitted narrow dark spitfire recess '+str(s),INK,xy,head,.005,3);p.append(socket);cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted narrow ivory spitfire gaze '+str(s),IVORY,[(cx+.88*(x-cx),cy+.82*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  p.append(paint('Pupil / fitted small dark spitfire stare '+str(s),INK,oval(s*.084,.985,.005,.010,24),eye,.003,2))
 belly=flowing([(-.145,.347),(-.199,.485),(-.158,.599),(-.100,.583),(-.121,.693),(-.076,.738),(-.079,.649),(-.028,.608),(.027,.669),(.035,.762),(.088,.772),(.066,.710),(.108,.662),(.087,.601),(.150,.614),(.182,.727),(.197,.664),(.204,.532),(.171,.386),(.075,.300),(-.041,.303)],5)
 p.append(smooth_belly(paint('Belly / fitted yellow upward-flame spitfire field',GOLD,belly,body,.009,3),body))
 upper=spitfire_bill('Bill / rooted cream spitfire upper beak',CREAM,[(.120,.113,.923,.968),(.189,.116,.917,.976),(.264,.063,.905,.941),(.279,.019,.909,.931)])
 lower=spitfire_bill('Bill / rooted open cream spitfire lower beak',CREAM,[(.119,.090,.862,.906),(.183,.087,.824,.855),(.241,.046,.851,.876)])
 p.extend((upper,lower,ellipsoid('Mouth back / rooted dark spitfire oral chamber',INK,(0,.904,.131),(.083,.055,.038),24,16),ellipsoid('Tongue / rooted pink spitfire mouth floor',PINK,(0,.856,.190),(.047,.014,.036),24,16)))
 for s in (-1,1):
  p.append(ellipsoid('Shoulder / rooted rounded yellow spitfire joint '+str(s),GOLD,(s*.195,.795,-.015),(.101,.110,.095),20,14))
  p.append(curved_tube('Arm / rooted orange spitfire forelimb '+str(s),ORANGE,[(s*.182,.802,-.012),(s*.334,.790,.015),(s*.482,.779,.024)],[.060,.060,.052],24))
  for j in range(2):p.append(cuff(s,j))
  p.append(ellipsoid('Hand / broad orange spitfire claw junction '+str(s),ORANGE,(s*.524,.781,.027),(.076,.071,.068),20,14))
  for j,(y,z) in enumerate(((.826,.034),(.762,.080),(.746,-.014))):
   p.append(curved_tube(f'Hand claw / rooted ivory spitfire point {s} {j}',IVORY,[(s*.549,y,z),(s*.591,y+.018,z+.014),(s*.625,y+.031,z+.019)],[.021,.014,.002],20))
  p.append(ellipsoid('Thigh / rooted round yellow spitfire haunch '+str(s),GOLD,(s*.172,.362,.007),(.127,.132,.119),28,18))
  p.append(curved_tube('Shin band / rooted dark blue spitfire lower leg '+str(s),BLUE,[(s*.188,.321,.013),(s*.223,.175,.031),(s*.235,.075,.060)],[.052,.043,.040],24))
  p.append(ellipsoid('Foot / grounded broad orange spitfire support '+str(s),ORANGE,(s*.239,.042,.116),(.097,.042,.115),24,16))
  for j in range(2):
   x=s*(.202+.063*j);p.append(curved_tube(f'Toe claw / rooted ivory spitfire point {s} {j}',IVORY,[(x,.051,.191),(x,.028,.246),(x,.012,.273)],[.025,.017,.002],20))
 for j,(y,z,ty,tz) in enumerate(((.553,-.166,.605,-.298),(.738,-.127,.802,-.278),(.981,-.112,1.043,-.259))):
  p.append(curved_tube('Dorsal point / rooted orange spitfire spine '+str(j),ORANGE,[(0,y,z),(0,ty,tz)],[.054,.002],20))
 p.append(curved_tube('Tail / rooted long yellow spitfire balance',GOLD,[(0,.372,-.104),(-.058,.287,-.283),(-.067,.383,-.432),(.012,.645,-.556)],[.073,.062,.047,.025],24))
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
