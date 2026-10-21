#!/usr/bin/env python3
"""Original Noctowl: broad forked crown, stern red gaze and deep folded cases."""
import math,struct
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import ellipsoid,loft,tube,curved_profiles,fitted_triangle_contour,fitted_contour,crease_normals
from bovine_sculpt import oval
from lugia_sculpt import paddle
from pidgeotto_sculpt import vane
from skin_glb import COORDINATES
HEIGHT=1.30
BROWN=(.64,.43,.31);DARK=(.28,.17,.12);GOLD=(.94,.83,.56)
WHITE=(.99,.98,.91);INK=(.045,.030,.025);RED=(.87,.20,.20);PINK=(.93,.72,.59)
def paint(name,color,xy,parent,th=.004):return fitted_triangle_contour(name,color,xy,parent,th,precision=6)
@lru_cache(maxsize=1)
def raw_sculpture():
 body=crease_normals(loft('Body / continuous tawny owl head and breast',BROWN,curved_profiles(((.120,.085,.105,-.020),(.200,.190,.190,-.035),(.380,.255,.260,-.045),(.580,.250,.250,-.025),(.730,.205,.205,.005),(.810,.190,.180,.025),(.870,.250,.205,.040),(.990,.280,.230,.045),(1.110,.250,.215,.025),(1.190,.170,.150,.015),(1.220,.020,.025,0.)),3),32),45);parts=[body]
 for s in (-1,1):
  mask=paint('Eye mask / fitted gold owl disk '+str(s),GOLD,oval(s*.115,.987,.105,.106,28),body,.006);parts.append(mask)
  ring=paint('Eye ring / fitted black owl outline '+str(s),INK,oval(s*.115,.987,.088,.091,28),mask,.003);parts.append(ring)
  eye=paint('Eye / fitted ivory owl sclera '+str(s),WHITE,oval(s*.115,.987,.075,.078,28),ring,.003);parts.append(eye)
  iris=paint('Iris / small red owl iris '+str(s),RED,oval(s*.104,.975,.031,.037,20),eye,.002);parts.append(iris)
  pupil=paint('Pupil / focused black owl gaze '+str(s),INK,oval(s*.104,.975,.014,.024,16),iris,.0015);parts.append(pupil)
  parts.append(paint('Glint / small ivory owl eye '+str(s),WHITE,oval(s*.104-.004,.984,.004,.006,12),pupil,.001))
  outline=[(0.,.958),(.245,1.040),(.255,1.110),(.115,1.070),(.040,1.056),(0.,1.128)]
  parts.append(crease_normals(fitted_contour('Brow / broad gold crown bridge '+str(s),GOLD,[(s*x,y) for x,y in outline],lambda y,x:.240-.23*abs(x)-.10*(y-.970),.070,0),40))
  horn=[(.140,1.057),(.278,1.108),(.285,1.390),(.250,1.350),(.241,1.445),(.206,1.400),(.172,1.460),(.149,1.390)]
  parts.append(crease_normals(fitted_contour('Crown / broad three-point owl plume '+str(s),GOLD,[(s*x,y) for x,y in horn],lambda y,x:.200-.45*(y-1.05),.070,0),40))
  parts.append(paddle('Wing / deep dark folded owl case '+str(s),DARK,[(s*.145,.780,-.025),(s*.285,.530,-.070),(s*.260,.174,-.220)],[.035,.075,.012],[.050,.135,.025],segments=18,steps=5))
  for j in range(2):
   parts.append(vane(f'Flight feather / broad folded owl vane {s} {j}',DARK,(s*(.304+.008*j),.522-.014*j,-.040-.025*j),(s*(.302+.008*j),.165+.012*j,-.254-.027*j),.060,.015))
  parts.append(tube('Leg / stout gold owl shank '+str(s),GOLD,[(s*.108,.228,.005),(s*.108,.127,.005),(s*.108,.052,.005)],[.034,.029,.026],16))
  parts.append(ellipsoid('Foot / broad planted owl pad '+str(s),GOLD,(s*.108,.037,.042),(.071,.037,.072),20,12))
  for j,dx in enumerate((-.049,0.,.049)):
   tip=(s*.108+dx*1.12,.020,.184 if j==1 else .163)
   parts.append(tube(f'Toe / grasping forward owl digit {s} {j}',GOLD,[(s*.108+dx*.25,.036,.074),(s*.108+dx,.026,.132),tip],[.018,.016,.009],14))
   parts.append(paddle(f'Claw / curved pale owl talon {s} {j}',PINK,[tip,(tip[0],.016,tip[2]+.029),(tip[0],.002,tip[2]+.043)],[.010,.009,.0015],[.012,.008,.0015],segments=12,steps=3))
  parts.append(tube('Rear toe / opposing owl hallux '+str(s),GOLD,[(s*.108,.029,.008),(s*.120,.019,-.043),(s*.137,.012,-.077)],[.015,.013,.006],14))
 for row,y in enumerate((.615,.390)):
  for j,x in enumerate((-.151,0.,.151)):
   w=.050 if j!=1 else .058
   parts.append(paint(f'Breast mark / fitted dark owl feather {row} {j}',DARK,[(x-w,y+.060),(x,y-.083),(x+w,y+.060)],body,.005))
 parts.append(paddle('Beak / short pale hooked owl bill',PINK,[(0.,.963,.250),(0.,.914,.308),(0.,.880,.302)],[.029,.028,.003],[.024,.026,.002],segments=16,steps=4))
 parts.append(paint('Mouth / fitted owl bill line',DARK,[(-.035,.870),(0.,.864),(.035,.870),(.029,.862),(0.,.857),(-.029,.862)],body,.003))
 for j,x in enumerate((-.062,0.,.062)):
  parts.append(paddle('Tail / broad layered owl fan '+str(j),DARK,[(x*.5,.260,-.205),(x,.225+.004*abs(j-1),-.365),(x*1.25,.205+.004*abs(j-1),-.530)],[.031,.043,.020],[.019,.014,.004],segments=14,steps=4))
 return dict(name='noctowl',version=1,coordinate_system=COORDINATES,primitives=parts)
def sculpture(*,normalize=True):
 model=deepcopy(raw_sculpture())
 if normalize:
  p=model['primitives'];low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);k=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if i%3==1 else 0.))*k,7) for i,v in enumerate(q['positions'])]
   q['positions']=list(struct.unpack('<'+'f'*len(q['positions']),struct.pack('<'+'f'*len(q['positions']),*q['positions'])))
   q.update(crease_normals(q,40.));q['normals']=[round(v,7) or 0. for v in q['normals']]
 return model
