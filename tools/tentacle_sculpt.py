#!/usr/bin/env python3
"""Original jelly bells, rooted curved tentacles, fitted ruby lobes and mantle eyes."""
import math
from functools import lru_cache
from sculpt_geometry import loft,curved_profiles,ellipsoid,part
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint
from goldfish_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('tentacool','tentacruel');HEIGHTS={s:1.13 for s in SPECIES}
# Normal local pack RGB555: blue(11,20,31), pink(30,6,10), red(26,2,2).
BLUE=(90/255,165/255,1.);PINK=(247/255,49/255,82/255);RED=(214/255,16/255,16/255)
INK=(0.,0.,0.);WHITE=(1.,1.,1.)

def paths(s):
 if s=='tentacool':
  return [((sign*.10,.59,.025),(sign*.13,.46,.01),(sign*.18,.29,.015),(sign*.24,.105,.065),(sign*.34,.035,.14),(sign*.40,.065,.19)) for sign in (-1,1)]
 if s!='tentacruel':raise ValueError('unknown jelly species')
 out=[]
 for j in range(10):
  a=math.tau*j/10;x=math.cos(a);z=math.sin(a)
  out.append(((x*.18,.60,z*.13),(x*.22,.48,z*.16),(x*.27,.32,z*.21),(x*.30,.15,z*.26),(x*.36,.042,z*.30),(x*.42,.027+.009*(j%3),z*.32+.035)))
 return out

def jewel(name,color,xy,radii,parent):
 x,y=xy;rx,ry=radii
 skin=surface(parent);q=ellipsoid(name,color,(x,y,0.),(rx,ry,.045),24,14)
 vs=[(a,b,skin(b,a)+c+.014) for a,b,c in zip(q['positions'][::3],q['positions'][1::3],q['positions'][2::3])]
 fs=[tuple(q['indices'][i:i+3]) for i in range(0,len(q['indices']),3)]
 return part(name,color,vs,fs,True)

def anatomy(s):
 if s not in SPECIES:raise ValueError('unknown jelly species')
 cruel=s=='tentacruel';w=.44 if cruel else .29
 profiles=curved_profiles(((.635,w*.90,.215 if cruel else .175,0.),(.68,w,.245 if cruel else .20,0.),(.78,w*.96,.265 if cruel else .225,0.)),3)
 for i in range(1,15):
  a=(math.pi/2-.01)*i/14;profiles.append((.78+.325*math.sin(a),w*.96*math.cos(a),(.265 if cruel else .225)*math.cos(a),0.))
 bell=loft('Bell / continuous flared blue dome',BLUE,profiles,32)
 mantle=loft('Mantle / dark recessed eye hood',INK,curved_profiles(((.505,.07,.11,.055),(.55,.18 if cruel else .16,.19,.052),(.62,.29 if cruel else .215,.24,.045),(.72,.30 if cruel else .22,.23,.040)),3),28)
 p=[bell,mantle];color=RED if cruel else PINK
 for sign in (-1,1):
  x=sign*(.21 if cruel else .135);y=.868 if cruel else .866;rx=.115 if cruel else .080;ry=.135 if cruel else .123
  gem=jewel('Jewel / large rooted red bell lobe '+str(sign),color,(x,y),(rx,ry),bell);p.append(gem)
  p.append(paint('Jewel glint / fitted white lobe highlight '+str(sign),WHITE,oval(x-.021,y+.047,.024 if cruel else .019,.028,16),gem,.003,2))
 center=jewel('Jewel / small central blue ridge',BLUE,(0.,.71),(.039,.049),bell);p.append(center)
 for sign in (-1,1):
  x=sign*(.14 if cruel else .105);y=.633
  xy=[(x-sign*.071,y+.031),(x+sign*.068,y+.050),(x+sign*.052,y-.030),(x-sign*.048,y-.026)]
  white=paint('Eye / fitted angular ivory mantle gaze '+str(sign),WHITE,xy,mantle,.014,2);p.append(white)
  p.append(paint('Pupil / fitted narrow black gaze '+str(sign),INK,oval(x+sign*.012,y+.001,.016,.028,16),white,.003,2))
 if cruel:
  for sign in (-1,1):
   p.append(curved_tube('Arm / swept blue side feeler '+str(sign),BLUE,[(sign*.28,.67,.01),(sign*.36,.57,.11),(sign*.40,.43,.16),(sign*.45,.35,.17)],[.052,.046,.026,.0025],14))
 for j,path in enumerate(paths(s)):
  p.append(curved_tube('Tentacle / continuous rooted blue tendril '+str(j),BLUE,path,[.033,.031,.025,.019,.012,.003],12))
 return p
@lru_cache(maxsize=2)
def transform(s):
 ps=anatomy(s);lo=min(v for p in ps for v in p['positions'][1::3]);hi=max(v for p in ps for v in p['positions'][1::3]);return lo,HEIGHTS[s]/(hi-lo)
def normalized(s,p):
 lo,k=transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(p))
def sculpture(s,*,normalize=True):
 ps=anatomy(s)
 if normalize:
  lo,k=transform(s)
  for p in ps:
   p['positions']=[round((v-(lo if i%3==1 else 0.))*k,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return dict(name=s,version=1,coordinate_system=COORDINATES,primitives=ps)
if __name__=='__main__':
 import argparse
 from pathlib import Path
 from tentacle_glb import export_tentacle
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_tentacle(s))
