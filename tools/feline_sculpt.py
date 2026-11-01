#!/usr/bin/env python3
"""Original coin cat and lean quadruped: continuous bodies, fitted faces and rooted whiskers."""
import math
from functools import lru_cache
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,fitted_triangle_contour,tube
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from skin_glb import COORDINATES
SPECIES=('meowth','persian');HEIGHTS={s:1.01 for s in SPECIES}
WHITE=(1.,1.,1.);INK=(0.,0.,0.)
GOLD={'meowth':(1.,222/255,107/255),'persian':(1.,230/255,82/255)}
BROWN={'meowth':(165/255,66/255,24/255),'persian':(99/255,74/255,33/255)}
TAILS={
 'meowth':((0.,.29,-.11),(.13,.28,-.28),(.31,.35,-.32),(.38,.50,-.25),(.34,.60,-.19),(.24,.61,-.17),(.20,.54,-.18),(.25,.50,-.19)),
 'persian':((0.,.43,-.38),(.025,.55,-.56),(.08,.76,-.64),(.11,.95,-.57),(.10,1.08,-.42),(.07,1.09,-.27),(.04,.99,-.23),(.045,.92,-.28))}
def fitted(name,color,xy,parent,th=.004):return crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),35)
def head_center(s):return (0.,.76,.016) if s=='meowth' else (0.,.665,.39)
def support_paths(s):
 if s=='meowth':return {str(side):((side*.095,.285,-.005),(side*.125,.155,.025),(side*.14,.065,.070)) for side in (-1,1)}
 return {f'{side} {row}':((side*.105,.45,.22 if row==0 else -.25),(side*.145,.27,.25 if row==0 else -.34),(side*.16,.070,.29 if row==0 else -.25)) for side in (-1,1) for row in (0,1)}
def arm_path(side):
 return ((side*.105,.47,.015),(side*.235,.48 if side==-1 else .59,.045),(side*.28,.57 if side==-1 else .76,.09))
def ear_paths(s,side):
 if s=='meowth':return ((side*.135,.865,-.015),(side*.205,.988,-.025),(side*.215,1.075,-.025))
 return ((side*.14,.745,.32),(side*.225,.81,.315),(side*.265,.855,.315))
def fitted_line(name,color,xy,parent,r=.0025):
 # The line follows the actual triangle surface, including its faceting.
 from abra_sculpt import surface as stored_surface
 surface=stored_surface(parent)
 return tube(name,color,[(x,y,surface(y,x)+r*.45) for x,y in xy],[r]*len(xy),10)
