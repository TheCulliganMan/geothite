#!/usr/bin/env python3
"""Original armored parent with raised clawed fists and a baby in an open pouch."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,crease_normals,tube
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from cubone_sculpt import skin_shading
from goldfish_sculpt import surface
from skin_glb import COORDINATES
HEIGHT=1.40
TAN=(.66,.56,.51);PLATE=(.73,.64,.57);CREAM=(.91,.86,.63)
SKULL=(.34,.34,.35);BABY=(.66,.65,.79);INK=(.16,.12,.13)
WHITE=(.97,.97,.95);RED=(.70,.19,.22);PINK=(.92,.64,.72)

def cup():
 rows=[]
 for j in range(21):
  t=.015+.985*j/20;k=math.sqrt(1-(1-t)**2);rows.append((.27+.35*t,.245*k,.150*k,.340))
 for j in reversed(range(21)):
  t=.015+.985*j/20;k=math.sqrt(1-(1-t)**2);rows.append((.293+.327*t,.227*k,.132*k,.340))
 return crease_normals(loft('Pouch / closed cream Kangaskhan cup with actual open rim',CREAM,rows,48),45)

def anatomy():
 body=loft('Body / heavy rounded gray brown Kangaskhan parent torso',TAN,curved_profiles(((.16,.10,.10,0),(.25,.28,.235,0),(.45,.365,.29,.012),(.67,.35,.265,.01),(.82,.285,.22,0),(.925,.12,.11,.01),(.946,.025,.023,.01)),5),48)
 head=loft('Head / broad rounded tan Kangaskhan parent head',TAN,curved_profiles(((.81,.035,.045,.04),(.88,.17,.16,.04),(.98,.237,.205,.04),(1.10,.23,.213,.04),(1.19,.175,.16,.04),(1.242,.03,.025,.04)),5),48)
 p=[body,head]
 p.append(skin_shading(paint('Belly / fitted broad cream Kangaskhan belly',CREAM,oval(0,.51,.275,.295,48),body,.007,3),body))
 pouch=cup();p.append(pouch)
 skull=loft('Skull plate / rooted dark Kangaskhan forehead armor',SKULL,curved_profiles(((1.125,.21,.19,.04),(1.19,.205,.18,.04),(1.245,.15,.13,.04),(1.271,.018,.015,.04)),4),48);p.append(skull)
 snout=ellipsoid('Snout / rooted broad upper Kangaskhan muzzle',TAN,(0,.976,.247),(.233,.093,.178),48,32);p.append(snout)
 jaw=ellipsoid('Jaw / rooted separate lower Kangaskhan jaw',TAN,(0,.830,.205),(.170,.055,.148),44,28);p.append(jaw)
 p.append(ellipsoid('Mouth back / dark Kangaskhan oral lining behind actual jaw gap',INK,(0,.900,.230),(.168,.072,.024),40,28))
 p.append(ellipsoid('Tongue / rooted broad pink Kangaskhan tongue on lower jaw',PINK,(0,.858,.258),(.133,.022,.100),40,26))
 for s in (-1,1):
  p.append(curved_tube('Fang / rooted upper white Kangaskhan fang '+str(s),WHITE,[(s*.145,.927,.325),(s*.147,.900,.355),(s*.148,.876,.365)],[.025,.018,.002],20))
  white=skin_shading(paint('Eye / fitted angular white Kangaskhan parent eye '+str(s),WHITE,[(s*.050,1.075),(s*.160,1.110),(s*.158,1.060),(s*.070,1.055)],head,.004,3),head);p.append(white)
  pupil=skin_shading(paint('Pupil / fitted red Kangaskhan parent pupil '+str(s),RED,oval(s*.133,1.084,.012,.014,24),white,.003,3),white);p.append(pupil)
  p.append(paint('Eye slit / fitted black Kangaskhan parent gaze '+str(s),INK,oval(s*.132,1.084,.003,.009,20),pupil,.002,2))
  p.append(paint('Nostril / fitted small dark Kangaskhan muzzle nostril '+str(s),INK,oval(s*.078,.980,.005,.010,20),snout,.003,3))
  ear=paddle('Ear / rooted broad cream Kangaskhan parent ear '+str(s),CREAM,[(s*.158,1.135,.050),(s*.244,1.171,.058),(s*.322,1.231,.066),(s*.356,1.260,.065)],[.058,.075,.046,.003],[.043,.036,.018,.002]);p.append(ear)
  shoulder=ellipsoid('Shoulder / rooted heavy Kangaskhan shoulder '+str(s),TAN,(s*.338,.763,.002),(.105,.126,.106),36,26);p.append(shoulder)
  arm=curved_tube('Arm / rooted raised heavy Kangaskhan arm '+str(s),TAN,[(s*.335,.740,.060),(s*.405,.770,.040),(s*.470,.850,.045),(s*.475,.950,.050)],[.080,.078,.072,.066],28);p.append(arm)
  fist=ellipsoid('Fist / rooted raised rounded Kangaskhan fist '+str(s),TAN,(s*.480,.995,.068),(.100,.110,.100),36,26);p.append(fist)
  for j,dx in enumerate((-.044,0,.044)):
   p.append(curved_tube(f'Hand claw / rooted white Kangaskhan claw {s} {j}',WHITE,[(s*.480+dx,1.039,.125),(s*.480+dx,1.073,.159),(s*.480+dx,1.094,.162)],[.024,.018,.0015],18))
  p.append(crease_normals(tube('Shoulder plate / rooted angular Kangaskhan shoulder armor '+str(s),PLATE,[(s*.282,.799,.078),(s*.318,.842,.089),(s*.367,.846,.091),(s*.414,.804,.078)],[.022,.024,.024,.022],4),35))
  haunch=ellipsoid('Haunch / rooted broad Kangaskhan hind thigh '+str(s),TAN,(s*.250,.335,0),(.147,.190,.150),40,28);p.append(haunch)
  outline=skin_shading(paint('Haunch armor outline / fitted dark Kangaskhan plate border '+str(s),INK,oval(s*.263,.344,.09,.115,32),haunch,.003,3),haunch);p.append(outline)
  p.append(skin_shading(paint('Haunch plate / fitted tan Kangaskhan thigh armor '+str(s),PLATE,oval(s*.263,.344,.074,.094,32),outline,.003,3),outline))
  p.append(ellipsoid('Ankle / rooted thick Kangaskhan ankle '+str(s),TAN,(s*.258,.163,.055),(.092,.116,.105),36,24))
  p.append(ellipsoid('Foot / grounded broad Kangaskhan foot '+str(s),TAN,(s*.262,.075,.055),(.135,.075,.190),40,26))
  for j,dx in enumerate((-.081,0,.081)):
   p.append(curved_tube(f'Foot claw / rooted white Kangaskhan toe claw {s} {j}',WHITE,[(s*.262+dx,.055,.172),(s*.262+dx,.045,.248),(s*.262+dx,.035,.295)],[.040,.030,.003],18))
 p.append(curved_tube('Tail / rooted heavy balancing Kangaskhan rear tail',TAN,[(0,.250,-.200),(-.340,.200,-.400),(-.620,.310,-.620)],[.150,.120,.018],32))
 # Read the actual stored rear torso surface instead of guessing its depth.
 mirrored=dict(body);mirrored['positions']=[-v if i%3==2 else v for i,v in enumerate(body['positions'])]
 rear=surface(mirrored)
 for j,y in enumerate((.34,.51,.68,.81)):
  z=-rear(y,0)
  p.append(paddle('Back plate / rooted pointed Kangaskhan dorsal armor '+str(j),PLATE,[(0,y,z+.018),(0,y+.025,z-.11)],[.048,.002],[.030,.002]))
 baby=ellipsoid('Baby body / pale violet Kangaskhan baby seated inside pouch',BABY,(0,.440,.340),(.090,.145,.075),40,28);p.append(baby)
 p.append(ellipsoid('Baby neck / rooted short Kangaskhan baby neck',BABY,(0,.591,.340),(.050,.060,.055),32,24))
 baby_head=ellipsoid('Baby head / rooted rounded pale violet Kangaskhan baby head',BABY,(0,.705,.325),(.110,.119,.095),40,28);p.append(baby_head)
 for s in (-1,1):
  p.append(paddle('Baby ear / rooted small cream Kangaskhan baby ear '+str(s),CREAM,[(s*.060,.775,.325),(s*.126,.790,.325),(s*.160,.784,.325)],[.036,.035,.002],[.025,.027,.002]))
  eye=skin_shading(paint('Baby eye / fitted black oval Kangaskhan baby eye '+str(s),INK,oval(s*.045,.744,.022,.030,28),baby_head,.004,3),baby_head);p.append(eye)
  p.append(paint('Baby glint / fitted white Kangaskhan baby gleam '+str(s),WHITE,oval(s*.043,.756,.005,.007,20),eye,.002,2))
  p.append(curved_tube('Baby arm / rooted Kangaskhan baby arm reaching pouch rim '+str(s),BABY,[(s*.061,.647,.348),(s*.086,.633,.415),(s*.120,.635,.450)],[.021,.023,.016],20))
  p.append(ellipsoid('Baby hand / rooted Kangaskhan baby hand holding pouch rim '+str(s),BABY,(s*.120,.633,.457),(.025,.027,.029),32,22))
 p.append(skin_shading(paint('Baby mouth / fitted small dark Kangaskhan baby mouth',INK,oval(0,.677,.015,.019,28),baby_head,.004,3),baby_head))
 return p

def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'kangaskhan','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'kangaskhan.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
