#!/usr/bin/env python3
"""Original Tyranitar with paired armor, planted shins and a low radial-tip tail."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,tube
from bovine_sculpt import curved_tube,oval,reverse_front
from murkrow_sculpt import bill
from fire_dragon_sculpt import fitted
from skin_glb import COORDINATES
HEIGHT=1.40
COAT=(.63,.72,.40);RIDGE=(.66,.75,.43);SLATE=(.30,.39,.55);INK=(.065,.075,.08);WHITE=(.99,.97,.88);RED=(.83,.23,.19);PINK=(.83,.43,.47)
TAIL=((0.,.29,-.18),(0.,.225,-.47),(.015,.20,-.77),(.035,.205,-1.02),(.045,.23,-1.19),(.045,.24,-1.24))
TAIL_RADII=(.175,.145,.11,.080,.050,.025)

def dent(mesh,color,holes,back=False):
 vertices=[]
 for i in range(0,len(mesh['positions']),3):
  x,y,z=mesh['positions'][i:i+3]
  if (z<0 if back else z>0):
   for cx,cy,rx,ry,depth in holes:
    r=((x-cx)/rx)**2+((y-cy)/ry)**2
    if r<1:z+=(1 if back else -1)*depth*(1-r)**2
  vertices.append((x,y,z))
 return part(mesh['part'],color,vertices,[tuple(mesh['indices'][i:i+3]) for i in range(0,len(mesh['indices']),3)],True)

def arm_path(side):
 return ((side*.265,.875,.005),(side*.405,.765,.045),(side*.47,.685,.105)),(.108,.085,.062)
def wrist(side):return (side*.48,.64,.155)
def spine(name,points,radii):
 # Broad diamond sections make armor wedges, without rounded peg-like roots.
 return crease_normals(tube(name,RIDGE,points,radii,4),30.)
def head_spines():
 # A swept sagittal blade and two lateral pairs continue the tapered skull.
 specs=[('center',((0.,1.32,.075),(0.,1.50,-.005),(0.,1.64,-.10)),(.083,.047,.0015))]
 for side in (-1,1):
  specs += [(f'front {side}',((side*.11,1.30,.06),(side*.15,1.43,-.01),(side*.19,1.51,-.10)),(.064,.038,.0015)),(f'rear {side}',((side*.15,1.245,-.025),(side*.23,1.34,-.12),(side*.31,1.38,-.24)),(.076,.044,.0015))]
 return specs

def tail_spines():
 # Short radial terminal points, rather than a row of tall upper fins.
 specs=[]
 for j,angle in enumerate((0.,math.pi/2,math.pi,3*math.pi/2)):
  dx,dy=math.sin(angle),math.cos(angle)
  root=(.042+dx*.033,.225+dy*.033,-1.14)
  mid=(.045+dx*.09,.23+dy*.09,-1.205)
  tip=(.045+dx*.15,.24+dy*.15,-1.275)
  specs.append((str(j),root,mid,tip))
 return specs

def body_geometry(recess=True):
 profiles=((.14,.07,.09,-.035),(.24,.27,.235,-.025),(.43,.365,.315,-.025),(.65,.365,.31,-.01),(.84,.325,.27,.005),(1.025,.25,.205,.018),(1.185,.17,.145,.035),(1.29,.105,.10,.045))
 body=loft('Body / continuous broad armored Tyranitar torso and neck',COAT,curved_profiles(profiles,4),28)
 if recess:
  body=dent(body,COAT,[(side*.175,1.015,.065,.052,.029) for side in (-1,1)]+[(side*.228,.885,.065,.045,.025) for side in (-1,1)])
  body=dent(body,COAT,[(0.,.995,.065,.055,.025),(0.,.83,.065,.055,.025)],True)
 return body

@lru_cache(maxsize=1)
def anatomy():
 body=body_geometry()
 head=bill('Head / tapered tall Tyranitar brow and compact muzzle',COAT,[(-.12,.105,1.075,1.34),(-.025,.18,1.055,1.405),(.105,.195,1.065,1.405),(.215,.16,1.085,1.32),(.295,.105,1.12,1.235),(.325,.034,1.135,1.195)])
 jaw=bill('Jaw / short rooted armored Tyranitar mandible',COAT,[(-.075,.105,1.045,1.145),(.025,.145,.995,1.105),(.16,.135,1.01,1.068),(.26,.085,1.055,1.095),(.30,.025,1.083,1.108)])
 jaw['positions']=[v-(.018 if k%3==1 else 0.) for k,v in enumerate(jaw['positions'])]
 p=[body,head,jaw,ellipsoid('Mouth back / recessed dark Tyranitar throat',INK,(0.,1.09,.035),(.135,.083,.105),22,14),ellipsoid('Tongue / rooted pink Tyranitar tongue',PINK,(0.,1.033,.15),(.060,.011,.06),20,12)]
 p.append(fitted('Brow keel / fitted descending Tyranitar forehead ridge',RIDGE,[(-.105,1.355),(0.,1.395),(.105,1.355),(.065,1.30),(0.,1.222),(-.065,1.30)],head,.014))
 # The opening is a long diamond with narrow dark segment seams.
 belly=fitted('Belly / fitted slate Tyranitar diamond shield',SLATE,[(0.,.91),(.175,.64),(0.,.365),(-.175,.64)],body,.009);p.append(belly)
 for j,y in enumerate((.55,.65,.75)):
  half=min(.158,(.91-y)/.27*.17,(y-.365)/.275*.17)
  p.append(fitted('Belly armor / recessed dark transverse Tyranitar shield seam '+str(j),(.20,.24,.35),[(-half,y-.005),(half,y-.005),(half,y+.004),(-half,y+.004)],belly,.0015))
 for side in (-1,1):
  # Thick surface-fitted upper armor descends around the belly opening.
  outline=[(side*x,y) for x,y in ((.025,1.205),(.19,1.12),(.26,.975),(.31,.77),(.235,.66),(.17,.81),(.035,.96))]
  mantle=fitted('Mantle / fitted pointed Tyranitar chest armor '+str(side),COAT,outline,body,.012);p.append(mantle)
  for j,(cx,cy,rx,ry) in enumerate(((side*.175,1.015,.039,.035),(side*.228,.885,.038,.027))):
   p.append(fitted('Torso vent / recessed triangular Tyranitar armor '+str(side)+' '+str(j),INK,[(cx-side*rx,cy+ry*.75),(cx+side*rx,cy+ry),(cx+side*rx*.40,cy-ry)],mantle,.002))
  cx=side*.115;cy=1.283
  outline=[(cx+side*dx,cy+dy) for dx,dy in ((-.038,-.020),(.036,-.003),(.027,.041),(-.025,.010))]
  socket=fitted('Eye socket / fitted angular Tyranitar recess '+str(side),INK,outline,head,.005);p.append(socket);ex=sum(x for x,y in outline)/4;ey=sum(y for x,y in outline)/4
  white=fitted('Eye white / fitted narrow ivory Tyranitar glare '+str(side),WHITE,[(ex+(x-ex)*.80,ey+(y-ey)*.65) for x,y in outline],socket,.0025);p.append(white)
  iris=fitted('Iris / fitted red Tyranitar gaze '+str(side),RED,oval(ex,ey,.008,.011,18),white,.0015);p.append(iris)
  pupil=fitted('Pupil / fitted charcoal Tyranitar slit '+str(side),INK,oval(ex,ey,.003,.0075,14),iris,.0013);p.append(pupil)
  p.append(fitted('Glint / fitted ivory Tyranitar reflection '+str(side),WHITE,oval(ex-side*.001,ey+.0025,.0012,.002,12),pupil,.001))
  p.append(fitted('Nostril / fitted black Tyranitar nose '+str(side),INK,oval(side*.030,1.186,.006,.003,14),head,.002))
  for j,y in enumerate((1.145,1.235)):
   root=(side*.14,y,-.015);p.append(spine('Cheek plate / rooted swept Tyranitar cheek point '+str(side)+' '+str(j),[root,(side*.215,y+.015,-.085),(side*.28,y+.05,-.19)],[.075,.044,.0015]))
  p.append(curved_tube('Fang / rooted ivory Tyranitar upper canine '+str(side),WHITE,[(side*.09,1.112,.21),(side*.09,1.095,.225),(side*.086,1.075,.235)],[.014,.009,.0015],12))
  points,radii=arm_path(side)
  # Both anatomical sections continue past the hinge, avoiding a capped cuff.
  overlap=tuple(a*.75+b*.25 for a,b in zip(points[1],wrist(side)))
  p.append(curved_tube('Arm / short sloping rooted Tyranitar upper arm '+str(side),COAT,[points[0],points[1],overlap],[radii[0],radii[1],.079],22))
  p.append(curved_tube('Forearm / short tapered rooted Tyranitar forearm '+str(side),COAT,[points[1],points[2],wrist(side)],[radii[1],radii[2],.054],22))
  hand=wrist(side);p.append(ellipsoid('Hand / complete compact Tyranitar palm '+str(side),COAT,hand,(.069,.060,.064),22,14))
  for j,dx in enumerate((-.040,0.,.040)):
   p.append(curved_tube('Hand claw / rooted ivory Tyranitar digit '+str(side)+' '+str(j),WHITE,[(hand[0]+dx*.75,hand[1]-.005,hand[2]+.035),(hand[0]+dx,hand[1]-.033,hand[2]+.071),(hand[0]+dx*.95,hand[1]-.048,hand[2]+.084)],[.018,.011,.0015],12))
  thigh=loft('Thigh / broad angular rooted Tyranitar haunch '+str(side),COAT,curved_profiles(((.09,.075,.095,.025),(.17,.15,.17,.015),(.30,.19,.205,-.005),(.43,.17,.185,-.025),(.51,.085,.10,-.04)),3),22)
  thigh['positions']=[v+(side*.275 if k%3==0 else 0.) for k,v in enumerate(thigh['positions'])]
  thigh=dent(thigh,COAT,[(side*.31,.385,.070,.043,.021),(side*.325,.265,.068,.04,.020)]);p.append(thigh)
  for j,(cy,rx) in enumerate(((.385,.048),(.265,.043))):
   cx=side*(.31 if j==0 else .325)
   p.append(fitted('Thigh vent / recessed triangular Tyranitar armor '+str(side)+' '+str(j),INK,[(cx-side*rx,cy+.025),(cx+side*rx,cy+.028),(cx+side*rx*.6,cy-.026)],thigh,.0025))
  p.append(curved_tube('Shin / near-vertical rooted Tyranitar ankle '+str(side),COAT,[(side*.29,.22,.015),(side*.30,.085,.025)],[.113,.099],22))
  foot=loft('Foot / full-heeled planted Tyranitar sole '+str(side),COAT,[(0.,.11,.135,.035),(.018,.133,.164,.045),(.065,.135,.168,.048),(.10,.118,.143,.035),(.125,.082,.10,.025)],24)
  foot['positions']=[v+(side*.30 if k%3==0 else 0.) for k,v in enumerate(foot['positions'])];p.append(foot)
  for j,dx in enumerate((-.074,0.,.074)):
   p.append(curved_tube('Foot claw / three rooted ivory Tyranitar toes '+str(side)+' '+str(j),WHITE,[(side*.30+dx*.8,.05,.165),(side*.30+dx,.025,.235),(side*.30+dx*1.02,.012,.27)],[.026,.017,.0015],12))
 for j,x in enumerate((-.047,0.,.047)):
  p.append(curved_tube('Lower tooth / rooted ivory Tyranitar jaw tooth '+str(j),WHITE,[(x,1.039,.22),(x,1.057,.235),(x,1.066,.24)],[.010,.0065,.0015],12))
 for label,points,radii in head_spines():p.append(spine('Crown spike / rooted Tyranitar head '+label,points,radii))
 # Two lateral rows frame the rear armor instead of one central fence.
 for side in (-1,1):
  for j,(y,z,rx,length) in enumerate(((1.12,-.07,.115,.22),(.955,-.14,.155,.25),(.79,-.20,.18,.24),(.625,-.23,.17,.21))):
   root=(side*rx,y,z);p.append(spine('Back spike / paired rooted Tyranitar dorsal armor '+str(side)+' '+str(j),[root,(side*(rx+.055),y+.045,z-length*.50),(side*(rx+.12),y+.10,z-length)],[.080,.050,.0015]))
 back_parent=reverse_front(body)
 back=fitted('Back armor / fitted pointed Tyranitar rear mantle',COAT,[(-.15,1.14),(0.,1.20),(.15,1.14),(.255,.95),(.25,.735),(.17,.66),(0.,.77),(-.17,.66),(-.25,.735),(-.255,.95)],back_parent,.012)
 p.append(reverse_front(back))
 for j,cy in enumerate((.995,.83)):
  vent=fitted('Back vent / recessed triangular Tyranitar opening '+str(j),INK,[(-.037,cy+.018),(.037,cy+.018),(0.,cy-.032)],back,.002)
  p.append(reverse_front(vent))
 p.append(curved_tube('Tail / continuous low heavy Tyranitar tail',COAT,TAIL,TAIL_RADII,26))
 p.append(spine('Tail hub / solid blunt diamond Tyranitar tail cap',[(.04,.22,-1.145),(.045,.235,-1.20),(.045,.24,-1.26)],[.075,.066,.055]))
 for label,root,mid,tip in tail_spines():p.append(spine('Tail spike / terminal radial Tyranitar point '+label,[root,mid,tip],[.058,.036,.0015]))
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
