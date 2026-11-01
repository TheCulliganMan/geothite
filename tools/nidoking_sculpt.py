#!/usr/bin/env python3
"""Original forward-leaning armored king with an open toothed jaw and heavy tail."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle as legacy_paddle
from raptor_bird_sculpt import sheet as legacy_sheet
from sculpt_geometry import fitted_triangle_contour
from cubone_sculpt import skin_shading
from rhyhorn_sculpt import longitudinal
from wing_insect_sculpt import contour
from murkrow_sculpt import bill
from skin_glb import COORDINATES
HEIGHT=1.40
PURPLE=(.60,.43,.69);ARMOR=(.49,.34,.59);TEAL=(.27,.48,.49)
CREAM=(.95,.91,.77);WHITE=(.99,.98,.94);INK=(.065,.055,.09)
IRIS=(.53,.26,.24);PINK=(.78,.43,.50)

def paddle(name,color,points,widths,depths):
 return legacy_paddle(name,color,points,widths,depths,segments=20,steps=6)

def sheet(name,color,xy,depth,th=.018,sub=2):
 return legacy_sheet(name,color,xy,depth,th,0)
def paint(name,color,xy,parent,th=.006,sub=3):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),35.),parent)

def ears(p):
 for s in (-1,1):
  xy=[(.131,1.275),(.145,1.469),(.185,1.649),(.227,1.580),(.311,1.781),(.347,1.660),(.423,1.724),(.429,1.611),(.492,1.650),(.470,1.510),(.498,1.502),(.396,1.369),(.269,1.283),(.171,1.264)]
  ear=sheet('Ear / thick ragged purple king pinna '+str(s),PURPLE,[(s*x,y) for x,y in xy],lambda y,x:.095-.38*(y-1.28),.036,3);p.append(ear)
  xy=[(.175,1.334),(.211,1.502),(.302,1.626),(.368,1.642),(.410,1.551),(.352,1.435),(.276,1.361),(.210,1.331)]
  p.append(paint('Ear inset / fitted dark teal king field '+str(s),TEAL,[(s*x,y) for x,y in xy],ear,.005,3))

def armor(p,body):
 xy=contour([((-.186,1.023),(-.137,1.078),(.137,1.078),(.186,1.023)),((.186,1.023),(.277,.959),(.262,.867),(.206,.824)),((.206,.824),(.127,.804),(-.127,.804),(-.206,.824)),((-.206,.824),(-.262,.867),(-.277,.959),(-.186,1.023))],7)
 p.append(paint('Chest plate / fitted broad cream king breast shield',CREAM,xy,body,.013,3))
 for j,(lo,hi,w) in enumerate(((.282,.407,.175),(.420,.552,.252),(.565,.698,.289),(.711,.806,.274))):
  xy=contour([((-w*.85,lo),(-w,lo+.024),(-w,hi-.025),(-w*.84,hi)),((-w*.84,hi),(-w*.35,hi+.002),(w*.35,hi+.002),(w*.84,hi)),((w*.84,hi),(w,hi-.025),(w,lo+.024),(w*.85,lo)),((w*.85,lo),(w*.3,lo-.002),(-w*.3,lo-.002),(-w*.85,lo))],6)
  p.append(paint('Belly plate / fitted segmented cream king abdomen '+str(j),CREAM,xy,body,.012,3))

def limbs(p):
 for s in (-1,1):
  p.append(ellipsoid('Shoulder / rooted angular purple king armor '+str(s),ARMOR,(s*.294,.916,.052),(.160,.183,.174),26,18))
  p.append(paddle('Arm / heavy forward purple king forelimb '+str(s),PURPLE,[(s*.309,.880,.082),(s*.475,.695,.135),(s*.495,.491,.310)],[.130,.134,.088],[.135,.129,.089]))
  p.append(ellipsoid('Hand / broad three-clawed purple king fist '+str(s),PURPLE,(s*.490,.434,.366),(.146,.149,.151),26,18))
  for j,dx in enumerate((-.088,0,.088)):
   p.append(curved_tube(f'Hand claw / three rooted ivory king points {s} {j}',WHITE,[(s*.490+dx*.81,.384,.451),(s*.490+dx,.326,.537),(s*.490+dx*.95,.285,.556)],[.040,.028,.003],18))
  p.append(ellipsoid('Haunch / powerful purple king hindquarter '+str(s),PURPLE,(s*.267,.333,-.076),(.186,.232,.219),28,20))
  p.append(ellipsoid('Knee / rooted purple king armor '+str(s),ARMOR,(s*.296,.299,.097),(.133,.150,.146),26,18))
  p.append(paddle('Leg / short planted purple king ankle '+str(s),PURPLE,[(s*.285,.250,.008),(s*.329,.137,.111),(s*.341,.077,.205)],[.112,.105,.102],[.119,.102,.049]))
  p.append(ellipsoid('Foot / grounded broad purple king pad '+str(s),PURPLE,(s*.341,.048,.206),(.154,.048,.188),28,18))
  p.append(paddle('Foot claw / single broad rooted ivory king toe '+str(s),WHITE,[(s*.341,.051,.321),(s*.341,.036,.402),(s*.341,.019,.450)],[.108,.103,.061],[.040,.028,.009]))

def sculpture(normalize=True):
 body=crease_normals(loft('Body / continuous forward-leaning purple king armor',PURPLE,curved_profiles(((.161,.087,.099,-.055),(.296,.285,.259,-.069),(.502,.369,.300,-.056),(.706,.364,.286,-.017),(.931,.309,.238,.051),(1.090,.206,.161,.095),(1.167,.088,.089,.104)),5),36),45)
 head=longitudinal('Head / long lowered purple king wedge',PURPLE,[(-.023,1.252,.097,.099),(.087,1.285,.190,.149),(.221,1.277,.236,.162),(.364,1.213,.203,.130),(.490,1.158,.157,.095),(.584,1.132,.074,.054),(.608,1.135,.007,.030)],segments=30)
 upper=bill('Jaw / hooked purple king upper mandible',PURPLE,[(.253,.163,1.085,1.218),(.385,.191,1.036,1.166),(.526,.141,1.038,1.118),(.624,.061,1.044,1.084),(.647,.002,1.051,1.064)])
 lower=bill('Jaw / open purple king lower mandible',PURPLE,[(.218,.141,.958,1.148),(.367,.171,.895,.980),(.490,.140,.878,.942),(.582,.072,.917,.948),(.615,.002,.948,.956)])
 mouth=ellipsoid('Mouth / recessed dark king oral back',INK,(0,1.019,.311),(.137,.115,.033),28,20)
 tongue=ellipsoid('Tongue / supported small pink king tongue',PINK,(0,.940,.491),(.084,.017,.068),26,18)
 p=[body,head,upper,lower,mouth,tongue]
 p.append(curved_tube('Horn / large rooted forward purple king lance',PURPLE,[(0,1.377,.290),(0,1.523,.429),(0,1.670,.614),(0,1.687,.758)],[.087,.070,.040,.002],18))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.065,1.257),(.166,1.310),(.206,1.257),(.167,1.212),(.102,1.210))]
  socket=paint('Eye socket / fitted angular dark king recess '+str(s),INK,xy,head,.007,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted ivory king glare '+str(s),WHITE,[(cx+.88*(x-cx),cy+.87*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted rust king gaze '+str(s),IRIS,oval(s*.153,1.255,.016,.023,24),eye,.003,2);p.append(iris)
  pupil=paint('Pupil / fitted dark king slit '+str(s),INK,oval(s*.153,1.255,.006,.019,20),iris,.003,2);p.append(pupil)
  p.append(paint('Glint / tiny ivory king eye point '+str(s),WHITE,oval(s*.153-.002,1.263,.002,.0035,16),pupil,.002,2))
  p.append(paint('Nostril / fitted dark king snout recess '+str(s),INK,oval(s*.050,1.109,.004,.003,20),upper,.003,2))
  for j,(x,y,z,tx,ty,tz,r) in enumerate(((.201,1.196,.178,.344,1.174,.240,.053),(.163,1.125,.223,.309,1.067,.313,.047))):
   p.append(curved_tube(f'Cheek barb / rooted purple king point {s} {j}',PURPLE,[(s*x,y,z),(s*(x+tx)/2,(y+ty)/2,(z+tz)/2),(s*tx,ty,tz)],[r,r*.55,.002],18))
  for j,(x,y,z,tipy) in enumerate(((.143,1.068,.445,.982),(.104,1.063,.534,1.009))):
   p.append(curved_tube(f'Fang / rooted ivory king upper tooth {s} {j}',WHITE,[(s*x,y,z),(s*x,y-.039,z+.009),(s*x*.96,tipy,z+.018)],[.024,.016,.002],18))
  p.append(curved_tube('Fang / rooted ivory king lower canine '+str(s),WHITE,[(s*.124,.925,.481),(s*.125,.969,.491),(s*.122,.997,.496)],[.021,.014,.002],18))
 ears(p);armor(p,body);limbs(p)
 tail=curved_tube('Tail / long massive upturned purple king tail',PURPLE,[(0,.325,-.282),(.011,.201,-.657),(.099,.175,-1.085),(.255,.342,-1.426),(.436,.647,-1.522)],[.175,.174,.151,.118,.005],18);p.append(tail)
 for j,(points,radii) in enumerate((([(0,.255,-.440),(.006,.214,-.590)],[.179,.181]),([(.024,.186,-.740),(.061,.175,-.922)],[.175,.164]),([(.135,.200,-1.185),(.210,.285,-1.361)],[.151,.133]))):
  p.append(curved_tube('Tail collar / rooted purple segmented armor '+str(j),(.55,.39,.64),points,radii,32))
 for j,(y,z,ty,tz,r) in enumerate(((1.006,-.138,1.294,-.363,.104),(.839,-.247,1.117,-.496,.104),(.635,-.333,.901,-.553,.097))):
  p.append(curved_tube('Back spine / large rooted purple king barb '+str(j),PURPLE,[(0,y,z),(0,y+.108,z-.088),(0,ty,tz)],[r,r*.65,.002],20))
 for j,(x,y,z,ty,tz,r) in enumerate(((.006,.349,-.597,.572,-.725,.088),(.081,.284,-1.017,.480,-1.135,.078),(.244,.439,-1.408,.611,-1.474,.055))):
  p.append(curved_tube('Tail spine / rooted purple king barb '+str(j),PURPLE,[(x,y,z),(x,y+.075,z-.043),(x,ty,tz)],[r,r*.60,.002],18))
 if not normalize:return {'name':'nidoking','version':1,'coordinate_system':COORDINATES,'primitives':p}
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'nidoking','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'nidoking.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
