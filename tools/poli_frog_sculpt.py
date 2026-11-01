#!/usr/bin/env python3
"""Original spiral-bellied round frogs and a distinct green crested toad."""
import math
from sculpt_geometry import ellipsoid,crease_normals,fitted_triangle_contour,tube
from bovine_sculpt import oval,curved_tube
from cubone_sculpt import skin_shading
from goldfish_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('poliwhirl','poliwrath','politoed')
HEIGHTS={'poliwhirl':1.10,'poliwrath':1.20,'politoed':1.20}
BLUE=(.35,.53,.74);WRATH=(.29,.45,.66);GREEN=(.40,.65,.27)
CREAM=(.98,.98,.94);YELLOW=(.91,.86,.49);INK=(.055,.065,.095);PINK=(.86,.51,.54)

def arm_points(species,side):
 toad=species=='politoed';wrath=species=='poliwrath'
 return ((side*(.145 if toad else .20),.56 if toad else .63 if wrath else .57,0.),(side*(.33 if toad else .44 if wrath else .40),.41 if toad else .61 if wrath else .48,.04),(side*(.35 if toad else .50 if wrath else .44),.26 if toad else .53 if wrath else .355,.17 if toad else .19 if wrath else .16))

def arm_curve(species,side):
 root,elbow,wrist=arm_points(species,side)
 if species=='poliwrath':return (root,(side*.32,.64,.015),elbow,(side*.49,.56,.12),wrist),(.095,.095,.085,.080,.075)
 return (root,elbow,wrist),(.053,.050,.043) if species=='politoed' else (.065,.060,.060)

def paint(name,color,xy,parent,th=.004):
 return skin_shading(crease_normals(fitted_triangle_contour(name,color,xy,parent,th,precision=7),40.),parent)
def spiral(parent,color,center,radius):
 skin=surface(parent);points=[]
 for i in range(161):
  t=i/160;r=.014+(radius-.014)*t;a=1.4-math.tau*2.4*t;x=center[0]+r*math.cos(a);y=center[1]+r*math.sin(a)
  points.append((x,y,skin(y,x)+.003))
 return tube('Spiral / continuous rooted belly coil',color,points,[.0065]*len(points),12)
