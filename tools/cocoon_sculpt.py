#!/usr/bin/env python3
"""Original hooked green chrysalis and rounded golden cocoon with fitted shell anatomy."""
import argparse,math
from pathlib import Path
from functools import lru_cache
from copy import deepcopy
from sculpt_geometry import part,tube,crease_normals,fitted_adaptive_contour
from bovine_sculpt import oval
from wing_insect_sculpt import contour
from goldfish_sculpt import surface
from skin_glb import COORDINATES
SPECIES=('metapod','kakuna');HEIGHT={'metapod':.80,'kakuna':.80}
GREEN=(.59,.76,.36);LIGHT=(.67,.82,.44);DARK_GREEN=(.35,.51,.22)
GOLD=(.96,.78,.30);ARM=(.90,.68,.23);OCHRE=(.65,.47,.18)
INK=(.075,.09,.075);WHITE=(.97,.98,.91)

def shell(species):
 if species=='metapod':
  profiles=((.012,.010,-.270,-.290),(.10,.065,-.185,-.290),(.21,.105,-.100,-.245),
   (.30,.130,-.045,-.220),(.38,.145,.080,-.210),(.46,.158,.225,-.235),
   (.55,.155,.190,-.330),(.63,.125,.120,-.160),(.73,.085,.060,-.050),(.815,.009,.010,-.005))
 else:
  profiles=((.012,.012,.010,-.012),(.10,.078,.061,-.071),(.24,.127,.105,-.117),
   (.36,.157,.145,-.148),(.45,.181,.173,-.162),(.54,.178,.154,-.157),
   (.605,.170,.133,-.139),(.665,.157,.111,-.118),(.715,.137,.091,-.099),
   (.755,.109,.067,-.078),(.785,.067,.038,-.052),(.803,.012,.006,-.023))
 n=24;v=[]
 for y,w,front,back in profiles:
  mid=(front+back)/2;depth=(front-back)/2
  for i in range(n):
   a=math.tau*i/n;s=math.sin(a);c=math.cos(a)
   v.append((w*math.copysign(abs(s)**.82,s),y,mid+depth*math.copysign(abs(c)**.82,c)))
 f=[tuple(reversed(range(n))),tuple((len(profiles)-1)*n+i for i in range(n))]
 for j in range(len(profiles)-1):
  for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part('Shell / continuous hooked green chrysalis' if species=='metapod' else 'Shell / continuous round crowned golden cocoon',GREEN if species=='metapod' else GOLD,v,f,True),40.)

def fitted_line(name,color,xy,parent,radius=.0018):
 skin=surface(parent);points=[(x,y,skin(y,x)+radius*.35) for x,y in xy]
 return tube(name,color,points,[radius]*len(points),8)

def cross_line(y,w,n=25,drop=.0):return [(-w+2*w*i/(n-1),y+drop*abs(2*i/(n-1)-1)) for i in range(n)]

