#!/usr/bin/env python3
"""Original Hoothoot volume from complete game orbits and Crystal animation."""
import argparse,math,struct
from pathlib import Path
from sculpt_geometry import ellipsoid,tube,fitted_triangle_contour,fitted_contour,crease_normals
from bovine_sculpt import oval
from lugia_sculpt import paddle
from pidgeotto_sculpt import join
from skin_glb import COORDINATES
HEIGHT=1.0
# Actual full-colour game pigments, distinct from Crystal's shared indices.
OCHRE=(.67,.44,.25);CREAM=(.91,.80,.58);NAVY=(.045,.038,.033)
CORAL=(1.,.43,.34);PINK=(.93,.77,.85);WHITE=(.99,.97,.91);INK=(.025,.021,.018)
def mark(name,color,xy,parent,th=.003):return fitted_triangle_contour(name,color,xy,parent,th,precision=6)
def sculpture(*,normalize=True):
 body=ellipsoid('Body / continuous round ochre owl',OCHRE,(0.,.420,0.),(.315,.310,.295),32,22);p=[body]
 for s in (-1,1):
  mask=mark('Eye mask / fitted charcoal clock ring '+str(s),NAVY,oval(s*.120,.565,.117,.117,32),body,.005)
  lashes=[]
  for angle in (-.65,0.,.65):
   x=s*(.120+.117*math.sin(angle));y=.565-.117*math.cos(angle)
   lashes.append(mark('clock lash',NAVY,[(x-.012,y+.009),(x+.012,y+.009),(x+.017,y-.030),(x-.013,y-.032)],body,.009))
  p.append(join(mask['part'],[mask]+lashes))
  eye=mark('Eye / fitted round coral owl iris '+str(s),CORAL,oval(s*.120,.565,.095,.095,32),mask,.003);p.append(eye)
  pupil=mark('Pupil / small focused black owl gaze '+str(s),INK,oval(s*.113,.566,.018,.022,20),eye,.002);p.append(pupil)
  p.append(mark('Glint / restrained white owl eye '+str(s),WHITE,oval(s*.113-.005,.574,.004,.005,12),pupil,.001))
  p.append(paddle('Clock brow / slender diagonal hand '+str(s),NAVY,[(s*.025,.635,.202),(s*.118,.704,.153),(s*.225,.779,.080)],[.018,.020,.017],[.016,.015,.014],segments=12,steps=3))
  blade=[(.205,.762),(.229,.804),(.326,.833),(.305,.778),(.262,.761),(.244,.748)]
  p.append(crease_normals(fitted_contour('Clock brow / broad diamond terminal '+str(s),NAVY,[(s*x,y) for x,y in blade],lambda y,x:.080-.85*(y-.779),.025,0),40))
  p.append(paddle('Wing / short outward owl shoulder '+str(s),OCHRE,[(s*.259,.323,-.008),(s*.354,.278,-.025),(s*.453,.260,-.070)],[.040,.049,.003],[.034,.028,.003],segments=14,steps=3))
  for j in range(2):
   p.append(paddle(f'Flight feather / short owl trailing tip {s} {j}',OCHRE,[(s*(.318+.013*j),.292,-.033),(s*(.365+.014*j),.263,-.061),(s*(.402+.019*j),.243+.004*j,-.085-.020*j)],[.022,.025,.002],[.022,.016,.002],segments=10,steps=3))
 p.append(mark('Breast mark / broad fitted pale belly',CREAM,oval(0.,.285,.219,.147,40),body,.005))
 hem=[]
 for j in range(-3,4):
  x=j*.026;hem.append(paddle('pale lower feather',CREAM,[(x*.8,.150,.092),(x,.119,.081),(x*1.12,.098+.005*abs(j),.063)],[.021,.023,.0015],[.015,.016,.0015],segments=8,steps=2))
 p.append(join('Breast mark / pale scalloped lower hem',hem))
 p.append(paddle('Upper beak / short pale pink owl hook',PINK,[(0.,.488,.266),(0.,.461,.321),(0.,.434,.348)],[.029,.034,.002],[.023,.028,.002],segments=16,steps=3))
 p.append(paddle('Lower beak / short pale pink owl jaw',PINK,[(0.,.455,.264),(0.,.434,.309),(0.,.434,.329)],[.025,.025,.002],[.016,.015,.002],segments=14,steps=3))
 p.append(tube('Leg / central weight-bearing owl shank',PINK,[(0.,.165,.005),(0.,.096,.005),(0.,.036,.005)],[.026,.022,.022],14))
 bands=[tube('dark ankle ring',NAVY,[(0.,y-.004,.005),(0.,y+.004,.005)],[.0235,.0235],16) for y in (.073,.103)]
 p.append(join('Leg band / two dark owl ankle rings',bands))
 p.append(ellipsoid('Foot / single planted owl pad',PINK,(0.,.026,.036),(.061,.026,.057),18,12))
 for j,dx in enumerate((-.040,0.,.040)):
  tip=(dx*1.1,.018,.161 if j==1 else .144)
  p.append(tube('Toe / three forward owl digits '+str(j),PINK,[(dx*.2,.031,.064),(dx,.023,.116),tip],[.014,.013,.008],12))
  p.append(paddle('Claw / small ivory owl talon '+str(j),WHITE,[tip,(tip[0],.009,tip[2]+.025),(tip[0],.002,tip[2]+.038)],[.008,.007,.0015],[.009,.006,.0015],segments=10,steps=2))
 p.append(tube('Rear toe / opposing owl hallux',PINK,[(0.,.027,.012),(.012,.017,-.033),(.020,.012,-.065)],[.013,.011,.006],12))
 p.append(tube('Leg / tucked second owl shank',PINK,[(-.085,.230,-.055),(-.070,.202,-.035),(-.060,.180,-.010)],[.022,.018,.014],12))
 p.append(ellipsoid('Foot / tucked second owl pad',PINK,(-.060,.177,-.010),(.037,.017,.045),16,10))
 for j,x in enumerate((-.038,0.,.038)):
  p.append(paddle('Tail / compact ochre owl fan '+str(j),OCHRE,[(x*.5,.205,-.180),(x,.182,-.310),(x*1.35,.173,-.433+abs(x)*.2)],[.023,.031,.012],[.014,.011,.003],segments=12,steps=3))
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if i%3==1 else 0.))*scale,7) for i,v in enumerate(q['positions'])]
   q['positions']=list(struct.unpack('<'+'f'*len(q['positions']),struct.pack('<'+'f'*len(q['positions']),*q['positions'])))
   q.update(crease_normals(q,40.));q['normals']=[round(v,7) or 0. for v in q['normals']]
 return dict(name='hoothoot',version=1,coordinate_system=COORDINATES,primitives=p)
def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from hoothoot_glb import export_hoothoot
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_hoothoot())
if __name__=='__main__':main()
