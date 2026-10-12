#!/usr/bin/env python3
"""Original sea guardian with five-digit paddle wings and an actually open jaw."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals
from bovine_sculpt import smooth,curved_tube,oval,reverse_front
from raptor_bird_sculpt import sheet,paint
from goldfish_sculpt import surface,zy
from murkrow_sculpt import bill
from chikorita_sculpt import add,sub,mul,dot,cross,unit
from skin_glb import COORDINATES
SPECIES=('lugia',);HEIGHT=1.45
WHITE=(.95,.97,.98);BLUE=(.47,.64,.84);NAVY=(.19,.28,.53)
INK=(.065,.075,.12);IVORY=(.99,.99,.97);PINK=(.83,.46,.49)

def paddle(name,color,points,widths,depths):
 pp,ww=smooth(points,widths,6);_,dd=smooth(points,depths,6);v=[];frames=[];side=(0,0,1);n=28
 for k,p in enumerate(pp):
  tangent=unit(sub(pp[min(k+1,len(pp)-1)],pp[max(0,k-1)]));side=unit(sub(side,mul(tangent,dot(side,tangent))));other=cross(tangent,side);frames.append((side,other))
  v.extend(add(p,add(mul(side,dd[k]*math.cos(i*math.tau/n)),mul(other,ww[k]*math.sin(i*math.tau/n)))) for i in range(n))
 f=[tuple(reversed(range(n))),tuple((len(pp)-1)*n+i for i in range(n))]
 for k in range(len(pp)-1):
  for i in range(n):a=k*n+i;b=k*n+(i+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part(name,color,v,f,True),45)

def wings(p):
 for s in (-1,1):
  p.append(paddle('Wing arm / broad bent white shoulder '+str(s),WHITE,[(s*.16,.714,-.012),(s*.46,.906,-.047),(s*.70,1.093,-.080),(s*.83,1.139,-.092)],[.101,.092,.090,.128],[.090,.067,.051,.052]))
  palm=ellipsoid('Wing palm / rounded five-finger white fan '+str(s),WHITE,(s*.84,1.145,-.095),(.169,.183,.058),40,26);p.append(palm)
  paths=[[(.735,1.17),(.727,1.30),(.77,1.43),(.823,1.46)],
         [(.814,1.22),(.84,1.35),(.94,1.48),(1.026,1.50)],
         [(.897,1.19),(1.012,1.29),(1.15,1.365),(1.23,1.344)],
         [(.925,1.11),(1.061,1.16),(1.211,1.181),(1.293,1.132)],
         [(.883,1.047),(.987,1.033),(1.13,.972),(1.18,.91)]]
  for j,path in enumerate(paths):
   pp=[(s*x,y,-.095-.060*(x-.84)) for x,y in path]
   p.append(paddle(f'Wing digit / broad blunt white finger {s} {j}',WHITE,pp,[.066,.065,.061,.037],[.049,.043,.035,.022]))

def back_plates(p,body):
 back=surface(reverse_front(body))
 for s in (-1,1):
  for j,y in enumerate((.322,.425,.532,.639,.744)):
   d=back(y,-s*.075)
   xy=[(d-.039,y-.038),(d+.054,y-.041),(d+.178,y+.018),(d+.135,y+.059),(d-.037,y+.045)]
   p.append(zy(sheet(f'Back plate / five paired navy dorsal paddles {s} {j}',NAVY,xy,lambda y,x:s*.075,.026,2)))

def tail(p):
 points=[(0,.314,-.139),(0,.215,-.44),(.26,.153,-.87),(.52,.250,-1.10),(.68,.390,-1.18)]
 p.append(curved_tube('Tail / continuous swept ivory sea-guardian tail',WHITE,points,[.105,.082,.055,.034,.002],32))
 for s in (-1,1):
  p.append(paddle('Tail fin / paired navy terminal lance '+str(s),NAVY,[(.52+s*.015,.25,-1.10),(.55+s*.024,.35,-1.145),(.60+s*.031,.48,-1.20)],[.033,.026,.002],[.018,.013,.0015]))

def feet(p):
 for s in (-1,1):
  p.append(ellipsoid('Haunch / powerful rooted white guardian thigh '+str(s),WHITE,(s*.21,.264,.016),(.122,.182,.148),40,26))
  p.append(curved_tube('Leg / broad white guardian ankle '+str(s),WHITE,[(s*.23,.226,.058),(s*.25,.116,.095),(s*.25,.053,.159)],[.071,.062,.049],28))
  p.append(ellipsoid('Foot / grounded broad white guardian pad '+str(s),WHITE,(s*.25,.041,.160),(.106,.041,.142),40,24))
  for j,dx in enumerate((-.062,0,.062)):
   p.append(ellipsoid(f'Toe / three blunt white guardian digits {s} {j}',WHITE,(s*.25+dx,.028,.253 if j==1 else .239),(.038,.027,.081),32,20))

def sculpture(species='lugia'):
 if species not in SPECIES:raise ValueError('unsupported sea guardian')
 body=crease_normals(loft('Body / smooth ivory sea-guardian pear',WHITE,curved_profiles(((.162,.055,.067,-.035),(.270,.223,.215,-.021),(.449,.284,.249,-.008),(.625,.249,.205,.011),(.77,.163,.128,.019),(.867,.069,.063,.029)),5),48),45)
 neck=paddle('Neck / long forward-curving ivory sea throat',WHITE,[(0,.740,.020),(0,.92,.096),(0,1.115,.203),(0,1.279,.280)],[.091,.077,.070,.061],[.089,.075,.070,.064])
 head=ellipsoid('Head / streamlined ivory guardian cranium',WHITE,(0,1.313,.294),(.129,.132,.140),48,28);p=[body,neck,head]
 xy=[(0,.820),(.081,.778),(.162,.670),(.229,.492),(.195,.329),(.103,.234),(0,.209),(-.103,.234),(-.195,.329),(-.229,.492),(-.162,.670),(-.081,.778)]
 p.append(paint('Belly / supported broad pale-blue breast shield',BLUE,xy,body,.012,4))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.025,1.367),(.077,1.399),(.115,1.34),(.061,1.313),(.022,1.332))]
  mask=paint('Mask / fitted navy pointed eye shield '+str(s),NAVY,xy,head,.009,3);p.append(mask)
  xy=[(s*x,y) for x,y in ((.041,1.360),(.084,1.363),(.098,1.341),(.061,1.322),(.036,1.333))]
  eye=paint('Eye / fitted ivory angular sea-guardian lens '+str(s),IVORY,xy,mask,.005,3);p.append(eye)
  pupil=paint('Pupil / small dark guardian stare '+str(s),INK,oval(s*.071,1.345,.007,.010,20),eye,.004,3);p.append(pupil)
  xy=[(s*x,y) for x,y in ((.026,1.337),(.045,1.422),(.120,1.475),(.103,1.372),(.061,1.315))]
  p.append(sheet('Eye flag / rooted rising navy head vane '+str(s),NAVY,xy,lambda y,x:.377-2.90*(y-1.33),.030,2))
 # A white swept point is part of the head silhouette behind the paired eye flags.
 xy=[(-.245,1.375),(-.205,1.424),(-.012,1.455),(-.144,1.349),(-.253,1.341)]
 p.append(zy(sheet('Head crest / swept ivory occipital point',WHITE,xy,lambda y,x:0,.043,2)))
 p.append(bill('Jaw / hooked ivory upper sea beak',WHITE,[(.354,.093,1.265,1.332),(.444,.078,1.229,1.300),(.530,.031,1.242,1.264),(.559,.0015,1.232,1.242)]))
 p.append(bill('Jaw / open ivory lower mandible',WHITE,[(.312,.091,1.184,1.232),(.401,.082,1.109,1.161),(.490,.034,1.151,1.171),(.541,.0015,1.217,1.224)]))
 p.append(ellipsoid('Mouth / recessed dark oral floor',INK,(0,1.237,.420),(.074,.065,.012),40,26))
 p.append(ellipsoid('Tongue / supported small pink sea-guardian tongue',PINK,(0,1.153,.449),(.055,.019,.055),36,24))
 wings(p);back_plates(p,body);tail(p);feet(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'lugia.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
