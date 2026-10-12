#!/usr/bin/env python3
"""Original distinct small female and male poison-pin creatures."""
import argparse,json
from pathlib import Path
from sculpt_geometry import ellipsoid,crease_normals,loft,curved_profiles,translate
from bovine_sculpt import curved_tube,oval,reverse_front
from raptor_bird_sculpt import paint,sheet
from nidorino_sculpt import side_patch
from rhyhorn_sculpt import longitudinal
from skin_glb import COORDINATES
SPECIES=('nidoran_f','nidoran_m');HEIGHTS={'nidoran_f':.78,'nidoran_m':.80}
PALE=(.70,.73,.86);PURPLE=(.67,.46,.69);NAVY=(.41,.40,.64);SPOT=(.47,.30,.51);TEAL=(.17,.48,.50)
WHITE=(.98,.98,.94);INK=(.075,.075,.105);ORANGE=(.79,.30,.19)

def ears(p,female,color):
 for s in (-1,1):
  if female:
   xy=[(.122,.713),(.211,.851),(.426,.953),(.401,.901),(.470,.910),(.573,.810),(.594,.785),(.527,.781),(.508,.719),(.419,.667),(.300,.667),(.176,.706)]
   inner=[(.170,.735),(.245,.813),(.408,.872),(.485,.807),(.473,.735),(.404,.695),(.292,.700),(.205,.721)]
   fold=[(.180,.734),(.254,.795),(.327,.815),(.367,.793),(.374,.744),(.342,.714),(.259,.718)]
   depth=lambda y,x:.060-.14*(abs(x)-.12)
  else:
   xy=[(.105,.471),(.136,.720),(.162,.954),(.192,.917),(.293,1.153),(.307,1.075),(.351,1.087),(.353,.996),(.391,1.008),(.371,.881),(.393,.867),(.350,.708),(.249,.545),(.146,.479)]
   inner=[(.145,.550),(.194,.797),(.283,1.015),(.326,.992),(.333,.877),(.304,.746),(.236,.602),(.177,.555)]
   depth=lambda y,x:.170-.35*(y-.471)
  ear=sheet('Ear / broad pale pinna '+str(s) if female else 'Ear / large ragged purple pinna '+str(s),color,[(s*x,y) for x,y in xy],depth,.028,3);p.append(ear)
  innerp=paint('Ear inset / fitted navy field '+str(s) if female else 'Ear inset / fitted teal field '+str(s),NAVY if female else TEAL,[(s*x,y) for x,y in inner],ear,.005,3);p.append(innerp)
  if female:p.append(paint('Ear fold / fitted teal lower lobe '+str(s),TEAL,[(s*x,y) for x,y in fold],innerp,.005,3))

def limbs(p,body,female,color):
 for s in (-1,1):
  for front in (True,False):
   tag=f'{s} '+('front' if front else 'rear');z=(.108 if front else -.216) if female else (.185 if front else -.306)
   shoulder=ellipsoid('Haunch / rooted poison-pin quarter '+tag,color,(s*(.155 if female else .183),.185 if female else .153,z),(.081,.125,.115) if female or front else (.130,.143,.151),32,22);p.append(shoulder)
   leg=translate(crease_normals(loft('Leg / short planted poison-pin ankle '+tag,color,curved_profiles(((.024,.039,.043,z+.036),(.075,.052,.061,z+.02),(.169,.058,.071,z),(.239 if female else .183,.045,.054,z-.01)),4),24),45),s*.170,0,0);p.append(leg)
   foot=ellipsoid('Foot / grounded poison-pin paw '+tag,color,(s*.170,.025,z+.046),(.061,.025,.083),28,18);p.append(foot)
   for j,dx in enumerate((-.035,.035)):
    p.append(curved_tube(f'Claw / two rooted ivory poison-pin toes {tag} {j}',WHITE,[(s*.170+dx*.7,.025,z+.097),(s*.170+dx,.013,z+.139),(s*.170+dx,.008,z+.157)],[.019,.013,.002],20))
   if female or not front:
    zz=z-.018;yy=.205 if female else .176
    p.append(side_patch('Haunch spot / fitted poison-pin marking '+tag,NAVY if female else SPOT,oval(zz,yy,.034,.043,20),shoulder,s))
  for j,(z,y,w,h) in enumerate(((-.125,.459,.026,.035),(-.221,.350,.027,.042),(-.060,.302,.015,.021)) if female else ((-.119,.337,.025,.033),(-.020,.270,.012,.018))):
   p.append(side_patch(f'Flank spot / fitted poison-pin marking {s} {j}',NAVY if female else SPOT,oval(z,y,w,h,20),body,s))

