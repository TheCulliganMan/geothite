#!/usr/bin/env python3
"""Original angular metal serpent with a broad toothed jaw and body spikes."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,translate
from bovine_sculpt import curved_tube,oval
from murkrow_sculpt import bill
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.45
STEEL=(.62,.65,.68);LIGHT=(.73,.76,.79);SHADE=(.49,.53,.57)
INK=(.065,.075,.095);IVORY=(.97,.96,.92);RED=(.78,.13,.12)
CHAIN=((.43,.075,.20,.069),(.29,.11,.19,.103),(.08,.17,.16,.15),(-.16,.205,.035,.182),(-.33,.25,-.18,.209),(-.22,.47,-.35,.233),(.01,.73,-.43,.264),(.22,1.00,-.35,.288),(.15,1.27,-.19,.298),(-.11,1.35,-.07,.30))
HEAD_X=-.33

def rock(j,c):
 x,y,z,r=c
 base=ellipsoid('Segment / original faceted steel boulder '+str(j),STEEL if j%3 else LIGHT,(x,y,z),(r,r*.94,r),10,6)
 vertices=[]
 for i in range(0,len(base['positions']),3):
  px,py,pz=base['positions'][i:i+3];a=math.atan2(pz-z,px-x);q=(py-y)/(r*.94)
  bump=1+(.065*math.cos(3*a+j*.53)+.025*math.sin(5*a))*max(0,1-q*q)
  vertices.append((x+(px-x)*bump,py,z+(pz-z)*bump))
 return part(base['part'],STEEL if j%3 else LIGHT,vertices,[tuple(base['indices'][i:i+3]) for i in range(0,len(base['indices']),3)],False)

def block(name,color,center,radii):
 x,y,z=center;rx,ry,rz=radii
 vertices=[(x+s*rx,y+t*ry,z+u*rz) for s,t,u in ((-1,-1,-1),(1,-1,-1),(1,1,-1),(-1,1,-1),(-1,-1,1),(1,-1,1),(1,1,1),(-1,1,1))]
 return part(name,color,vertices,[(0,3,2,1),(4,5,6,7),(0,1,5,4),(3,7,6,2),(0,4,7,3),(1,2,6,5)],False)

def anatomy():
 p=[rock(j,c) for j,c in enumerate(CHAIN)]
 for j,(a,b) in enumerate(zip(CHAIN,CHAIN[1:])):
  p.append(curved_tube('Joint / buried steel segment root '+str(j),SHADE,[a[:3],b[:3]],[min(a[3],b[3])*.40]*2,16))
 p.append(curved_tube('Tail / rooted pointed steel tail',LIGHT,[CHAIN[0][:3],(.54,.060,.21),(.69,.043,.22)],[.055,.038,.0015],10))
 for j in (3,5,7,9):
  x,y,z,r=CHAIN[j]
  for s in (-1,1):
   p.append(curved_tube(f'Spike / paired alternating steel body spike {j} {s}',LIGHT,[(x+s*r*.63,y,z),(x+s*(r+.10),y+.025,z+.01),(x+s*(r+.33),y+.11,z+.025)],[r*.30,r*.17,.0015],8))
 head=translate(bill('Head / broad angular metal Steelix skull',STEEL,[(-.17,.235,1.048,1.244),(.03,.409,1.025,1.345),(.28,.398,1.018,1.277),(.445,.302,1.021,1.172)]),HEAD_X,0,0);p.append(head)
 jaw=translate(bill('Jaw / broad low steel lower jaw',SHADE,[(-.065,.218,.875,.947),(.16,.349,.808,.884),(.397,.310,.821,.885),(.465,.244,.852,.895)]),HEAD_X,0,0);p.append(jaw)
 for s in (-1,1):
  p.append(ellipsoid('Hinge / rooted steel jaw hinge '+str(s),STEEL,(HEAD_X+s*.24,.997,.02),(.105,.149,.102),12,8))
 p.append(block('Mouth back / recessed dark steel jaw chamber',INK,(HEAD_X,.982,.035),(.245,.115,.060)))
 # Alternating tooth divisions leave a real front gap between the two jaws.
 for j,x in enumerate((-.245,-.081,.081,.245)):
  p.append(block('Upper tooth / rooted squared metal tooth '+str(j),LIGHT,(HEAD_X+x,1.009,.365),(.047,.061,.052)))
 for j,x in enumerate((-.164,0,.164)):
  p.append(block('Lower tooth / rooted squared metal tooth '+str(j),STEEL,(HEAD_X+x,.900,.405),(.047,.051,.051)))
 for s in (-1,1):
  xy=[(HEAD_X+s*x,y) for x,y in ((.055,1.200),(.262,1.242),(.247,1.137),(.115,1.124))]
  socket=paint('Socket / fitted angular dark Steelix gaze '+str(s),INK,xy,head,.010,3);p.append(socket)
  cx=sum(x for x,y in xy)/4;cy=sum(y for x,y in xy)/4
  eye=paint('Eye / fitted narrow ivory Steelix eye '+str(s),IVORY,[(cx+.88*(x-cx),cy+.78*(y-cy)) for x,y in xy],socket,.008,3);p.append(eye)
  iris=paint('Iris / fitted red Steelix iris '+str(s),RED,oval(HEAD_X+s*.169,1.181,.017,.025,24),eye,.003,2);p.append(iris)
  p.append(paint('Pupil / fitted narrow dark Steelix pupil '+str(s),INK,oval(HEAD_X+s*.169,1.181,.005,.018,20),iris,.003,2))
 return p

def body_reference_span():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);points=[(.69,.043,.22)]+[q[:3] for q in CHAIN]+[(HEAD_X,1.16,.21)]
 return sum(math.dist(a,b) for a,b in zip(points,points[1:]))*HEIGHT/(hi-lo)

def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'steelix','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'steelix.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
