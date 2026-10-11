#!/usr/bin/env python3
"""Original crowned sea dragon with a tapered bore, branched horns and coiled tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,tube,translate,fitted_patch,fitted_contour,crease_normals
from bovine_sculpt import oval,curved_tube,reverse_front,smooth
from abra_sculpt import surface
from pitcher_sculpt import funnel
from wing_insect_sculpt import contour
from seahorse_sculpt import fin
from skin_glb import COORDINATES
HEIGHT=1.07
BLUE=(.34,.67,.80);CREAM=(.98,.80,.43);IVORY=(.95,.96,.87);FOLD=(.62,.47,.27);SEAM=(.24,.50,.63);INK=(.08,.13,.16);WHITE=(.98,.98,.94);RUBY=(.82,.29,.36)

def sculpture(species='kingdra'):
 if species!='kingdra':raise ValueError(species)
 body=loft('Body / continuous curved dragon chest and slender neck',BLUE,curved_profiles(((.24,.010,.014,-.11),(.30,.150,.123,-.095),(.40,.198,.150,-.068),(.50,.165,.132,-.010),(.62,.091,.093,.036),(.73,.060,.064,.040),(.82,.065,.058,.030)),5),64)
 head=loft('Head / swept dragon brow and tapered lower jaw',BLUE,curved_profiles(((.762,.045,.060,.023),(.811,.141,.112,.030),(.870,.190,.150,.035),(.945,.178,.139,.022),(1.005,.112,.084,-.025),(1.025,.020,.022,-.04)),5),64);p=[body,head]
 # A broad initial bend flows into an inward Archimedean spiral; the cross
 # section stays smaller than the bend radius, avoiding the old folded coil.
 points,radii=smooth([(0,.335,-.105),(.020,.270,-.117),(.030,.210,-.120)],[.098,.090,.079],6)
 for k in range(1,97):
  t=k/96;a=math.pi+2.2*math.pi*t;r=.170-.135*t
  points.append((.20+r*math.cos(a),.21+r*math.sin(a),-.12));radii.append(.075*(1-t)**1.15+.004)
 p.append(crease_normals(tube('Tail / thick inward dragon spiral',BLUE,points,radii,40),35))
 bs=surface(body)
 belly=crease_normals(fitted_contour('Belly / fitted golden armored breast',CREAM,[(-.047,.64),(.047,.64),(.100,.57),(.140,.46),(.126,.36),(.060,.28),(-.065,.30),(-.132,.38),(-.145,.47),(-.099,.57)],bs,.008,subdivisions=4),35);p.append(belly);skin=surface(belly)
 for j,(y,w) in enumerate(((.33,.075),(.39,.110),(.46,.125),(.53,.103),(.60,.063))):
  pts=[]
  for k in range(41):
   x=w*(2*k/40-1);yy=y-.012*(1-(x/w)**2);pts.append((x,yy,skin(yy,x)+.003))
  p.append(crease_normals(tube('Belly fold / supported curved plate seam '+str(j),FOLD,pts,[.0018]*len(pts),8),35))
 rear=reverse_front(body);rs=surface(rear)
 for j,(y,w) in enumerate(((.355,.130),(.445,.155),(.54,.090))):
  pts=[]
  for k in range(49):
   x=w*(2*k/48-1);yy=y+.045*(x/w)**2;pts.append((x,yy,rs(yy,x)+.0025))
  p.append(reverse_front(crease_normals(tube('Body seam / fitted rear armor fold '+str(j),SEAM,pts,[.0017]*len(pts),8),35)))
 p.append(funnel('Snout / long hollow tapered dragon muzzle',BLUE,[(.116,.080,.060),(.22,.087,.050),(.34,.060,.039),(.49,.037,.023),(.510,.037,.023)],(0,.866,0),thickness=.012,tilt=.025,segments=64))
 p.append(funnel('Snout / recessed tapered dark bore',INK,[(.235,.072,.037),(.34,.050,.029),(.507,.027,.014)],(0,.866,0),thickness=.003,tilt=.025,segments=64))
 p.append(ellipsoid('Snout / deep enclosed chamber floor',INK,(0,.873,.235),(.073,.036,.008),32,22))
 hs=surface(head)
 for sign in (-1,1):
  x=sign*.109;y=.929
  outline=[(x+sign*a,y+b) for a,b in ((-.052,-.011),(-.039,.023),(.031,.012),(.052,.008),(.037,-.027),(-.024,-.027))]
  socket=crease_normals(fitted_contour('Eye socket / fitted angular dark brow '+str(sign),INK,outline,hs,.005,subdivisions=3),35);p.append(socket)
  cx=sum(a for a,b in outline)/len(outline);cy=sum(b for a,b in outline)/len(outline)
  white=crease_normals(fitted_contour('Eye white / fitted angular ivory '+str(sign),WHITE,[(cx+(a-cx)*.86,cy+(b-cy)*.86) for a,b in outline],surface(socket),.003,subdivisions=3),35);p.append(white)
  iris=crease_normals(fitted_patch('Iris / fitted ruby dragon iris '+str(sign),RUBY,oval(x-sign*.009,y-.002,.012,.017,32),surface(white),.003),35);p.append(iris)
  pupil=crease_normals(fitted_patch('Pupil / narrow watchful black '+str(sign),INK,oval(x-sign*.010,y-.003,.003,.010,24),surface(iris),.002),35);p.append(pupil)
  p.append(crease_normals(fitted_patch('Glint / tiny dragon eye light '+str(sign),WHITE,oval(x-sign*.010,y+.001,.0011,.0025,16),surface(pupil),.0015),35))
  p.append(curved_tube('Cheek / swept rooted dragon jaw horn '+str(sign),BLUE,[(sign*.12,.827,.055),(sign*.225,.879,-.035),(sign*.291,.985,-.105)],[.057,.041,.002],28))
  p.append(curved_tube('Crown / tall rooted blue antler '+str(sign),BLUE,[(sign*.09,.971,-.05),(sign*.125,1.095,-.085),(sign*.105,1.245,-.080)],[.020,.014,.005],24))
  for j,(y,height) in enumerate(((1.076,1.090),(1.177,1.193))):
   p.append(curved_tube('Crown branch / curved dragon tendril '+str(sign)+' '+str(j),BLUE,[(sign*.121,y,-.083),(sign*.18,y+.018,-.111),(sign*.225,y-.004,-.12),(sign*.285,height,-.119)],[.013,.011,.008,.004],20))
  neck_outline=contour([((.126,.820),(.19,.86),(.28,.864),(.295,.842)),((.295,.842),(.315,.815),(.257,.786),(.207,.788)),((.207,.788),(.172,.791),(.144,.807),(.126,.820))],10)
  neck_fin=translate(fin('Neck fin / small rounded ivory paddle '+str(sign),IVORY,neck_outline,sign),0,0,.090);p.append(neck_fin)
  wing_outline=contour([((.052,.497),(.12,.56),(.20,.76),(.238,.831)),((.238,.831),(.254,.803),(.248,.748),(.296,.691)),((.296,.691),(.352,.626),(.312,.571),(.262,.541)),((.262,.541),(.21,.507),(.112,.493),(.052,.497))],10)
  wing=fin('Dorsal fin / curved ivory dragon sail '+str(sign),IVORY,wing_outline,sign);p.append(wing);ws=surface(wing)
  for j,(xx,yy) in enumerate(((.226,.782),(.286,.667),(.265,.567))):
   pts=[]
   for k in range(33):
    t=k/32;x=sign*(.150+(xx-.150)*t);y=.565+(yy-.565)*t+.010*math.sin(math.pi*t);pts.append((x,y,ws(y,x)+.0025))
   p.append(crease_normals(tube('Fin ray / supported curved dorsal fold '+str(sign)+' '+str(j),SEAM,pts,[.0015]*len(pts),8),35))
  p.append(curved_tube('Dorsal horn / rooted blue fin spine '+str(sign),BLUE,[(sign*.10,.540,-.080),(sign*.213,.731,-.103),(sign*.249,.895,-.12)],[.023,.016,.006],24))
  p.append(curved_tube('Dorsal fork / curved upper blue ray '+str(sign),BLUE,[(sign*.237,.830,-.115),(sign*.277,.873,-.12),(sign*.304,.918,-.125)],[.012,.009,.004],20))
 floor=min(v for q in p for v in q['positions'][1::3]);height=max(v for q in p for v in q['positions'][1::3])-floor
 for q in p:
  q['positions']=[round((v-(floor if i%3==1 else 0.))*HEIGHT/height,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,default=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species');a=a.parse_args();m=sculpture();(a.out/'kingdra.mesh.json').write_text(json.dumps(m,separators=(',',':')));print('kingdra',len(m['primitives']),sum(len(q['indices'])//3 for q in m['primitives']),flush=True)
if __name__=='__main__':main()