def sculpture(species):
 if species not in SPECIES:raise ValueError('unsupported small poison-pin creature')
 female=species=='nidoran_f';color=PALE if female else PURPLE
 body=ellipsoid('Body / compact pale female pear' if female else 'Body / low crouching purple male barrel',color,(0,.350,-.061) if female else (0,.242,-.124),(.225,.264,.294) if female else (.220,.174,.330),44,26)
 head=ellipsoid('Head / round compact pale female cranium',color,(0,.633,.120),(.198,.178,.176),44,26) if female else longitudinal('Head / lowered purple male poison-pin wedge',color,[(.002,.381,.077,.064),(.139,.415,.163,.130),(.279,.384,.178,.136),(.395,.312,.116,.086),(.482,.270,.065,.048),(.509,.261,.008,.016)])
 muzzle=ellipsoid('Muzzle / broad low poison-pin snout',color,(0,.559,.267) if female else (0,.275,.422),(.146,.065,.101) if female else (.089,.039,.090),36,24);p=[body,head,muzzle]
 if female:
  p.append(curved_tube('Horn / small rooted pale forehead point',color,[(0,.767,.125),(0,.854,.165),(0,.925,.176)],[.043,.028,.002],28))
 else:p.append(curved_tube('Horn / rooted forward purple nasal point',color,[(0,.497,.257),(0,.525,.351),(0,.535,.491)],[.043,.031,.002],28))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in (((.038,.650),(.124,.707),(.157,.652),(.130,.611),(.079,.610)) if female else ((.058,.345),(.122,.405),(.151,.416),(.142,.354),(.101,.331))) ]
  socket=paint('Eye socket / fitted pointed poison-pin recess '+str(s),INK,xy,head,.006,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted ivory poison-pin lens '+str(s),WHITE,[(cx+.90*(x-cx),cy+.88*(y-cy)) for x,y in xy],socket,.004,3);p.append(eye)
  x=s*(.107 if female else .112);y=.650 if female else .363
  iris=paint('Iris / fitted orange poison-pin eye '+str(s),ORANGE,oval(x,y,.015 if female else .013,.022 if female else .018,24),eye,.003,3);p.append(iris)
  pupil=paint('Pupil / fitted dark poison-pin slit '+str(s),INK,oval(x,y,.007,.018 if female else .014,20),iris,.003,2);p.append(pupil)
  p.append(paint('Glint / tiny ivory poison-pin eye '+str(s),WHITE,oval(x-.002,y+.007,.002,.0035,16),pupil,.002,2))
  p.append(paint('Nostril / fitted tiny poison-pin snout '+str(s),INK,oval(s*(.039 if female else .028),.574 if female else .283,.004,.003,16),muzzle,.003,2))
  if female:
   for j,(y,z) in enumerate(((.606,.167),(.557,.176),(.515,.172))):
    p.append(curved_tube(f'Cheek barb / rooted pale poison whisker {s} {j}',color,[(s*.128,y,z),(s*.221,y-.027,z+.032),(s*.284,y-.065,z+.064)],[.013,.010,.0015],20))
  y=.515 if female else .246;z=.322 if female else .469
  p.append(curved_tube('Incisor / small rooted ivory poison-pin tooth '+str(s),WHITE,[(s*.059,y+.012,z),(s*.059,y-.009,z+.018),(s*.055,y-.024,z+.019)],[.014,.011,.002],20))
 yy=.516 if female else .250;ww=.113 if female else .070
 p.append(paint('Mouth / fitted shallow poison-pin smile',INK,[(-ww,yy+.018),(-ww*.6,yy),(0,yy-.005),(ww*.6,yy),(ww,yy+.018),(ww*.8,yy+.003),(ww*.5,yy-.008),(0,yy-.012),(-ww*.5,yy-.008),(-ww*.8,yy+.003)],muzzle,.003,3))
 ears(p,female,color);limbs(p,body,female,color)
 for j,(z,y,h) in enumerate(((-.123,.526,.124),(-.255,.452,.140),(-.323,.348,.092)) if female else ((-.075,.377,.199),(-.211,.350,.189),(-.346,.264,.133))):
  p.append(curved_tube('Back spine / rooted small poison barb '+str(j),color,[(0,y,z),(0,y+h*.50,z-.022),(0,y+h,z-.054)],[.058,.037,.002],24))
 p.append(curved_tube('Tail / short rooted poison-pin rear point',color,[(0,.277 if female else .215,-.277 if female else -.393),(0,.301 if female else .234,-.401 if female else -.460),(0,.346 if female else .294,-.452 if female else -.512)],[.059,.037,.002],24))
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.mesh.json')).write_text(json.dumps(sculpture(s),separators=(',',':')))
if __name__=='__main__':main()
