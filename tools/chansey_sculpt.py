#!/usr/bin/env python3
"""Original soft egg bearer with rounded ribbons, gathered hands and a pink cup."""
import argparse,json,math
from pathlib import Path
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import ellipsoid as sphere,loft,curved_profiles,crease_normals,fitted_adaptive_contour,fitted_triangle_contour
from bovine_sculpt import curved_tube as swept_tube,oval
from lugia_sculpt import paddle as swept_paddle
from slow_pair_sculpt import forelimb
from goldfish_sculpt import surface
from cubone_sculpt import skin_shading
from moltres_sculpt import flowing
from skin_glb import COORDINATES
HEIGHT=1.10
PINK=(.96,.73,.79);POUCH=(.90,.63,.70);EGG=(.99,.98,.90)
INK=(.20,.13,.18);WHITE=(.98,.99,.98)

def ellipsoid(name,color,center,radii,segments=28,rings=18):
 cap=(32,24) if name.startswith('Egg /') else (28,20) if name.startswith('Crown /') else (28,18) if name.startswith('Foot /') else (24,16)
 return sphere(name,color,center,radii,min(segments,cap[0]),min(rings,cap[1]))
def paddle(name,color,points,widths,depths,*,segments=28,steps=6):
 return swept_paddle(name,color,points,widths,depths,segments=min(segments,20),steps=min(steps,4))
def curved_tube(name,color,points,radii,segments=28):return swept_tube(name,color,points,radii,min(segments,24))
def paint(name,color,xy,parent,th=.006,sub=3):
 if name.startswith('Mouth lining /'):return fitted_triangle_contour(name,color,xy,parent,.001,precision=7)
 return fitted_adaptive_contour(name,color,xy,None,th,surface_at=surface(parent),initial_subdivisions=1)
def fitted_shading(parts):
 def find(prefix,side=None):return next(p for p in parts if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+side)))
 for i,p in enumerate(parts):
  body='Upper body /' if any(q['part'].startswith('Upper body /') for q in parts) else 'Body /'
  for child,parent in (('Eye /',body),('Eye glint /','Eye /'),('Smile /',body),('Chin /',body),('Mouth lining /',body),('Tongue /','Mouth lining /')):
   if p['part'].startswith(child):parts[i]=skin_shading(p,find(parent,p['part'].split()[-1] if parent=='Eye /' else None));break
 return parts
def ribbon_root(side,index):return (side*(.217 if index==0 else .266 if index==1 else .303),(.978,.915,.839)[index],.015)
def arm_points(side):return ((side*.335,.68,.070),(side*.251,.585,.264),(side*.175,.565,.352),(side*.115,.548,.39),(side*.068,.548,.405))
TAIL_POINTS=((0,.26,-.23),(.18,.29,-.42),(.30,.385,-.50))

def cup(name,color=POUCH):
 rows=[]
 for j in range(14):
  t=.015+.985*j/13;k=math.sqrt(1-(1-t)**2);rows.append((.28+.23*t,.148*k,.120*k,.355))
 for j in reversed(range(14)):
  t=.015+.985*j/13;k=math.sqrt(1-(1-t)**2);rows.append((.303+.207*t,.133*k,.105*k,.355))
 return crease_normals(loft(name,color,rows,32),45)

@lru_cache(maxsize=1)
def anatomy():
 body=loft('Body / broad softly rounded pink Chansey egg body',PINK,curved_profiles(((.145,.092,.075,0),(.205,.285,.220,0),(.365,.407,.318,.010),(.595,.420,.324,.010),(.805,.335,.252,.010),(.995,.204,.145,.015),(1.085,.070,.043,.010),(1.097,.018,.015,.010)),4),48)
 pouch=cup('Pouch / closed pink Chansey cup with actual open rim')
 egg=ellipsoid('Egg / pale Chansey egg supported inside pink pouch',EGG,(0,.470,.355),(.112,.18,.105),48,36)
 p=[body,pouch,egg]
 for s in (-1,1):
  eye=paint('Eye / fitted tiny dark Chansey eye '+str(s),INK,oval(s*.061,.943,.009,.025,32),body,.004,3);p.append(eye)
  p.append(paint('Eye glint / fitted small white Chansey gleam '+str(s),WHITE,oval(s*.060,.953,.0022,.005,20),eye,.002,2))
  for j,(root_y,end_y,end_x) in enumerate(((.978,1.015,.60),(.915,.945,.645),(.839,.828,.635))):
   root_x=.217 if j==0 else (.266 if j==1 else .303)
   p.append(paddle(f'Ribbon / rooted long rounded pink Chansey side ribbon {s} {j}',PINK,[(s*root_x,root_y,.015),(s*.397,root_y-.015,.018),(s*.545,end_y-.027,.024),(s*end_x,end_y,.033)],[.051,.058,.043,.012],[.036,.035,.028,.009]))
  p.append(ellipsoid('Shoulder / rooted short pink Chansey shoulder '+str(s),PINK,(s*.332,.671,.036),(.085,.109,.10),36,26))
  p.append(forelimb('Arm / continuous pink Chansey forelimb and egg-holding palm '+str(s),PINK,arm_points(s),[.065,.062,.042,.056,.003],[.065,.060,.043,.047,.003],segments=20))
  p.append(ellipsoid('Ankle / rooted short pink Chansey foot stem '+str(s),PINK,(s*.239,.163,.023),(.090,.125,.105),36,24))
  p.append(ellipsoid('Foot / grounded broad pink Chansey foot '+str(s),PINK,(s*.243,.065,.060),(.107,.065,.142),40,28))
 smile=flowing([(-.055,.882),(-.028,.868),(.020,.868),(.050,.882),(.038,.873),(.018,.862),(-.027,.862),(-.047,.874)],3)
 p.append(skin_shading(paint('Smile / fitted thin curved closed Chansey smile',INK,smile,body,.004,3),body))
 p.append(paint('Chin / fitted small curved Chansey chin mark',INK,flowing([(-.023,.848),(0,.844),(.020,.848),(.013,.843),(0,.839),(-.018,.844)],3),body,.003,3))
 p.append(curved_tube('Tail / rooted short pointed pink Chansey rear tail',PINK,TAIL_POINTS,[.074,.059,.0015],32))
 return fitted_shading([crease_normals(q,45.) for q in p])

@lru_cache(maxsize=2)
def sculpture(normalize=True):
 p=deepcopy(anatomy());lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 if normalize:
  for q in p:
   q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) or 0. for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'chansey','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'chansey.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
