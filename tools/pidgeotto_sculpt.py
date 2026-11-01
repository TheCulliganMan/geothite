#!/usr/bin/env python3
"""Original grounded Pidgeotto, authored from inspected game silhouettes.

The recipe has no dependency on a stored asset or reference mesh. The 34 public
anatomy names remain stable; closed components share material/rig ownership.
"""
import math,struct
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals,fitted_triangle_contour,fitted_contour,tube
from bovine_sculpt import curved_tube,oval
from lugia_sculpt import paddle
from skin_glb import COORDINATES
HEIGHT=1.0
BROWN=(.64,.43,.25);CREAM=(.96,.87,.63);INK=(.12,.095,.08)
RED=(.89,.33,.32);PINK=(.81,.50,.54);IVORY=(.96,.94,.84)


def join(name,parts):
 q=dict(part=name,positions=[],normals=[],indices=[],base_color=parts[0]['base_color'])
 for p in parts:
  if p['base_color']!=q['base_color']:raise ValueError('merged components must share pigment')
  start=len(q['positions'])//3
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices']+=[start+i for i in p['indices']]
 return q


def frame(q,angle,inverse=False):
 """Rigid Y frame, also used by independent fitted-surface checks."""
 a=-angle if inverse else angle;c,s=math.cos(a),math.sin(a)
 q=dict(q)
 for k in ('positions','normals'):
  q[k]=[v for x,y,z in zip(q[k][::3],q[k][1::3],q[k][2::3]) for v in (c*x+s*z,y,-s*x+c*z)]
 return q


def mark(name,color,outline,parent,th=.003):
 return fitted_triangle_contour(name,color,outline,parent,th,precision=6)


def eye_parts(head,side):
 # The iris and hooked temple are on the lateral skull, not on the muzzle.
 angle=side*math.radians(53);local=frame(head,angle,True)
 uv=lambda xy:[(side*x,y) for x,y in xy]
 mask=mark('mask',INK,uv(((-.050,.724),(.003,.754),(.084,.744),(.100,.713),(.077,.681),(.064,.622),(.037,.646),(.039,.694),(-.036,.688))),local,.004)
 white=mark('eye',IVORY,uv(((-.038,.721),(.009,.737),(.065,.730),(.054,.704),(.005,.696),(-.031,.702))),mask,.003)
 pupil=mark('pupil',INK,oval(side*.013,.715,.012,.013,14),white,.002)
 glint=mark('glint',IVORY,oval(side*.009,.720,.003,.003,10),pupil,.0015)
 return [frame(p,angle) for p in (white,join('ink',[mask,pupil]),glint)]


def vane(name,color,root,tip,width,th=.009):
 """Broad YZ feather with a real narrow X wall; both profiles retain its area."""
 dy,dz=tip[1]-root[1],tip[2]-root[2];length=math.hypot(dy,dz);uy,uz=-dz/length,dy/length
 outline=[]
 for t,w in ((0,-.35),(.24,-.92),(.74,-1.),(1.,-.10),(1.,.10),(.74,1.),(.24,.92),(0.,.35)):
  outline.append((root[2]+t*dz+w*uz*width,root[1]+t*dy+w*uy*width))
 # local +Z maps to +X; local +X maps to -Z to keep the frame right handed.
 p=fitted_contour(name,color,[(-z,y) for z,y in outline],lambda y,x:root[0],th,0)
 for k in ('positions','normals'):
  p[k]=[v for x,y,z in zip(p[k][::3],p[k][1::3],p[k][2::3]) for v in (z,y,-x)]
 return crease_normals(p,38)


