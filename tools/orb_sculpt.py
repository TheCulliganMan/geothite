#!/usr/bin/env python3
"""Original smooth closed Voltorb and Electrode shells with fitted faces."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,fitted_patch,fitted_contour,crease_normals
from abra_sculpt import surface
from bovine_sculpt import oval,curved_tube
from wing_insect_sculpt import contour
from bivalve_sculpt import ribbon_inside
from skin_glb import COORDINATES
SPECIES=('voltorb','electrode');HEIGHT={'voltorb':.82,'electrode':.94}
RED=(.85,.32,.34);WHITE=(.95,.97,.98);INK=(.12,.13,.16)
def hemisphere(upper,color):
 n=64;rings=24;v=[]
 for k in range(rings+1):
  t=.0008+(math.pi/2-.0008)*k/rings
  y=.420+(1 if upper else -1)*.415*math.cos(t)
  v.extend((.415*math.sin(t)*math.sin(j*math.tau/n),y,.415*math.sin(t)*math.cos(j*math.tau/n)) for j in range(n))
 f=[tuple(reversed(range(n))),tuple(rings*n+j for j in range(n))]
 for k in range(rings):
  for j in range(n):a=k*n+j;b=k*n+(j+1)%n;f.append((a,b,b+n,a+n))
 return crease_normals(part(('Upper shell /' if upper else 'Lower shell /')+' smooth closed molded hemisphere',color,v,f,True),45)
def sculpture(s):
 if s not in SPECIES:raise ValueError(s)
 adult=s=='electrode';top=hemisphere(True,WHITE if adult else RED);bottom=hemisphere(False,RED if adult else WHITE);p=[top,bottom];ss=surface(top)
 for sign in (-1,1):
  if adult:
   p.append(fitted_patch('Pupil / fitted narrow dark explosive orb pupil '+str(sign),INK,oval(sign*.122,.536,.0047,.015,24),ss,.0038))
   xy=contour((((.051,.615),(.081,.636),(.155,.736),(.190,.728)),((.190,.728),(.218,.724),(.248,.686),(.267,.669))),12)
   pts=[(sign*x,y,ss(y,sign*x)+.0028) for x,y in xy]
   p.append(curved_tube('Brow / supported expressive raised orb arch '+str(sign),INK,pts,[.0020]*len(pts),10))
  else:
   xy=[(sign*x,y) for x,y in ((.045,.490),(.215,.638),(.214,.459),(.067,.445))]
   border=fitted_contour('Eye border / fitted sharp dark orb socket '+str(sign),INK,xy,ss,.006,subdivisions=3);p.append(border)
   cx=sum(x for x,y in xy)/4;cy=sum(y for x,y in xy)/4
   eye=fitted_contour('Eye white / fitted angular pale orb eye '+str(sign),WHITE,[(cx+(x-cx)*.955,cy+(y-cy)*.955) for x,y in xy],surface(border),.003,subdivisions=3);p.append(eye)
   p.append(fitted_patch('Pupil / fitted stern narrow orb pupil '+str(sign),INK,oval(sign*.136,.528,.0038,.014,24),surface(eye),.0025))
 if adult:
  xy=contour((((-.236,.344),(-.093,.281),(.085,.282),(.236,.344)),((.236,.344),(.156,.162),(-.157,.162),(-.236,.344))),24)
  border=fitted_contour('Grin border / fitted wide curved explosive orb grin',INK,xy,surface(bottom),.007,subdivisions=3);p.append(border)
  outline=ribbon_inside(xy,.221,amount=.43);teeth=fitted_contour('Teeth / fitted pale curved explosive orb grin',WHITE,outline,surface(border),.004,subdivisions=3);p.append(teeth);ts=surface(teeth)
  for j,x in enumerate((-.075,.035)):
   hits=[]
   for a,b in zip(outline,outline[1:]+outline[:1]):
    if min(a[0],b[0])<=x<max(a[0],b[0]):hits.append(a[1]+(b[1]-a[1])*(x-a[0])/(b[0]-a[0]))
   lo=min(hits)+.003;hi=lo+(max(hits)-min(hits))*.35;pts=[(x,y,ts(y,x)+.0020) for y in (lo,(lo+hi)/2,hi)]
   p.append(curved_tube('Tooth division / supported short dark grin seam '+str(j),INK,pts,[.0015]*len(pts),10))
 for i,q in enumerate(p):
  if q['part'].startswith(('Eye border /','Eye white /','Pupil /','Grin border /','Teeth /')):p[i]=crease_normals(q,35)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT[s]/(hi-lo)
 for q in p:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'icon_voltorb' if s=='voltorb' else s,'version':1,'coordinate_system':COORDINATES,'primitives':p}
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);a.add_argument('--only',choices=SPECIES);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 for s in ([args.only] if args.only else SPECIES):
  m=sculpture(s);(args.output/(s+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(s,len(m['primitives']),sum(len(p['indices'])//3 for p in m['primitives']),flush=True)
if __name__=='__main__':main()
