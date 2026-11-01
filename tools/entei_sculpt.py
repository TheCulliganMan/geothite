#!/usr/bin/env python3
"""Original heavy Entei with an angular mask, hanging mane, back blades and smoke."""
import argparse,math
from copy import deepcopy
from functools import lru_cache
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,translate,part,fitted_contour,crease_normals
from bovine_sculpt import curved_tube,oval
from raptor_bird_sculpt import paint
from goldfish_sculpt import surface
from skin_glb import COORDINATES,smoothstep
HEIGHT=1.42
BODY=(198/255,115/255,0.);RED=(173/255,0.,0.)
MANE=tuple(v*.72 for v in BODY);CREAM=(.95,.87,.65)
SILVER=(.80,.83,.85);CUFF=(.30,.32,.34);WHITE=(1.,1.,1.);INK=(0.,0.,0.)
TAIL_POINTS=((0.,.65,-.58),(.08,.58,-.80),(.15,.46,-.98),(.16,.39,-1.13),(.23,.44,-1.27))

def plate(name,color,xy,z,th=.04):
 skin=z if callable(z) else lambda y,x:z
 return crease_normals(fitted_contour(name,color,xy,skin,th,2),40.)

def cuff(name,x,z):
 v=[];faces=[];n=28;sides=10
 for i in range(n):
  a=i*math.tau/n
  for j in range(sides):
   b=j*math.tau/sides;r=.024*math.cos(b)
   v.append((x+(.113+r)*math.cos(a),.255+.041*math.sin(b),z+(.122+r)*math.sin(a)))
 for i in range(n):
  for j in range(sides):
   a=i*sides+j;b=((i+1)%n)*sides+j;c=((i+1)%n)*sides+(j+1)%sides;d=i*sides+(j+1)%sides;faces.append((a,b,c,d))
 return part(name,CUFF,v,faces,True)

