#!/usr/bin/env python3
"""Original tail-balanced Sentret, informed by complete game volume references."""
import argparse,math,struct
from pathlib import Path
from sculpt_geometry import ellipsoid,tube,part,crease_normals,add,sub,mul,dot,cross,unit,fitted_triangle_contour
from bovine_sculpt import oval,smooth
from lugia_sculpt import paddle
from goldfish_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.0
LIGHT=(.69,.52,.43);DARK=(.35,.31,.28);WHITE=(.98,.92,.80);INK=(.045,.035,.030);PINK=(.94,.53,.65)
TAIL_KNOTS=((0.,.440,-.070),(0.,.335,-.100),(0.,.240,-.025),(0.,.160,.085),(0.,.165,.230),(0.,.170,.370))
TAIL,RADII=smooth(TAIL_KNOTS,[.049,.065,.087,.105,.150,.170],8)
# Sample the rounded terminal dome directly. A finite-slope radius taper
# ends in a cone even when its front projection is circular.
for i in range(1,13):
 angle=i*math.pi/24
 TAIL.append((0.,.170+.010*math.sin(angle),.370+.170*math.sin(angle)))
 RADII.append(max(.0015,.170*math.cos(angle)))
TAIL_SECTIONS=((0,7,LIGHT),(7,13,DARK),(13,20,LIGHT),(20,27,DARK),(27,34,LIGHT),(34,43,DARK),(43,len(TAIL)-1,LIGHT))
def tail_frames():
 frames=[];side=None
 for k,p in enumerate(TAIL):
  tangent=unit(sub(TAIL[min(k+1,len(TAIL)-1)],TAIL[max(0,k-1)]))
  if side is None:side=cross(tangent,(0.,1.,0.) if abs(tangent[1])<.9 else (1.,0.,0.))
  side=unit(sub(side,mul(tangent,dot(side,tangent))));frames.append((side,cross(tangent,side)))
 return tuple(frames)
TAIL_FRAMES=tail_frames()
def tail_parts():
 n=24;vertices=[]
 for k,p in enumerate(TAIL):
  u,v=TAIL_FRAMES[k]
  vertices.extend(add(p,mul(add(mul(u,math.cos(i*math.tau/n)),mul(v,math.sin(i*math.tau/n))),RADII[k])) for i in range(n))
 def faces(count):
  fs=[tuple(reversed(range(n))),tuple((count-1)*n+i for i in range(n))]
  for k in range(count-1):
   for i in range(n):
    a=k*n+i;b=k*n+(i+1)%n;fs.append((a,b,b+n,a+n))
  return fs
 whole=part('Tail / reference surface',LIGHT,vertices,faces(len(TAIL)),True)
 normals={tuple(whole['positions'][i:i+3]):whole['normals'][i:i+3] for i in range(0,len(whole['positions']),3)};result=[]
 for j,(start,end,color) in enumerate(TAIL_SECTIONS):
  label='Tail band / joined charcoal ring ' if color==DARK else 'Tail / joined rounded brown support '
  q=part(label+str(j),color,vertices[start*n:(end+1)*n],faces(end-start+1))
  for i in set(q['indices'][2*(n-2)*3:]):q['normals'][i*3:i*3+3]=normals[tuple(q['positions'][i*3:i*3+3])]
  result.append(q)
 return result

def paint(name,color,xy,parent,th=.003):return fitted_triangle_contour(name,color,xy,parent,th,precision=5)
def belly_ring(body):
 skin=surface(body);segments=48;rows=5;vertices=[]
 for depth in (.005,-.002):
  for row in range(rows):
   t=row/(rows-1);rx=.052+(.108-.052)*t;ry=.072+(.142-.072)*t
   for i in range(segments):
    a=math.tau*i/segments;x=rx*math.cos(a);y=.598+ry*math.sin(a);vertices.append((x,y,skin(y,x)+depth))
 count=rows*segments;faces=[]
 for row in range(rows-1):
  for i in range(segments):
   a=row*segments+i;b=row*segments+(i+1)%segments;faces += [(a,a+segments,b+segments,b),(a+count,b+count,b+count+segments,a+count+segments)]
 for i in range(segments):
  j=(i+1)%segments;last=(rows-1)*segments;faces += [(i,j,j+count,i+count),(last+j,last+i,last+i+count,last+j+count)]
 return crease_normals(part('Belly ring / continuous fitted cream annulus',WHITE,vertices,faces),45.)

