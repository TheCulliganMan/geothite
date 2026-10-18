#!/usr/bin/env python3
"""Original horned rock larva and segmented blue armor pupa from local sprite silhouettes."""
import argparse
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals
from bovine_sculpt import curved_tube,oval,reverse_front
from lugia_sculpt import paddle
from murkrow_sculpt import bill
from fire_dragon_sculpt import fitted
from skin_glb import COORDINATES
SPECIES=('larvitar','pupitar');HEIGHTS={'larvitar':1.04,'pupitar':1.20}
GREEN=(.51,.72,.28);LIGHT=(.59,.77,.35);RED=(.76,.24,.40);BLUE=(.46,.61,.82);RIDGE=(.55,.69,.88);SEAM=(.23,.31,.45);INK=(.075,.08,.10);WHITE=(.99,.98,.91)
LARVA_TAIL=((0.,.20,-.12),(.025,.17,-.31),(.06,.24,-.46),(.08,.31,-.50));LARVA_WIDTHS=(.075,.105,.115,.075);LARVA_DEPTHS=(.075,.075,.06,.045)
def arm_path(side):return ((side*.13,.51,.015),(side*.23,.43,.10),(side*.26,.465,.18)),(.055,.045,.034)
def tail_prongs():
 return tuple((j,((.08+side*.035,.29,-.50),(.08+side*.14,.39 if side==0 else .34,-.55),(.08+side*.22,.51 if side==0 else .41,-.50))) for j,side in enumerate((-1,0,1)))
def pupa_crowns():
 return [('center',((0.,.96,-.015),(0.,1.15,-.03),(0.,1.34,-.055)),(.075,.045,.0015))]+[(str(side),((side*.115,.94,-.025),(side*.17,1.09,-.04),(side*.24,1.22,-.09)),(.075,.045,.0015)) for side in (-1,1)]
def pupa_sides():
 return [(side,j,((side*.18,y,-.015),(side*.35,y+.045,-.035),(side*.48,y+.10,-.08))) for j,y in enumerate((.43,.64,.86)) for side in (-1,1)]
def gaze(p,head,species):
 adult=species=='pupitar'
 for side in (-1,1):
  cx=side*(.10 if adult else .103);cy=.784 if adult else .824
  outline=[(cx+side*dx*(1.45 if adult else 1.15),cy+dy*(1.25 if adult else 1.15)) for dx,dy in ((-.035,-.018),(.035,.0),(.032,.046),(-.029,.024))]
  socket=fitted('Eye socket / fitted angular rock evolution recess '+str(side),INK,outline,head,.005);p.append(socket);x=sum(a for a,b in outline)/4;y=sum(b for a,b in outline)/4
  white=fitted('Eye white / fitted ivory rock evolution glare '+str(side),WHITE,[(x+(a-x)*.8,y+(b-y)*.70) for a,b in outline],socket,.003);p.append(white)
  iris=fitted('Iris / fitted red rock evolution gaze '+str(side),RED,oval(x,y,.010 if adult else .008,.017 if adult else .012,20),white,.002);p.append(iris)
  pupil=fitted('Pupil / fitted charcoal rock evolution slit '+str(side),INK,oval(x,y,.004 if adult else .003,.012 if adult else .008,16),iris,.0015);p.append(pupil)
  p.append(fitted('Glint / fitted ivory rock evolution reflection '+str(side),WHITE,oval(x-side*.0007,y+.002,.0012,.0021,12),pupil,.0013))
 return p

