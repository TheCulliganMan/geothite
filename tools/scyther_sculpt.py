#!/usr/bin/env python3
"""Original green mantis with curved ivory scythes, wing cases and clawed greaves."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,crease_normals,part,fitted_triangle_contour,fitted_adaptive_contour
from bovine_sculpt import oval,curved_tube,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import sheet
from wing_insect_sculpt import contour
from abra_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.18
GREEN=(84/255,190/255,71/255);DARK=(18/255,98/255,13/255);CREAM=(244/255,225/255,167/255)
IVORY=(.95,.97,.94);WING=(.97,.95,.79);VEIN=(.57,.65,.58);INK=(.075,.10,.07)

def paint(name,color,outline,parent,th=.003,sub=1):
 return fitted_triangle_contour(name,color,outline,parent,th,precision=6)


def frame(q,origin,u,v,w):
 for field in ('positions','normals'):
  values=[]
  for i in range(0,len(q[field]),3):
   a,b,c=q[field][i:i+3];values.extend((origin[k] if field=='positions' else 0.)+a*u[k]+b*v[k]+c*w[k] for k in range(3))
  q[field]=values
 return q


def both_faces(mark,parent,offset):
 front=surface(parent);back=surface(reverse_front(parent))
 for i in range(0,len(mark['positions']),3):
  x,y,z=mark['positions'][i:i+3]
  if z<front(y,x):mark['positions'][i+2]=-back(y,-x)-offset
 return mark


def mantis_wing(side,j,root,tip,width):
 length=math.dist(root,tip);v=tuple((b-a)/length for a,b in zip(root,tip))
 seed=(0.,1.,0.);dot=sum(a*b for a,b in zip(seed,v));u=tuple(a-dot*b for a,b in zip(seed,v));norm=math.sqrt(sum(a*a for a in u));u=tuple(a/norm for a in u)
 w=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
 outline=[(0.,0.),(-width*.65,length*.16),(-width,length*.42),(-width*.70,length*.69),(0.,length),(width*.70,length*.69),(width,length*.42),(width*.65,length*.16)]
 depth=lambda y,x:.010*math.sin(math.pi*max(0.,min(1.,y/length)))*(1.-.5*(x/width)**2)
 case=sheet(f'Wing case / four green-rimmed mantis leaves {side} {j}',GREEN,outline,depth,.011,1)
 inner=[(x*.78,length*.05+y*.90) for x,y in outline]
 pale=both_faces(paint(f'Wing / fitted pale mantis membrane {side} {j}',WING,inner,case,.003),case,.003)
 skin=surface(pale);vein=tube(f'Wing vein / fitted central gray mantis rib {side} {j}',VEIN,[(0.,length*t,skin(length*t,0.)+.0008) for t in (.12,.20,.28,.36,.44,.52,.60,.68,.76,.84,.90)],[.0015]*11,8)
 return [frame(q,root,u,v,w) for q in (case,pale,vein)]

def abdomen_groove(name,parent,y):
 # Cut the actual stored triangles, then cast radial samples onto that section.
 ps=[parent['positions'][i:i+3] for i in range(0,len(parent['positions']),3)];cuts=set()
 for k in range(0,len(parent['indices']),3):
  tri=[ps[j] for j in parent['indices'][k:k+3]]
  for a,b in zip(tri,tri[1:]+tri[:1]):
   if not (a[1]<=y<b[1] or b[1]<=y<a[1]):continue
   t=(y-a[1])/(b[1]-a[1]);cuts.add((round(a[0]+t*(b[0]-a[0]),9),round(a[2]+t*(b[2]-a[2]),9)))
 cx=sum(p[0] for p in cuts)/len(cuts);cz=sum(p[1] for p in cuts)/len(cuts);cuts=sorted(cuts,key=lambda p:math.atan2(p[1]-cz,p[0]-cx));n=48;m=8;vertices=[]
 def cross(a,b):return a[0]*b[1]-a[1]*b[0]
 for j in range(n):
  a=math.tau*j/n;d=(math.cos(a),math.sin(a));hits=[]
  for p,q in zip(cuts,cuts[1:]+cuts[:1]):
   e=(q[0]-p[0],q[1]-p[1]);offset=(p[0]-cx,p[1]-cz);den=cross(d,e)
   if abs(den)<1e-12:continue
   t=cross(offset,e)/den;u=cross(offset,d)/den
   if t>0. and -.000001<=u<=1.000001:hits.append(t)
  radius=max(hits)+.0004
  for k in range(m):
   b=math.tau*k/m;r=radius+.0016*math.cos(b);vertices.append((cx+r*d[0],y+.0016*math.sin(b),cz+r*d[1]))
 faces=[]
 for j in range(n):
  for k in range(m):faces.append((j*m+k,j*m+(k+1)%m,((j+1)%n)*m+(k+1)%m,((j+1)%n)*m+k))
 return part(name,DARK,vertices,faces,True)

def scythe(side):
 cx=0.;cy=0.
 xy=contour([((-.015,-.035),(.008,.008),(.103,.037),(.177,.041)),
  ((.177,.041),(.286,.057),(.360,.008),(.369,-.082)),
  ((.369,-.082),(.396,-.180),(.385,-.284),(.352,-.390)),
  ((.352,-.390),(.316,-.292),(.258,-.234),(.193,-.211)),
  ((.193,-.211),(.177,-.136),(.121,-.076),(.055,-.061)),
  ((.055,-.061),(.022,-.060),(-.002,-.047),(-.015,-.035))],8)
 def world(points):return [(cx+x,cy+y) for x,y in points]
 blade=sheet('Scythe / long curved ivory mantis blade '+str(side),IVORY,world(xy),lambda y,x:.018*math.sin(math.pi*max(0.,min(1.,(x-cx)/.40))),.023,1)
 edge=[]
 for x,y in xy[:23]:
  dx=.18-x;dy=-.15-y;l=math.hypot(dx,dy);edge.append((x+.0012*dx/l,y+.0012*dy/l))
 inner=[]
 for i,(x,y) in enumerate(edge):
  dx=.18-x;dy=-.15-y;l=math.hypot(dx,dy);w=.013*math.sin(math.pi*(i+.6)/(len(edge)+.2))**.5
  inner.append((x+dx*w/l,y+dy*w/l))
 rim=fitted_adaptive_contour('Scythe rim / fitted green mantis blade spine '+str(side),GREEN,world(edge+list(reversed(inner))),blade,.003,initial_subdivisions=0,max_surface_error=.0004)
 rim=both_faces(rim,blade,.003)
 # Blades sweep mostly forward in the sagittal plane, with a slight outward guard.
 a=math.radians(20.);u=(side*math.sin(a),0.,math.cos(a));v=(0.,1.,0.);w=(-math.cos(a),0.,side*math.sin(a))
 for q in (blade,rim):
  q['positions']=[value*(1.75 if k%3==0 else .65 if k%3==1 else 1.) for k,value in enumerate(q['positions'])]
  ns=[]
  for k in range(0,len(q['normals']),3):
   x,y,z=q['normals'][k:k+3];n=(x/1.75,y/.65,z);l=math.sqrt(sum(v*v for v in n));ns.extend(v/l for v in n)
  q['normals']=ns
 return [frame(q,(side*.20,.81,.27),u,v,w) for q in (blade,rim)]

@lru_cache(maxsize=1)
def raw_sculpture():
 thorax=ellipsoid('Thorax / rounded green mantis breast',GREEN,(0.,.823,.010),(.145,.160,.120),28,18)
 waist=loft('Waist / slender pale mantis stalk',CREAM,curved_profiles(((.53,.050,.045,-.02),(.64,.053,.047,-.01),(.75,.065,.055,.01),(.80,.074,.060,.015)),4),24)
 abdomen=loft('Abdomen / rounded tapered green mantis shell',GREEN,curved_profiles(((.286,.030,.027,-.06),(.34,.095,.075,-.065),(.46,.170,.140,-.060),(.58,.153,.120,-.04),(.64,.081,.060,-.025),(.65,.035,.030,-.02)),4),32)
 head=crease_normals(loft('Head / angular green mantis mask and tapered muzzle',GREEN,((.963,.022,.025,.195),(1.005,.080,.095,.145),(1.025,.205,.100,.120),(1.055,.160,.120,.075),(1.130,.170,.125,.015),(1.185,.130,.085,-.015),(1.215,.060,.045,-.025),(1.230,.020,.020,-.020)),16),35.)
 p=[thorax,waist,abdomen,head]
 p.append(curved_tube('Neck / short pale green mantis throat',CREAM,[(0.,.924,.027),(0.,.977,.053),(0.,1.018,.076)],[.041,.034,.032],20))
 p.append(paddle('Crest / short central green mantis point',GREEN,[(0.,1.185,-.020),(0.,1.250,-.025),(0.,1.310,-.025)],[.048,.030,.0018],[.033,.020,.0018],segments=16,steps=4))
 for side in (-1,1):
  p.append(paddle('Crest / swept green mantis cranial point '+str(side),GREEN,[(side*.090,1.145,-.005),(side*.170,1.205,-.025),(side*.250,1.240,-.040)],[.045,.034,.002],[.029,.021,.002],segments=20,steps=4))
  x=side*.098;y=1.085
  xy=[(x+side*u,y+v) for u,v in ((-.041,.012),(.035,.038),(.033,-.021),(-.028,-.031))]
  rim=paint('Eye socket / fitted stern green mantis brow '+str(side),DARK,xy,head,.005,3);p.append(rim)
  xy=[(x+.85*(u-x),y+.85*(v-y)) for u,v in xy]
  eye=paint('Eye / fitted ivory mantis lens '+str(side),IVORY,xy,rim,.003,2);p.append(eye)
  p.append(paint('Pupil / fitted narrow black mantis slit '+str(side),INK,oval(x+side*.005,y-.002,.006,.019,16),eye,.0025,2))
  p.append(paint('Nostril / tiny fitted mantis muzzle pore '+str(side),DARK,oval(side*.027,1.005,.003,.0025,12),head,.002,1))
  p.append(ellipsoid('Shoulder / large rounded green mantis armor '+str(side),GREEN,(side*.151,.850,.005),(.096,.100,.091),28,18))
  p.append(curved_tube('Upper arm / slender pale mantis elbow stalk '+str(side),CREAM,[(side*.170,.840,.030),(side*.200,.835,.150),(side*.200,.810,.285)],[.026,.023,.024],16))
  p += scythe(side)
  p.append(ellipsoid('Haunch / teardrop green mantis coxa '+str(side),GREEN,(side*.142,.453,-.040),(.095,.124,.100),28,18))
  p.append(paddle('Haunch thorn / pointed green upper leg armor '+str(side),GREEN,[(side*.165,.490,-.040),(side*.215,.585,-.140)],[.048,.002],[.037,.002],segments=16,steps=3))
  p.append(curved_tube('Shank / slender pale mantis knee stalk '+str(side),CREAM,[(side*.155,.384,-.025),(side*.175,.299,.012),(side*.178,.249,.015)],[.035,.028,.026],16))
  p.append(paddle('Greave / flared green mantis lower leg armor '+str(side),GREEN,[(side*.178,.278,.015),(side*.185,.145,.025),(side*.187,.060,.030)],[.032,.076,.070],[.027,.049,.045],segments=24,steps=4))
  p.append(ellipsoid('Foot / planted broad green mantis pad '+str(side),GREEN,(side*.188,.035,.050),(.073,.035,.095),24,16))
  for j,dx in enumerate((-.052,0.,.052)):
   p.append(curved_tube(f'Foot claw / three pointed ivory mantis nails {side} {j}',IVORY,[(side*.188+dx,.042,.115),(side*.189+dx*1.12,.031,.154+(j==1)*.020),(side*.190+dx*1.13,.012,.182+(j==1)*.022)],[.021,.016,.0018],16))
  for j,(root,tip,w) in enumerate((((side*.071,.861,-.055),(side*.29,1.27,-.82),.095),((side*.072,.831,-.055),(side*.26,1.035,-.76),.078))):
   p += mantis_wing(side,j,root,tip,w)
 p.append(paint('Breast plate / fitted pale mantis throat shield',CREAM,[(0.,.735),(.044,.805),(.039,.924),(-.039,.924),(-.044,.805)],thorax,.005,3))
 for j,y in enumerate((.366,.456,.548)):
  p.append(abdomen_groove('Abdomen seam / three fitted green mantis shell divisions '+str(j),abdomen,y))
 hs=surface(head);pts=[]
 for i in range(19):
  x=-.045+.090*i/18;y=.994+.006*(abs(x)/.045)**2;pts.append((x,y,hs(y,x)+.0015))
 p.append(tube('Mouth / fitted restrained mantis muzzle seam',DARK,pts,[.0015]*len(pts),8))
 return {'name':'scyther','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
  # Transcendental rounding can change only the sign of zero between Python
  # runtimes. Canonicalize it before pinning the actual authored buffers.
  q['normals']=[round(v,7) or 0. for v in q['normals']]
 from chikorita_sculpt import f32
 for i,q in enumerate(p):
  if not q['part'].startswith(('Eye','Pupil /','Nostril /','Breast plate /','Scythe rim /','Wing /')):continue
  vs=[tuple(f32(v) for v in q['positions'][j:j+3]) for j in range(0,len(q['positions']),3)]
  faces=[tuple(q['indices'][j:j+3]) for j in range(0,len(q['indices']),3)]
  rebuilt=crease_normals(part(q['part'],(0.,0.,0.),vs,faces,True),40.);rebuilt['base_color']=q['base_color'];p[i]=rebuilt
 return model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from scyther_glb import export_scyther
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_scyther())
if __name__=='__main__':main()
