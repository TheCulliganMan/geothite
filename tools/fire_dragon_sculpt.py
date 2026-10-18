#!/usr/bin/env python3
"""Original fire-lizard evolutions with shaped skulls, open jaws and rooted bat wings."""
import argparse,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals,fitted_triangle_contour,fitted_adaptive_contour
from bovine_sculpt import curved_tube,oval
from rhyhorn_sculpt import longitudinal
from murkrow_sculpt import bill
from cubone_sculpt import skin_shading
from wing_insect_sculpt import contour,membrane
from cyndaquil_sculpt import flame
from chikorita_sculpt import linear
from skin_glb import COORDINATES
SPECIES=('charmeleon','charizard');HEIGHTS={'charmeleon':1.20,'charizard':1.40}
RED=(.96,.25,.24);ORANGE=(.98,.49,.20);CREAM=(.99,.93,.73);WHITE=(.99,.98,.93);TEAL=(.17,.49,.56);INK=(.085,.075,.085);PINK=(.84,.43,.48)

def fitted(name,color,outline,parent,thickness=.003):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,outline,parent,thickness,precision=7),40),parent)

def tail_path(species):
 if species=='charmeleon':return [(0.,.23,-.14),(.04,.18,-.35),(.18,.23,-.58),(.32,.38,-.73),(.36,.58,-.78)],[.082,.069,.050,.033,.016]
 return [(0.,.30,-.19),(.07,.20,-.45),(.24,.26,-.74),(.40,.46,-.93),(.45,.68,-.97)],[.110,.091,.065,.043,.020]

def arm_path(species,side):
 return ([(side*.17,.74,.015),(side*.33,.60,.10),(side*.39,.53,.22)],[.070,.060,.044]) if species=='charizard' else ([(side*.13,.57,.02),(side*.23,.48,.075),(side*.29,.41,.15)],[.055,.048,.035])

def fire_path(species):
 return ((.36,.57,-.78),(.42,.94,-.81),.18,.044) if species=='charmeleon' else ((.45,.67,-.97),(.52,1.11,-1.02),.23,.055)

