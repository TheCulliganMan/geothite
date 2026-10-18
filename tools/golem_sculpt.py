#!/usr/bin/env python3
"""Original stone-shell reptile with fitted polygon armor and an open toothed jaw."""
import argparse,itertools,json,math
from pathlib import Path
from sculpt_geometry import part,ellipsoid,crease_normals
from chikorita_sculpt import add,sub,mul,dot,cross,unit
from bovine_sculpt import oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint
from rhyhorn_sculpt import longitudinal
from murkrow_sculpt import bill
from skin_glb import COORDINATES
HEIGHT=1.25
SEAM=(.28,.25,.22);STONE=(.49,.52,.47);LIGHT=(.56,.59,.53)
SKIN=(.64,.58,.55);SHADE=(.53,.47,.46);IVORY=(.98,.97,.91)
INK=(.075,.06,.065);RED=(.78,.30,.25);PINK=(.81,.46,.57)
CENTER=(0,.66,-.03);RADII=(.47,.49,.425)

def shell_cells():
 phi=(1+math.sqrt(5))/2
 vv=[unit(v) for v in [(0,s,t*phi) for s in (-1,1) for t in (-1,1)]+[(s,t*phi,0) for s in (-1,1) for t in (-1,1)]+[(s*phi,0,t) for s in (-1,1) for t in (-1,1)]]
 faces=[]
 for i,j,k in itertools.combinations(range(12),3):
  normal=cross(sub(vv[j],vv[i]),sub(vv[k],vv[i]));sides=[dot(normal,sub(v,vv[i])) for v in vv]
  if all(s<=1e-8 for s in sides) or all(s>=-1e-8 for s in sides):
   if dot(normal,vv[i])<0:j,k=k,j
   faces.append((i,j,k))
 assert len(faces)==20
 edges={tuple(sorted((a,b))) for f in faces for a,b in zip(f,f[1:]+f[:1])}
 truncated={(a,b):unit(add(mul(vv[a],2),vv[b])) for edge in edges for a,b in (edge,tuple(reversed(edge)))}
 cells=[]
 for i,v in enumerate(vv):
  neighbors=[b for a,b in truncated if a==i];side=unit(cross(v,(0,1,0) if abs(v[1])<.9 else (1,0,0)));other=cross(v,side)
  neighbors.sort(key=lambda b:math.atan2(dot(truncated[i,b],other),dot(truncated[i,b],side)));cells.append([truncated[i,b] for b in neighbors])
 for i,j,k in faces:cells.append([truncated[a,b] for a,b in ((i,j),(j,i),(j,k),(k,j),(k,i),(i,k))])
 assert len(cells)==32 and sum(len(c)==5 for c in cells)==12
 return cells

def stone_plate(j,outline):
 center=unit(tuple(sum(v[k] for v in outline) for k in range(3)));outline=[unit(add(mul(center,.065),mul(v,.935))) for v in outline];n=len(outline);points=[center]
 for t in (.25,.50,.75,1.):points.extend(unit(add(mul(center,1-t),mul(v,t))) for v in outline)
 vertices=[]
 for side in (1,-1):
  for k,v in enumerate(points):
   radial=(1.033 if k<=3*n else 1.020) if side==1 else .965
   vertices.append(tuple(CENTER[a]+RADII[a]*v[a]*radial for a in range(3)))
 count=len(points);faces=[(0,1+i,1+(i+1)%n) for i in range(n)]
 for ring in range(3):
  for i in range(n):a=1+ring*n+i;b=1+ring*n+(i+1)%n;faces.append((a,a+n,b+n,b))
 faces += [tuple(i+count for i in reversed(f)) for f in list(faces)]
 last=1+3*n
 for i in range(n):a=last+i;b=last+(i+1)%n;faces.append((b,a,a+count,b+count))
 return crease_normals(part('Shell plate / fitted polygon Golem stone '+str(j),LIGHT if j%4==0 else STONE,vertices,faces,True),40)

