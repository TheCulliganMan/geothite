#!/usr/bin/env python3
"""Original athletic bone keeper with narrow skull gaze and raised gripped club."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles
from hoofed_sculpt import trunk
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint
from cubone_sculpt import recessed,skin_shading,CREAM,BONE,WHITE,INK
from skin_glb import COORDINATES
HEIGHT=1.18
TAN=(.72,.52,.30);IRIS=(.55,.35,.23);SEAM=(.76,.70,.56)

def anatomy():
 body=loft('Body / athletic tan Marowak torso',TAN,curved_profiles(((.164,.071,.089,-.013),(.285,.227,.210,-.022),(.492,.295,.253,-.016),(.728,.259,.213,.020),(.895,.189,.147,.061),(.999,.068,.075,.087)),5),40)
 neck=ellipsoid('Neck / rooted tan Marowak neck',TAN,(0,1.036,.080),(.127,.145,.136),36,24)
 head=ellipsoid('Head / rooted tan Marowak head inside skull',TAN,(0,1.232,.076),(.215,.248,.217),40,28)
 skull=recessed(loft('Skull / long sloped bone Marowak skull',BONE,curved_profiles(((.925,.085,.123,.240),(1.020,.185,.223,.211),(1.16,.233,.252,.139),(1.30,.257,.259,.087),(1.43,.215,.212,.064),(1.49,.095,.110,.037),(1.505,.015,.024,.022)),5),64),[(s*.161,1.297,.077,.047,.082) for s in (-1,1)])
 snout=trunk('Snout / long narrow tapered bone Marowak muzzle',BONE,curved_profiles(((.205,.056,.084,1.112),(.270,.140,.118,1.097),(.397,.147,.121,1.050),(.547,.122,.101,1.034),(.631,.082,.066,1.057),(.653,.026,.025,1.075)),5))
 p=[body,neck,head,skull,snout]
 belly=skin_shading(paint('Belly / fitted long cream Marowak belly',CREAM,oval(0,.565,.190,.295,48),body,.009,3),body);p.append(belly)
 for j,y in enumerate((.384,.449)):
  xy=[(-.150,y+.018),(-.08,y),(.08,y),(.150,y+.018),(.143,y+.010),(.076,y-.007),(-.076,y-.007),(-.143,y+.010)]
  p.append(paint('Belly seam / fitted shallow Marowak belly band '+str(j),SEAM,xy,belly,.003,3))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.092,1.295),(.208,1.329),(.222,1.295),(.140,1.269),(.092,1.273))]
  socket=skin_shading(paint('Orbit / fitted recessed narrow tan Marowak socket '+str(s),TAN,xy,skull,.004,4),skull);p.append(socket);cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted narrow white Marowak glare '+str(s),WHITE,[(cx+.84*(x-cx),cy+.77*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted rust Marowak gaze '+str(s),IRIS,oval(s*.174,1.299,.011,.012,24),eye,.003,3);p.append(iris)
  pupil=paint('Pupil / fitted dark Marowak slit '+str(s),INK,oval(s*.174,1.299,.0045,.009,20),iris,.003,2);p.append(pupil)
  p.append(paint('Eye glint / fitted tiny white Marowak point '+str(s),WHITE,oval(s*.172,1.303,.0018,.0025,16),pupil,.002,2))
  p.append(paint('Nostril / fitted tiny dark Marowak nostril '+str(s),INK,oval(s*.035,1.065,.005,.008,20),snout,.003,3))
  p.append(curved_tube('Horn / broad pointed bone Marowak side horn '+str(s),BONE,[(s*.170,1.430,.050),(s*.276,1.525,.080),(s*.400,1.590,.104)],[.087,.065,.0015],32))
  p.append(curved_tube('Cheek / rooted angular bone Marowak cheek '+str(s),BONE,[(s*.211,1.165,.177),(s*.284,1.104,.203),(s*.251,1.021,.221)],[.068,.059,.002],28))
  p.append(ellipsoid('Shoulder / rooted strong tan Marowak shoulder '+str(s),TAN,(s*.253,.889,.025),(.110,.140,.126),36,24))
  pts=[(-.256,.904,0),(-.420,1.128,-.140),(-.330,1.530,-.310)] if s==-1 else [(.267,.901,.020),(.484,1.023,.010),(.664,1.104,.084)]
  p.append(curved_tube('Arm / posed strong tan Marowak forelimb '+str(s),TAN,pts,[.081,.073,.060],28))
  hand=pts[-1];p.append(ellipsoid('Hand / shaped tan Marowak gripping palm '+str(s),TAN,hand,(.076,.095,.079),36,24))
  for j in range(3):
   if s==-1:
    x=hand[0]+(j-1)*.027;points=[(x,hand[1]-.012,hand[2]+.051),(x-.010,hand[1]-.065,hand[2]+.093),(x-.017,hand[1]-.058,hand[2]+.131)]
   else:
    y=hand[1]+(j-1)*.034;points=[(hand[0]+.048,y,hand[2]+.016),(hand[0]+.106,y-.022,hand[2]+.042),(hand[0]+.137,y-.008,hand[2]+.043)]
   p.append(curved_tube(f'Finger / rooted shaped tan Marowak digit {s} {j}',TAN,points,[.023,.019,.006],24))
  points=[(-.372,1.530,-.255),(-.39,1.525,-.223),(-.383,1.512,-.214)] if s==-1 else [(.626,1.060,.136),(.621,1.040,.197),(.621,1.032,.227)]
  p.append(curved_tube('Hand claw / rooted white Marowak claw '+str(s),WHITE,points,[.025,.016,.0015],24))
  p.append(ellipsoid('Haunch / rooted powerful tan Marowak thigh '+str(s),TAN,(s*.240,.266,-.005),(.176,.226,.229),40,26))
  p.append(ellipsoid('Foot / grounded broad tan Marowak foot '+str(s),TAN,(s*.287,.060,.230),(.118,.060,.188),36,24))
  p.append(curved_tube('Toe claw / rooted large white Marowak toe '+str(s),WHITE,[(s*.292,.045,.363),(s*.301,.033,.402),(s*.309,.025,.431)],[.045,.025,.0015],28))
 p.append(curved_tube('Thumb / rooted tan Marowak bone-gripping thumb',TAN,[(-.373,1.570,-.301),(-.410,1.550,-.332),(-.393,1.527,-.347)],[.030,.025,.008],24))
 club=curved_tube('Bone shaft / raised bone Marowak club held by hand',BONE,[(-.680,1.720,-.300),(-.330,1.530,-.310),(.045,1.319,-.300)],[.045,.038,.044],32);p.append(club)
 for end,(x,y,z) in enumerate(((-.680,1.720,-.300),(.045,1.319,-.300))):
  for j,(dx,dy) in enumerate(((-.021,-.036),(.021,.036))):p.append(ellipsoid(f'Bone lobe / rooted bifurcated Marowak club end {end} {j}',BONE,(x+dx,y+dy,z),(.054,.056,.053),32,22))
 tail=curved_tube('Tail / rooted long muscular tan Marowak tail',TAN,[(0,.340,-.200),(0,.275,-.460),(.10,.333,-.760),(.265,.452,-.920)],[.150,.144,.094,.002],32);p.append(tail)
 p.append(curved_tube('Tail spine / rooted tan Marowak dorsal point',TAN,[(.085,.382,-.707),(.095,.480,-.720),(.100,.569,-.727)],[.045,.030,.0015],28))
 return p

def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'marowak','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'marowak.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
