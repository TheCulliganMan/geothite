#!/usr/bin/env python3
"""Original continuous five-arm Staryu with a raised collar and faceted ruby core."""
import argparse,math
from pathlib import Path
from sculpt_geometry import part,tube
from raptor_bird_sculpt import paint
from skin_glb import COORDINATES
HEIGHT=1.0
# Verified local pack normal/front/back RGB555 (23,17,11) and (31,5,3).
TAN=(189/255,140/255,90/255);RED=(1.,41/255,24/255);GOLD=(.91,.73,.34);WHITE=(1.,1.,1.)
CENTER=(0.,.40,0.);RADIUS=.48;INNER=.195
ANGLES=tuple(math.pi/2+i*math.tau/5 for i in range(5))

def star_body():
 outline=[(math.cos(math.pi/2+i*math.pi/5)*r,.40+math.sin(math.pi/2+i*math.pi/5)*r) for i in range(10) for r in [RADIUS if i%2==0 else INNER]]
 boundary=[]
 for a,b in zip(outline,outline[1:]+outline[:1]):
  for j in range(6):boundary.append(tuple(a[k]+(b[k]-a[k])*j/6 for k in range(2)))
 n=len(boundary);v=[];f=[]
 # Closed lens rings preserve the complete star outline without five disconnected limbs.
 for scale,z in [(1.,0.),(.92,.022),(.70,.059),(.40,.082),(.10,.092),(.10,-.078),(.40,-.071),(.70,-.049),(.92,-.020)]:
  v += [(x*scale,.40+(y-.40)*scale,z) for x,y in boundary]
 for ring in (0,1,2,3,5,6,7):
  for j in range(n):
   a=ring*n+j;b=ring*n+(j+1)%n;f.append((a,b,b+n,a+n))
 for j in range(n):
  a=8*n+j;b=8*n+(j+1)%n;f.append((a,b,(j+1)%n,j))
 front=len(v);v.append((0.,.40,.092));back=len(v);v.append((0.,.40,-.078))
 for j in range(n):f.extend([(front,4*n+j,4*n+(j+1)%n),(back,5*n+(j+1)%n,5*n+j)])
 return part('Body / continuous five-arm star lens',TAN,v,f,True)

def ring(name,color,radius,thickness,z,segments=32,sides=10):
 v=[];f=[]
 for i in range(segments):
  a=i*math.tau/segments
  for j in range(sides):
   b=j*math.tau/sides;r=radius+thickness*math.cos(b)
   v.append((r*math.cos(a),.40+r*math.sin(a),z+thickness*math.sin(b)))
 for i in range(segments):
  for j in range(sides):
   a=i*sides+j;b=((i+1)%segments)*sides+j;c=((i+1)%segments)*sides+(j+1)%sides;d=i*sides+(j+1)%sides
   f.append((a,b,c,d))
 return part(name,color,v,f,True)

def sculpture(*,normalize=True):
 body=star_body();p=[body]
 # Fitted radial ridges are shallow, closed contours carried by the same arm weights.
 for i,a in enumerate(ANGLES):
  dx,dy=math.cos(a),math.sin(a);tx,ty=-dy,dx
  xy=[(dx*.142+tx*.026,.40+dy*.142+ty*.026),(dx*.405,.40+dy*.405),(dx*.142-tx*.026,.40+dy*.142-ty*.026)]
  p.append(paint('Ridge / seated radial arm '+str(i),TAN,xy,body,.006,3))
 p.append(tube('Collar / raised central gold drum',GOLD,[(0.,.40,.07),(0.,.40,.102),(0.,.40,.128)],[.148,.145,.128],24))
 p.append(ring('Collar / rounded gold ruby bezel',GOLD,.119,.021,.135))
 # Explicit planar gemstone facets, rather than a smooth red ball.
 v=[];n=12
 for radius,z in [(.097,.119),(.103,.145),(.071,.175),(.024,.183)]:
  v += [(radius*math.cos(i*math.tau/n),.40+radius*math.sin(i*math.tau/n),z) for i in range(n)]
 f=[tuple(reversed(range(n))),tuple(3*n+i for i in range(n))]
 for k in range(3):
  for i in range(n):
   a=k*n+i;b=k*n+(i+1)%n;f.append((a,b,b+n,a+n))
 gem=part('Core / twelve-sided faceted red ruby',RED,v,f);p.append(gem)
 p.append(paint('Core glint / small fitted white facet',WHITE,[(-.051,.444),(-.019,.455),(-.031,.425)],gem,.0015,1))
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if j%3==1 else 0.))*scale,7) for j,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'staryu','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from staryu_glb import export_staryu
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_staryu())
if __name__=='__main__':main()
