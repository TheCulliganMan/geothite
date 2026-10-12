#!/usr/bin/env python3
"""Original pink-and-white egg nurse with curling locks and wing-like frills."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,linear,part,crease_normals
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from cubone_sculpt import recessed,skin_shading
from chansey_sculpt import cup,PINK,POUCH,EGG,INK
from skin_glb import COORDINATES
HEIGHT=1.20
WHITE=(.97,.985,.95);TONGUE=(.88,.43,.51)
def anatomy():
 lower=loft('Lower body / soft white Blissey lower body',WHITE,curved_profiles(((.158,.070,.063,0),(.220,.245,.183,0),(.330,.360,.260,0),(.425,.376,.273,0),(.480,.348,.256,0),(.515,.303,.231,0)),5),64)
 upper=loft('Upper body / broad pink Blissey upper body',PINK,curved_profiles(((.420,.329,.242,0),(.475,.380,.276,0),(.655,.401,.294,0),(.845,.346,.253,0),(.985,.255,.178,0),(1.075,.164,.110,0),(1.115,.056,.037,0),(1.120,.010,.010,0)),6),80)
 upper=recessed(upper,[(0,.984,.054,.060,.065)]);upper['base_color']=linear(PINK)
 pouch=cup('Pouch / closed pink Blissey cup with actual open rim')
 egg=ellipsoid('Egg / pale Blissey egg supported inside pink pouch',EGG,(0,.470,.355),(.112,.18,.105),48,36)
 p=[lower,upper,pouch,egg]
 lining=skin_shading(paint('Mouth lining / fitted dark Blissey happy mouth',INK,[(-.040,1.003),(.040,1.003),(.037,.982),(.025,.960),(0,.952),(-.025,.960),(-.037,.982)],upper,.005,4),upper);p.append(lining)
 p.append(skin_shading(paint('Tongue / fitted pink Blissey mouth tongue',TONGUE,oval(0,.968,.019,.011,28),lining,.004,3),lining))
 for s in (-1,1):
  eye=paint('Eye / fitted tiny dark Blissey eye '+str(s),INK,oval(s*.055,1.040,.0085,.024,28),upper,.004,3);p.append(eye)
  p.append(paint('Eye glint / fitted small white Blissey gleam '+str(s),WHITE,oval(s*.054,1.050,.002,.005,20),eye,.002,2))
  p.append(ellipsoid('Crown / rooted large pink Blissey side crown lobe '+str(s),PINK,(s*.220,1.105,-.017),(.145,.125,.110),48,32))
  p.append(curved_tube('Curl / rooted loose curling pink Blissey front lock '+str(s),PINK,[(s*.220,1.140,.005),(s*.340,1.110,.170),(s*.360,1.035,.210),(s*.290,1.015,.230),(s*.266,1.058,.230),(s*.295,1.071,.230)],[.060,.048,.035,.026,.012,.002],36))
  shoulder=ellipsoid('Shoulder / rooted pink Blissey shoulder '+str(s),PINK,(s*.346,.760,.006),(.090,.109,.103),36,26);p.append(shoulder)
  for j,end_y in enumerate((.935,.842,.742)):
   p.append(paddle(f'Arm frill / rooted white Blissey wing petal {s} {j}',WHITE,[(s*.344,.785,.007),(s*.470,.810,.017),(s*.574,end_y,.035)],[.063,.070,.005],[.039,.035,.003]))
  p.append(paddle('Arm / rooted short pink Blissey arm '+str(s),PINK,[(s*.349,.732,.049),(s*.445,.711,.110),(s*.520,.705,.136)],[.063,.054,.017],[.057,.047,.013]))
  p.append(ellipsoid('Hand / rooted soft pink Blissey hand tip '+str(s),PINK,(s*.503,.708,.138),(.054,.039,.040),36,24))
  p.append(ellipsoid('Leg / rooted slim pink Blissey leg '+str(s),PINK,(s*.174,.153,.015),(.060,.133,.073),36,28))
  p.append(ellipsoid('Foot / grounded rounded pink Blissey foot '+str(s),PINK,(s*.174,.050,.037),(.061,.050,.090),36,26))
 for j in range(10):
  a=(j+.5)*math.tau/10;x=math.sin(a);z=math.cos(a)
  name='Skirt / rooted white Blissey pointed skirt petal '+str(j)
  petal=paddle(name,WHITE,[(.309,.460,0),(.395,.435,0),(.472,.390,0)],[.030,.023,.003],[.096,.099,.003])
  vertices=[]
  for i in range(0,len(petal['positions']),3):
   u,y,v=petal['positions'][i:i+3];vertices.append((u*x-v*z,y,.76*(u*z+v*x)))
  p.append(crease_normals(part(name,WHITE,vertices,[tuple(petal['indices'][i:i+3]) for i in range(0,len(petal['indices']),3)],True),45))
 return p
def sculpture():
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'blissey','version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'blissey.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
