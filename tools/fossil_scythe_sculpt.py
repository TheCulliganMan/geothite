#!/usr/bin/env python3
"""Original domed four-claw fossil and swept-head, paired-scythe adult sculptures."""
import argparse
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,fitted_triangle_contour
from bovine_sculpt import curved_tube,oval,reverse_front
from lugia_sculpt import paddle
from murkrow_sculpt import bill
from fire_dragon_sculpt import fitted
from cubone_sculpt import skin_shading
from abra_sculpt import surface
from raptor_bird_sculpt import sheet
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('kabuto','kabutops');HEIGHTS={'kabuto':.60,'kabutops':1.20}
BROWN=(.65,.45,.20);RIDGE=(.72,.53,.27);DARK=(.19,.14,.105);INK=(.025,.022,.020);IVORY=(.96,.93,.82);RED=(.80,.19,.13)
TAIL=((0.,.38,-.08),(0.,.30,-.16),(0.,.18,-.21),(0.,.055,-.24))
def small_leg(side,j):
 z=(-.11,.13)[j];return ((side*.22,.14,z),(side*.34,.095,z+.025),(side*.36,.033,z+.10))
def adult_arm(side):return ((side*.10,.76,.02),(side*.29,.73,.055),(side*.41,.82,.10))
def adult_paint(name,color,outline,parent,thickness=.004):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,outline,parent,thickness,precision=6),40.),parent)
def scythe(side):
 xy=contour([((.398,.800),(.418,.815),(.445,.857),(.470,.890)),
  ((.470,.890),(.540,.855),(.590,.755),(.592,.640)),
  ((.592,.640),(.600,.465),(.556,.290),(.490,.180)),
  ((.490,.180),(.529,.355),(.548,.515),(.530,.650)),
  ((.530,.650),(.514,.758),(.462,.789),(.398,.800))],8)
 return sheet('Scythe / broad swept gray Kabutops blade '+str(side),(.69,.73,.72),[(side*x,y) for x,y in xy],lambda y,x:.10+.12*(.82-y),.015,1)
def helmet(color):
 # A continuous shield has a central low keel, high lateral corners and a
 # tapered rear roof. Flat-bottomed beak rings erase that pointed front.
 rows=((-.28,.17,.985,1.025),(-.19,.48,.94,1.08),(-.09,.43,.86,1.10),(.10,.30,.77,1.05),(.30,.018,.78,.82));vertices=[];n=9
 for z,w,low,high in rows:
  h=high-low;mid=(low+high)/2
  vertices.extend((x,y,z) for x,y in ((0.,low),(w*.6,low+h*.22),(w,mid),(w*.75,high-h*.15),(w*.3,high),(-w*.3,high),(-w*.75,high-h*.15),(-w,mid),(-w*.6,low+h*.22)))
 faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
 for j in range(len(rows)-1):
  for i in range(n):a=j*n+i;b=j*n+(i+1)%n;faces.append((a,b,b+n,a+n))
 return crease_normals(part('Head / broad angular Kabutops hammer shell',color,vertices,faces,True),40.)
def kabuto():
 shell=crease_normals(loft('Shell / broad domed Kabuto carapace',BROWN,curved_profiles(((.145,.30,.27,-.02),(.185,.365,.31,-.02),(.27,.35,.29,-.02),(.36,.27,.23,-.02),(.425,.16,.145,-.025),(.45,.012,.012,-.025)),4),40),35.)
 underside=ellipsoid('Underside / dark recessed Kabuto face',INK,(0.,.126,.025),(.29,.100,.255),32,20);p=[shell,underside]
 for side in (-1,1):
  eye=fitted('Eye lens / fitted pale Kabuto underside lens '+str(side),IVORY,oval(side*.115,.145,.040,.047,24),underside,.005);p.append(eye)
  iris=fitted('Iris / fitted red Kabuto underside gaze '+str(side),RED,oval(side*.115,.146,.021,.032,20),eye,.0025);p.append(iris)
  p.append(fitted('Pupil / fitted dark Kabuto gaze slit '+str(side),INK,oval(side*.115,.147,.006,.022,16),iris,.0015))
  # Two dark optic pits sit on the brown carapace as well as the underside gaze.
  p.append(fitted('Shell pore / fitted black Kabuto optic pit '+str(side),INK,oval(side*.195,.315,.020,.014,24),shell,.003))
  for j in range(2):
   pts=small_leg(side,j);p.append(curved_tube(f'Leg / four rooted Kabuto claw stems {side} {j}',BROWN,pts,[.047,.039,.022],18))
   x,y,z=pts[-1];p.append(ellipsoid(f'Foot / four planted Kabuto claw roots {side} {j}',BROWN,(x,.021,z),(.030,.021,.039),20,12))
   p.append(paddle(f'Claw / four hooked ivory Kabuto tips {side} {j}',IVORY,[(x,.031,z),(x-side*.012,.025,z+.055),(x-side*.042,.012,z+.092)],[.027,.020,.002],[.019,.013,.0015],segments=14,steps=4))
 # A fitted dark rear inset keeps the carapace readable from behind.
 p.append(reverse_front(fitted('Rear inset / supported dark Kabuto horseshoe',DARK,oval(0.,.26,.195,.085,32),reverse_front(shell),.0035)))
 return p

