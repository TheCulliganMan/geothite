#!/usr/bin/env python3
"""Original armored Tyranitar with a toothed skull, inset armor and a heavy spined tail."""
import argparse
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals
from bovine_sculpt import curved_tube,oval
from murkrow_sculpt import bill
from fire_dragon_sculpt import fitted
from skin_glb import COORDINATES
HEIGHT=1.40
COAT=(.52,.72,.25);RIDGE=(.59,.76,.32);SLATE=(.30,.34,.47);INK=(.065,.075,.08);WHITE=(.99,.97,.88);RED=(.83,.23,.19);PINK=(.83,.43,.47)
TAIL=((0.,.30,-.18),(0.,.25,-.48),(.04,.30,-.76),(.09,.45,-.98),(.12,.65,-1.07),(.12,.78,-1.03))
TAIL_RADII=(.17,.15,.11,.080,.045,.002)

def dent(mesh,color,holes):
 vertices=[]
 for i in range(0,len(mesh['positions']),3):
  x,y,z=mesh['positions'][i:i+3]
  if z>0:
   for cx,cy,rx,ry,depth in holes:
    r=((x-cx)/rx)**2+((y-cy)/ry)**2
    if r<1:z-=depth*(1-r)**2
  vertices.append((x,y,z))
 return part(mesh['part'],color,vertices,[tuple(mesh['indices'][i:i+3]) for i in range(0,len(mesh['indices']),3)],True)

def arm_path(side):return ((side*.235,.82,.025),(side*.375,.70,.13),(side*.405,.64,.24)),(.105,.085,.066)
def wrist(side):return (side*.40,.66,.33)
def spine(name,points,radii):return crease_normals(curved_tube(name,RIDGE,points,radii,12),30.)
def head_spines():
 specs=[('center',((0.,1.27,.075),(0.,1.46,.045),(0.,1.61,.020)),(.085,.048,.0015))]
 for side in (-1,1):
  specs += [(f'front {side}',((side*.105,1.275,.11),(side*.145,1.41,.10),(side*.165,1.51,.075)),(.063,.036,.0015)),(f'rear {side}',((side*.12,1.25,-.015),(side*.20,1.34,-.10),(side*.27,1.405,-.19)),(.075,.043,.0015))]
 return specs

def body_geometry(recess=True):
 profiles=((.15,.08,.10,-.04),(.25,.25,.23,-.035),(.42,.335,.285,-.035),(.62,.32,.28,-.020),(.80,.28,.245,0.),(.94,.205,.17,.025),(1.075,.13,.115,.045),(1.16,.09,.08,.055))
 body=loft('Body / continuous heavy armored Tyranitar torso and neck',COAT,curved_profiles(profiles,4),32)
 return dent(body,COAT,[(side*.21,.79,.085,.070,.029) for side in (-1,1)]) if recess else body

