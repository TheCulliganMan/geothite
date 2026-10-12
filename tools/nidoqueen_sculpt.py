#!/usr/bin/env python3
"""Original blue armored queen with paired chest plates and a substantial tail."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import sheet,paint
from rhyhorn_sculpt import longitudinal
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.40
BLUE=(.40,.61,.76);ARMOR=(.35,.55,.70);CREAM=(.93,.88,.68)
BROWN=(.47,.34,.28);WHITE=(.99,.98,.93);INK=(.065,.08,.11);IRIS=(.57,.31,.23)

def ears(p):
 for s in (-1,1):
  xy=contour([((.131,1.281),(.121,1.397),(.178,1.568),(.264,1.630)),((.264,1.630),(.323,1.673),(.429,1.626),(.444,1.550)),((.444,1.550),(.465,1.441),(.335,1.327),(.232,1.293)),((.232,1.293),(.187,1.274),(.156,1.274),(.131,1.281))],8)
  ear=sheet('Ear / broad rounded blue pinna '+str(s),BLUE,[(s*x,y) for x,y in xy],lambda y,x:.014-.19*(y-1.28),.038,2);p.append(ear)
  xy=contour([((.185,1.333),(.160,1.416),(.205,1.535),(.272,1.576)),((.272,1.576),(.329,1.610),(.387,1.583),(.394,1.524)),((.394,1.524),(.407,1.449),(.312,1.362),(.235,1.339)),((.235,1.339),(.211,1.330),(.194,1.330),(.185,1.333))],7)
  p.append(paint('Ear inset / fitted warm brown inner field '+str(s),BROWN,[(s*x,y) for x,y in xy],ear,.006,3))

def armor(p,body):
 for s in (-1,1):
  xy=contour([((.020,.986),(.066,1.030),(.175,1.027),(.236,.964)),((.236,.964),(.267,.925),(.231,.837),(.153,.822)),((.153,.822),(.085,.807),(.025,.859),(.020,.913)),((.020,.913),(.015,.942),(.015,.964),(.020,.986))],7)
  p.append(paint('Chest plate / separate fitted cream shield '+str(s),CREAM,[(s*x,y) for x,y in xy],body,.014,3))
 for j,(lo,hi,w) in enumerate(((.293,.398,.167),(.411,.529,.246),(.542,.665,.276),(.678,.787,.253))):
  xy=contour([((-w*.88,lo),(-w,lo+.035),(-w,hi-.025),(-w*.79,hi)),((-w*.79,hi),(-w*.3,hi+.002),(w*.3,hi+.002),(w*.79,hi)),((w*.79,hi),(w,hi-.025),(w,lo+.035),(w*.88,lo)),((w*.88,lo),(w*.34,lo-.002),(-w*.34,lo-.002),(-w*.88,lo))],6)
  p.append(paint('Belly plate / fitted segmented cream abdomen '+str(j),CREAM,xy,body,.012,3))

def limbs(p,body):
 for s in (-1,1):
  shoulder=ellipsoid('Shoulder / rooted blue armor cap '+str(s),ARMOR,(s*.295,.895,-.015),(.144,.171,.165),36,24);p.append(shoulder)
  arm=paddle('Arm / powerful bent blue forelimb '+str(s),BLUE,[(s*.309,.866,.008),(s*.450,.683,.058),(s*.444,.511,.230)],[.114,.110,.082],[.117,.115,.083]);p.append(arm)
  hand=ellipsoid('Hand / broad blue three-clawed fist '+str(s),BLUE,(s*.442,.456,.281),(.126,.132,.139),36,24);p.append(hand)
  for j,dx in enumerate((-.073,0,.073)):
   p.append(curved_tube(f'Hand claw / three rooted ivory points {s} {j}',WHITE,[(s*.442+dx*.82,.411,.358),(s*.442+dx,.352,.424),(s*.442+dx*.95,.320,.437)],[.032,.024,.003],24))
  thigh=ellipsoid('Haunch / powerful blue hindquarter '+str(s),BLUE,(s*.258,.329,-.092),(.176,.226,.217),40,26);p.append(thigh)
  knee=ellipsoid('Knee / rooted blue scale armor '+str(s),ARMOR,(s*.280,.306,.057),(.126,.149,.138),36,24);p.append(knee)
  leg=paddle('Leg / short planted blue ankle '+str(s),BLUE,[(s*.271,.264,-.031),(s*.306,.139,.065),(s*.318,.074,.158)],[.105,.092,.094],[.109,.097,.047]);p.append(leg)
  foot=ellipsoid('Foot / grounded broad blue two-toed pad '+str(s),BLUE,(s*.318,.043,.166),(.137,.043,.171),40,24);p.append(foot)
  for j,dx in enumerate((-.068,.068)):
   p.append(curved_tube(f'Foot claw / two rooted ivory toes {s} {j}',WHITE,[(s*.318+dx*.7,.049,.293),(s*.318+dx,.035,.370),(s*.318+dx,.021,.402)],[.044,.032,.003],24))

def sculpture():
 body=crease_normals(loft('Body / continuous heavy blue armored pear',BLUE,curved_profiles(((.169,.093,.106,-.058),(.298,.285,.252,-.075),(.491,.369,.286,-.077),(.702,.353,.272,-.053),(.907,.298,.230,-.010),(1.058,.220,.166,.018),(1.139,.093,.090,.035)),5),48),45)
 head=longitudinal('Head / broad low blue reptilian cranium',BLUE,[(-.097,1.231,.092,.093),(.012,1.262,.189,.155),(.145,1.262,.237,.167),(.271,1.200,.211,.124),(.363,1.144,.172,.087),(.463,1.121,.087,.059),(.491,1.120,.016,.029)])
 muzzle=ellipsoid('Muzzle / blunt broad blue queen snout',BLUE,(0,1.106,.389),(.191,.079,.133),44,26)
 p=[body,head,muzzle]
 p.append(curved_tube('Horn / short rooted blue forehead point',BLUE,[(0,1.351,.194),(0,1.460,.286),(0,1.502,.379)],[.068,.042,.002],28))
 chin=paint('Chin / fitted cream reptilian lower jaw',CREAM,[(-.171,1.085),(-.100,1.057),(0,1.042),(.100,1.057),(.171,1.085),(.109,1.047),(.045,1.033),(-.045,1.033),(-.109,1.047)],muzzle,.006,4);p.append(chin)
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.058,1.253),(.169,1.301),(.203,1.239),(.168,1.198),(.103,1.197))]
  socket=paint('Eye socket / fitted angular dark recess '+str(s),INK,xy,head,.007,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted ivory queen gaze '+str(s),WHITE,[(cx+.88*(x-cx),cy+.88*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted warm queen gaze '+str(s),IRIS,oval(s*.148,1.244,.018,.025,24),eye,.003,2);p.append(iris)
  pupil=paint('Pupil / fitted dark reptilian slit '+str(s),INK,oval(s*.148,1.244,.007,.020,20),iris,.003,2);p.append(pupil)
  p.append(paint('Glint / tiny ivory queen eye point '+str(s),WHITE,oval(s*.148-.002,1.252,.002,.004,16),pupil,.002,2))
  p.append(paint('Nostril / fitted queen snout recess '+str(s),INK,oval(s*.045,1.136,.006,.0045,20),muzzle,.003,2))
  p.append(curved_tube('Fang / small rooted ivory queen canine '+str(s),WHITE,[(s*.150,1.092,.443),(s*.151,1.062,.460),(s*.145,1.043,.462)],[.021,.014,.002],24))
 p.append(paint('Mouth / fitted shallow queen jaw seam',INK,[(-.165,1.093),(-.104,1.077),(0,1.067),(.104,1.077),(.165,1.093),(.133,1.076),(.084,1.063),(0,1.056),(-.084,1.063),(-.133,1.076)],muzzle,.010,4))
 ears(p);armor(p,body);limbs(p,body)
 tail=curved_tube('Tail / long substantial curved blue armored tail',BLUE,[(0,.342,-.264),(0,.220,-.566),(.069,.173,-.956),(.253,.302,-1.249),(.410,.501,-1.358)],[.159,.147,.120,.079,.004],36);p.append(tail)
 for j,(y,z,ty,tz,r) in enumerate(((.980,-.157,1.174,-.336,.088),(.808,-.260,1.019,-.454,.088),(.608,-.318,.787,-.511,.083))):
  p.append(curved_tube('Back spine / rooted blue dorsal armor barb '+str(j),BLUE,[(0,y,z),(0,y+.067,z-.071),(0,ty,tz)],[r,r*.65,.002],28))
 for j,(x,y,z,ty,tz,r) in enumerate(((0,.297,-.530,.484,-.641,.078),(.059,.255,-.909,.416,-1.007,.062),(.227,.338,-1.216,.482,-1.282,.047))):
  p.append(curved_tube('Tail spine / rooted blue dorsal tail barb '+str(j),BLUE,[(x,y,z),(x,y+.060,z-.039),(x,ty,tz)],[r,r*.60,.002],24))
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'nidoqueen','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'nidoqueen.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
