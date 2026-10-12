#!/usr/bin/env python3
"""Original low horned poison-pin quadruped with ragged ears and fitted spots."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,translate
from bovine_sculpt import oval,curved_tube,reverse_front
from raptor_bird_sculpt import sheet,paint
from goldfish_sculpt import zy
from rhyhorn_sculpt import longitudinal
from skin_glb import COORDINATES
HEIGHT=1.12
PURPLE=(.63,.46,.68);SPOT=(.40,.27,.45);TEAL=(.28,.46,.47)
WHITE=(.98,.97,.93);INK=(.075,.065,.095);IRIS=(.40,.33,.26)

def side_chart(p,s):
 q=zy(p)
 return reverse_front(q) if s==1 else q

def side_patch(name,color,outline,parent,s):
 q=paint(name,color,[(z if s==-1 else -z,y) for z,y in outline],side_chart(parent,s),.004,2)
 if s==1:q=reverse_front(q)
 return zy(zy(zy(q)))

def ears(p):
 for s in (-1,1):
  xy=[(.121,.685),(.160,.911),(.195,1.149),(.215,1.028),(.286,1.342),(.308,1.219),(.345,1.164),(.442,1.364),(.432,1.196),(.475,1.226),(.458,1.065),(.490,1.079),(.410,.889),(.298,.777),(.178,.699)]
  ear=sheet('Ear / tall ragged purple pinna '+str(s),PURPLE,[(s*x,y) for x,y in xy],lambda y,x:.260-.65*(y-.685)+.03*(abs(x)-.15),.030,3);p.append(ear)
  field=[(.166,.759),(.213,.976),(.290,1.130),(.384,1.176),(.401,1.086),(.353,.929),(.281,.834),(.203,.782)]
  p.append(paint('Ear inset / fitted dark teal inner field '+str(s),TEAL,[(s*x,y) for x,y in field],ear,.005,3))

def legs(p,body):
 for s in (-1,1):
  for front in (True,False):
   z=.181 if front else -.406;tag=f'{s} '+('front' if front else 'rear')
   thigh=ellipsoid('Haunch / rooted purple quadruped shoulder '+tag,PURPLE,(s*.217,.283,z),(.115,.146,.141),36,24);p.append(thigh)
   leg=translate(crease_normals(loft('Leg / planted tapered purple pillar '+tag,PURPLE,curved_profiles(((.031,.057,.071,z+.038),(.092,.077,.088,z+.026),(.190,.084,.090,z+.012),(.301,.084,.087,z)),4),28),45),s*.239,0,0);p.append(leg)
   foot=ellipsoid('Foot / grounded broad purple paw '+tag,PURPLE,(s*.239,.030,z+.053),(.086,.030,.107),32,20);p.append(foot)
   for j,dx in enumerate((-.049,0,.049)):
    p.append(curved_tube(f'Claw / three rooted ivory quadruped toes {tag} {j}',WHITE,[(s*.239+dx*.7,.029,z+.129),(s*.239+dx,.018,z+.174),(s*.239+dx,.010,z+.197)],[.023,.018,.002],20))
   for j,(zz,y,w,h) in enumerate(((z+.026,.164,.027,.038),(z+.025,.234,.027,.029))):
    xy=[(zz-w*.9,y-h*.6),(zz-w*.6,y+h*.7),(zz+w*.5,y+h),(zz+w,y+h*.1),(zz+w*.5,y-h),(zz-w*.3,y-h*.9)]
    p.append(side_patch(f'Leg spot / fitted irregular purple marking {tag} {j}',SPOT,xy,leg,s))
  for j,(z,y,w,h) in enumerate(((-.13,.454,.056,.045),(-.32,.457,.035,.034),(.021,.467,.023,.032),(-.254,.539,.022,.028),(-.384,.362,.025,.033))):
   xy=[(z-w*.9,y-h*.4),(z-w*.5,y+h*.8),(z+w*.4,y+h),(z+w,y+h*.1),(z+w*.7,y-h*.8),(z-w*.2,y-h)]
   p.append(side_patch(f'Flank spot / fitted irregular purple marking {s} {j}',SPOT,xy,body,s))

def sculpture():
 body=ellipsoid('Body / long low purple poison-pin barrel',PURPLE,(0,.390,-.148),(.282,.230,.450),48,28)
 head=longitudinal('Head / long lowered purple poison-pin wedge',PURPLE,[(.023,.542,.100,.091),(.135,.560,.184,.158),(.281,.548,.220,.175),(.433,.467,.195,.145),(.565,.374,.140,.098),(.664,.328,.071,.053),(.693,.319,.010,.017)])
 jaw=longitudinal('Jaw / rooted low purple mandible',SPOT,[(.289,.352,.126,.058),(.468,.301,.153,.067),(.615,.276,.089,.040),(.666,.281,.026,.018)])
 p=[body,head,jaw]
 p.append(curved_tube('Horn / rooted forward swept purple nasal lance',PURPLE,[(0,.591,.378),(0,.686,.525),(0,.766,.661),(0,.792,.786)],[.072,.055,.029,.002],32))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.079,.474),(.145,.546),(.190,.558),(.176,.485),(.134,.459))]
  socket=paint('Eye socket / fitted low angular dark recess '+str(s),INK,xy,head,.006,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted narrow ivory quadruped gaze '+str(s),WHITE,[(cx+.89*(x-cx),cy+.86*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  iris=paint('Iris / fitted muted poison-pin iris '+str(s),IRIS,oval(s*.151,.499,.012,.016,24),eye,.003,2);p.append(iris)
  pupil=paint('Pupil / fitted dark quadruped slit '+str(s),INK,oval(s*.151,.499,.006,.014,20),iris,.003,2);p.append(pupil)
  p.append(paint('Glint / tiny ivory eye point '+str(s),WHITE,oval(s*.151-.002,.505,.002,.0035,16),pupil,.002,2))
  p.append(paint('Nostril / fitted dark low snout recess '+str(s),INK,oval(s*.044,.342,.004,.006,20),head,.003,2))
  p.append(curved_tube('Fang / small rooted ivory quadruped canine '+str(s),WHITE,[(s*.164,.415,.417),(s*.166,.385,.436),(s*.162,.355,.447)],[.016,.012,.002],24))
  for j,(x,y,w,h) in enumerate(((.099,.622,.016,.022),(.101,.413,.013,.018))):
   xy=[(s*(x+dx*w),y+dy*h) for dx,dy in ((-.9,-.5),(-.5,.8),(.4,1),(1,.1),(.6,-.9),(-.2,-1))]
   p.append(paint(f'Head spot / fitted irregular purple marking {s} {j}',SPOT,xy,head,.004,2))
 p.append(paint('Mouth / fitted narrow wedge snout line',INK,[(-.090,.304),(-.043,.290),(0,.286),(.043,.290),(.09,.304),(.074,.293),(.04,.280),(0,.277),(-.04,.280),(-.074,.293)],jaw,.003,3))
 ears(p);legs(p,body)
 for j,(z,y,r,h) in enumerate(((.050,.514,.085,.268),(-.104,.555,.088,.348),(-.265,.519,.078,.326),(-.412,.440,.068,.269))):
  p.append(curved_tube('Back spine / rooted upright purple poison barb '+str(j),PURPLE,[(0,y,z),(0,y+h*.43,z-.020),(0,y+h,z-.050)],[r,r*.58,.002],28))
 p.append(curved_tube('Tail / short rooted purple rear point',PURPLE,[(0,.360,-.479),(0,.366,-.612),(.011,.466,-.715)],[.087,.062,.002],28))
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'nidorino','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'nidorino.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