@lru_cache(maxsize=1)
def anatomy():
 body=body_geometry()
 head=bill('Head / broad angular Tyranitar brow and upper skull',COAT,[(-.075,.13,1.105,1.29),(.07,.215,1.095,1.36),(.23,.245,1.08,1.33),(.37,.195,1.085,1.24),(.435,.085,1.115,1.19)])
 jaw=bill('Jaw / open rooted armored Tyranitar mandible',COAT,[(-.015,.11,.995,1.12),(.12,.16,.91,1.015),(.29,.18,.91,1.005),(.39,.12,.95,1.035),(.425,.030,1.005,1.04)])
 p=[body,head,jaw,ellipsoid('Mouth back / recessed dark Tyranitar throat',INK,(0.,1.055,.065),(.14,.10,.065),24,16),ellipsoid('Tongue / rooted pink Tyranitar tongue',PINK,(0.,1.005,.26),(.085,.017,.075),24,14)]
 belly=fitted('Belly / fitted slate Tyranitar diamond shield',SLATE,[(-.155,.64),(0.,.72),(.155,.64),(.195,.43),(0.,.255),(-.195,.43)],body,.008);p.append(belly)
 for j,y in enumerate((.43,.535)):
  outline=[(-.155,y-.018),(-.09,y-.026),(0.,y-.036),(.09,y-.026),(.155,y-.018),(.15,y+.008),(0.,y-.005),(-.15,y+.008)]
  p.append(fitted('Belly armor / fitted green transverse Tyranitar shield band '+str(j),COAT,outline,belly,.006))
 for side in (-1,1):
  cx=side*.21;cy=.79
  p.append(fitted('Torso vent / actual recessed triangular black Tyranitar opening '+str(side),INK,[(cx-side*.045,cy+.028),(cx+side*.047,cy+.042),(cx+side*.028,cy-.034)],body,.0035))
  cx=side*.12;cy=1.25
  outline=[(cx+side*dx,cy+dy) for dx,dy in ((-.045,-.018),(.040,0.),(.035,.047),(-.035,.023))]
  socket=fitted('Eye socket / fitted angular Tyranitar recess '+str(side),INK,outline,head,.006);p.append(socket);ex=sum(x for x,y in outline)/4;ey=sum(y for x,y in outline)/4
  white=fitted('Eye white / fitted narrow ivory Tyranitar glare '+str(side),WHITE,[(ex+(x-ex)*.82,ey+(y-ey)*.68) for x,y in outline],socket,.003);p.append(white)
  iris=fitted('Iris / fitted red Tyranitar gaze '+str(side),RED,oval(ex,ey,.009,.013,20),white,.002);p.append(iris)
  pupil=fitted('Pupil / fitted charcoal Tyranitar slit '+str(side),INK,oval(ex,ey,.0035,.009,16),iris,.0015);p.append(pupil)
  p.append(fitted('Glint / fitted ivory Tyranitar reflection '+str(side),WHITE,oval(ex-side*.001,ey+.003,.0013,.0025,12),pupil,.0013))
  p.append(fitted('Nostril / fitted black Tyranitar nose '+str(side),INK,oval(side*.045,1.185,.008,.004,16),head,.003))
  for j,y in enumerate((1.145,1.235)):
   root=(side*.17,y,.065);p.append(spine('Cheek plate / rooted swept Tyranitar cheek point '+str(side)+' '+str(j),[root,(side*.26,y+.018,-.035),(side*.325,y+.04,-.105)],[.078,.049,.0015]))
  p.append(curved_tube('Fang / rooted ivory Tyranitar upper canine '+str(side),WHITE,[(side*.115,1.106,.285),(side*.118,1.071,.303),(side*.115,1.035,.32)],[.021,.014,.0015],16))
  points,radii=arm_path(side);p.append(curved_tube('Arm / stocky rooted Tyranitar upper arm '+str(side),COAT,points[:2],radii[:2],26))
  p.append(curved_tube('Forearm / rooted Tyranitar lower arm '+str(side),COAT,[points[1],points[2],wrist(side)],[radii[1],radii[2],.055],26))
  hand=wrist(side);p.append(ellipsoid('Hand / complete broad Tyranitar palm '+str(side),COAT,hand,(.082,.075,.080),24,16))
  for j,dx in enumerate((-.048,0.,.048)):
   p.append(curved_tube('Hand claw / rooted ivory Tyranitar digit '+str(side)+' '+str(j),WHITE,[(hand[0]+dx*.75,hand[1]+.020,hand[2]+.045),(hand[0]+dx,hand[1]-.005,hand[2]+.105),(hand[0]+dx*.95,hand[1]-.030,hand[2]+.135)],[.023,.016,.0015],16))
  thigh=ellipsoid('Thigh / thick rooted armored Tyranitar haunch '+str(side),COAT,(side*.25,.25,.015),(.195,.225,.205),30,20)
  thigh=dent(thigh,COAT,[(side*.29,.285,.075,.055,.025)]);p.append(thigh)
  p.append(fitted('Thigh vent / actual recessed triangular black Tyranitar armor '+str(side),INK,[(side*.245,.30),(side*.325,.32),(side*.305,.255)],thigh,.0035))
  p.append(curved_tube('Shin / rooted short Tyranitar ankle '+str(side),COAT,[(side*.25,.20,.075),(side*.28,.09,.16)],[.115,.095],24))
  p.append(ellipsoid('Foot / broad planted Tyranitar pad '+str(side),COAT,(side*.28,.060,.20),(.15,.06,.20),28,18))
  for j,dx in enumerate((-.085,0.,.085)):
   p.append(curved_tube('Foot claw / three rooted ivory Tyranitar toes '+str(side)+' '+str(j),WHITE,[(side*.28+dx*.8,.055,.34),(side*.28+dx,.035,.415),(side*.28+dx*1.02,.014,.455)],[.033,.022,.002],16))
 for j,x in enumerate((-.065,0.,.065)):
  p.append(curved_tube('Lower tooth / rooted ivory Tyranitar jaw tooth '+str(j),WHITE,[(x,.995,.30),(x,1.024,.325),(x,1.038,.335)],[.015,.010,.0015],14))
 for label,points,radii in head_spines():p.append(spine('Crown spike / rooted Tyranitar head '+label,points,radii))
 for j,(y,z) in enumerate(((.94,-.08),(.76,-.18),(.53,-.23),(.34,-.23))):p.append(spine('Back spike / rooted Tyranitar dorsal armor '+str(j),[(0.,y,z),(0.,y+.05,z-.14),(0.,y+.15,z-.31)],[.095,.063,.0015]))
 p.append(curved_tube('Tail / continuous heavy curved Tyranitar tail',COAT,TAIL,TAIL_RADII,28))
 for j in (2,3,4):
  x,y,z=TAIL[j];p.append(spine('Tail spike / rooted Tyranitar tail crest '+str(j),[(x,y+TAIL_RADII[j]*.55,z),(x,y+TAIL_RADII[j]+.10,z-.025),(x,y+TAIL_RADII[j]+.20,z-.07)],[TAIL_RADII[j]*.75,TAIL_RADII[j]*.44,.0015]))
 return tuple(p)

def sculpture(normalize=True):
 p=deepcopy(list(anatomy()));lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  if normalize:q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
  for field in ('positions','normals'):q[field]=[0. if v==0 else v for v in q[field]]
 return dict(name='tyranitar',version=1,coordinate_system=COORDINATES,primitives=p)

def main():
 from tyranitar_glb import export_tyranitar
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'tyranitar.glb').write_bytes(export_tyranitar())
if __name__=='__main__':main()