def raw_parts(species):
 if species not in SPECIES:raise ValueError('unknown spiral frog')
 toad=species=='politoed';wrath=species=='poliwrath';coat=GREEN if toad else WRATH if wrath else BLUE
 body=ellipsoid('Body / '+('pear-shaped green frog torso' if toad else 'heavy blue combat frog orb' if wrath else 'round blue spiral frog orb'),coat,(0.,.43 if toad else .52 if wrath else .48,0.),(.255,.30,.23) if toad else (.36,.38,.285) if wrath else (.31,.35,.255),40,24)
 p=[body]
 by,rx,ry=(.415,.187,.216) if toad else (.49,.282,.292) if wrath else (.452,.247,.270)
 belly=paint('Belly / fitted '+('yellow toad spiral field' if toad else 'ivory spiral field'),YELLOW if toad else CREAM,oval(0.,by,rx,ry,64),body,.008);p += [belly,spiral(belly,GREEN if toad else INK,(0.,by),.162 if toad else .219 if wrath else .195)]
 head=ellipsoid('Head / distinct broad green toad face',GREEN,(0.,.79,.04),(.24,.19,.21),36,22) if toad else body
 if toad:p.append(head)
 for side in (-1,1):
  ex,ey,ez,er=(.16,.94,.09,(.095,.105,.105)) if toad else (.195,.87,.015,(.12,.12,.126)) if wrath else (.175,.81,.015,(.11,.115,.117))
  bulb=ellipsoid('Eye socket / rooted rounded frog turret '+str(side),coat,(side*ex,ey,ez),er,28,20);p.append(bulb)
  if wrath:xy=[(side*x,y) for x,y in ((.153,.919),(.235,.946),(.243,.871),(.211,.833),(.166,.843))]
  else:xy=oval(side*ex,ey+.015,.044 if toad else .045,.060,32)
  white=paint('Eye / fitted ivory frog gaze '+str(side),CREAM,xy,bulb,.005);p.append(white)
  pupil=paint('Pupil / fitted black vertical frog gaze '+str(side),INK,oval(side*(ex+.002),ey+.015,.018,.039,28),white,.003);p.append(pupil)
  if wrath:
   p.append(paint('Brow / raised fitted severe blue lid '+str(side),coat,[(side*x,y) for x,y in ((.126,.929),(.248,.970),(.252,.945),(.143,.908))],bulb,.009))
  if toad:p.append(paint('Cheek / fitted rosy toad disc '+str(side),PINK,oval(side*.163,.795,.033,.039,32),head,.004))
  shoulder,elbow,wrist=arm_points(species,side)
  arm_path,arm_radii=arm_curve(species,side)
  p.append(curved_tube('Arm / bent continuous frog arm '+str(side),coat,arm_path,arm_radii,20))
  palm=ellipsoid('Hand / '+('green webbed palm ' if toad else 'ivory padded boxing glove ')+str(side),coat if toad else CREAM,(wrist[0],.235 if toad else .50 if wrath else .305,.17 if not wrath else .20),(.064,.075,.049) if toad else (.135,.12,.128) if wrath else (.110,.10,.105),28,20);p.append(palm)
  if toad:
   for j,dx in enumerate((-.040,0.,.040)):
    p.append(curved_tube(f'Finger / rooted rounded green digit {side} {j}',GREEN,[(wrist[0]+dx*.6,.225,.19),(wrist[0]+dx,.174,.222),(wrist[0]+dx*1.10,.140,.227)],[.024,.022,.012],16))
  else:
   p.append(ellipsoid('Thumb / rooted ivory glove thumb '+str(side),CREAM,(wrist[0]-side*.075,.461 if wrath else .266,.274 if wrath else .244),(.050,.052,.050),22,16))
   for j,dx in enumerate((-.037,.037)):
    x=wrist[0]+dx;lift=.195 if wrath else 0.
    p.append(paint(f'Glove crease / fitted rounded knuckle seam {side} {j}',(.54,.58,.61),[(x-.003,.329+lift),(x+.003,.329+lift),(x+.004,.300+lift),(x-.003,.295+lift)],palm,.002))
  hx=.18 if toad else .215 if wrath else .18
  thigh=ellipsoid('Haunch / rooted frog hindquarter '+str(side),coat,(side*hx,.225 if toad else .255,-.018),(.112,.155,.121) if toad or wrath else (.09,.13,.10),28,20);p.append(thigh)
  p.append(curved_tube('Leg / supported frog ankle '+str(side),coat,[(side*hx,.23 if toad else .25,0.),(side*(.235 if wrath else .20),.13,.025),(side*(.235 if wrath else .20),.065,.105)],[.07,.06,.05] if not wrath else [.09,.08,.065],20))
  p.append(ellipsoid('Foot / broad planted frog pad '+str(side),coat,(side*(.235 if wrath else .20),.045,.105),(.11,.045,.12) if toad else (.11,.045,.17) if wrath else (.085,.045,.155),32,18))
  if toad:
   for j,dx in enumerate((-.066,0.,.066)):
    p.append(ellipsoid(f'Toe / rooted rounded green web digit {side} {j}',GREEN,(side*.20+dx,.026,.210+(.025 if j==1 else 0.)),(.035,.026,.064),20,14))
 if toad:
  xy=[(-.102,.798),(-.055,.780),(0.,.777),(.055,.780),(.102,.798),(.064,.773),(0.,.769),(-.064,.773)]
  p.append(paint('Mouth / fitted curved toad smile',INK,xy,head,.004))
  p.append(curved_tube('Crest / continuous rooted green head curl',GREEN,[(0.,.93,-.02),(0.,1.10,-.02),(.035,1.205,0.),(.125,1.225,.025),(.18,1.175,.045),(.15,1.105,.052),(.095,1.108,.055),(.085,1.15,.055)],[.024,.022,.021,.020,.019,.018,.016,.010],18))
 return p

def sculpture(species,normalize=True):
 p=raw_parts(species)
 if normalize:
  lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
  for q in p:
   q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