@lru_cache(maxsize=2)
def raw_sculpture(species):
 if species not in SPECIES:raise ValueError('unsupported cocoon')
 body=shell(species);p=[body];green=species=='metapod'
 for side in (-1,1):
  if green:
   cx=side*.080
   # The upper diagonal is the actual white eye boundary, not a rectangular
   # white plate. Rounded lower arcs give the characteristic sleepy gaze.
   xy=contour([((-.037,.019),(-.010,.014),(.014,.006),(.038,-.003)),((.038,-.003),(.041,-.027),(.020,-.045),(-.006,-.043)),((-.006,-.043),(-.034,-.042),(-.043,-.013),(-.037,.019))],8)
   xy=[(cx+side*x,.619+y) for x,y in xy]
   eye=fitted_adaptive_contour('Eye / rounded sleepy ivory crescent '+str(side),WHITE,xy,body,.004,surface_at=surface(body));p.append(eye)
   pupil=fitted_adaptive_contour('Pupil / fitted dark sleepy cocoon gaze '+str(side),INK,oval(cx,.609,.014,.017,20),eye,.002,surface_at=surface(eye));p.append(pupil)
   brow=[(cx+side*x,.619+y) for x,y in ((-.039,.021),(.040,-.003),(.040,.004),(-.038,.028))]
   p.append(fitted_adaptive_contour('Brow / sloping green chrysalis lid '+str(side),DARK_GREEN,brow,body,.005,surface_at=surface(body)))
   xy=[(side*x,y) for x,y in ((.008,.449),(.121,.413),(.097,.340),(.012,.359))]
   p.append(fitted_adaptive_contour('Shell fold / fitted green cheek shield '+str(side),LIGHT,xy,body,.003,surface_at=surface(body)))
   line=[(side*(.014+.095*k/16),.353+.067*k/16) for k in range(17)]
   p.append(fitted_line('Shell crease / dark diagonal cheek joint '+str(side),DARK_GREEN,line,body,.0018))
  else:
   xy=contour([((.026,.625),(.053,.645),(.091,.695),(.104,.682)),((.104,.682),(.134,.653),(.129,.647),(.117,.640)),((.117,.640),(.087,.625),(.054,.618),(.026,.625))],9)
   xy=[(side*x,y) for x,y in xy];eye=fitted_adaptive_contour('Eye / fitted triangular dark cocoon slit '+str(side),INK,xy,body,.004,surface_at=surface(body));p.append(eye)
   p.append(fitted_adaptive_contour('Glint / tiny ivory inner cocoon reflection '+str(side),WHITE,oval(side*.061,.637,.008,.009,20),eye,.002,surface_at=surface(eye)))
   xy=[(side*x,y) for x,y in ((.014,.550),(.155,.565),(.170,.480),(.110,.430),(.035,.465))]
   plate=fitted_adaptive_contour('Arm fold / broad layered golden shell plate '+str(side),ARM,xy,body,.010,surface_at=surface(body));p.append(plate)
   # Trace the lower plate on its own stored surface; the seam shares its rig.
   xy=[(side*(.042+.065*k/15),.470-.032*k/15) for k in range(16)]
   p.append(fitted_line('Arm crease / fine ochre folded plate edge '+str(side),OCHRE,xy,plate,.0018))
 if green:
  p.append(fitted_line('Shell crease / lower folded face seam',DARK_GREEN,cross_line(.313,.111,25,.020),body,.0019))
  for j,(y,w) in enumerate(((.097,.057),(.168,.084),(.245,.112))):
   p.append(fitted_line('Tail crease / three curled chrysalis segments '+str(j),DARK_GREEN,cross_line(y,w,25,.012),body,.0018))
 else:
  p.append(fitted_line('Shell crease / rounded golden head collar',OCHRE,cross_line(.586,.167,33,.006),body,.0019))
  for j,(y,w) in enumerate(((.115,.077),(.210,.111),(.310,.142))):
   p.append(fitted_line('Shell crease / three tapered abdominal bands '+str(j),OCHRE,cross_line(y,w,25,.014),body,.0018))
  # A structural eight-sided ridge, with rows at the real shell's profile
  # breaks. This keeps a clean raised shield without thin coating triangles.
  skin=surface(body);rows=((.295,.001),(.360,.038),(.371,.040),(.450,.022),(.492,.018),(.540,.0036),(.552,.001));v=[];n=8
  for y,w in rows:
   for j in range(n):
    angle=math.tau*j/n;x=w*math.sin(angle);c=math.cos(angle)
    v.append((x,y,skin(y,x)+.005+(.015 if c>=0 else .013)*c))
  faces=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+j for j in range(n))]
  for k in range(len(rows)-1):
   for j in range(n):a=k*n+j;b=k*n+(j+1)%n;faces.append((a,b,b+n,a+n))
  p.append(crease_normals(part('Shell keel / elongated central golden shield',GOLD,v,faces,True),40.))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(species,normalize=True):
 m=deepcopy(raw_sculpture(species));p=m['primitives']
 if not normalize:return m
 low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT[species]/(high-low)
 for q in p:
  q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return m

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 from cocoon_glb import export_cocoon
 for s in SPECIES:
  (args.output/(s+'.glb')).write_bytes(export_cocoon(s))
  (args.output/(s+'.mesh.json')).unlink(missing_ok=True)
if __name__=='__main__':main()
