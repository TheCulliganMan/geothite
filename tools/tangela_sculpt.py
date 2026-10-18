#!/usr/bin/env python3
"""Original rounded vine tangle with a clear eye opening and broad red shoes."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from sculpt_geometry import ellipsoid,tube,crease_normals,part,fitted_adaptive_contour
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint
from cubone_sculpt import skin_shading
from skin_glb import COORDINATES,smoothstep
from goldfish_sculpt import surface
VINE_PATHS={}
HEIGHT=1.00
BLUE=(.34,.64,.77);BACK_BLUE=(.26,.47,.61);INK=(.085,.095,.11)
WHITE=(.98,.98,.96);RED=(.91,.36,.25)
def bounded_rope(name,color,pts,rr,segments=12):
 # The hidden root centers belong inside the stored core, including lateral
 # curls whose old cap only grazed its surface. Blend back to the authored
 # outer loop over the first few samples.
 seated=[]
 for k,(x,y,z) in enumerate(pts):
  t=k/(len(pts)-1);uncompressed=.70+(y-.70)/.60 if y>.70 else y
  radius=math.sqrt((x/.32)**2+((uncompressed-.48)/.37)**2+(z/.255)**2)
  limit=.80+.30*smoothstep(0.,.16,t)
  if t<.16 and radius>limit:
   factor=limit/radius;x*=factor;uncompressed=.48+(uncompressed-.48)*factor;z*=factor;y=.70+(uncompressed-.70)*.60 if uncompressed>.70 else uncompressed
  seated.append((x,y,z))
 pts=seated
 # Rope width follows its local bend while the independent mesh crossing
 # test still checks every nonadjacent face after construction.
 for k in range(1,len(pts)-1):
  a,b,c=pts[k-1:k+2];u=tuple(a[j]-b[j] for j in range(3));v=tuple(c[j]-b[j] for j in range(3));w=tuple(c[j]-a[j] for j in range(3))
  cross=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);area2=math.sqrt(sum(q*q for q in cross))
  if area2>1e-12:
   radius=math.sqrt(sum(q*q for q in u)*sum(q*q for q in v)*sum(q*q for q in w))/(2*area2);rr[k]=min(rr[k],.45*radius)
 VINE_PATHS[name]=tuple(tuple(p) for p in pts)
 return crease_normals(tube(name,color,pts,rr,segments),60)
@lru_cache(maxsize=1)
def anatomy():
 body=ellipsoid('Body / concealed rounded blue Tangela tangle core',BACK_BLUE,(0,.48,0),(.32,.37,.255),28,18)
 vertices=[]
 for i in range(0,len(body['positions']),3):
  x,y,z=body['positions'][i:i+3];vertices.append((x,.70+(y-.70)*.60 if y>.70 else y,z))
 body=part(body['part'],BACK_BLUE,vertices,[tuple(body['indices'][i:i+3]) for i in range(0,len(body['indices']),3)],True);p=[body]
 p.append(skin_shading(fitted_adaptive_contour('Face / fitted dark Tangela eye opening',INK,oval(0,.565,.177,.128,32),body,.004,surface_at=surface(body),initial_subdivisions=1),body))
 for s in (-1,1):
  shoe=ellipsoid('Shoe / rooted broad rounded red Tangela shoe '+str(s),RED,(s*.190,.105,.105),(.132,.105,.175),28,18);p.append(shoe)
  eye=ellipsoid('Eye / rooted white Tangela oval in dark opening '+str(s),WHITE,(s*.088,.565,.244),(.058,.082,.030),24,16);p.append(eye)
  p.append(paint('Pupil / fitted tiny high black Tangela pupil '+str(s),INK,oval(s*.088,.608,.005,.008,20),eye,.003,1))
 # A front strand turns through one open spiral. The shrinking radius and
 # tube width leave physical space between its outer turn and curled tip.
 def spiral(name,cx,cy,rx,ry,start,z,side=1,facing=1):
  cx+=.007 if side==1 else -.004;cy+=.009 if side==1 else -.005
  start+=.20 if side==1 else -.12;rx*=.94 if side==1 else 1.02
  def at(t):
   a=start+6.55*t;r=1-.78*t
   depth=.18*(1-min(1,t/.25))**2
   return (side*(cx+rx*r*math.cos(a)),cy+ry*r*math.sin(a),facing*(z+.015*math.cos(a)-depth))
  pts=[at(k/24) for k in range(25)]
  rr=[.037*(1-.78*k/24)*(.30+.70*min(1,k/24/.20))*(.4+.6*min(1,(1-k/24)/.12)) for k in range(25)]
  return bounded_rope(name,BLUE,pts,rr)
 for side in (-1,1):
  for j,(cx,cy,rx,ry,start,z) in enumerate(((.22,.76,.115,.095,-2.3,.175),(.31,.525,.125,.115,-2.6,.225),(.17,.35,.105,.090,1.6,.270))):
   p.append(spiral(f'Front vine / rooted individually curled blue Tangela strand {side} {j}',cx,cy,rx,ry,start,z,side))
 p.append(spiral('Front weave / rooted blue Tangela crest or lower loop 0',0,.35,.105,.09,.35,.295))
 p.append(spiral('Front weave / rooted blue Tangela crest or lower loop 1',0,.79,.10,.09,-1.8,.18))
 for side in (-1,1):
  for j,(cx,cy,rx,ry,start,z) in enumerate(((.12,.75,.14,.10,-1.8,.270),(.06,.81,.12,.07,-1.8,.250),(.29,.58,.10,.13,-1.7,.260),(.30,.41,.12,.12,1.6,.260),(.15,.31,.13,.10,1.6,.250),(.065,.30,.11,.08,1.8,.265))):
   p.append(spiral(f'Front weave / rooted layered blue Tangela tangle {side} {j}',cx,cy,rx,ry,start,z,side))
 # Ropes overlap the perimeter of the dark opening, so it is framed by
 # actual vines rather than reading as a smooth oval badge on the core.
 frame=[([(-.19,.655,.12),(-.15,.682,.226),(-.07,.675,.268),(.035,.664,.281),(.13,.686,.230),(.19,.672,.10)],[.008,.028,.034,.034,.025,.008]),
        ([(-.19,.468,.13),(-.12,.452,.262),(-.035,.470,.294),(.055,.477,.302),(.135,.458,.253),(.20,.482,.11)],[.008,.028,.034,.034,.025,.008])]
 for side in (-1,1):frame.append(([(side*.18,.475,.11),(side*.215,.515,.208),(side*.214,.594,.222),(side*.184,.656,.202),(side*.16,.675,.10)],[.008,.029,.032,.025,.008]))
 for j,(pts,rr) in enumerate(frame):p.append(crease_normals(curved_tube('Front weave / rooted Tangela eye-frame rope '+str(j),BLUE,pts,rr,12),60))
 # Differently tilted rear paths cross over one another around the volume,
 # avoiding a stack of parallel horizontal bands.
 for j,tilt in enumerate((.65,-.42,.30,-.72,.55,-.32,.80,-.58,.37)):
  pts=[];rr=[]
  for k in range(33):
   t=k/32;a=.35+5.35*t;lat=(j-4)*.145+.10*math.sin(t*12+j*.91)
   x=.349*math.cos(lat)*math.sin(a);dy=.400*math.sin(lat);z=.283*math.cos(lat)*math.cos(a)
   x,dy=x*math.cos(tilt)-dy*math.sin(tilt),x*math.sin(tilt)+dy*math.cos(tilt)
   radius=math.sqrt((x/.32)**2+(dy/.37)**2+(z/.255)**2);x/=radius;dy/=radius;z/=radius
   y=.48+dy;y=.70+(y-.70)*.60 if y>.70 else y
   root=.60+.51*min(1,t/.04);pts.append((x*root,.48+(y-.48)*root,z*root))
   rr.append((.008+.024*min(1,t/.04))*(1-.90625*max(0,(t-.9)/.1)))
  p.append(bounded_rope('Rear vine / rooted wavy blue Tangela wrapping strand '+str(j),BACK_BLUE if j%3==0 else BLUE,pts,rr,10))
 # Tangential side curls fill the oblique silhouette with real rope geometry.
 for side in (-1,1):
  for j,cy in enumerate((.38,.66)):
   curl=spiral(f'Front weave / rooted Tangela side curl {side} {j}',0,cy,.12,.12,-1.8,.33,1)
   angle=side*1.35;c=math.cos(angle);s=math.sin(angle)
   curl['positions']=[v for x,y,z in zip(curl['positions'][::3],curl['positions'][1::3],curl['positions'][2::3]) for v in (c*x+s*z,y,-s*x+c*z)]
   curl['normals']=[v for x,y,z in zip(curl['normals'][::3],curl['normals'][1::3],curl['normals'][2::3]) for v in (c*x+s*z,y,-s*x+c*z)]
   VINE_PATHS[curl['part']]=tuple((c*x+s*z,y,-s*x+c*z) for x,y,z in VINE_PATHS[curl['part']])
   p.append(curl)
 for j,(cx,cy,rx,ry,start,z,side) in enumerate(((0,.75,.16,.095,-2,.325,1),(.15,.66,.15,.13,-1.8,.325,1),(.13,.60,.15,.13,-1.5,.320,-1),(0,.28,.17,.08,1.4,.320,1))):
  p.append(spiral('Rear weave / rooted blue Tangela rear curl '+str(j),cx,cy,rx,ry,start,z,side,-1))
 return p
@lru_cache(maxsize=2)
def sculpture(normalize=True):
 import copy
 p=copy.deepcopy(anatomy());lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo) if normalize else 1.
 for q in p:
  q['positions']=[round((v-(lo if normalize and i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'tangela','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);from tangela_glb import export_tangela; (args.output/'tangela.glb').write_bytes(export_tangela())
if __name__=='__main__':main()