def anatomy():
 shell=ellipsoid('Shell / continuous dark brown Golem seam body',SEAM,CENTER,RADII,48,32);p=[shell]+[stone_plate(j,c) for j,c in enumerate(shell_cells())]
 head=longitudinal('Head / broad browed tan grey Golem reptile skull',SKIN,[(.25,.884,.10,.10),(.38,.938,.191,.145),(.51,.926,.224,.146),(.65,.838,.182,.094),(.732,.797,.07,.042)]);p.append(head)
 upper=bill('Muzzle / rooted broad tan grey Golem upper snout',SKIN,[(.45,.157,.776,.915),(.61,.185,.746,.852),(.75,.12,.753,.811),(.788,.046,.772,.793)]);p.append(upper)
 lower=bill('Jaw / open rooted tan grey Golem lower jaw',SHADE,[(.30,.128,.647,.791),(.48,.165,.551,.650),(.64,.137,.543,.611),(.75,.07,.591,.635),(.772,.013,.633,.643)]);p.append(lower)
 p.append(ellipsoid('Mouth back / recessed dark Golem oral chamber',INK,(0,.754,.420),(.128,.116,.068),28,18))
 p.append(ellipsoid('Tongue / rooted pink Golem tongue',PINK,(0,.612,.608),(.085,.020,.067),32,20))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.060,.941),(.169,.982),(.199,.929),(.158,.885),(.091,.885))]
  socket=paint('Socket / fitted angular dark Golem gaze '+str(s),INK,xy,head,.008,3);p.append(socket);cx=sum(x for x,y in xy)/5;cy=sum(y for x,y in xy)/5
  eye=paint('Eye / fitted narrow ivory Golem glare '+str(s),IVORY,[(cx+.86*(x-cx),cy+.78*(y-cy)) for x,y in xy],socket,.005,3);p.append(eye)
  iris=paint('Iris / fitted red Golem iris '+str(s),RED,oval(s*.145,.931,.016,.024,24),eye,.003,2);p.append(iris)
  p.append(paint('Pupil / fitted dark Golem slit '+str(s),INK,oval(s*.145,.931,.005,.019,20),iris,.003,2))
  p.append(curved_tube('Fang / rooted ivory Golem upper canine '+str(s),IVORY,[(s*.12,.784,.623),(s*.122,.741,.635),(s*.119,.699,.649)],[.027,.019,.002],24))
  p.append(paddle('Arm / rooted short tan grey Golem upper arm '+str(s),SKIN,[(s*.344,.854,.145),(s*.485,.735,.247)],[.107,.082],[.116,.090]))
  p.append(paddle('Forearm / rooted raised tan grey Golem forearm '+str(s),SKIN,[(s*.481,.738,.246),(s*.511,.864,.351)],[.084,.072],[.090,.080]))
  p.append(ellipsoid('Hand / broad three-clawed tan grey Golem hand '+str(s),SKIN,(s*.525,.920,.391),(.096,.102,.100),24,16))
  for j,dx in enumerate((-.060,0,.060)):
   p.append(curved_tube(f'Hand claw / three rooted ivory Golem claws {s} {j}',IVORY,[(s*.525+dx*.80,.969,.430),(s*.525+dx,1.043,.469),(s*.525+dx*.92,1.079,.508)],[.027,.019,.002],20))
  p.append(ellipsoid('Thigh / stocky rooted tan grey Golem haunch '+str(s),SKIN,(s*.285,.272,.051),(.181,.210,.193),32,22))
  p.append(paddle('Leg / short rooted tan grey Golem shin '+str(s),SKIN,[(s*.29,.244,.080),(s*.333,.110,.160)],[.107,.096],[.121,.093]))
  p.append(ellipsoid('Foot / grounded broad tan grey Golem pad '+str(s),SKIN,(s*.334,.059,.191),(.149,.059,.167),28,18))
  for j,dx in enumerate((-.075,0,.075)):
   p.append(paddle(f'Foot claw / three rooted ivory Golem forward toes {s} {j}',IVORY,[(s*.334+dx*.8,.050,.295),(s*.334+dx,.032,.370),(s*.334+dx*1.05,.012,.417)],[.039,.029,.012],[.027,.018,.002]))
  p.append(curved_tube('Heel claw / rooted ivory Golem rear point '+str(s),IVORY,[(s*.35,.054,.057),(s*.35,.037,-.004),(s*.35,.027,-.045)],[.027,.019,.002],20))
 for j,x in enumerate((-.072,0,.072)):
  p.append(curved_tube('Lower tooth / rooted ivory Golem jaw tooth '+str(j),IVORY,[(x,.604,.637),(x,.641,.655),(x,.664,.665)],[.020,.014,.002],20))
 return p

def sculpture(normalize=True):
 p=anatomy();lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  if normalize:q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])]
  q['normals']=[round(v,7) for v in q['normals']]
  for key in ('positions','normals'):q[key]=[0. if v==0 else v for v in q[key]]
 return {'name':'golem','version':1,'coordinate_system':COORDINATES,'primitives':p}
def write_canonical(output):
    from stone_pair_glb import export_stone_pair
    (output/'golem.glb').write_bytes(export_stone_pair('golem'))


def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);write_canonical(args.output)
if __name__=='__main__':main()
