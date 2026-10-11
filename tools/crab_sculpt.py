#!/usr/bin/env python3
"""Original four-legged crab shells and curved open orange pincers."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,loft,curved_profiles,fitted_contour,fitted_patch,crease_normals
from bovine_sculpt import oval,curved_tube
from abra_sculpt import surface
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('krabby','kingler');HEIGHT=.97
ORANGE=(.94,.48,.27);CREAM=(.94,.91,.84);WHITE=(.98,.99,.97);INK=(.14,.13,.15)
def sculpture(s):
 if s not in SPECIES:raise ValueError(s)
 king=s=='kingler';p=[]
 body=ellipsoid('Carapace / broad orange domed crab shell',ORANGE,(0,.365,0),(.265,.200,.197),40,28);p.append(body);ss=surface(body)
 belly=ellipsoid('Belly / rounded ivory lower crab shell',CREAM,(0,.278,.071),(.274,.140,.173),40,28);p.append(belly)
 for sign in (-1,1):
  for j,z in enumerate((-.060,.065)):
   pts=[(sign*.170,.305,z),(sign*.373,.238,z-.025),(sign*.492,.076,z+.025),(sign*.540,.038,z+.081)]
   p.append(curved_tube('Walking leg / articulated ivory crab limb '+str(sign)+' '+str(j),CREAM,pts,[.030,.027,.022,.009],20))
   p.append(ellipsoid('Leg knee / ivory angular joint '+str(sign)+' '+str(j),CREAM,pts[1],(.037,.032,.036),24,16))
  root=(sign*.234,.347,.092);wrist=(sign*.467,.573,.145)
  p.append(curved_tube('Pincer arm / articulated ivory raised limb '+str(sign),CREAM,[root,(sign*.370,.396,.123),wrist],[.038,.030,.033],24))
  p.append(ellipsoid('Wrist / ivory pincer joint '+str(sign),CREAM,wrist,(.043,.044,.048),28,18))
  size=1.50 if king and sign<0 else (1.0 if king else .96);cx=sign*.485;cy=.584;cz=.153
  def point(u,y,z=0):return (cx+sign*u*size,cy+y*size,cz+z*size)
  pts=[point(0,-.025),point(.111,.047),point(.141,.188),point(.100,.301),point(-.025,.334),point(-.115,.276)]
  p.append(curved_tube('Claw palm / curved hooked orange fixed pincer '+str(sign),ORANGE,pts,[r*size for r in (.079,.102,.097,.079,.054,.007)],28))
  pts=[point(.006,-.020,.010),point(-.074,-.007,.014),point(-.153,.059,.019),point(-.144,.132,.023)]
  p.append(curved_tube('Claw jaw / curved orange movable pincer '+str(sign),ORANGE,pts,[r*size for r in (.080,.073,.047,.006)],24))
  # Faces sit on the broad front dome, with no floating eye stalks.
  x=sign*.103;y=.459
  xy=oval(x,y,.055,.048,32) if not king else [(sign*u,v) for u,v in ((.044,.444),(.166,.463),(.164,.503),(.110,.519),(.062,.497))]
  eye=fitted_contour('Eye white / fitted raised pale crab eye '+str(sign),WHITE,xy,ss,.009,subdivisions=3);p.append(eye)
  p.append(fitted_patch('Pupil / fitted fine black crab pupil '+str(sign),INK,oval(x+sign*.005,y+.011,.0037,.013 if king else .006,20),surface(eye),.0028))
  # Ivory cheek peaks rise from the lower shell rather than detached teeth.
  p.append(curved_tube('Cheek peak / rooted ivory shell point '+str(sign),CREAM,[(sign*.209,.318,.178),(sign*.211,.410,.168),(sign*.190,.444,.151)],[.037,.020,.0025],24))
 crown=[(-.110,.655),(.110,.670)] if not king else [(-.216,.604),(-.140,.687),(0,.729),(.140,.704),(.219,.619)]
 for j,(x,y) in enumerate(crown):
  rootx=x*.72;rooty=.517 if abs(rootx)<.105 else .487;rootz=ss(rooty,rootx)-.015
  p.append(curved_tube('Crown / rooted orange carapace point '+str(j),ORANGE,[(rootx,rooty,rootz),(x*.92,(rooty+y)/2,rootz-.020),(x,y,rootz-.045)],[.030 if not king else .034,.022,.0025],24))
 xy=contour((((-.147,.371),(-.073,.352),(-.030,.321),(0,.313)),((0,.313),(.034,.322),(.077,.352),(.147,.371)),((.147,.371),(.078,.347),(.026,.309),(0,.304)),((0,.304),(-.029,.312),(-.080,.346),(-.147,.371))),10)
 p.append(fitted_contour('Mouth / fitted dark angular shell seam',INK,xy,surface(belly),.005,subdivisions=3))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye white /','Pupil /','Mouth /')):p[i]=crease_normals(q,35)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':s,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:
  m=sculpture(s);(args.output/(s+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(s,len(m['primitives']),sum(len(p['indices'])//3 for p in m['primitives']),flush=True)
if __name__=='__main__':main()
