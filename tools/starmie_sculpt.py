#!/usr/bin/env python3
"""Original dual-layer purple star with an eight-spoke gold mount and rose core."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from staryu_sculpt import star_body
from sculpt_geometry import part,crease_normals
from raptor_bird_sculpt import paint,sheet
from abra_sculpt import surface
from chikorita_sculpt import linear
from skin_glb import COORDINATES
HEIGHT=1.03;CENTER=(0.,.40,0.);RADIUS=.48
FRONT=(.56,.53,.70);BACK=(.46,.43,.60);GOLD=(.95,.82,.38)
ROSE=(.84,.37,.42);GLINT=(1.,.89,.88)
FRONT_ANGLES=tuple(math.pi/2+i*math.tau/5 for i in range(5))
BACK_ANGLES=tuple(a+math.pi/5 for a in FRONT_ANGLES)

def layer(back):
 p=star_body();angle=math.pi/5 if back else 0.;c=math.cos(angle);s=math.sin(angle)
 p['part']='Back star / continuous offset five-arm purple lens' if back else 'Front star / continuous five-arm purple lens'
 p['base_color']=linear(BACK if back else FRONT)
 for i in range(0,len(p['positions']),3):
  x,y,z=p['positions'][i:i+3];y-=.40;p['positions'][i:i+3]=[c*x-s*y,.40+s*x+c*y,z-(.10 if back else 0.)]
 for i in range(0,len(p['normals']),3):
  x,y,z=p['normals'][i:i+3];p['normals'][i:i+3]=[c*x-s*y,s*x+c*y,z]
 return crease_normals(p,30.)

def gem_solid(name,color,rows,n=8):
 v=[];faces=[]
 for radius,z in rows:
  v += [(radius*math.cos(math.pi/8+i*math.tau/n),.40+radius*math.sin(math.pi/8+i*math.tau/n),z) for i in range(n)]
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
 for k in range(len(rows)-1):
  for i in range(n):
   a=k*n+i;b=k*n+(i+1)%n;faces.append((a,b,b+n,a+n))
 return part(name,color,v,faces)

@lru_cache(maxsize=1)
def raw_sculpture():
 front=layer(False);back=layer(True);p=[front,back]
 for rear,body,angles in ((False,front,FRONT_ANGLES),(True,back,BACK_ANGLES)):
  skin=surface(body)
  for j,a in enumerate(angles):
   dx,dy=math.cos(a),math.sin(a);tx,ty=-dy,dx
   xy=[(dx*.145+tx*.035,.40+dy*.145+ty*.035),(dx*.434,.40+dy*.434),(dx*.145-tx*.035,.40+dy*.145-ty*.035)]
   ridge=paint(('Back ridge /' if rear else 'Front ridge /')+' seated radial purple arm '+str(j),BACK if rear else FRONT,xy,body,.006,2)
   for i in range(0,len(ridge['positions']),3):
    x,y,z=ridge['positions'][i:i+3]
    if z>skin(y,x):
     radial=x*dx+(y-.40)*dy;cross=x*tx+(y-.40)*ty
     width=max(.001,.035*(.434-radial)/(.434-.145));lift=.010*max(0.,1.-abs(cross)/width)
     ridge['positions'][i+2]+=lift
   p.append(crease_normals(ridge,35.))
 mount=gem_solid('Mount / raised octagonal golden core bezel',GOLD,[(.155,.070),(.188,.112),(.180,.145),(.151,.151)]);p.append(mount)
 for j in range(8):
  a=j*math.tau/8;dx,dy=math.cos(a),math.sin(a);tx,ty=-dy,dx
  xy=[(r*dx+w*tx,.40+r*dy+w*ty) for r,w in ((.145,-.022),(.246,-.019),(.259,-.012),(.259,.012),(.246,.019),(.145,.022))]
  p.append(sheet('Mount spoke / eight beveled gold core rays '+str(j),GOLD,xy,lambda y,x:.130+.020*(math.hypot(x,y-.40)-.145),.016,1))
 gem=gem_solid('Core / large octagonal faceted rose gemstone',ROSE,[(.141,.130),(.153,.155),(.108,.205),(.040,.222)]);p.append(gem)
 p.append(paint('Core glint / fitted pale rose facet reflection',GLINT,[(-.075,.455),(-.015,.481),(-.041,.427)],gem,.0015,2))
 return {'name':'starmie','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(normalize=True):
 model=deepcopy(raw_sculpture());p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from starmie_glb import export_starmie
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_starmie())
