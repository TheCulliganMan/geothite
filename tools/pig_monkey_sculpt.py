#!/usr/bin/env python3
"""Original angry pig monkeys: rooted fur locks, bent limbs and coherent clenched paws."""
import math
from functools import lru_cache
from pathlib import Path
from sculpt_geometry import ellipsoid,part,crease_normals,fitted_triangle_contour
from lugia_sculpt import paddle
from bovine_sculpt import curved_tube,oval
from skin_glb import COORDINATES
SPECIES=('mankey','primeape');HEIGHTS={s:1.02 for s in SPECIES}
WHITE=(1.,1.,1.);INK=(0.,0.,0.)
TAN={'mankey':(239/255,165/255,90/255),'primeape':(1.,148/255,99/255)}
BROWN={'mankey':(156/255,74/255,57/255),'primeape':(132/255,66/255,41/255)}
def fitted(name,color,xy,parent,th=.004):return crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),35)
def body_center(s):return .55 if s=='mankey' else .57
def arm_path(s,side):
 return ((side*.235,body_center(s)+.035,-.005),(side*.365,.64,.025),(side*.43,.775,.065))
def leg_path(s,side):return ((side*.14,.34,-.010),(side*.195,.19,.015),(side*.215,.070,.070))
TAIL=((0.,.355,-.175),(.06,.33,-.34),(.23,.38,-.43),(.36,.52,-.40),(.40,.65,-.31),(.35,.72,-.26))
def fur_paths(s):
 cy=body_center(s);out=[]
 for side in (-1,1):
  for j,(x,y,tipx,tipy) in enumerate(((.20,cy+.15,.32,cy+.22),(.23,cy,.36,cy-.06),(.19,cy-.17,.29,cy-.27))):
   tipx+=.045 if s=='primeape' else 0.
   out.append((f'cheek {side} {j}',((side*x,y,.005),(side*(x+.065),(y+tipy)/2,.005),(side*tipx,tipy,.010)),(.058,.044,.0015),(.050,.036,.0015)))
 for j in ((-2,-1,0,1,2) if s=='primeape' else (-1,0,1)):
  out.append((f'crown {j}',((j*.060,cy+.24,-.025),(j*.075,cy+.32,-.030),(j*.090+.018,cy+.38,-.035)),(.045,.033,.0015),(.036,.026,.0015)))
 for j in (-1,0,1):
  out.append((f'back {j}',((j*.105,cy+.11,-.155),(j*.12,cy+.025,-.265),(j*.13,cy-.09,-.31)),(.053,.041,.0015),(.044,.033,.0015)))
 for side in ((-1.5,-.5,.5,1.5) if s=='primeape' else (-1,1)):
  out.append((f'chin {side}',((side*.10,cy-.18,.115),(side*.115,cy-.26,.125),(side*.13,cy-.34,.13)),(.047,.035,.0015),(.039,.026,.0015)))
 return out

def anatomy(s):
 if s not in SPECIES:raise ValueError('unknown pig monkey')
 cy=body_center(s);large=s=='primeape';brown=BROWN[s];tan=TAN[s]
 body=ellipsoid('Body / continuous round white fur pelt',WHITE,(0.,cy,0.),(.33 if large else .275,.30,.235 if large else .22),32,22);p=[body]
 for label,path,widths,depths in fur_paths(s):p.append(paddle('Fur / rooted pointed '+label,WHITE,path,widths,depths,segments=16,steps=4))
 belly=fitted('Belly / fitted warm lower pelt field',tan,oval(0.,.42,.19 if large else .17,.115 if large else .10,28),body,.005);p.append(belly)
 for side in (-1,1):
  eye_xy=[(side*.052,.632),(side*.183,.685),(side*.174,.595),(side*.068,.580)]
  socket=fitted('Eye socket / fitted angry dark angular gaze '+str(side),INK,eye_xy,body,.006);p.append(socket)
  cx=sum(x for x,y in eye_xy)/4;ey=sum(y for x,y in eye_xy)/4
  white=fitted('Eye / fitted stern white gaze '+str(side),WHITE,[(cx+.86*(x-cx),ey+.82*(y-ey)) for x,y in eye_xy],socket,.004);p.append(white)
  p.append(fitted('Pupil / fitted narrow black stare '+str(side),INK,oval(side*.113,.619,.011,.020,20),white,.003))
 nose=ellipsoid('Snout / rooted broad tan pig muzzle',tan,(0.,.505,.233 if large else .222),(.095,.062,.070),24,16);p.append(nose)
 for side in (-1,1):p.append(fitted('Nostril / fitted deep dark pig oval '+str(side),INK,oval(side*.032,.511,.011,.019,18),nose,.003))
 for side in (-1,1):
  ear=paddle('Ear / rooted pointed white pinna '+str(side),WHITE,[(side*.185,cy+.19,-.015),(side*.25,cy+.28,-.010),(side*.31,cy+.35,.005)],[.056,.048,.0015],[.043,.031,.0015],segments=18,steps=4);p.append(ear)
  p.append(fitted('Ear inset / fitted warm triangular pinna '+str(side),tan,[(side*.218,cy+.235),(side*.274,cy+.315),(side*.276,cy+.265)],ear,.004))
  p.append(curved_tube('Arm / continuous bent white forelimb '+str(side),WHITE,arm_path(s,side),[.057,.053,.046],18))
  hand=ellipsoid('Hand / broad clenched brown fist '+str(side),brown,(side*.435,.800,.078),(.095,.092,.081) if large else (.080,.085,.073),24,16);p.append(hand)
  for j,dx in enumerate((-.050,0.,.050) if large else (-.041,0.,.041)):
   x=side*.435+dx
   p.append(curved_tube(f'Knuckle / rooted folded brown digit {side} {j}',brown,[(x,.803,.111),(x,.837,.112),(x,.854,.141),(x,.831,.173),(x,.795,.166),(x,.778,.137)],[.016,.017,.017,.016,.014,.013],14))
  p.append(ellipsoid('Thumb / folded rooted brown inner digit '+str(side),brown,(side*.382,.770,.124),(.038,.046,.034),20,14))
  p.append(curved_tube('Leg / bent white supporting shin '+str(side),WHITE,leg_path(s,side),[.060,.049,.034],18))
  foot=ellipsoid('Foot / broad grounded brown monkey paw '+str(side),brown,(side*.22,.046,.119),(.084,.046,.118),24,16);p.append(foot)
  for j,dx in enumerate((-.042,0.,.042)):
   x=side*.22+dx;p.append(curved_tube(f'Toe / rooted forward brown digit {side} {j}',brown,[(x,.058,.175),(x,.040,.232),(x,.028,.274 if j==1 else .255)],[.024,.021,.014],14))
  if large:
   p.append(ellipsoid('Wrist cuff / rooted dark Primeape band '+str(side),INK,(side*.425,.734,.054),(.073,.025,.067),22,14))
   p.append(ellipsoid('Ankle cuff / rooted dark Primeape band '+str(side),INK,(side*.210,.100,.058),(.043,.027,.047),22,14))
 if not large:p.append(curved_tube('Tail / continuous long curled brown monkey tail',brown,TAIL,[.031,.029,.026,.023,.017,.004],18))
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
 import argparse,json
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from pig_monkey_glb import export_pig_monkey
 for s in SPECIES:
  (args.output/(s+'.glb')).write_bytes(export_pig_monkey(s));(args.output/(s+'.mesh.json')).unlink(missing_ok=True)