def sculpture(*,with_transform=False):
 body=crease_normals(loft('Bird body',BROWN,curved_profiles(((.120,.055,.080,-.040),(.190,.146,.175,-.045),(.320,.217,.245,-.070),(.460,.220,.231,-.043),(.574,.162,.168,.006),(.653,.104,.109,.052),(.690,.068,.080,.070)),3),28),45)
 head=ellipsoid('Bird head',BROWN,(0,.698,.076),(.132,.118,.151),28,18)
 breast=mark('bib',CREAM,[(-.060,.629),(-.120,.562),(-.169,.473),(-.181,.361),(-.146,.223),(-.074,.153),(0,.144),(.074,.153),(.146,.223),(.181,.361),(.169,.473),(.120,.562),(.060,.629)],body,.009)
 throat=mark('throat',CREAM,[(-.037,.734),(-.063,.702),(-.088,.650),(-.056,.612),(0,.594),(.056,.612),(.088,.650),(.063,.702),(.037,.734)],head,.005)
 p=[body,join('Light breast',[breast,throat]),head]
 for side,suffix in ((-1,'004'),(1,'005')):
  for q,n in zip(eye_parts(head,side),('Eye sclera.','Eye iris.','Eye catchlight.')):q['part']=n+suffix;p.append(q)
 p.append(paddle('Beak',PINK,[(0,.697,.201),(0,.693,.253),(0,.656,.285)],[.042,.039,.002],[.031,.022,.002],segments=16,steps=4))
 for side in (-1,1):
  suffix='' if side<0 else '.001';x=side*.112
  # Straight lower shank below a rear hock. Forward toes are separate supports.
  leg=[curved_tube('shank',PINK,[(x,.218,.018),(x,.116,-.004),(x,.038,-.004)],[.031,.023,.025],14)]
  leg.append(ellipsoid('pad',PINK,(x,.028,.007),(.054,.028,.048),16,10))
  ends=[]
  for j,dx in enumerate((-.043,0.,.043)):
   end=(x+dx,.019,.140 if j==1 else .121);ends.append(end)
   leg.append(curved_tube('digit',PINK,[(x+dx*.20,.031,.023),(x+dx*.65,.024,.079),end],[.015,.014,.010],12))
  leg.append(curved_tube('rear toe',PINK,[(x,.031,-.012),(x+side*.012,.019,-.064),(x+side*.017,.015,-.090)],[.014,.011,.007],12))
  p.append(join('Bird shin'+suffix,leg))
  for j,end in enumerate(ends):
   index=j+(0 if side<0 else 3);n='Toe'+('' if index==0 else '.'+str(index).zfill(3));ex,ey,ez=end
   p.append(curved_tube(n,IVORY,[(ex,ey,ez-.005),(ex,.014,ez+.023),(ex,.003,ez+.039)],[.011,.008,.0015],12))
  shoulder=ellipsoid('shoulder',BROWN,(side*.168,.491,-.038),(.094,.118,.144),20,14)
  cover=vane('cover',BROWN,(side*.212,.524,-.003),(side*.247,.332,-.350),.105,.032)
  p.append(join('Layered wing'+suffix,[shoulder,cover]))
  for group in range(4):
   feathers=[]
   for row in (0,1):
    # Four overlapping long vanes and a shorter row retain rounded, folded banks.
    j=group;root=(side*(.243+.005*j),.494-.025*j,-.051-.029*j)
    tip=(side*(.255+.003*j),.272+.015*j,-.245-.057*j)
    if row:root=(root[0]+side*.011,root[1]+.022,root[2]+.022);tip=(tip[0]+side*.016,tip[1]+.074,tip[2]+.064)
    feathers.append(vane('vane',CREAM,root,tip,.054 if row==0 else .045,.012))
   index=group+(0 if side<0 else 4);p.append(join('Flight feather'+('' if index==0 else '.'+str(index).zfill(3)),feathers))
 for j in range(3):
  # Broad red outer pair, cream intermediate pair, and one red center feather.
  parts=[];xs=(-.071,.071) if j==0 else (-.036,.036) if j==1 else (0.,)
  for x in xs:parts.append(paddle('tail',CREAM if j==1 else RED,[(x*.30,.262,-.225),(x,.352,-.410),(x*1.38,.486,-.580 if j==2 else -.550)],[.033,.041,.012],[.014,.012,.004],segments=12,steps=4))
  p.append(join('Tail feather'+('' if j==0 else '.'+str(j).zfill(3)),parts))
 for j in range(3):
  parts=[]
  for side in ((-1,1) if j<2 else (0,)):
   x=side*(.035 if j==0 else .068)
   parts.append(paddle('crest',RED,[(x*.7,.772,.148),(x+side*.035,.922-j*.025,.035),(x+side*.075,.963-j*.035,-.145),(x+side*.110,.950-j*.052,-.310)],[.018,.033,.028,.0015],[.018,.027,.022,.0015],segments=12,steps=4))
  p.append(join('Head crest'+('' if j==0 else '.'+str(j).zfill(3)),parts))
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);k=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*k,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) or 0. for v in q['normals']]
  q['positions']=list(struct.unpack('<'+str(len(q['positions']))+'f',struct.pack('<'+str(len(q['positions']))+'f',*q['positions'])))
  shaded=crease_normals(q,40.);q.update(shaded)
  q['normals']=[round(v,7) or 0. for v in q['normals']]
 model=dict(name='pidgeotto',version=1,coordinate_system=COORDINATES,primitives=p)
 return (model,(lo,k)) if with_transform else model
