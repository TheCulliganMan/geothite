#!/usr/bin/env python3
"""Original cratered gas bodies with a cheerful grin and strained joined faces."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,part,translate
from bovine_sculpt import curved_tube,oval
from cubone_sculpt import skin_shading
from raptor_bird_sculpt import paint
from moltres_sculpt import flowing
from skin_glb import COORDINATES
from goldfish_sculpt import surface
PURPLE=(.65,.58,.76);CREAM=(.94,.91,.74);WHITE=(.96,.97,.96)
INK=(.16,.12,.20);TONGUE=(.83,.48,.59)
GAS=(.86,.86,.68)
HEIGHTS={'koffing':1.00,'weezing':1.20}
VENTS=((0,1,.08),(-.88,.47,.10),(.88,.47,.10),(-.96,-.2,.20),(.96,-.2,.20),(-.70,-.71,.12),(.70,-.71,.12),(0,-1,.10),(-.55,.30,-.80),(.55,.30,-.80),(0,.55,-.84),(.40,-.60,-.70),(-.55,-.50,-.75))
def unit(v):
 n=math.sqrt(sum(x*x for x in v));return tuple(x/n for x in v)

def body(name,c,r,adult=False):
 p=ellipsoid(name,PURPLE,c,r,88,64);vertices=[];directions=[unit(v) for v in VENTS]
 for i in range(0,len(p['positions']),3):
  x,y,z=p['positions'][i:i+3];u=unit(((x-c[0])/r[0],(y-c[1])/r[1],(z-c[2])/r[2]));delta=0
  for j,d in enumerate(directions):
   q=(1-sum(a*b for a,b in zip(u,d)))/.022
   if q<5:delta+=(.105*math.exp(-q*.85)-.155*math.exp(-q*3.5))*(1+.10*math.sin(j*2.1))
  x=c[0]+(x-c[0])*(1+delta);y=c[1]+(y-c[1])*(1+delta);z=c[2]+(z-c[2])*(1+delta)
  # The actual front surface bends inward around the expression.
  cy=c[1]+r[1]*(.055 if adult else .075);q=((x-c[0])/(r[0]*.61))**2+((y-cy)/(r[1]*(.16 if adult else .17)))**2
  if z>c[2] and q<1:z-=r[2]*.28*(1-q)**2
  vertices.append((x,y,z))
 return part(name,PURPLE,vertices,[tuple(p['indices'][i:i+3]) for i in range(0,len(p['indices']),3)],True)

def fitted(name,color,xy,parent,offset=.004,sub=3):
 return skin_shading(paint(name,color,xy,parent,offset,sub),parent)

def billows(p,species):
 # Emitted vapor floats around the body. Its lobes form connected cloud
 # volumes; it is not an anatomical appendage attached by a rigid rod.
 groups=[(-.55,.88,-.12),(.60,.32,-.12)] if species=='koffing' else [(-.76,.88,-.16),(.77,1.10,-.13)]
 for j,c in enumerate(groups):
  p.append(ellipsoid(f'Gas cloud / floating pale {species} cloud core {j}',GAS,c,(.098,.103,.079),36,26))
  for k,(dx,dy,dz,rx,ry,rz) in enumerate(((-.060,.034,0,.081,.084,.075),(.039,.060,-.015,.082,.082,.074),(.065,-.038,0,.078,.080,.071),(-.025,-.061,.008,.080,.082,.073))):
   p.append(ellipsoid(f'Gas lobe / rooted rounded {species} cloud lobe {j} {k}',GAS,(c[0]+dx,c[1]+dy,c[2]+dz),(rx,ry,rz),32,24))

def face(p,parent,c,r,label,adult=False,small=False):
 x,y,z=c;rx,ry,rz=r
 def xy(points):return [(x+a*rx,y+b*ry) for a,b in points]
 for s in (-1,1):
  # Broad half-open whites are shaped markings following the real surface.
  outline=[(s*a,b) for a,b in (((.15,.41),(.27,.36),(.48,.29),(.56,.31),(.50,.14),(.36,.11),(.21,.17)) if adult else ((.17,.32),(.21,.45),(.35,.51),(.47,.46),(.55,.33),(.39,.32)))]
  eye=fitted(f'Eye / fitted half-open white {label} eye {s}',WHITE,xy(flowing(outline,3)),parent,.005,3);p.append(eye)
  ex=x+s*rx*.35;ey=y+ry*(.27 if adult else .43)
  pupil=fitted(f'Pupil / fitted tiny dark {label} gaze {s}',INK,oval(ex,ey,rx*.018,ry*.035,24),eye,.003,2);p.append(pupil)
 if adult:
  outline=[(-.59,-.12),(-.35,.055),(0,.13),(.35,.055),(.59,-.12),(.52,-.20),(.29,-.10),(0,-.055),(-.29,-.10),(-.52,-.20)]
 else:outline=[(-.61,.16),(-.40,.10),(0,.10),(.40,.10),(.61,.16),(.45,-.055),(0,-.13),(-.45,-.055)]
 mouth=fitted(f'Mouth / fitted broad recessed {label} '+('frown' if adult else 'grin'),INK,xy(flowing(outline,3)),parent,.007,3);p.append(mouth)
 if not adult:
  p.append(fitted(f'Tongue / fitted rosy lower {label} grin',TONGUE,xy(flowing([(-.39,-.02),(0,.00),(.39,-.02),(.26,-.085),(0,-.115),(-.26,-.085)],3)),mouth,.003,3))
 for s in (-1,1):
  # Solid tapered teeth grow from the actual lip surface rather than bending
  # white paint across a steep recess, which reads as tear trails.
  fx=x+s*rx*(.35 if adult and not small else .46)
  fy=y+ry*(-.13 if adult and not small else (.09 if not adult else -.11))
  fz=surface(parent)(fy,fx)
  dy=ry*(.25 if adult and not small else -.115)
  radius=rx*(.071 if adult and not small else .037)
  p.append(curved_tube(f'Fang / rooted white {label} canine {s}',WHITE,[(fx,fy,fz-.014),(fx,fy+dy*.55,fz+.018),(fx,fy+dy,fz+.028)],[radius,radius*.66,.0015],28))
 # The cream emblem is a skull-shaped ring, not a through-hole in the body.
 skull=fitted(f'Skull mark / fitted cream {label} skull',CREAM,xy(flowing([(-.22,-.43),(-.16,-.31),(0,-.28),(.16,-.31),(.22,-.43),(.16,-.54),(.07,-.57),(0,-.60),(-.07,-.57),(-.16,-.54)],3)),parent,.005,3);p.append(skull)
 p.append(fitted(f'Skull center / fitted purple {label} skull center',PURPLE,oval(x,y-ry*.415,rx*.115,ry*.060,32),skull,.003,3))
 if not small:
  for s in (-1,1):
   points=[(-.32,-.58),(-.37,-.60),(-.35,-.67),(-.28,-.68),(.28,-.88),(.35,-.87),(.36,-.81),(.30,-.79)]
   points=[(s*a,b) for a,b in points]
   p.append(fitted(f'Crossbone / fitted rounded cream {label} bone {s}',CREAM,xy(flowing(points,3)),parent,.006 if s==-1 else .009,3))

def anatomy(species):
 if species=='koffing':
  c=(0,.47,0);r=(.43,.41,.37);main=body('Body / soft cratered purple Koffing gas orb',c,r);p=[main];face(p,main,c,r,'Koffing');billows(p,species);return p
 if species!='weezing':raise ValueError(species)
 c=(-.22,.51,0);r=(.43,.42,.375);main=body('Body / large strained cratered Weezing gas orb',c,r,True)
 c2=(.40,.86,-.025);r2=(.27,.28,.25);small=body('Small body / upper cratered Weezing gas orb',c2,r2,True)
 p=[main,small,curved_tube('Main stalk / rooted purple Weezing joining neck',PURPLE,[(-.05,.70,-.045),(.12,.81,-.05),(.23,.84,-.05)],[.082,.072,.075],32)]
 p.append(ellipsoid('Rear node / rooted small purple Weezing rear gas node',PURPLE,(.27,.35,-.13),(.105,.11,.095),40,28))
 p.append(curved_tube('Rear stalk / rooted purple Weezing rear gas connector',PURPLE,[(.065,.35,-.13),(.18,.37,-.13),(.27,.35,-.13)],[.050,.043,.040],28))
 face(p,main,c,r,'large Weezing',True);face(p,small,c2,r2,'small Weezing',True,True);billows(p,species);return p

def sculpture(species):
 p=anatomy(species);lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[species]/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);a.add_argument('--species',choices=HEIGHTS,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/(args.species+'.mesh.json')).write_text(json.dumps(sculpture(args.species),separators=(',',':')))
if __name__=='__main__':main()
