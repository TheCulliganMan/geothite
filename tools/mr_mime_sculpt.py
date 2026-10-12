#!/usr/bin/env python3
"""Original expressive barrier mime with splayed gloves and curled slippers."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,part
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint,sheet
from moltres_sculpt import flowing
from skin_glb import COORDINATES
HEIGHT=1.20
SKIN=(1.,.84,.85);WHITE=(.97,.98,.96);RED=(.89,.38,.50);BLUE=(.17,.40,.55);INK=(.11,.09,.13);PINK=(.91,.58,.63)
def anatomy():
 body=loft('Body / sculpted cream Mr Mime torso',WHITE,curved_profiles(((.37,.11,.12,.0),(.43,.22,.17,0),(.63,.27,.19,0),(.81,.24,.17,0),(.94,.13,.10,0)),5),40)
 neck=ellipsoid('Neck / rooted pale pink Mr Mime neck',SKIN,(0,.965,0),(.115,.13,.105),32,22)
 head=ellipsoid('Head / rounded pale pink Mr Mime head',SKIN,(0,1.167,0),(.267,.25,.22),80,56)
 # Recess the actual closed face around the smile rather than attaching a bill.
 vertices=[]
 for i in range(0,len(head['positions']),3):
  x,y,z=head['positions'][i:i+3];q=(x/.120)**2+((y-1.066)/.061)**2
  if z>0 and q<1:z*=1-.43*(1-q)**2
  vertices.append((x,y,z))
 head=part(head['part'],SKIN,vertices,[tuple(head['indices'][i:i+3]) for i in range(0,len(head['indices']),3)],True)
 upper=ellipsoid('Upper lip / rooted pale pink smiling upper lip',SKIN,(0,1.113,.169),(.080,.009,.017),36,22)
 lower=ellipsoid('Lower lip / rooted pale pink smiling lower lip',SKIN,(0,1.026,.164),(.098,.009,.023),36,22)
 chamber=ellipsoid('Mouth back / recessed dark Mr Mime smile',INK,(0,1.068,.130),(.093,.036,.023),32,22)
 tongue=ellipsoid('Tongue / rooted pink Mr Mime tongue',PINK,(0,1.040,.163),(.065,.011,.026),32,20)
 p=[body,neck,head,upper,lower,chamber,tongue]
 p.append(paint('Mouth lining / fitted dark curved Mr Mime smile',INK,flowing([(-.110,1.093),(-.080,1.084),(0,1.075),(.080,1.084),(.110,1.093),(.080,1.032),(0,1.022),(-.080,1.032)],2),head,.003,4))
 for s in (-1,1):
  eye=paint('Eye / fitted white oval Mr Mime eye '+str(s),WHITE,oval(s*.098,1.231,.032,.046,32),head,.006,3);p.append(eye)
  iris=paint('Pupil / fitted narrow black Mr Mime pupil '+str(s),INK,oval(s*.087,1.233,.009,.025,24),eye,.003,3);p.append(iris)
  p.append(paint('Eye glint / fitted white Mr Mime gleam '+str(s),WHITE,oval(s*.086,1.244,.0025,.006,16),iris,.002,2))
  p.append(paint('Cheek / fitted rosy Mr Mime cheek '+str(s),RED,oval(s*.171,1.106,.047,.056,32),head,.008,3))
  outline=[(s*x,y) for x,y in ((.18,1.305),(.255,1.413),(.365,1.46),(.475,1.472),(.548,1.385),(.448,1.385),(.446,1.290),(.351,1.30),(.330,1.224),(.251,1.224),(.186,1.28))]
  p.append(crease_normals(sheet('Ear fan / closed scalloped blue Mr Mime fan '+str(s),BLUE,flowing(outline,4),lambda x,y:-.025,.025,2),35))
  shoulder=ellipsoid('Shoulder / rooted rosy round Mr Mime shoulder '+str(s),RED,(s*.253,.839,0),(.119,.122,.116),36,24);p.append(shoulder)
  hand=(-.48,1.257,.16) if s==-1 else (.49,.772,.30)
  pts=[(s*.285,.851,.007),(-.494,.899,.076),(-.48,1.190,.148)] if s==-1 else [(s*.285,.851,.007),(.46,.9,.113),(.485,.687,.286)]
  p.append(curved_tube('Arm / thin bent pale pink Mr Mime arm '+str(s),SKIN,pts,[.037,.033,.031],28))
  palm=ellipsoid('Palm / rooted rounded white Mr Mime glove '+str(s),WHITE,hand,(.086,.088,.037),36,24);p.append(palm)
  hx,hy,hz=hand
  # Five independent digits radiate from each actual glove, including a thumb.
  for i,deg in enumerate((170,125,85,45,-10)):
   a=math.radians(deg);dx,dy=math.cos(a),math.sin(a);root=(hx+dx*.060,hy+dy*.061,hz);mid=(hx+dx*.092,hy+dy*.095,hz+.008);tip=(hx+dx*.132,hy+dy*.132,hz+.011)
   p.append(curved_tube(f'Finger / rooted spread white Mr Mime digit {s} {i}',WHITE,[root,mid,tip],[.022,.020,.017],24))
   bulb=ellipsoid(f'Fingertip / rounded white Mr Mime fingertip {s} {i}',WHITE,tip,(.027,.029,.024),28,20);p.append(bulb)
   p.append(paint(f'Finger pad / fitted pink Mr Mime fingertip {s} {i}',RED,oval(tip[0],tip[1],.017,.018,24),bulb,.003,2))
  hip=ellipsoid('Hip / rooted rosy round Mr Mime hip '+str(s),RED,(s*.211,.412,0),(.100,.105,.094),32,22);p.append(hip)
  knee=(s*.307,.255,.064)
  p.append(curved_tube('Thigh / rooted pale pink Mr Mime upper leg '+str(s),SKIN,[(s*.224,.403,.012),knee],[.038,.035],28))
  p.append(ellipsoid('Knee / rooted rounded white Mr Mime knee '+str(s),WHITE,knee,(.057,.040,.049),32,22))
  footroot=(s*.293,.099,.117)
  p.append(curved_tube('Shin / rooted pale pink Mr Mime lower leg '+str(s),SKIN,[knee,(s*.32,.188,.113),footroot],[.033,.029,.028],28))
  p.append(ellipsoid('Shoe / grounded blue Mr Mime slipper '+str(s),BLUE,(s*.294,.062,.167),(.092,.062,.137),36,24))
  p.append(curved_tube('Shoe curl / rooted upturned blue Mr Mime toe '+str(s),BLUE,[(s*.292,.061,.249),(s*.290,.105,.316),(s*.292,.172,.317),(s*.292,.181,.285),(s*.292,.156,.278)],[.038,.027,.020,.012,.004],28))
 p.append(paint('Chest / fitted rosy Mr Mime chest medallion',RED,oval(0,.667,.111,.111,40),body,.009,3))
 return p

def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'mr__mime','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'mr__mime.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
