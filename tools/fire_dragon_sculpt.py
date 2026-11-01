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
 return [(0.,.34,-.13),(.025,.23,-.48),(.08,.20,-.86),(.17,.23,-1.24),(.23,.36,-1.55)],[.135,.125,.090,.050,.020]

def arm_path(species,side):
 return ([(side*.11,.74,.0),(side*.34,.64,.08),(side*.39,.80,.21)],[.070,.065,.045]) if species=='charizard' else ([(side*.13,.57,.02),(side*.23,.48,.075),(side*.29,.41,.15)],[.055,.048,.035])

def fire_path(species):
 return ((.36,.57,-.78),(.42,.94,-.81),.18,.044) if species=='charmeleon' else ((.23,.35,-1.55),(.25,.80,-1.60),.16,.065)

def inset_wing(outline,distance):
 # Offset actual perimeter edges. Radial shrinking can cross a concave scallop.
 sign=1. if sum(a[0]*b[1]-b[0]*a[1] for a,b in zip(outline,outline[1:]+outline[:1]))>0. else -1.
 result=[]
 for i,p in enumerate(outline):
  prev=outline[i-1];nxt=outline[(i+1)%len(outline)];edges=((p[0]-prev[0],p[1]-prev[1]),(nxt[0]-p[0],nxt[1]-p[1]));normals=[(-sign*y/math.hypot(x,y),sign*x/math.hypot(x,y)) for x,y in edges];a,b=normals;den=1.+sum(x*y for x,y in zip(a,b));result.append(tuple(p[k]+distance*(a[k]+b[k])/den for k in range(2)))
 return result

def fitted_top(name,color,outline,parent,thickness):
 top=dict(parent)
 for field in ('positions','normals'):top[field]=[v for x,y,z in zip(parent[field][::3],parent[field][1::3],parent[field][2::3]) for v in (x,-z,y)]
 mark=fitted(name,color,[(x,-z) for x,z in outline],top,thickness)
 for field in ('positions','normals'):mark[field]=[v for x,y,z in zip(mark[field][::3],mark[field][1::3],mark[field][2::3]) for v in (x,z,-y)]
 return mark