def anatomy(s):
 if s not in SPECIES:raise ValueError('unknown feline')
 gold=GOLD[s];brown=BROWN[s];meow=s=='meowth'
 if meow:
  body=crease_normals(loft('Body / continuous slender upright coin-cat torso',gold,curved_profiles(((.16,.035,.04,-.01),(.24,.115,.095,-.015),(.35,.145,.115,-.012),(.47,.115,.09,0.),(.55,.063,.057,.004)),4),28),40)
  neck=curved_tube('Neck / rooted short coin-cat throat',gold,[(0.,.49,.005),(0.,.59,.014),(0.,.65,.020)],[.073,.073,.09],20)
 else:
  # Long axis lies along Z; the silhouette narrows at the loin and rises at the shoulders.
  raw=loft('Body / continuous lean long Persian trunk',gold,curved_profiles(((-.41,.045,.055,.435),(-.32,.13,.145,.45),(-.15,.135,.15,.43),(.04,.12,.145,.435),(.20,.145,.18,.465),(.30,.09,.105,.48),(.33,.035,.045,.49)),4),32)
  # Rotate the loft's long axis into Z while retaining outward winding.
  for key in ('positions','normals'):
   vv=raw[key];raw[key]=[v for i in range(0,len(vv),3) for v in (vv[i],vv[i+2],-vv[i+1])]
  # Reverse Z so the shoulder end points toward the head, preserving outward winding.
  for key in ('positions','normals'):raw[key]=[(-v if i%3 in (0,2) else v) for i,v in enumerate(raw[key])]
  body=raw
  neck=paddle('Neck / flowing raised feline throat',gold,[(0.,.47,.22),(0.,.55,.30),(0.,.64,.38)],[.11,.095,.09],[.105,.095,.09],segments=24,steps=4)
 head=ellipsoid('Head / broad rounded feline face',gold,head_center(s),(.244,.175,.165) if meow else (.205,.145,.15),32,22)
 p=[body,neck,head];hx,hy,hz=head_center(s)
 for side in (-1,1):
  path=ear_paths(s,side)
  ear=paddle('Ear / rooted dark pointed pinna '+str(side),brown,path,[.071,.057,.002] if meow else [.067,.059,.018],[.045,.036,.002] if meow else [.045,.035,.012],segments=20,steps=4);p.append(ear)
  outline=[(side*.146,.891),(side*.205,1.025),(side*.225,.941)] if meow else [(side*.155,.762),(side*.242,.831),(side*.224,.771)]
  p.append(fitted('Ear inset / fitted pale triangular pinna '+str(side),gold,outline,ear,.004))
  ex=side*(.109 if meow else .090);ey=hy+.016;ew=.062 if meow else .060;eh=.079 if meow else .046
  outline=oval(ex,ey,ew,eh,26) if meow else [(ex-side*.055,ey+.015),(ex+side*.057,ey+.035),(ex+side*.052,ey-.024),(ex-side*.045,ey-.027)]
  socket=fitted('Eye socket / fitted dark feline outline '+str(side),INK,outline,head,.005);p.append(socket)
  white=fitted('Eye / fitted large white feline lens '+str(side),WHITE,[(ex+(x-ex)*.87,ey+(y-ey)*.87) for x,y in outline],socket,.003);p.append(white)
  pupil=fitted('Pupil / fitted vertical black feline slit '+str(side),INK,oval(ex-side*.008,ey,.013 if meow else .010,.059 if meow else .018,20),white,.0025);p.append(pupil)
  muzzle=ellipsoid('Muzzle / rooted rounded whisker pad '+str(side),gold,(side*.057,hy-.082,hz+.136),(.080,.041,.046),24,16);p.append(muzzle)
  for j,dy in enumerate((.023,-.023) if meow else (.032,0.,-.032)):
   base=(side*.108,hy-.082+dy*.55,hz+.155)
   p.append(curved_tube(f'Whisker / rooted tapered cheek vibrissa {side} {j}',brown,[base,(side*.236,base[1]+dy*.35,base[2]+.013),(side*(.365 if meow else .34),base[1]+dy*.9,base[2]-.012)],[.008,.005,.0015],10))
 if meow:
  for side in (-1,1):p.append(curved_tube('Brow whisker / rooted rising feline vibrissa '+str(side),brown,[(side*.075,.89,.080),(side*.088,1.015,.095),(side*.11,1.10,.105)],[.007,.004,.0015],10))
 nose=ellipsoid('Nose / small rooted dark feline button',brown,(0.,hy-.065,hz+.179),(.022,.014,.018),20,14);p.append(nose)
 mf=1. if meow else .8
 mouth=fitted('Mouth / fitted feline smile aperture',INK,[(x,hy+y*mf) for x,y in ((-.065,-.111),(-.018,-.117),(0.,-.108),(.018,-.117),(.065,-.111),(.035,-.144),(-.035,-.144))],head,.005);p.append(mouth)
 for side in (-1,1):
  p.append(fitted('Fang / fitted small ivory canine '+str(side),WHITE,[(side*.041,hy-.121*mf),(side*.022,hy-.123*mf),(side*.030,hy-.138*mf)],mouth,.002))
 if meow:
  coin=ellipsoid('Coin / tall raised oval gold forehead charm',gold,(0.,.924,.175),(.047,.105,.022),28,20);p.append(coin)
  for j,y in enumerate((.86,.902,.944,.986)):
   p.append(fitted_line('Coin groove / fitted horizontal charm engraving '+str(j),brown,[(-.028,y),(0.,y),(.028,y)],coin,.0026))
  for side in (-1,1):
   path=arm_path(side);p.append(curved_tube('Arm / continuous bent coin-cat forelimb '+str(side),gold,path,[.047,.037,.030],18))
   x,y,z=path[-1];hand=ellipsoid('Hand / rounded beckoning feline palm '+str(side),gold,(x,y+.018,z+.008),(.048,.054,.042),22,16);p.append(hand)
   for j,dx in enumerate((-.026,0.,.026)):
    p.append(ellipsoid(f'Finger / three formed rounded palm digits {side} {j}',gold,(x+dx,y+.049,z+.012),(.020,.027,.030),16,12))
 else:
  gem=ellipsoid('Gem / raised oval dark forehead jewel',brown,(0.,hy+.078,hz+.142),(.033,.039,.020),24,16);p.append(gem)
  p.append(fitted('Gem glint / fitted small ivory highlight',WHITE,oval(-.009,hy+.088,.006,.009,12),gem,.002))
 for label,path in support_paths(s).items():
  p.append(curved_tube('Leg / complete curved feline support '+label,gold,path,[.050,.041,.026] if meow else [.055,.034,.026],18))
  x,y,z=path[-1];foot=ellipsoid('Foot / grounded broad feline paw '+label,brown if meow else gold,(x,.042,z+.040),(.065,.042,.089),22,16);p.append(foot)
  for j,dx in enumerate((-.034,0.,.034)):
   p.append(ellipsoid(f'Toe / three formed grounded feline digits {label} {j}',brown if meow else gold,(x+dx,.029,z+.104),(.025,.029,.045),16,12))
 p.append(curved_tube('Tail / continuous curled feline whip',brown if meow else gold,TAILS[s],[.032,.031,.028,.026,.023,.021,.018,.009] if meow else [.031,.028,.025,.023,.021,.018,.016,.008],18))
 return p
@lru_cache(maxsize=2)
def authoring_transform(s):
 ps=anatomy(s);lo=min(v for p in ps for v in p['positions'][1::3]);hi=max(v for p in ps for v in p['positions'][1::3]);return lo,HEIGHTS[s]/(hi-lo)
def normalized(s,point):
 lo,k=authoring_transform(s);return tuple((v-(lo if i==1 else 0.))*k for i,v in enumerate(point))
def sculpture(s,normalize=True):
 ps=anatomy(s)
 if normalize:
  lo,k=authoring_transform(s)
  for p in ps:p['positions']=[round((v-(lo if i%3==1 else 0.))*k,7) for i,v in enumerate(p['positions'])];p['normals']=[round(v,7) for v in p['normals']]
 return dict(name=s,version=1,coordinate_system=COORDINATES,primitives=ps)
if __name__=='__main__':
 import argparse
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from feline_glb import export_feline
 for s in SPECIES:
  (args.output/(s+'.glb')).write_bytes(export_feline(s));(args.output/(s+'.mesh.json')).unlink(missing_ok=True)
