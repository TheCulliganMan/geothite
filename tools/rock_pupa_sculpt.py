#!/usr/bin/env python3
"""Original horned rock larva and segmented blue armor pupa from local sprite silhouettes."""
import argparse
import math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,crease_normals,fitted_triangle_contour,translate
from bovine_sculpt import curved_tube,oval,reverse_front
from lugia_sculpt import paddle
from murkrow_sculpt import bill
from fire_dragon_sculpt import fitted
from cubone_sculpt import skin_shading
from bulldog_sculpt import rounded_loft
from skin_glb import COORDINATES
SPECIES=('larvitar','pupitar');HEIGHTS={'larvitar':1.04,'pupitar':1.20}
GREEN=(.51,.72,.28);LIGHT=(.59,.77,.35);RED=(.76,.24,.40);BLUE=(.46,.61,.82);RIDGE=(.55,.69,.88);SEAM=(.23,.31,.45);INK=(.075,.08,.10);WHITE=(.99,.98,.91)
LARVA_TAIL=((0.,.20,-.145),(.015,.18,-.285),(.030,.260,-.420));LARVA_WIDTHS=(.095,.110,.100);LARVA_DEPTHS=(.085,.085,.065)
LARVA_FAN=(.030,.260,-.450)
def arm_path(side):return ((side*.180,.570,.005),(side*.250,.500,.030),(side*.300,.470,.045)),(.068,.049,.008)
def tail_tooth(j):
 cx,cy,cz=LARVA_FAN;a=math.tau*j/6+.30;v=[]
 for r,w,d in ((.055,.035,.043),(.160,.060,.045),(.223,.029,.026)):
  x=cx+r*math.sin(a);y=cy+r*math.cos(a)
  v.extend((x+t*w*math.cos(a),y-t*w*math.sin(a),cz+z*d) for t,z in ((-1,-1),(1,-1),(1,1),(-1,1)))
 f=[(3,2,1,0),(8,9,10,11)]
 for k in (0,4):
  for i in range(4):f.append((k+i,k+(i+1)%4,k+4+(i+1)%4,k+4+i))
 return crease_normals(part('Tail prong / six rooted Larvitar radial points '+str(j),LIGHT,v,f,True),28)
def tail_hub():
 cx,cy,cz=LARVA_FAN;n=32;v=[(cx,cy,cz+.060)]
 rings=((.065,cz+.055),(.105,cz+.020),(.105,cz-.025),(.090,cz-.050),(.045,cz-.040))
 for r,z in rings:v.extend((cx+r*math.sin(i*math.tau/n),cy+r*math.cos(i*math.tau/n),z) for i in range(n))
 v.append((cx,cy,cz-.008));last=len(v)-1;f=[]
 for i in range(n):f.append((0,1+i,1+(i+1)%n))
 for k in range(len(rings)-1):
  for i in range(n):a=1+k*n+i;b=1+k*n+(i+1)%n;f.append((a,a+n,b+n,b))
 for i in range(n):f.append((last,1+(len(rings)-1)*n+(i+1)%n,1+(len(rings)-1)*n+i))
 return crease_normals(part('Tail hub / solid green Larvitar radial fan center',GREEN,v,f,True),40)
def larva_paint(name,color,xy,parent,th=.004):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),45),parent)
def pupa_crowns():
 return [('center',((0.,.96,-.015),(0.,1.15,-.03),(0.,1.34,-.055)),(.075,.045,.0015))]+[(str(side),((side*.115,.94,-.025),(side*.17,1.09,-.04),(side*.24,1.22,-.09)),(.075,.045,.0015)) for side in (-1,1)]
def pupa_sides():
 return [(side,j,((side*.18,y,-.015),(side*.35,y+.045,-.035),(side*.48,y+.10,-.08))) for j,y in enumerate((.43,.64,.86)) for side in (-1,1)]