@lru_cache(maxsize=1)
def raw_sculpture():
 torso=ellipsoid('Body / heavy amber Entei torso',BODY,(0.,.59,-.17),(.305,.295,.535),32,18)
 neck=ellipsoid('Neck / raised powerful Entei neck',BODY,(0.,.93,.16),(.265,.335,.265),28,16)
 head=ellipsoid('Head / broad Entei mask foundation',MANE,(0.,1.20,.34),(.265,.225,.215),32,18)
 p=[torso,neck,head]
 p.append(ellipsoid('Bib / pale central Entei throat',CREAM,(0.,.84,.395),(.145,.325,.086),24,16))
 p.append(plate('Beard / pointed silver Entei chest star',SILVER,[(-.18,.64),(-.21,.45),(-.09,.49),(-.06,.37),(0.,.44),(.06,.37),(.09,.49),(.21,.45),(.18,.64)],.36,.035))
 for side in (-1,1):
  xy=[(side*x,y) for x,y in [(.09,1.13),(.28,1.21),(.40,1.03),(.48,.71),(.39,.73),(.44,.56),(.34,.62),(.35,.43),(.26,.50),(.19,.40),(.16,.60),(.09,.80)]]
  p.append(plate('Mane / hanging sculpted Entei cloak '+str(side),MANE,xy,lambda y,x:.39-(.25+.65*smoothstep(.60,.95,y))*abs(x)+.03*math.sin(y*3)+.10*smoothstep(.60,.95,y),.055))
  # Three rooted, broad, tapered dorsal blades per flank.
  for j,(tipx,tipy,tipz) in enumerate(((.40,1.19,-.58),(.54,1.08,-.79),(.60,.94,-.89))):
   root=(side*.19,.79,-.27-j*.05);tip=(side*tipx,tipy,tipz)
   # A closed diamond-section blade, with a broad root and sharp actual tip.
   mid=tuple(root[k]*.4+tip[k]*.6 for k in range(3));v=[]
   for center,width,depth in [(root,.105,.037),(mid,.072,.026),(tip,.006,.003)]:
    x,y,z=center;v.extend([(x-width,y,z),(x,y+depth,z),(x+width,y,z),(x,y-depth,z)])
   f=[(3,2,1,0),(8,9,10,11)]
   for row in range(2):
    for k in range(4):a=row*4+k;b=row*4+(k+1)%4;f.append((a,b,b+4,a+4))
   p.append(part('Back blade / silver Entei flank '+str(side)+' '+str(j),SILVER,v,f))
  for rear,z in ((False,.25),(True,-.49)):
   label='rear' if rear else 'fore';x=side*.235
   p.append(ellipsoid('Leg / rooted '+label+' Entei leg '+str(side),MANE if rear else BODY,(x,.37,z),(.119,.295,.128),24,14))
   p.append(cuff('Cuff / thick silver '+label+' Entei ankle band '+str(side),x,z))
   p.append(translate(loft('Foot / broad planted '+label+' Entei paw '+str(side),SILVER,[(0.,.108,.127,z+.035),(.026,.133,.159,z+.045),(.079,.129,.153,z+.032),(.119,.090,.108,z+.002)],24),x,0.,0.))
   # Rounded pale toes and real thin gaps remain readable at the battle scale.
   for j,dx in enumerate((-.075,0.,.075)):
    p.append(ellipsoid('Toe / rounded silver '+label+' Entei toe '+str(side)+' '+str(j),SILVER,(x+dx,.050,z+.158),(.035,.049,.061),16,10))
 # The red cheek plates sit behind the central gray muzzle mask.
 cheeks={}
 for side in (-1,1):
  xy=[(side*x,y) for x,y in [(.02,1.27),(.20,1.34),(.39,1.19),(.25,1.14),(.19,.88),(.10,1.07)]]
  cheeks[side]=plate('Face plate / angular red Entei cheek '+str(side),RED,xy,lambda y,x:.54-1.3*x*x-.07*max(0.,1.15-y),.035);p.append(cheeks[side])
 gray=plate('Mask / angular silver Entei muzzle shield',SILVER,[(-.19,1.17),(-.10,1.25),(0.,1.20),(.10,1.25),(.19,1.17),(.13,.87),(.075,1.04),(-.075,1.04),(-.13,.87)],lambda y,x:.55-1.6*x*x-.16*max(0.,1.16-y),.045);p.append(gray)
 p.append(ellipsoid('Muzzle / cream Entei nose pad',CREAM,(0.,1.055,.595),(.087,.096,.055),24,14))
 p.append(ellipsoid('Nose / small dark Entei nose',INK,(0.,1.093,.648),(.035,.018,.014),20,12))
 crown=[(-.36,1.37),(-.20,1.43),(-.05,1.47),(0.,1.60),(.055,1.47),(.22,1.43),(.38,1.52),(.30,1.36),(.09,1.30),(0.,1.34),(-.09,1.30),(-.28,1.32)]
 p.append(plate('Crown / swept golden Entei forehead star',BODY,crown,lambda y,x:.565-.90*abs(x)-.55*max(0.,y-1.35),.042))
 for side in (-1,1):
  # Eyes are fitted to the actual head foundation between crest and shield.
  x=side*.123;xy=[(x-side*.039,1.291),(x+side*.039,1.279),(x+side*.027,1.255),(x-side*.030,1.257)]
  socket=paint('Eye socket / fitted dark Entei stare '+str(side),INK,xy,cheeks[side],.012,3);p.append(socket)
  white=paint('Eye / fitted white Entei sclera '+str(side),WHITE,[(x+(a-x)*.82,1.273+(b-1.273)*.78) for a,b in xy],socket,.004,3);p.append(white)
  iris=paint('Iris / fitted red Entei iris '+str(side),RED,oval(x,1.271,.009,.007,20),white,.003,2);p.append(iris)
  p.append(paint('Pupil / narrow dark Entei pupil '+str(side),INK,oval(x,1.271,.003,.004,16),iris,.002,2))
 # A connected rolling cloud above the back, and a trailing white smoke tail.
 for j,(x,y,z,rx,ry,rz) in enumerate(((0.,1.16,-.02,.17,.09,.22),(.02,1.13,-.20,.19,.09,.23),(-.03,1.04,-.41,.18,.09,.22),(.02,.94,-.61,.16,.085,.20))):
  p.append(ellipsoid('Smoke / rolling dorsal Entei cloud '+str(j),WHITE,(x,y,z),(rx,ry,rz),24,12))
 p.append(curved_tube('Tail / connected white Entei smoke trail',WHITE,TAIL_POINTS,[.09,.10,.085,.065,.035],20))
 for j,(center,radii) in enumerate((((.08,.58,-.80),(.115,.105,.14)),((.15,.46,-.99),(.12,.105,.14)),((.16,.39,-1.13),(.088,.085,.105)))):
  p.append(ellipsoid('Tail cloud / trailing Entei smoke lobe '+str(j),WHITE,center,radii,20,12))
 return {'name':'entei','version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(*,normalize=True):
 model=deepcopy(raw_sculpture())
 if normalize:
  p=model['primitives'];low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

if __name__=='__main__':
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
 from entei_glb import export_entei
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_entei())
