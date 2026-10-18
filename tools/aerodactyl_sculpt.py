#!/usr/bin/env python3
"""Original fossil flyer with an open toothed skull and supported scalloped sails."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,loft,curved_profiles,ellipsoid,fitted_contour,fitted_patch,crease_normals
from rhyhorn_sculpt import longitudinal
from bovine_sculpt import curved_tube,oval
from wing_insect_sculpt import contour,membrane
from abra_sculpt import surface,polygon_caps
from chikorita_sculpt import cross,unit
from skin_glb import COORDINATES
HEIGHT=1.30
GRAY=(.64,.64,.71);LIGHT=(.72,.72,.79);PURPLE=(.56,.39,.68);INK=(.10,.075,.12);MOUTH=(.32,.16,.23);TONGUE=(.73,.42,.46);IVORY=(.97,.96,.90)

def turn(p,up):
 q=dict(p)
 for key in ('positions','normals'):
  q[key]=[v for i in range(0,len(p[key]),3) for v in ((p[key][i],-p[key][i+2],p[key][i+1]) if up else (p[key][i],p[key][i+2],-p[key][i+1]))]
 return q

def mouth_patch(name,color,parent,outline,up,thickness):
 skin=surface(turn(parent,up));xy=[(x,-z if up else z) for x,z in outline]
 return turn(crease_normals(fitted_contour(name,color,xy,skin,thickness,subdivisions=3),35),not up)

def arrow():
 # Build a closed dart in its own plane, then orient it along the actual tail.
 xy=[(-.11,.07),(0,.26),(.11,.07),(.045,.09),(0,-.04),(-.045,.09)]
 v=[(x,y,z) for z in (-.017,.017) for x,y in xy];n=len(xy);caps=polygon_caps(v[n:]);caps=[tuple(i+n for i in f) for f in caps]
 ids=list(range(n));ids.reverse()
 fs=caps+[tuple(i-n for i in reversed(f)) for f in caps]+[(a,b,b+n,a+n) for a,b in zip(ids,ids[1:]+ids[:1])]
 q=crease_normals(part('Tail tip / thick pointed fossil arrowhead',GRAY,v,fs),40)
 ey=unit((.18,.14,-.12));ex=unit((-.14,.18,0));ez=cross(ex,ey);origin=(.50,.57,-.80)
 for key in ('positions','normals'):
  q[key]=[round(sum(q[key][i+j]*basis[k] for j,basis in enumerate((ex,ey,ez)))+(origin[k] if key=='positions' else 0),7) for i in range(0,len(q[key]),3) for k in range(3)]
 return q

def sculpture(normalize=True):
 body=loft('Body / continuous lean fossil chest and neck',GRAY,curved_profiles(((.20,.025,.030,-.05),(.29,.145,.128,-.055),(.44,.185,.16,-.06),(.59,.175,.151,-.065),(.71,.122,.111,-.07),(.86,.088,.085,-.035),(1.015,.095,.080,.010)),4),40)
 head=longitudinal('Skull / swept narrow dinosaur wedge',GRAY,[(-.09,1.00,.07,.10),(.025,1.025,.174,.135),(.16,1.045,.158,.098),(.34,1.075,.129,.058),(.52,1.105,.084,.07),(.585,1.10,.013,.037)])
 jaw=longitudinal('Jaw / lowered long fossil mandible',GRAY,[(.035,.92,.095,.035),(.13,.85,.137,.043),(.30,.72,.125,.043),(.49,.65,.088,.050),(.56,.66,.013,.034)])
 p=[body,head,jaw];hs=surface(head)
 p.append(mouth_patch('Mouth ceiling / supported dark upper palate',MOUTH,head,[(-.095,.11),(-.10,.32),(-.058,.52),(.058,.52),(.10,.32),(.095,.11)],False,.004))
 p.append(mouth_patch('Mouth floor / supported dark lower lining',MOUTH,jaw,[(-.091,.12),(-.09,.29),(-.059,.47),(.059,.47),(.09,.29),(.091,.12)],True,.004))
 p.append(mouth_patch('Tongue / fitted long rose tongue',TONGUE,jaw,[(-.054,.17),(-.058,.30),(-.04,.46),(.04,.46),(.058,.30),(.054,.17)],True,.008))
 for sign in (-1,1):
  p.append(ellipsoid('Jaw hinge / rooted rear cheek '+str(sign),GRAY,(sign*.12,.938,.07),(.065,.105,.066),32,20))
  p.append(curved_tube('Skull horn / long swept backward fossil crest '+str(sign),GRAY,[(sign*.113,1.117,.015),(sign*.18,1.19,-.065),(sign*.247,1.235,-.165)],[.046,.030,.0015],24))
  x=sign*.127;y=1.072;xy=[(x+sign*dx,y+dy) for dx,dy in ((-.023,.018),(.023,.017),(.025,-.010),(-.022,-.020))]
  socket=crease_normals(fitted_contour('Eye socket / supported stern dark wedge '+str(sign),INK,xy,hs,.007,subdivisions=4),35);p.append(socket)
  cx=sum(a for a,b in xy)/4;cy=sum(b for a,b in xy)/4
  white=crease_normals(fitted_contour('Eye white / fitted narrow ivory wedge '+str(sign),IVORY,[(cx+(a-cx)*.82,cy+(b-cy)*.73) for a,b in xy],surface(socket),.003,subdivisions=4),35);p.append(white)
  p.append(crease_normals(fitted_patch('Pupil / fitted dark fossil slit '+str(sign),INK,oval(cx-sign*.002,cy,.004,.008,24),surface(white),.002),35))
  p.append(crease_normals(fitted_patch('Nostril / fitted dark nasal slit '+str(sign),INK,oval(sign*.041,1.125,.008,.012,24),hs,.003),35))
  for upper,parent in ((True,head),(False,jaw)):
   skin=surface(turn(parent,not upper))
   for j,(z,x) in enumerate(((.24,.092),(.34,.087),(.44,.067))):
    yy=skin(-z if not upper else z,sign*x)*(1 if not upper else -1);direction=-1 if upper else 1
    p.append(curved_tube('Tooth / rooted ivory '+('upper' if upper else 'lower')+' '+str(sign)+' '+str(j),IVORY,[(sign*x,yy-direction*.008,z),(sign*x*.95,yy+direction*.026,z+.004),(sign*x*.88,yy+direction*(.075 if j==0 else .057),z+.014)],[.018,.013,.0015],18))
  # Wing skin and leading arm share depth at the rooted wrist.
  def depth(x,y):return -.09-.06*abs(x)+.025*(y-.70)**2
  outline=contour([((.11,.69),(.26,.72),(.42,.83),(.52,.99)),((.52,.99),(.78,1.10),(1.13,1.08),(1.28,.97)),((1.28,.97),(1.01,.93),(.86,.73),(.84,.51)),((.84,.51),(.65,.68),(.52,.66),(.35,.40)),((.35,.40),(.23,.50),(.15,.47),(.11,.43)),((.11,.43),(.09,.51),(.10,.62),(.11,.69))],10)
  sail=membrane('Wing / broad scalloped violet fossil membrane '+str(sign),PURPLE,outline,sign,root_z=-.09,thickness=.013);p.append(sail)
  leading=[(.105,.685),(.30,.77),(.52,.99),( .91,1.066),(1.28,.97)]
  p.append(curved_tube('Wing arm / continuous bent leading fossil arm '+str(sign),GRAY,[(sign*x,y,depth(x,y)+.006) for x,y in leading],[.050,.038,.027,.022,.003],24))
  for j,tip in enumerate(((.84,.51),(.35,.40))):
   x,y=tip;pts=[(.52,.99),((.52+x)*.5,(.99+y)*.5+.015),tip]
   p.append(curved_tube('Wing finger / seated branching fossil spar '+str(sign)+' '+str(j),GRAY,[(sign*a,b,depth(a,b)+.006) for a,b in pts],[.020,.014,.003],18))
  for j,dx in enumerate((-.033,.009,.045)):
   pts=[(sign*(.52+dx),.990,depth(.52+dx,.99)+.02),(sign*(.525+dx),1.065,depth(.525+dx,1.065)+.031),(sign*(.56+dx),1.080,depth(.56+dx,1.08)+.046)]
   p.append(curved_tube('Wing claw / hooked wrist digit '+str(sign)+' '+str(j),GRAY,pts,[.015,.012,.002],16))
  leg=curved_tube('Leg / bent fossil hind limb '+str(sign),GRAY,[(sign*.11,.31,-.040),(sign*.19,.19,.003),(sign*.18,.072,.105)],[.074,.050,.031],28);p.append(leg)
  foot=ellipsoid('Foot / grounded narrow fossil pad '+str(sign),GRAY,(sign*.18,.045,.115),(.063,.045,.093),32,20);p.append(foot)
  for j,dx in enumerate((-.044,0,.044)):
   p.append(curved_tube('Foot claw / curved pointed ivory talon '+str(sign)+' '+str(j),IVORY,[(sign*.18+dx,.047,.151),(sign*.18+dx,.035,.210),(sign*.18+dx,.016,.246)],[.019,.014,.0015],18))
 p.append(curved_tube('Tail / continuous slender swept fossil tail',GRAY,[(0,.29,-.13),(.04,.26,-.40),(.28,.38,-.68),(.50,.57,-.80)],[.075,.062,.040,.023],32));p.append(arrow())
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
 for q in p:
  if normalize:q['positions']=[round((v-(low if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'aerodactyl','version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,required=True);args=a.parse_args();args.out.mkdir(parents=True,exist_ok=True)
 from aerodactyl_glb import export_aerodactyl
 blob=export_aerodactyl();(args.out/'aerodactyl.glb').write_bytes(blob);print('aerodactyl',len(blob),'bytes',flush=True)
if __name__=='__main__':main()