def sculpture(*,normalize=True):
 body=ellipsoid('Body / round warm-brown lookout belly and skull',LIGHT,(0.,.641,0.),(.255,.290,.197),32,22);p=[body,belly_ring(body)]
 for s in (-1,1):
  eye=paint('Eye / tiny fitted black lookout gaze '+str(s),INK,oval(s*.041,.843,.008,.010,20),body,.002);p.append(eye)
  p.append(paint('Glint / minute lookout eye light '+str(s),WHITE,oval(s*.041-.002,.846,.0016,.0020,12),eye,.0008))
  ear=ellipsoid('Ear / narrow round-tipped charcoal pinna '+str(s),DARK,(s*.122,1.004,-.010),(.044,.139,.027),24,18)
  vs=[tuple(ear['positions'][i+k]+(s*.19*(ear['positions'][i+1]-.875) if k==0 else 0.) for k in range(3)) for i in range(0,len(ear['positions']),3)]
  ear=part(ear['part'],DARK,vs,[tuple(ear['indices'][i:i+3]) for i in range(0,len(ear['indices']),3)],True);p.append(ear)
  p.append(paint('Ear inset / fitted long pink pinna '+str(s),PINK,[(s*x,y) for x,y in ((.105,.890),(.142,.925),(.166,1.076),(.150,1.110),(.129,1.079))],ear,.003))
  p.append(paddle('Arm / broad raised lookout flank '+str(s),LIGHT,[(s*.190,.650,.012),(s*.277,.693,.020),(s*.310,.761,.035)],[.058,.067,.024],[.050,.039,.022],segments=18,steps=4))
  palm=ellipsoid('Hand / rounded lookout paw '+str(s),LIGHT,(s*.310,.768,.037),(.030,.031,.028),18,12);p.append(palm)
  for j,dx in enumerate((-.011,0.,.011)):
   p.append(ellipsoid(f'Finger / small rounded lookout knuckle {s} {j}',LIGHT,(s*.310+dx,.782,.055),(.006,.009,.006),12,8))
  p.append(tube('Leg / upright suspended lookout hindlimb '+str(s),LIGHT,[(s*.096,.394,.012),(s*.100,.333,.025),(s*.102,.282,.039)],[.034,.033,.026],14))
  p.append(ellipsoid('Foot / short suspended brown lookout pad '+str(s),LIGHT,(s*.102,.280,.049),(.035,.042,.043),18,12))
  for j,dx in enumerate((-.016,0.,.016)):
   p.append(ellipsoid(f'Toe / blunt suspended lookout digit {s} {j}',LIGHT,(s*.102+dx,.263,.077),(.009,.013,.014),12,8))
 p.append(paddle('Muzzle / shallow lookout nose bridge',LIGHT,[(0.,.830,.125),(0.,.821,.149),(0.,.813,.160)],[.018,.019,.008],[.012,.011,.006],segments=14,steps=3))
 p.append(ellipsoid('Nose / minute brown lookout tip',LIGHT,(0.,.815,.162),(.006,.004,.004),14,8))
 p.append(paint('Mouth / tiny fitted inverted lookout chevron',INK,[(-.017,.802),(0.,.812),(.017,.802),(.014,.799),(0.,.807),(-.014,.799)],body,.002))
 p+=tail_parts()
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   values=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
   q['positions']=list(struct.unpack('<'+'f'*len(values),struct.pack('<'+'f'*len(values),*values)))
   if not q['part'].startswith(('Tail /','Tail band /')):q.update(crease_normals(q,40.))
   q['normals']=[round(v,7) or 0. for v in q['normals']]
 return {'name':'sentret','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from sentret_glb import export_sentret
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_sentret())
if __name__=='__main__':main()
