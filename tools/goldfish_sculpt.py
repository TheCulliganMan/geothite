#!/usr/bin/env python3
"""Original flowing goldfish and plated carp, with actual recessed oral cavities."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import part,curved_profiles,crease_normals,fitted_contour,ellipsoid
from bovine_sculpt import oval,curved_tube,reverse_front
from jet_fish_sculpt import side_rotate
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
SPECIES=('goldeen','seaking','magikarp');HEIGHTS={'goldeen':.8,'seaking':1.,'magikarp':.9}
WHITE=(.98,.97,.94);ORANGE=(.94,.43,.24);RED=(.86,.34,.23);PINK=(.94,.60,.62);INK=(.08,.09,.12);TEAL=(.12,.56,.56);GOLD=(.96,.78,.30);SEAM=(.49,.24,.17);VEIN=(.74,.75,.72)

def surface(mesh):
 # Exact stored triangle interpolation, with a local XY index for authoring.
 vs=[mesh['positions'][i:i+3] for i in range(0,len(mesh['positions']),3)];bins={};cell=.025
 for k in range(0,len(mesh['indices']),3):
  a,b,c=[vs[i] for i in mesh['indices'][k:k+3]];den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
  if abs(den)<1e-12:continue
  for x in range(math.floor(min(p[0] for p in (a,b,c))/cell)-1,math.floor(max(p[0] for p in (a,b,c))/cell)+2):
   for y in range(math.floor(min(p[1] for p in (a,b,c))/cell)-1,math.floor(max(p[1] for p in (a,b,c))/cell)+2):bins.setdefault((x,y),[]).append((a,b,c,den))
 def at(y,x):
  hits=[]
  for a,b,c,den in bins.get((math.floor(x/cell),math.floor(y/cell)),[]):
   u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den
   if min(u,v,1-u-v)>=-1e-6:hits.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
  if not hits:raise ValueError(('outside fish skin',x,y))
  return max(hits)
 return at

def fitted(name,color,xy,parent,th=.004,sub=2):return crease_normals(fitted_contour(name,color,xy,surface(parent),th,sub),35)
def sheet(name,color,xy,depth,th=.012,sub=2):return crease_normals(fitted_contour(name,color,xy,depth,th,sub),35)
def zy(p):
 p=dict(p)
 for field in ('positions','normals'):p[field]=[v for i in range(0,len(p[field]),3) for v in (p[field][i+2],p[field][i+1],-p[field][i])]
 return p

def body(s):
 carp=s=='magikarp';rx,ry=(.073,.130) if carp else ((.058,.045) if s=='goldeen' else (.070,.050));yc=.455
 rows=curved_profiles(((-.40,.025,.035,yc),(-.30,.105,.130,yc),(-.16,.220,.225,yc),(.01,.260,.260,yc),(.16,.235,.235,yc),(.27,.170,.195,yc),(.335,.105,.150 if carp else .110,yc)),4)
 rows+= [( .374,rx+.009,ry+.009,yc),(.386,rx-.009,ry-.009,yc),(.313,rx*.66,ry*.66,yc)]
 n=48;v=[(w*math.cos(i*math.tau/n),y+h*math.sin(i*math.tau/n),z) for z,w,h,y in rows for i in range(n)];f=[tuple(reversed(range(n))),tuple((len(rows)-1)*n+i for i in range(n))]
 for j in range(len(rows)-1):
  for i in range(n):a=j*n+i;b=j*n+(i+1)%n;f.append((a,b,b+n,a+n))
 p=crease_normals(part('Body / continuous '+s+' cheek mantle and hollow snout',RED if carp else ORANGE if s=='seaking' else WHITE,v,f,True),45)
 # Closed oval lip with a genuine central hole; it seats in the annular snout.
 v=[];ns=48;nt=12;r=.014 if carp else .012
 for i in range(ns):
  a=i*math.tau/ns
  for j in range(nt):
   t=j*math.tau/nt;v.append(((rx+r*math.cos(t))*math.cos(a),yc+(ry+r*math.cos(t))*math.sin(a),.385+r*math.sin(t)))
 f=[]
 for i in range(ns):
  for j in range(nt):f.append((i*nt+j,((i+1)%ns)*nt+j,((i+1)%ns)*nt+(j+1)%nt,i*nt+(j+1)%nt))
 lip=crease_normals(part('Lip / rooted rounded pink open oval',PINK,v,f,True),45)
 well=ellipsoid('Mouth / recessed dark oral floor',INK,(0,yc,.312),(rx*.67,ry*.67,.007),32,18)
 return [p,lip,well]

def face(p,s):
 b=p[0];carp=s=='magikarp';x=.168 if carp else .156;y=.548
 for sign in (-1,1):
  # Front-oblique fitted lenses retain visibility in the battle camera.
  socket=fitted('Eye socket / fitted dark cheek '+str(sign),INK,oval(sign*x,y,.055,.064,28),b,.009,3);p.append(socket)
  white=fitted('Eye white / supported ivory lens '+str(sign),WHITE,oval(sign*x,y,.050,.059,28),socket,.004,2);p.append(white)
  if s=='goldeen':
   iris=fitted('Iris / low teal oval '+str(sign),TEAL,oval(sign*x,y-.018,.027,.032,24),white,.003,2);p.append(iris)
   pupil=fitted('Pupil / quiet lower dark oval '+str(sign),INK,oval(sign*x,y-.018,.012,.025,20),iris,.002,2);p.append(pupil)
   lid=[(sign*x-.047,y+.009),(sign*x+.047,y+.009),(sign*x+.035,y+.037),(sign*x,y+.055),(sign*x-.035,y+.037)];p.append(fitted('Lid / sleepy orange upper cheek '+str(sign),ORANGE,lid,white,.006,3))
  else:
   pupil=fitted('Pupil / tiny vacant carp dot '+str(sign) if carp else 'Pupil / broad glossy seaking gaze '+str(sign),INK,oval(sign*x,y,.006 if carp else .040,.009 if carp else .047,24),white,.003,2);p.append(pupil)
  if not carp:p.append(fitted('Glint / restrained ivory eye light '+str(sign),WHITE,oval(sign*x-.003,y-.010 if s=='goldeen' else y+.019,.003 if s=='goldeen' else .006,.004 if s=='goldeen' else .009,16),pupil,.002,2))
 if not carp:
  p.append(curved_tube('Horn / rooted ivory forward lance',WHITE,[(0,.64,.205),(0,.74,.28),(0,.89 if s=='goldeen' else .83,.335)],[.044,.026,.0015],28))
 else:
  for sign in (-1,1):p.append(curved_tube('Whisker / continuous curved gold barbel '+str(sign),GOLD,[(sign*.074,.423,.335),(sign*.20,.36,.36),(sign*.235,.17,.33),(sign*.29,.05,.25),(sign*.37,.08,.15)],[.018,.017,.012,.007,.0015],24))

def fins(p,s):
 carp=s=='magikarp';base=p[0];tails=[];sides=[]
 if carp:
  tail=contour([((0,.46),(.07,.56),(.23,.73),(.39,.79)),((.39,.79),(.38,.64),(.14,.53),(.17,.45)),((.17,.45),(.14,.36),(.38,.23),(.40,.14)),((.40,.14),(.20,.18),(.09,.34),(0,.46))],9)
 else:
  tail=contour([((0,.45),(.10,.47),(.09,.68),(.23,.83)),((.23,.83),(.40,.91),(.46,.83),(.43,.72)),((.43,.72),(.40,.65),(.29,.57),(.32,.51)),((.32,.51),(.33,.44),(.48,.37),(.46,.28)),((.46,.28),(.44,.15),(.31,.14),(.23,.22)),((.23,.22),(.12,.28),(.09,.42),(0,.45))],8)
 for sign in (-1,1):
  xy=[(sign*x,y) for x,y in tail];q=sheet('Tail / flowing split '+s+' caudal fan '+str(sign),WHITE,xy,lambda y,x:-.365-.75*abs(x)+.035*math.sin(y*8),.013,2);p.append(q);tails.append(q)
  side=contour([((.16,.43),(.24,.42),(.39,.41),(.52,.33)),((.52,.33),(.45,.30),(.49,.23),(.40,.20)),((.40,.20),(.34,.12),(.26,.18),(.26,.26)),((.26,.26),(.25,.32),(.18,.37),(.16,.43))],8)
  q=sheet('Pectoral / swept scalloped white paddle '+str(sign),WHITE,[(sign*x,y) for x,y in side],lambda y,x:.10-.50*(abs(x)-.16)+.025*math.sin(y*12),.012,2);p.append(q);sides.append(q)
  if carp:
   # Raised red leading rim, rooted in the white paddle.
   sk=surface(q);path=[(sign*x,y,sk(y,sign*x)-.001) for x,y in ((.175,.422),(.24,.405),(.33,.389),(.43,.366))];p.append(curved_tube('Pectoral rim / fitted red leading edge '+str(sign),RED,path,[.010,.010,.009,.005],18))
 dorsal=[(-.23,.62),(-.17,.82),(-.10,.77),(-.04,.96),(.02,.78),(.11,.84),(.16,.63)] if carp else contour([((-.25,.59),(-.23,.76),(-.13,.84),(-.04,.80)),((-.04,.80),(.09,.77),(.16,.70),(.18,.61)),((.18,.61),(.05,.62),(-.13,.61),(-.25,.59))],9)
 # Draw in XY, rotate into the midline YZ plane.
 d=zy(sheet('Dorsal / '+('three pointed golden crown' if carp else 'curved ivory sail'),GOLD if carp else WHITE,dorsal,lambda y,x:.012*math.sin(x*7),.014,2));p.append(d)
 if carp:
  xy=[(-.24,.28),(-.18,.075),(-.11,.13),(-.025,.035),(.025,.15),(.14,.10),(.15,.29)];p.append(zy(sheet('Pelvic / inverted gold crown',GOLD,xy,lambda y,x:.008*math.sin(x*6),.013,2)))
 elif s=='seaking':
  for sign in (-1,1):p.append(sheet('Pelvic / rounded spotted lower streamer '+str(sign),WHITE,[(sign*x,y) for x,y in contour([((.08,.26),(.10,.14),(.18,.04),(.24,.085)),((.24,.085),(.32,.15),(.26,.22),(.19,.31)),((.19,.31),(.15,.29),(.11,.27),(.08,.26))],8)],lambda y,x:-.15-.35*abs(x),.012,2))
 # Thin fitted veins follow actual stored fan surfaces, not a guessed plane.
 for j,q in enumerate(tails+sides):
  sk=surface(q);sign=-1 if max(q['positions'][::3])<=.001 else 1
  paths=(([(.04,.46),(.10,.53),(.29,.70)],[(.04,.45),(.10,.38),(.29,.24)]) if carp else ([(.055,.45),(.16,.52),(.27,.68)],[(.055,.45),(.19,.42),(.33,.29)])) if j<2 else ([(.18,.41),(.29,.33),(.39,.26)],)
  for k,path in enumerate(paths):
   pts=[(sign*x,y,sk(y,sign*x)-.001) for x,y in path];p.append(curved_tube('Fin ray / supported pale fan fold '+str(j)+' '+str(k),SEAM if carp else VEIN,pts,[.0025]*len(pts),10))
 return tails,sides

def markings(p,s,tails,sides):
 b=p[0]
 if s=='goldeen':
  p.append(fitted('Mask / fitted orange forehead saddle',ORANGE,[(-.145,.623),(-.11,.672),(0,.700),(.11,.672),(.145,.623),(.075,.640),(0,.665),(-.075,.640)],b,.009,3))
  for j,q in enumerate(tails+sides):
   sign=-1 if max(q['positions'][::3])<=.001 else 1
   xy=oval(sign*(.18 if j<2 else .25),.47 if j<2 else .365,.08 if j<2 else .045,.045 if j<2 else .030,22)
   p.append(fitted('Fin paint / supported orange root wash '+str(j),ORANGE,xy,q,.0035,3))
 elif s=='seaking':
  for sign in (-1,1):
   lateral=side_rotate(b,-sign)
   patch=[(-.12,.665),(.07,.675),(.13,.62),(.07,.53),(.11,.42),(.06,.29),(.015,.35),(-.005,.51),(-.065,.56)]
   p.append(side_rotate(fitted('Flank / seated dark hooked saddle '+str(sign),INK,patch,lateral,.008,3),sign))
   for j,(x,y,r) in enumerate(((-.21,.48,.022),(-.24,.54,.014),(-.19,.39,.025))):p.append(side_rotate(fitted('Flank spot / fitted dark speck '+str(sign)+' '+str(j),INK,oval(x,y,r,r*.85,14),lateral,.004,2),sign))
   # Two dark gill crescents on the forward cheek.
   for j,y in enumerate((.43,.39)):
    x=sign*.15;xy=[(x-.035,y+.012),(x+.035,y+.012),(x+.020,y-.006),(x,y-.012),(x-.020,y-.006)];p.append(fitted('Gill / supported charcoal crescent '+str(sign)+' '+str(j),INK,xy,b,.005,3))
  p.append(fitted('Bib / supported white lower chin',WHITE,[(-.065,.414),(.065,.414),(.11,.33),(.055,.25),(-.055,.25),(-.11,.33)],b,.009,4))
  for j,q in enumerate(tails+sides):
   sign=-1 if max(q['positions'][::3])<=.001 else 1
   points=((.25,.74,.021),(.36,.70,.016),(.38,.29,.025),(.30,.34,.014)) if j<2 else ((.33,.32,.022),(.37,.255,.011))
   for k,(x,y,r) in enumerate(points):
    xy=[(sign*(x+dx*r),y+dy*r) for dx,dy in ((-1,-.6),(-.6,1),(.6,.7),(1,-.4),(.2,-1))];p.append(fitted('Fin spot / seated irregular charcoal '+str(j)+' '+str(k),INK,xy,q,.003,2))
 else:
  # Carp plate seams wrap both flanks of the actual mantle.
  for sign in (-1,1):
   lateral=side_rotate(b,-sign);sk=surface(lateral)
   paths=[[(z,.29),(z-.024,.36),(z+.016,.43),(z-.022,.50),(z+.015,.58),(z-.012,.64)] for z in (-.16,-.015,.12)]
   for j,path in enumerate(paths):
    pts=[(x,y,sk(y,x)-.001) for x,y in path];p.append(side_rotate(curved_tube('Scale seam / supported staggered carp plate '+str(sign)+' '+str(j),SEAM,pts,[.003]*len(pts),10),sign))
 return p

def sculpture(s):
 if s not in SPECIES:raise ValueError('unsupported goldfish')
 p=body(s);face(p,s);tails,sides=fins(p,s);markings(p,s,tails,sides)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHTS[s]/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':s,'version':1,'coordinate_system':COORDINATES,'primitives':p}
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--out',type=Path,required=True);args=a.parse_args();args.out.mkdir(parents=True,exist_ok=True)
 for s in SPECIES:
  m=sculpture(s);(args.out/(s+'.mesh.json')).write_text(json.dumps(m,separators=(',',':')));print(s,len(m['primitives']),sum(len(p['indices'])//3 for p in m['primitives']),flush=True)