def kabutops():
 coat=(.61,.44,.30);ridge=(.64,.47,.32);steel=(.69,.73,.72)
 body=crease_normals(loft('Body / tapered Kabutops armored torso',coat,curved_profiles(((.265,.07,.07,-.06),(.38,.165,.150,-.03),(.54,.190,.150,-.02),(.70,.220,.170,-.02),(.82,.170,.135,.005),(.90,.080,.070,.045)),4),28),32.)
 neck=loft('Neck / narrow Kabutops armored neck',coat,curved_profiles(((.75,.065,.06,.005),(.91,.07,.065,.045),(.99,.06,.055,.06)),3),24)
 head=helmet(coat);p=[body,neck,head]
 breast=adult_paint('Breast plate / continuous gray Kabutops ventral armor',steel,[(-.125,.82),(.125,.82),(.155,.73),(.160,.64),(.115,.49),(0.,.47),(-.115,.49),(-.160,.64),(-.155,.73)],body,.006);p.append(breast)
 for j,(y,w) in enumerate(((.735,.132),(.635,.145),(.535,.108))):
  xy=[(-w*.88,y+.037),(0.,y+.028),(w*.88,y+.037),(w,y+.005),(w*.80,y-.027),(0.,y-.038),(-w*.80,y-.027),(-w,y+.005)]
  p.append(adult_paint('Rib plate / three supported gray Kabutops chest lobes '+str(j),steel,xy,breast,.004))
 for side in (-1,1):
  socket=adult_paint('Eye socket / fitted narrow Kabutops glare '+str(side),INK,[(side*x,y) for x,y in ((.075,.837),(.205,.861),(.195,.927),(.092,.874))],head,.005);p.append(socket)
  white=adult_paint('Eye lens / fitted ivory Kabutops glare '+str(side),IVORY,[(side*x,y) for x,y in ((.093,.846),(.188,.867),(.180,.907),(.106,.875))],socket,.0025);p.append(white)
  p.append(adult_paint('Pupil / fitted black Kabutops slit '+str(side),INK,oval(side*.142,.875,.005,.008,20),white,.002))
  pts=adult_arm(side);p.append(curved_tube('Upper arm / rooted Kabutops shoulder '+str(side),coat,pts[:2],[.060,.042],22));p.append(ellipsoid('Elbow / Kabutops elbow hinge '+str(side),coat,pts[1],(.045,.044,.044),20,12))
  p.append(curved_tube('Forearm / raised Kabutops scythe carrier '+str(side),steel,pts[1:],[.037,.032],22));p.append(ellipsoid('Wrist / Kabutops scythe root '+str(side),steel,pts[-1],(.042,.044,.037),20,12));p.append(scythe(side))
  p.append(paddle('Thigh / powerful Kabutops hind limb '+str(side),coat,[(side*.09,.44,-.04),(side*.18,.36,.045),(side*.265,.28,.105)],[.075,.100,.065],[.080,.095,.065],segments=20,steps=4))
  p.append(curved_tube('Shin / backward hock and upright Kabutops ankle '+str(side),coat,[(side*.265,.28,.105),(side*.275,.14,.040),(side*.275,.055,.050)],[.055,.036,.030],20))
  p.append(ellipsoid('Foot / planted Kabutops toe junction '+str(side),coat,(side*.275,.034,.085),(.064,.034,.075),24,14))
  for j,dx in enumerate((-.048,0.,.048)):
   p.append(paddle(f'Toe / three pointed gray Kabutops talons {side} {j}',steel,[(side*.275+dx*.7,.035,.13),(side*.275+dx,.025,.18),(side*.275+dx*1.12,.012,.215 if j==1 else .200)],[.024,.017,.0015],[.018,.012,.0012],segments=12,steps=4))
 back=surface(reverse_front(body))
 for side in (-1,1):
  for j,y in enumerate((.36,.46,.56,.66,.76)):
   z=-back(y,-side*.07)+.020
   xy=[(side*.015,y+.045),(side*.15,y+.055),(side*.31,y-.060),(side*.12,y-.045)]
   p.append(sheet(f'Back spur / five paired Kabutops lateral shell ridges {side} {j}',ridge,xy,lambda yy,x,z=z:z-.45*(abs(x)-.07),.025,1))
 p.append(paddle('Tail / continuous tapered Kabutops tail',coat,TAIL,[.10,.080,.045,.0015],[.075,.050,.025,.0012],segments=20,steps=4))
 return p
@lru_cache(maxsize=2)
def anatomy(s):
 if s not in SPECIES:raise ValueError('unknown fossil scythe species')
 return tuple((kabuto if s=='kabuto' else kabutops)())
def sculpture(species,normalize=True):
 p=deepcopy(list(anatomy(species)));lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  if normalize:q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
  for field in ('positions','normals'):q[field]=[0. if v==0 else v for v in q[field]]
 return dict(name=species,version=1,coordinate_system=COORDINATES,primitives=p)
def main():
 from fossil_scythe_glb import export_fossil_scythe
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_fossil_scythe(s))
if __name__=='__main__':main()
