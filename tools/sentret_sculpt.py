#!/usr/bin/env python3
"""Original Sentret lookout with a fitted belly ring and curled ringed tail support."""
import argparse,math
from pathlib import Path
from sculpt_geometry import ellipsoid,tube,part,crease_normals,add,sub,mul,dot,cross,unit
from raptor_bird_sculpt import paint
from goldfish_sculpt import surface
from bovine_sculpt import oval,smooth
from lugia_sculpt import paddle
from skin_glb import COORDINATES
HEIGHT=1.0
LIGHT=(181/255,123/255,82/255);DARK=(123/255,57/255,33/255);WHITE=(1.,1.,1.);INK=(0.,0.,0.)
TAIL_KNOTS=((0.,.33,-.105),(-.065,.27,-.235),(-.15,.12,-.24),(-.22,.056,-.22),(-.30,.072,-.19),(-.34,.145,-.18),(-.34,.183,-.18),(-.337,.211,-.178))
TAIL,RADII=smooth(TAIL_KNOTS,[.054,.067,.063,.055,.049,.042,.028,.0015],8)

def tail_frames():
 frames=[];side=None
 for k,point in enumerate(TAIL):
  tangent=unit(sub(TAIL[min(k+1,len(TAIL)-1)],TAIL[max(0,k-1)]))
  if side is None:side=cross(tangent,(0.,1.,0.) if abs(tangent[1])<.9 else (1.,0.,0.))
  side=unit(sub(side,mul(tangent,dot(side,tangent))));frames.append((side,cross(tangent,side)))
 return tuple(frames)
TAIL_FRAMES=tail_frames()

def tail_section(name,color,start,end,inflate=0.):
 # Every band uses the same full-tail transported frame and centerline.
 n=24;vertices=[];faces=[]
 for k in range(start,end+1):
  u,v=TAIL_FRAMES[k];radius=RADII[k]+inflate
  for i in range(n):vertices.append(add(TAIL[k],mul(add(mul(u,math.cos(i*math.tau/n)),mul(v,math.sin(i*math.tau/n))),radius)))
 faces=[tuple(reversed(range(n))),tuple((end-start)*n+i for i in range(n))]
 for k in range(end-start):
  for i in range(n):
   a=k*n+i;b=k*n+(i+1)%n;faces.append((a,b,b+n,a+n))
 return part(name,color,vertices,faces,True)

def belly_ring(body):
 skin=surface(body);segments=48;rows=5;vertices=[]
 for depth in (.006,-.002):
  for row in range(rows):
   t=row/(rows-1);rx=.067+(.124-.067)*t;ry=.076+(.139-.076)*t
   for i in range(segments):
    a=math.tau*i/segments;x=rx*math.cos(a);y=.555+ry*math.sin(a);vertices.append((x,y,skin(y,x)+depth))
 count=rows*segments;faces=[]
 for row in range(rows-1):
  for i in range(segments):
   a=row*segments+i;b=row*segments+(i+1)%segments
   faces += [(a,a+segments,b+segments,b),(a+count,b+count,b+count+segments,a+count+segments)]
 for i in range(segments):
  j=(i+1)%segments;last=(rows-1)*segments
  faces += [(i,j,j+count,i+count),(last+j,last+i,last+i+count,last+j+count)]
 return crease_normals(part('Belly ring / continuous fitted white annulus',WHITE,vertices,faces),45.)

def sculpture(*,normalize=True):
 body=ellipsoid('Body / continuous light-brown sentry pear',LIGHT,(0.,.58,0.),(.225,.285,.175),32,22);p=[body,belly_ring(body)]
 for s in (-1,1):
  eye=paint('Eye / fitted small black lookout gaze '+str(s),INK,oval(s*.080,.752,.022,.028,24),body,.004,2);p.append(eye)
  p.append(paint('Glint / small white lookout eye '+str(s),WHITE,oval(s*.080-.006,.762,.005,.007,16),eye,.002,2))
  ear=ellipsoid('Ear / upright round-tipped lookout pinna '+str(s),LIGHT,(s*.135,.930,-.025),(.065,.115,.035),24,18);p.append(ear)
  p.append(paint('Ear inset / fitted dark-brown inner pinna '+str(s),DARK,[(s*x,y) for x,y in ((.117,.851),(.170,.951),(.155,1.008),(.131,.980),(.108,.887))],ear,.004,3))
  p.append(tube('Arm / small raised lookout forelimb '+str(s),LIGHT,[(s*.176,.636,.025),(s*.244,.687,.066),(s*.276,.721,.078)],[.035,.030,.022],14))
  palm=ellipsoid('Hand / blunt lookout paw '+str(s),LIGHT,(s*.279,.724,.082),(.034,.041,.028),18,12);p.append(palm)
  for j,dx in enumerate((-.015,0.,.015)):
   p.append(tube('Finger / three small lookout paw digits '+str(s)+' '+str(j),LIGHT,[(s*.279+dx,.737,.099),(s*.284+dx,.765,.105)],[.008,.004],10))
  p.append(tube('Leg / short suspended lookout hindlimb '+str(s),LIGHT,[(s*.078,.365,.025),(s*.087,.310,.068),(s*.091,.279,.101)],[.034,.027,.018],14))
  foot=ellipsoid('Foot / suspended dark-brown lookout pad '+str(s),DARK,(s*.093,.271,.135),(.044,.026,.065),18,12);p.append(foot)
  for j,dx in enumerate((-.020,0.,.020)):
   p.append(tube('Toe / three suspended lookout digits '+str(s)+' '+str(j),DARK,[(s*.093+dx*.5,.277,.169),(s*.093+dx,.267,.205)],[.010,.004],10))
 p.append(paddle('Muzzle / compact light-brown lookout snout',LIGHT,[(0.,.741,.123),(0.,.733,.179),(0.,.721,.208)],[.032,.034,.009],[.022,.022,.008],segments=16,steps=4))
 p.append(ellipsoid('Nose / small black lookout tip',INK,(0.,.727,.210),(.014,.011,.011),16,10))
 p.append(paint('Mouth / quiet fitted lookout smile',DARK,[(-.038,.716),(-.018,.704),(0.,.701),(.018,.704),(.038,.716),(.021,.711),(0.,.708),(-.021,.711)],body,.003,2))
 p.append(tail_section('Tail / continuous curled lookout support',LIGHT,0,len(TAIL)-1))
 for j,(start,end) in enumerate(((5,11),(16,22),(28,34))):
  p.append(tail_section('Tail band / fitted dark lookout ring '+str(j),DARK,start,end,.0013))
 p.append(tail_section('Tail band / rounded dark terminal cap',DARK,40,len(TAIL)-1,.0013))
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'sentret','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from sentret_glb import export_sentret
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_sentret())
if __name__=='__main__':main()