def larvitar():
 body=loft('Body / continuous round Larvitar torso and neck',GREEN,curved_profiles(((.115,.06,.06,-.025),(.20,.16,.14,-.025),(.36,.205,.18,-.02),(.51,.18,.16,0.),(.64,.12,.105,.02),(.73,.075,.075,.025)),4),32)
 head=bill('Head / shaped broad Larvitar brow and muzzle',GREEN,[(-.035,.09,.64,.87),(.09,.175,.63,.955),(.23,.19,.64,.91),(.34,.14,.67,.83),(.41,.065,.71,.79)])
 p=[body,head];p=gaze(p,head,'larvitar')
 p.append(crease_normals(curved_tube('Horn / single long swept Larvitar crown',LIGHT,[(0.,.865,.025),(0.,1.07,-.015),(0.,1.28,-.10)],[.087,.061,.0015],16),28.))
 p.append(fitted('Belly / fitted red Larvitar diamond',RED,[(-.08,.32),(0.,.49),(.08,.32),(0.,.18)],body,.006))
 p.append(fitted('Lip / fitted small closed Larvitar mouth',INK,[(-.05,.724),(0.,.711),(.05,.724),(.047,.729),(0.,.718),(-.047,.729)],head,.0025))
 for side in (-1,1):
  p.append(fitted('Cheek tear / long fitted black Larvitar eye stripe '+str(side),INK,[(side*.069,.795),(side*.122,.79),(side*.085,.711)],head,.0035))
  p.append(fitted('Nostril / small supported Larvitar nose '+str(side),INK,oval(side*.036,.771,.005,.003,16),head,.003))
  p.append(fitted('Body vent / fitted triangular black Larvitar side slit '+str(side),INK,[(side*.15,.455),(side*.187,.44),(side*.165,.40)],body,.0035))
  points,radii=arm_path(side);p.append(curved_tube('Arm / short complete Larvitar forelimb '+str(side),GREEN,points,radii,24));x,y,z=points[-1]
  p.append(ellipsoid('Hand / rounded Larvitar palm '+str(side),GREEN,(x,y,z+.012),(.047,.045,.050),24,16))
  for j,dx in enumerate((-.028,0.,.028)):p.append(curved_tube('Finger / three blunt green Larvitar digits '+str(side)+' '+str(j),GREEN,[(x+dx*.8,y-.006,z+.040),(x+dx,y-.012,z+.078)],[.016,.005],14))
  p.append(ellipsoid('Thigh / rooted rounded Larvitar haunch '+str(side),GREEN,(side*.14,.14,.005),(.108,.122,.115),28,18))
  p.append(ellipsoid('Foot / planted broad Larvitar pad '+str(side),GREEN,(side*.145,.034,.115),(.09,.034,.115),28,16))
  for j,dx in enumerate((-.043,0.,.043)):p.append(curved_tube('Toe / three blunt green Larvitar toes '+str(side)+' '+str(j),GREEN,[(side*.145+dx*.8,.029,.19),(side*.145+dx,.018,.254)],[.021,.007],14))
 p.append(paddle('Tail / broad curved Larvitar tail stem',GREEN,LARVA_TAIL,LARVA_WIDTHS,LARVA_DEPTHS,segments=24,steps=4))
 for j,points in tail_prongs():p.append(paddle('Tail prong / three rooted Larvitar fan points '+str(j),LIGHT,points,[.075,.075,.002],[.050,.035,.0015],segments=18,steps=4))
 return p

def pupitar():
 body=crease_normals(loft('Body / continuous tapered blue Pupitar cocoon',BLUE,curved_profiles(((.025,.015,.025,0.),(.11,.14,.12,-.02),(.25,.235,.185,-.03),(.46,.295,.22,-.025),(.66,.27,.215,-.02),(.83,.22,.18,-.005),(.94,.10,.11,.005)),3),16),28.)
 head=bill('Head / broad angular blue Pupitar visor shell',BLUE,[(-.10,.12,.62,.98),(-.015,.22,.55,1.05),(.10,.26,.55,.98),(.24,.20,.60,.88),(.27,.06,.69,.82)])
 p=gaze([body,head],head,'pupitar')
 for label,points,radii in pupa_crowns():p.append(crease_normals(curved_tube('Crown spike / three rooted Pupitar head points '+label,RIDGE,points,radii,12),25.))
 for side,j,points in pupa_sides():p.append(paddle('Side spike / three paired Pupitar shell points '+str(side)+' '+str(j),RIDGE,points,[.095,.075,.0015],[.075,.045,.0015],segments=12,steps=4))
 for j,(y,w) in enumerate(((.235,.18),(.39,.25),(.55,.245))):
  outline=[(-w,y+.015),(-w*.52,y-.014),(0.,y-.045),(w*.52,y-.014),(w,y+.015),(w*.98,y+.024),(0.,y-.032),(-w*.98,y+.024)]
  p.append(fitted('Shell seam / fitted dark Pupitar chevron '+str(j),SEAM,outline,body,.0035))
  p.append(reverse_front(fitted('Back seam / fitted dark rear Pupitar chevron '+str(j),SEAM,outline,reverse_front(body),.0035)))
 p.append(paddle('Tail keel / rooted pointed blue Pupitar lower fold',BLUE,[(0.,.16,-.02),(0.,.12,-.20),(0.,.02,-.31)],[.085,.075,.0015],[.075,.045,.0015],segments=16,steps=4))
 return p
@lru_cache(maxsize=2)
def anatomy(species):
 if species not in SPECIES:raise ValueError('unknown rock larva/pupa')
 return tuple((larvitar if species=='larvitar' else pupitar)())
def sculpture(species,normalize=True):
 p=deepcopy(list(anatomy(species)));lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  if normalize:q['positions']=[round((v-(lo if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
  for field in ('positions','normals'):q[field]=[0. if v==0 else v for v in q[field]]
 return dict(name=species,version=1,coordinate_system=COORDINATES,primitives=p)
def main():
 from rock_pupa_glb import export_rock_pupa
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_rock_pupa(s))
if __name__=='__main__':main()