def anatomy(species):
 adult=species=='charizard';coat=ORANGE if adult else RED
 profiles=((.13,.07,.08,-.025),(.24,.24,.20,-.025),(.40,.28,.23,-.025),(.58,.245,.20,-.015),(.74,.16,.135,0.),(.91,.10,.095,.025),(1.05,.08,.085,.045)) if adult else ((.12,.05,.065,-.02),(.22,.18,.145,-.02),(.36,.21,.17,-.015),(.51,.17,.14,0.),(.64,.10,.085,.02),(.75,.07,.07,.035),(.86,.07,.07,.04))
 body=loft('Body / continuous tapered fire dragon torso and neck',coat,curved_profiles(profiles,3),32)
 hp=[(-.04,1.05,.07,.10),(.07,1.10,.155,.13),(.20,1.11,.166,.12),(.34,1.075,.14,.075),(.44,1.055,.09,.05),(.48,1.055,.02,.027)] if adult else [(-.04,.88,.05,.075),(.06,.92,.145,.12),(.17,.94,.156,.115),(.29,.91,.13,.065),(.37,.89,.085,.04),(.41,.895,.017,.025)]
 head=longitudinal('Head / angular brow and projecting fire lizard muzzle',coat,hp,28,3)
 rows=[(.055,.075,.94,1.025),(.17,.115,.86,.92),(.30,.11,.855,.91),(.415,.065,.90,.935),(.455,.012,.935,.95)] if adult else [(.055,.075,.79,.87),(.16,.11,.72,.775),(.28,.10,.72,.765),(.37,.06,.76,.79),(.395,.012,.80,.815)]
 jaw=bill('Jaw / open rooted lower fire lizard mandible',coat,rows);parts=[body,head,jaw]
 belly=[(-.06,.16),(-.17,.23),(-.205,.37),(-.17,.57),(-.10,.76),(-.055,.91),(0.,.96),(.055,.91),(.10,.76),(.17,.57),(.205,.37),(.17,.23),(.06,.16)] if adult else [(-.045,.14),(-.12,.20),(-.15,.32),(-.12,.48),(-.075,.62),(-.035,.72),(0.,.76),(.035,.72),(.075,.62),(.12,.48),(.15,.32),(.12,.20),(.045,.14)]
 parts.append(fitted('Belly / broad supported cream fire lizard bib',CREAM,belly,body,.009))
 # Recessed oral lining and tongue are seated in their actual lower-jaw cap.
 upper_y=1.00 if adult else .85
 for side in (-1,1):
  x=side*(.097 if adult else .080);y=1.12 if adult else .972
  outline=[(x+side*dx,y+dy) for dx,dy in ((-.045,-.012),(.035,.010),(.025,.045),(-.027,.031))]
  socket=fitted('Eye socket / stern angular fire dragon gaze '+str(side),INK,outline,head,.005);parts.append(socket)
  cx=sum(p[0] for p in outline)/4;cy=sum(p[1] for p in outline)/4
  white=fitted('Eye white / supported ivory fire dragon glare '+str(side),WHITE,[(cx+(a-cx)*.80,cy+(b-cy)*.68) for a,b in outline],socket,.0025);parts.append(white)
  iris=fitted('Iris / narrow teal fire dragon iris '+str(side),TEAL,oval(cx,cy,.008,.009,20),white,.0018);parts.append(iris)
  pupil=fitted('Pupil / dark upright fire dragon slit '+str(side),INK,oval(cx+side*.001,cy,.0035,.007,16),iris,.0015);parts.append(pupil)
  parts.append(fitted('Glint / small ivory fire dragon reflection '+str(side),WHITE,oval(cx,cy+.003,.0012,.0025,12),pupil,.0012))
  nose_y=1.087 if adult else .923
  parts.append(fitted('Nostril / supported dark fire dragon nose '+str(side),INK,oval(side*.040,nose_y,.006,.003,16),head,.002))
  z=.33 if adult else .29;y=1.005 if adult else .852
  parts.append(curved_tube('Fang / rooted ivory upper fire dragon tooth '+str(side),WHITE,[(side*.078,y+.014,z),(side*.080,y-.006,z+.010),(side*.077,y-.035,z+.018)],[.014,.010,.0015],14))
  if adult:
   parts.append(curved_tube('Horn / swept twin orange Charizard crown '+str(side),coat,[(side*.095,1.16,.045),(side*.13,1.32,-.035),(side*.15,1.40,-.12)],[.055,.035,.0015],20))
  path,radii=arm_path(species,side);parts.append(curved_tube('Arm / tapered rooted fire lizard forelimb '+str(side),coat,path,radii,20))
  hx,hy,hz=(side*.40,.51,.26) if adult else (side*.30,.395,.18)
  parts.append(ellipsoid('Hand / rounded three-digit fire lizard palm '+str(side),coat,(hx,hy,hz),(.060,.064,.070) if adult else (.048,.048,.056),24,14))
  for j,dx in enumerate((-.033,0.,.033)):
   x=hx+dx;d=.052 if adult else .040
   points=[(x,hy-.018,hz+.035),(x+side*.006,hy-.040,hz+d),(x+side*.008,hy-.055,hz+d+.012)]
   parts.append(curved_tube('Finger / complete orange-red fire lizard digit '+str(side)+' '+str(j),coat,points,[.020,.016,.010],14))
   parts.append(curved_tube('Hand claw / ivory fire lizard finger point '+str(side)+' '+str(j),WHITE,[points[-1],(x+side*.009,hy-.073,hz+d+.036),(x+side*.010,hy-.078,hz+d+.055)],[.012,.008,.0015],12))
  cx=.20 if adult else .14;cy=.23 if adult else .17
  parts.append(ellipsoid('Haunch / strong rooted fire lizard thigh '+str(side),coat,(side*cx,cy,.03),(.145,.205,.15) if adult else (.10,.145,.12),26,18))
  fx,fy,fz=(side*.22,.045,.14) if adult else (side*.15,.032,.12)
  parts.append(ellipsoid('Foot / planted long fire lizard pad '+str(side),coat,(fx,fy,fz),(.11,.045,.18) if adult else (.085,.032,.145),28,16))
  for j,dx in enumerate((-.060,0.,.060) if adult else (-.046,0.,.046)):
   z=.27 if adult else .22
   parts.append(curved_tube('Foot claw / ivory planted fire lizard toe '+str(side)+' '+str(j),WHITE,[(fx+dx,fy,z),(fx+dx*1.06,.022,z+.055),(fx+dx*1.10,.012,z+.073)],[.025,.015,.0015] if adult else [.020,.012,.0015],14))
 if not adult:parts.append(curved_tube('Horn / single swept red Charmeleon crest',coat,[(0.,.990,.015),(0.,1.095,-.060),(0.,1.15,-.15)],[.064,.038,.0015],22))
 # A shallow closed tongue seats on the jaw while leaving the gape unfilled.
 parts.append(ellipsoid('Tongue / rooted pink lower fire dragon mouth',PINK,(0.,.906,.265) if adult else (0.,.762,.245),(.067,.014,.072) if adult else (.060,.012,.064),24,12))
 tp,tr=tail_path(species);parts.append(curved_tube('Tail / continuous curved tapering fire lizard tail',coat,tp,tr,28))
 root,tip,width,depth=fire_path(species)
 for label,mesh in flame(root,tip,width,depth,-18.,-.10).items():
  color=(.93,.25,.10) if label=='ember silhouette' else ((1.,.59,.10) if 'orange' in label else (1.,.90,.34));parts.append(dict(mesh,part='Flame / coherent '+label,base_color=linear(color)))
 if adult:
  outline=contour([((.08,.68),(.27,.91),(.38,1.13),(.51,1.15)),((.51,1.15),(.76,1.25),(.93,1.26),(1.06,1.21)),((1.06,1.21),(.97,.99),(.97,.72),(.91,.49)),((.91,.49),(.78,.69),(.62,.68),(.59,.49)),((.59,.49),(.45,.68),(.31,.62),(.23,.48)),((.23,.48),(.16,.53),(.07,.60),(.08,.68))],8)
  def depth(x,y):return -.14-.06*abs(x)+.025*(y-.70)**2
  for side in (-1,1):
   parts.append(membrane('Wing / broad scalloped teal Charizard sail '+str(side),TEAL,outline,side,root_z=-.14,thickness=.012))
   leading=[(.095,.72),(.31,.99),(.51,1.15),(.81,1.25),(1.06,1.21)]
   parts.append(curved_tube('Wing arm / continuous orange Charizard leading spar '+str(side),coat,[(side*x,y,depth(x,y)+.006) for x,y in leading],[.065,.047,.035,.025,.002],20))
   for j,(x,y) in enumerate(((.91,.49),(.59,.49),(.23,.48))):
    points=[(.51,1.15),((.51+x)/2,(1.15+y)/2+.018),(x,y)]
    parts.append(curved_tube('Wing rib / rooted orange Charizard finger '+str(side)+' '+str(j),coat,[(side*a,b,depth(a,b)+.006) for a,b in points],[.025,.017,.002],14))
   parts.append(curved_tube('Wing claw / ivory hooked Charizard wrist '+str(side),WHITE,[(side*.50,1.16,depth(.50,1.16)+.02),(side*.48,1.25,depth(.48,1.25)+.035),(side*.52,1.27,depth(.52,1.27)+.05)],[.025,.016,.0015],14))
 return parts

def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError('unknown fire dragon evolution')
 parts=anatomy(species);lo=min(v for p in parts for v in p['positions'][1::3]);hi=max(v for p in parts for v in p['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for p in parts:
  if normalize:p['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(p['positions'])]
  p['normals']=[round(v,7) for v in p['normals']]
  for key in ('positions','normals'):p[key]=[0. if v==0 else v for v in p[key]]
 return dict(name=species,version=1,coordinate_system=COORDINATES,primitives=parts)

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from fire_dragon_glb import export_fire_dragon
 for s in SPECIES:(args.output/(s+'.glb')).write_bytes(export_fire_dragon(s))
if __name__=='__main__':main()