def gaze(p,head,species):
 adult=species=='pupitar'
 for side in (-1,1):
  if not adult:
   outline=[(side*x,y) for x,y in ((.085,.823),(.188,.878),(.198,.821),(.144,.797))]
   socket=larva_paint('Eye socket / fitted angular rock evolution recess '+str(side),INK,outline,head,.005);p.append(socket);x=sum(a for a,b in outline)/4;y=sum(b for a,b in outline)/4
   white=larva_paint('Eye white / fitted ivory rock evolution glare '+str(side),WHITE,[(x+(a-x)*.82,y+(b-y)*.76) for a,b in outline],socket,.003);p.append(white)
   iris=larva_paint('Iris / fitted red rock evolution gaze '+str(side),RED,oval(side*.151,.834,.012,.016,20),white,.002);p.append(iris)
   pupil=larva_paint('Pupil / fitted charcoal rock evolution slit '+str(side),INK,oval(side*.151,.834,.004,.011,16),iris,.0015);p.append(pupil)
   p.append(larva_paint('Glint / fitted ivory rock evolution reflection '+str(side),WHITE,oval(side*.151-side*.001,.837,.0015,.0025,12),pupil,.0013))
   continue
  cx=side*(.10 if adult else .103);cy=.784 if adult else .824
  outline=[(cx+side*dx*(1.45 if adult else 1.15),cy+dy*(1.25 if adult else 1.15)) for dx,dy in ((-.035,-.018),(.035,.0),(.032,.046),(-.029,.024))]
  socket=fitted('Eye socket / fitted angular rock evolution recess '+str(side),INK,outline,head,.005);p.append(socket);x=sum(a for a,b in outline)/4;y=sum(b for a,b in outline)/4
  white=fitted('Eye white / fitted ivory rock evolution glare '+str(side),WHITE,[(x+(a-x)*.8,y+(b-y)*.70) for a,b in outline],socket,.003);p.append(white)
  iris=fitted('Iris / fitted red rock evolution gaze '+str(side),RED,oval(x,y,.010 if adult else .008,.017 if adult else .012,20),white,.002);p.append(iris)
  pupil=fitted('Pupil / fitted charcoal rock evolution slit '+str(side),INK,oval(x,y,.004 if adult else .003,.012 if adult else .008,16),iris,.0015);p.append(pupil)
  p.append(fitted('Glint / fitted ivory rock evolution reflection '+str(side),WHITE,oval(x-side*.0007,y+.002,.0012,.0021,12),pupil,.0013))
 return p

def larvitar():
 body=loft('Body / continuous round Larvitar torso and neck',GREEN,curved_profiles(((.070,.070,.080,-.025),(.180,.195,.175,-.025),(.340,.240,.205,-.015),(.510,.225,.195,.015),(.650,.205,.195,.035),(.800,.145,.120,.030)),4),36)
 head=loft('Head / shaped broad Larvitar brow and muzzle',GREEN,curved_profiles(((.510,.090,.100,.020),(.590,.175,.165,.040),(.700,.220,.230,.055),(.830,.225,.200,.015),(.940,.145,.130,-.015),(1.030,.050,.050,-.025)),4),40)
 p=[body,head];p=gaze(p,head,'larvitar')
 p.append(crease_normals(paddle('Horn / single long swept Larvitar crown',LIGHT,[(0.,.970,-.020),(0.,1.110,-.035),(0.,1.250,-.075)],[.075,.052,.009],[.061,.040,.007],segments=20,steps=4),28.))
 p.append(larva_paint('Belly / fitted red Larvitar diamond',RED,[(-.085,.310),(0.,.455),(.085,.310),(0.,.165)],body,.006))
 p.append(larva_paint('Lip / fitted small closed Larvitar mouth',INK,[(-.050,.687),(0.,.675),(.050,.687),(.047,.692),(0.,.682),(-.047,.692)],head,.0025))
 for side in (-1,1):
  p.append(larva_paint('Cheek tear / long fitted black Larvitar eye stripe '+str(side),INK,[(side*.120,.802),(side*.177,.807),(side*.154,.726)],head,.0035))
  p.append(larva_paint('Nostril / small supported Larvitar nose '+str(side),INK,oval(side*.036,.734,.005,.003,16),head,.003))
  p.append(larva_paint('Body vent / fitted triangular black Larvitar side slit '+str(side),INK,[(side*.141,.375),(side*.181,.353),(side*.168,.3017),(side*.131,.323)],body,.0035))
  p.append(reverse_front(larva_paint('Back vent / fitted black Larvitar rear flank diamond '+str(side),INK,[(side*.125,.566),(side*.168,.542),(side*.146,.492),(side*.107,.517)],reverse_front(body),.0035)))
  points,radii=arm_path(side);p.append(curved_tube('Arm / short complete Larvitar forelimb '+str(side),GREEN,points,radii,24));x,y,z=points[-1]
  p.append(ellipsoid('Thigh / rooted rounded Larvitar haunch '+str(side),GREEN,(side*.150,.125,.005),(.117,.122,.120),28,18))
  p.append(translate(rounded_loft('Foot / planted broad Larvitar pad '+str(side),GREEN,[(0.,.072,.085,.065),(.025,.091,.108,.075),(.060,.082,.092,.064),(.086,.051,.054,.015)],28,.70),side*.150,0,0))
  for j,dx in enumerate((-.043,0.,.043)):p.append(curved_tube('Toe / three blunt green Larvitar toes '+str(side)+' '+str(j),GREEN,[(side*.150+dx*.8,.031,.152),(side*.150+dx,.018,.210)],[.020,.006],14))
 p.append(paddle('Tail / broad curved Larvitar tail stem',GREEN,LARVA_TAIL,LARVA_WIDTHS,LARVA_DEPTHS,segments=24,steps=4))
 p.append(tail_hub())
 for j in range(6):p.append(tail_tooth(j))
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