def anatomy(species):
 adult=species=='charizard';coat=(232/255,128/255,0.) if adult else RED;cream=(232/255,232/255,136/255) if adult else CREAM;teal=(0.,100/255,128/255) if adult else TEAL
 profiles=((.13,.08,.09,-.025),(.25,.25,.215,-.025),(.44,.31,.265,-.015),(.63,.27,.23,0.),(.78,.18,.16,.045),(.94,.105,.10,.115),(1.08,.085,.09,.20)) if adult else ((.12,.05,.065,-.02),(.22,.18,.145,-.02),(.36,.21,.17,-.015),(.51,.17,.14,0.),(.64,.10,.085,.02),(.75,.07,.07,.035),(.86,.07,.07,.04))
 body=loft('Body / continuous tapered fire dragon torso and neck',coat,curved_profiles(profiles,3),32)
 hp=[(.09,1.10,.08,.09),(.18,1.15,.15,.12),(.29,1.15,.155,.105),(.41,1.11,.125,.070),(.49,1.105,.085,.045),(.53,1.105,.020,.025)] if adult else [(-.04,.88,.05,.075),(.06,.92,.145,.12),(.17,.94,.156,.115),(.29,.91,.13,.065),(.37,.89,.085,.04),(.41,.895,.017,.025)]
 head=longitudinal('Head / angular brow and projecting fire lizard muzzle',coat,hp,28,3)
 rows=[(.17,.075,1.005,1.075),(.29,.115,.940,.990),(.40,.110,.950,.995),(.50,.065,.990,1.020),(.525,.012,1.015,1.035)] if adult else [(.055,.075,.79,.87),(.16,.11,.72,.775),(.28,.10,.72,.765),(.37,.06,.76,.79),(.395,.012,.80,.815)]
 jaw=bill('Jaw / open rooted lower fire lizard mandible',coat,rows);parts=[body,head,jaw]
 if adult:
  parts.append(ellipsoid('Throat / recessed dark Charizard mouth backing',INK,(0.,1.03,.24),(.095,.065,.025),24,14))
  parts.append(fitted_top('Oral floor / fitted dark Charizard jaw lining',INK,[(-.08,.25),(.08,.25),(.085,.31),(.065,.44),(0.,.48),(-.065,.44),(-.085,.31)],jaw,.003))
 belly=[(-.07,.16),(-.20,.24),(-.24,.40),(-.22,.58),(-.14,.71),(-.08,.76),(0.,.78),(.08,.76),(.14,.71),(.22,.58),(.24,.40),(.20,.24),(.07,.16)] if adult else [(-.045,.14),(-.12,.20),(-.15,.32),(-.12,.48),(-.075,.62),(-.035,.72),(0.,.76),(.035,.72),(.075,.62),(.12,.48),(.15,.32),(.12,.20),(.045,.14)]
 parts.append(fitted('Belly / broad supported cream fire lizard bib',cream,belly,body,.009))
 # Recessed oral lining and tongue are seated in their actual lower-jaw cap.
 upper_y=1.00 if adult else .85
 for side in (-1,1):
  x=side*(.10 if adult else .080);y=1.17 if adult else .972
  outline=[(x+side*dx,y+dy) for dx,dy in ((-.045,-.012),(.035,.010),(.025,.045),(-.027,.031))]
  socket=fitted('Eye socket / stern angular fire dragon gaze '+str(side),INK,outline,head,.005);parts.append(socket)
  cx=sum(p[0] for p in outline)/4;cy=sum(p[1] for p in outline)/4
  white=fitted('Eye white / supported ivory fire dragon glare '+str(side),WHITE,[(cx+(a-cx)*.80,cy+(b-cy)*.68) for a,b in outline],socket,.0025);parts.append(white)
  iris=fitted('Iris / narrow teal fire dragon iris '+str(side),TEAL,oval(cx,cy,.008,.009,20),white,.0018);parts.append(iris)
  pupil=fitted('Pupil / dark upright fire dragon slit '+str(side),INK,oval(cx+side*.001,cy,.0035,.007,16),iris,.0015);parts.append(pupil)
  parts.append(fitted('Glint / small ivory fire dragon reflection '+str(side),WHITE,oval(cx,cy+.003,.0012,.0025,12),pupil,.0012))
  nose_y=1.13 if adult else .923
  parts.append(fitted('Nostril / supported dark fire dragon nose '+str(side),INK,oval(side*.040,nose_y,.006,.003,16),head,.002))
  z=.43 if adult else .29;y=1.055 if adult else .852
  parts.append(curved_tube('Fang / rooted ivory upper fire dragon tooth '+str(side),WHITE,[(side*.078,y+.014,z),(side*.080,y-.006,z+.010),(side*.077,y-.035,z+.018)],[.014,.010,.0015],14))
  if adult:
   parts.append(curved_tube('Horn / swept twin orange Charizard crown '+str(side),coat,[(side*.09,1.21,.18),(side*.13,1.33,.02),(side*.17,1.365,-.11)],[.055,.035,.0015],20))
  path,radii=arm_path(species,side);parts.append(curved_tube('Arm / tapered rooted fire lizard forelimb '+str(side),coat,path,radii,20))
  hx,hy,hz=(side*.39,.83,.25) if adult else (side*.30,.395,.18)
  parts.append(ellipsoid('Hand / rounded three-digit fire lizard palm '+str(side),coat,(hx,hy,hz),(.060,.064,.070) if adult else (.048,.048,.056),24,14))
  for j,dx in enumerate((-.033,0.,.033)):
   x=hx+dx;d=.052 if adult else .040
   points=([(x,hy+.015,hz+.03),(x+side*.006,hy+.050,hz+.05),(x+side*.008,hy+.080,hz+.04)] if adult else [(x,hy-.018,hz+.035),(x+side*.006,hy-.040,hz+d),(x+side*.008,hy-.055,hz+d+.012)])
   parts.append(curved_tube('Finger / complete orange-red fire lizard digit '+str(side)+' '+str(j),coat,points,[.020,.016,.010],14))
   claws=[points[-1],(x+side*.009,hy+.110,hz+.070),(x+side*.010,hy+.100,hz+.095)] if adult else [points[-1],(x+side*.009,hy-.073,hz+d+.036),(x+side*.010,hy-.078,hz+d+.055)]
   parts.append(curved_tube('Hand claw / ivory fire lizard finger point '+str(side)+' '+str(j),WHITE,claws,[.012,.008,.0015],12))
  cx=.20 if adult else .14;cy=.23 if adult else .17
  parts.append(ellipsoid('Haunch / strong rooted fire lizard thigh '+str(side),coat,(side*cx,cy,.03),(.145,.205,.15) if adult else (.10,.145,.12),26,18))
  if adult:parts.append(curved_tube('Shin / short upright Charizard ankle '+str(side),coat,[(side*.22,.18,.035),(side*.225,.095,.040),(side*.225,.055,.043)],[.065,.055,.045],20))
  fx,fy,fz=(side*.22,.045,.08) if adult else (side*.15,.032,.12)
  parts.append(ellipsoid('Foot / planted long fire lizard pad '+str(side),coat,(fx,fy,fz),(.11,.045,.18) if adult else (.085,.032,.145),28,16))
  for j,dx in enumerate((-.060,0.,.060) if adult else (-.046,0.,.046)):
   z=.215 if adult else .22
   parts.append(curved_tube('Foot claw / ivory planted fire lizard toe '+str(side)+' '+str(j),WHITE,[(fx+dx,fy,z),(fx+dx*1.06,.022,z+.055),(fx+dx*1.10,.012,z+.073)],[.025,.015,.0015] if adult else [.020,.012,.0015],14))
 if not adult:parts.append(curved_tube('Horn / single swept red Charmeleon crest',coat,[(0.,.990,.015),(0.,1.095,-.060),(0.,1.15,-.15)],[.064,.038,.0015],22))
 # A shallow closed tongue seats on the jaw while leaving the gape unfilled.
 parts.append(ellipsoid('Tongue / rooted pink lower fire dragon mouth',PINK,(0.,.987,.36) if adult else (0.,.762,.245),(.067,.012,.072) if adult else (.060,.012,.064),24,12))
 tp,tr=tail_path(species)
 if adult:
  from dragon_family_sculpt import painted_tube
  tail,underside=painted_tube('Tail / continuous curved tapering fire lizard tail',coat,tp,tr,(0.,-1.,0.),segments=28,steps=4);underside['base_color']=linear(cream);parts.extend((tail,underside))
 else:parts.append(curved_tube('Tail / continuous curved tapering fire lizard tail',coat,tp,tr,28))
 root,tip,width,depth=fire_path(species)
 for label,mesh in flame(root,tip,width,depth,-18.,-.10).items():
  color=(.93,.25,.10) if label=='ember silhouette' else ((1.,.59,.10) if 'orange' in label else (1.,.90,.34));parts.append(dict(mesh,part='Flame / coherent '+label,base_color=linear(color)))
 if adult:
  outline=contour([((.08,.74),(.23,.94),(.34,1.15),(.43,1.41)),((.43,1.41),(.61,1.24),(.89,1.15),(.97,.96)),((.97,.96),(.98,.86),(.97,.77),(.94,.69)),((.94,.69),(.85,.82),(.71,.83),(.63,.65)),((.63,.65),(.56,.81),(.41,.80),(.35,.65)),((.35,.65),(.25,.77),(.14,.74),(.08,.74))],8)
  def depth(x,y):return -.105-.06*abs(x)+.025*(y-.70)**2
  for side in (-1,1):
   case=membrane('Wing back / closed orange Charizard sail '+str(side),coat,outline,side,root_z=-.105,thickness=.018);parts.append(case)
   inset=inset_wing(outline,.012);inner=[]
   # Keep the shoulder web orange. A teal root otherwise reads as a stray
   # triangle below the jaw when the far wing is seen through a quarter view.
   for a,b in zip(inset,inset[1:]+inset[:1]):
    if a[0]>=.22:inner.append(a)
    if (a[0]<.22)!=(b[0]<.22):
     t=(.22-a[0])/(b[0]-a[0]);inner.append((.22,a[1]+t*(b[1]-a[1])))
   inner=[(side*x,y) for x,y in inner]
   parts.append(fitted('Wing / broad scalloped teal Charizard sail '+str(side),teal,inner,case,.004))
   leading=[(.095,.75),(.25,1.0),(.43,1.41),(.72,1.24),(.97,.96)]
   parts.append(curved_tube('Wing arm / continuous orange Charizard leading spar '+str(side),coat,[(side*x,y,depth(x,y)+.022) for x,y in leading],[.065,.047,.035,.025,.002],20))
   for j,(x,y) in enumerate(((.94,.69),(.63,.65),(.35,.65))):
    points=[(.43,1.41),((.43+x)/2,(1.41+y)/2+.018),(x,y)]
    parts.append(curved_tube('Wing rib / rooted orange Charizard finger '+str(side)+' '+str(j),coat,[(side*a,b,depth(a,b)+.022) for a,b in points],[.025,.017,.002],14))
   parts.append(curved_tube('Wing claw / ivory hooked Charizard wrist '+str(side),WHITE,[(side*.43,1.405,depth(.43,1.405)+.02),(side*.40,1.46,depth(.40,1.46)+.035),(side*.43,1.48,depth(.43,1.48)+.05)],[.025,.016,.0015],14))
  for q in parts:
   if not q['part'].startswith(('Wing /','Wing back /','Wing arm /','Wing rib /','Wing claw /')):continue
   side=int(q['part'].split()[-2] if q['part'].startswith('Wing rib /') else q['part'].split()[-1]);a=side*math.radians(30.);c=math.cos(a);s=math.sin(a)
   for field in ('positions','normals'):
    out=[]
    for i in range(0,len(q[field]),3):
     x,y,z=q[field][i:i+3]
     if field=='positions':x-=side*.095;z+=.105
     out.extend((c*x+s*z+(side*.095 if field=='positions' else 0.),y,-s*x+c*z+(-.105 if field=='positions' else 0.)))
    q[field]=out
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
