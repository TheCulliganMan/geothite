#!/usr/bin/env python3
"""Original stag beetle with curved spiked pincers, barred mouth and clawed hands."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,part
from bovine_sculpt import oval,curved_tube,smooth,reverse_front
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from abra_sculpt import surface
from skin_glb import COORDINATES
from chikorita_sculpt import sub,unit,cross,add,mul
HEIGHT=1.16
SHELL=(.55,.47,.42);DARK=(.25,.21,.19);INK=(.075,.065,.060)
HORN=(.79,.77,.78);IVORY=(.96,.94,.91)
PINCER=((.155,1.015,.0),(.245,1.14,.010),(.286,1.31,.0),(.267,1.47,-.010),(.205,1.590,-.025))
RADII=(.076,.088,.074,.049,.0025)

@lru_cache(maxsize=1)
def raw_sculpture():
 body=loft('Body / continuous oval brown beetle carapace',SHELL,curved_profiles(
  ((.23,.065,.06,-.01),(.28,.17,.12,-.015),(.40,.24,.18,-.02),(.60,.28,.21,-.02),
   (.83,.285,.21,-.01),(.97,.245,.19,0.),(1.08,.13,.10,.01),(1.10,.015,.015,.01)),4),36)
 p=[body];face=surface(body)
 for side in (-1,1):
  x=side*.122;y=.981
  xy=[(x+side*u,y+v) for u,v in ((-.048,.009),(.039,.042),(.041,-.034),(-.042,-.024))]
  rim=paint('Eye socket / fitted stern brown beetle brow '+str(side),DARK,xy,body,.005,3);p.append(rim)
  xy=[(x+.89*(u-x),y+.84*(v-y)) for u,v in xy]
  eye=paint('Eye / fitted angular ivory beetle lens '+str(side),IVORY,xy,rim,.003,2);p.append(eye)
  p.append(paint('Pupil / tiny fitted black beetle slit '+str(side),INK,[(x-side*.009,y+.004),(x+side*.001,y+.012),(x+side*.002,y-.015),(x-side*.006,y-.012)],eye,.0025,2))
 mouth=paint('Mouth / fitted dark upright beetle aperture',INK,oval(0.,.786,.081,.116,32),body,.006,3);p.append(mouth)
 for row,y in enumerate((.702,.756,.810,.864)):
  dy=.015;maxdy=max(abs(y-dy-.786),abs(y+dy-.786));width=.081*math.sqrt(1-(maxdy/.116)**2)-.006
  for side in (-1,1):
   xy=[(side*.013,y-dy),(side*width,y-dy+.004),(side*width,y+dy),(side*.013,y+dy-.004)]
   p.append(paint(f'Tooth / four paired ivory mouth bars {side} {row}',IVORY,xy,mouth,.003,2))
 # Closed toroidal seams follow actual front/back shell sections with no overlapping end caps.
 back=surface(reverse_front(body));sections={}
 for i in range(0,len(body['positions']),3):
  x,y,z=body['positions'][i:i+3];sections[y]=max(sections.get(y,0.),abs(x))
 sections=sorted(sections.items())
 for j,y in enumerate((.335,.455,.575)):
  for (ya,ra),(yb,rb) in zip(sections,sections[1:]):
   if ya<=y<=yb:rx=ra+(rb-ra)*(y-ya)/(yb-ya);break
  pts=[]
  for i in range(64):
   a=math.tau*i/64;x=.99999*rx*math.sin(a)
   z=face(y,x)+.0015 if math.cos(a)>=0 else -back(y,-x)-.0015
   pts.append((x,y,z))
  vertices=[];faces=[];n=8
  for i,point in enumerate(pts):
   tangent=unit(sub(pts[(i+1)%len(pts)],pts[i-1]));other=cross(tangent,(0.,1.,0.))
   vertices += [add(point,add((0.,.002*math.cos(k*math.tau/n),0.),mul(other,.002*math.sin(k*math.tau/n)))) for k in range(n)]
  for i in range(len(pts)):
   for k in range(n):
    a=i*n+k;b=i*n+(k+1)%n;c=((i+1)%len(pts))*n+(k+1)%n;d=((i+1)%len(pts))*n+k;faces.append((a,b,c,d))
  p.append(part('Shell seam / fitted transverse beetle armor band '+str(j),DARK,vertices,faces,True))
 pp,rr=smooth(PINCER,RADII,5)
 def horn_at(y):
  for a,b,ra,rb in zip(pp,pp[1:],rr,rr[1:]):
   if a[1]<=y<=b[1]:
    t=(y-a[1])/(b[1]-a[1]);return tuple(a[k]+t*(b[k]-a[k]) for k in range(3)),ra+t*(rb-ra)
  raise ValueError(y)
 for side in (-1,1):
  p.append(tube('Pincer / long curved pale stag-beetle mandible '+str(side),HORN,[(side*x,y,z) for x,y,z in pp],rr,24))
  for j,(y,direction) in enumerate(((1.15,-1),(1.23,1),(1.31,0),(1.40,-1),(1.47,1))):
   (x,y,z),r=horn_at(y)
   if direction:root=(side*(x+direction*.65*r),y,z);tip=(side*(x+direction*(r+.045)),y+.027,z+.005)
   else:root=(side*x,y,z+.65*r);tip=(side*x,y+.025,z+r+.046)
   p.append(curved_tube(f'Pincer spike / five rooted pale horn thorns {side} {j}',HORN,[root,tip],[.024 if y<1.4 else .017,.0018],12))
  p.append(paddle('Upper arm / slender brown beetle humerus '+str(side),SHELL,[(side*.25,.765,.0),(side*.345,.690,.008),(side*.43,.675,.020)],[.042,.026,.027],[.038,.023,.024],segments=20,steps=4))
  p.append(paddle('Forearm / long upturned brown beetle ulna '+str(side),SHELL,[(side*.43,.675,.020),(side*.49,.805,.032),(side*.53,.980,.045)],[.030,.032,.029],[.027,.028,.026],segments=20,steps=4))
  p.append(ellipsoid('Hand / rounded brown three-claw beetle palm '+str(side),SHELL,(side*.532,1.015,.055),(.059,.064,.052),24,16))
  for j,(dx,y,z,ex,ey,ez) in enumerate(((-.035,1.040,.077,-.047,1.103,.083),(.034,1.042,.078,.046,1.104,.082),(0.,1.002,.095,.003,1.030,.159))):
   p.append(curved_tube(f'Hand claw / three pointed ivory beetle digits {side} {j}',IVORY,[(side*(.532+dx),y,z),(side*(.532+(dx+ex)/2),.5*(y+ey),.5*(z+ez)+.006),(side*(.532+ex),ey,ez)],[.021,.017,.0018],16))
  p.append(curved_tube('Leg / short rooted brown beetle shank '+str(side),SHELL,[(side*.17,.358,-.005),(side*.225,.240,.015),(side*.25,.152,.037)],[.064,.050,.045],20))
  p.append(ellipsoid('Foot / large planted brown beetle tarsus '+str(side),SHELL,(side*.253,.105,.063),(.098,.105,.117),28,18))
  for j,(dx,y,z,ex,ey,ez) in enumerate(((-.059,.073,.131,-.085,.042,.203),(.059,.073,.131,.085,.042,.203),(0.,.140,.155,0.,.129,.220))):
   p.append(curved_tube(f'Foot claw / three broad ivory beetle nails {side} {j}',IVORY,[(side*(.253+dx),y,z),(side*(.253+(dx+ex)/2),.5*(y+ey),.5*(z+ez)+.015),(side*(.253+ex),ey,ez)],[.027,.023,.002],16))
 return {'name':'pinsir','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from pinsir_glb import export_pinsir
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_pinsir())
if __name__=='__main__':main()
