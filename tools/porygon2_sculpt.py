#!/usr/bin/env python3
"""Original continuous rounded virtual duck with seated cyan belly and paddles."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval
from skin_glb import COORDINATES
HEIGHT=1.05
RED=(.88,.39,.52);BLUE=(.36,.70,.83);INK=(.085,.10,.12);WHITE=(.96,.97,.95);PINK=(.98,.65,.73);PALE=(.68,.87,.93)
PROFILES=((.085,.025,.025,-.06),(.16,.155,.16,-.045),(.27,.267,.245,-.045),(.39,.245,.215,-.032),(.47,.178,.165,-.01),(.54,.084,.085,.056),(.63,.055,.064,.111),(.70,.058,.066,.138),(.76,.099,.113,.175),(.84,.175,.163,.195),(.95,.203,.176,.188),(1.06,.176,.149,.162),(1.13,.080,.069,.15),(1.15,.005,.005,.15))

def belly(rows):
 def at(y):
  for a,b in zip(rows,rows[1:]):
   if a[0]<=y<=b[0]:
    t=(y-a[0])/(b[0]-a[0]);return tuple(a[k]+(b[k]-a[k])*t for k in (1,2,3))
  raise ValueError('belly outside torso')
 n=48;v=[];levels=[.083]+[r[0] for r in rows if .083<r[0]<=.33]+[None]
 for level in levels:
  for j in range(n):
   a=math.tau*j/n;top=.318+.019*math.cos(a)+.012*math.cos(2*a);y=top if level is None else min(level,top-.001);rx,rz,z=at(max(y,rows[0][0]))
   v.append(((rx+.0045)*math.sin(a),y,z+(rz+.0045)*math.cos(a)))
 f=[tuple(reversed(range(n))),tuple((len(levels)-1)*n+j for j in range(n))]
 for k in range(len(levels)-1):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part('Belly / seated curved cyan lower torso',BLUE,v,f,True),35.)

def rotate(p,axis,angle,center):
 c=math.cos(angle);s=math.sin(angle)
 for field in ('positions','normals'):
  values=[]
  for i in range(0,len(p[field]),3):
   x,y,z=p[field][i:i+3];x,y,z=(x-center[0],y-center[1],z-center[2]) if field=='positions' else (x,y,z)
   if axis=='y':x,z=c*x+s*z,-s*x+c*z
   else:y,z=c*y-s*z,s*y+c*z
   values.extend((x+center[0],y+center[1],z+center[2]) if field=='positions' else (x,y,z))
  p[field]=values
 return p

def sculpture():
 rows=curved_profiles(PROFILES,4);body=loft('Body / continuous rounded rose torso neck and head',RED,rows,48);p=[body,belly(rows)];bs=surface(body)
 bill=ellipsoid('Bill / smooth flattened cyan spatulate beak',BLUE,(0,.903,.452),(.137,.061,.238),48,28);p.append(rotate(bill,'x',.14,(0,.903,.452)))
 tail=loft('Tail / slender rooted cyan leaf with curved rear sweep',BLUE,curved_profiles(((.30,.012,.012,-.22),(.42,.025,.023,-.35),(.55,.067,.033,-.48),(.70,.085,.040,-.58),(.83,.067,.037,-.63),(.93,.026,.023,-.66),(.96,.004,.006,-.66)),4),40);p.append(tail)
 for sign in (-1,1):
  center=(sign*.184,.083,.103 if sign<0 else -.010)
  foot=rotate(ellipsoid('Foot / grounded rounded splayed cyan paddle '+str(sign),BLUE,center,(.127,.083,.183),40,26),'y',sign*.29,center);p.append(foot)
  x=sign*.145
  eye=fitted_contour('Eye socket / thin fitted dark oval border '+str(sign),INK,oval(x,.970,.041,.064,40),bs,.006,subdivisions=3);p.append(eye)
  white=fitted_contour('Eye white / large fitted ivory virtual-duck oval '+str(sign),WHITE,oval(x,.970,.038,.061,40),surface(eye),.003,subdivisions=3);p.append(white)
  pupil=fitted_patch('Pupil / fitted dark vertical curious oval '+str(sign),INK,oval(x+sign*.002,.970,.011,.029,32),surface(white),.003);p.append(pupil)
  p.append(fitted_patch('Eye glint / fitted tiny ivory reflection '+str(sign),WHITE,oval(x+sign*.002-.002,.982,.004,.008,24),surface(pupil),.0015))
  p.append(fitted_patch('Foot highlight / seated pale cyan paddle reflection '+str(sign),PALE,oval(center[0]-.019,.124,.019,.008,28),surface(foot),.0015))
 p.append(fitted_patch('Head highlight / subtle fitted rosy reflection',PINK,oval(-.063,1.087,.020,.011,28),bs,.002))
 p=[crease_normals(q,35.) if q['part'].startswith(('Eye socket /','Eye white /','Pupil /','Eye glint /','Foot highlight /','Head highlight /')) else q for q in p]
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 # Preserve the narrow terminal cap after final coordinate quantization.
 p=[crease_normals(q,35.) if q['part'].startswith('Tail /') else q for q in p]
 for q in p:
  if q['part'].startswith('Tail /'):q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'porygon2','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'porygon2.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
