#!/usr/bin/env python3
"""Original thunder bird with angular gold wings and black lightning underfeathers."""
import argparse,json,math
from pathlib import Path
from copy import deepcopy
from functools import lru_cache
from sculpt_geometry import loft,curved_profiles,ellipsoid,crease_normals
from raptor_bird_sculpt import sheet,paint,feet
from murkrow_sculpt import bill
from bovine_sculpt import oval
from chikorita_sculpt import linear
from skin_glb import COORDINATES
SPECIES=('zapdos',);HEIGHT=1.28
GOLD=(.98,.81,.28);LIGHT=(1.,.86,.37);OCHRE=(.88,.63,.23)
BLACK=(.075,.095,.12);GREY=(.49,.43,.34);IVORY=(.96,.95,.87)

def wing_depth(y,x):return -.045-.12*abs(x)+.035*math.sin((y-.5)*3)

def wings(p):
 for s in (-1,1):
  xy=[(.095,.635),(.29,.856),(.49,1.012),(.65,1.244),(.63,1.039),
      (1.34,1.414),(.91,.983),(1.14,.975),(.61,.704),(.87,.497),(.41,.653),(.24,.587)]
  p.append(sheet('Wing / gold angular thunder fan '+str(s),GOLD,[(s*x,y) for x,y in xy],wing_depth,.050,3))
  # Three broad pointed underfeathers form a second silhouette behind each fan.
  for j,outline in enumerate(([(.27,.715),(.49,1.016),(.84,1.267),(.68,.916)],
                             [(.40,.697),(.67,.971),(1.30,1.188),(.91,.860)],
                             [(.29,.637),(.61,.773),(.96,.638),(.48,.580)])):
   p.append(sheet(f'Underfeather / black lightning blade {s} {j}',BLACK,[(s*x,y) for x,y in outline],lambda y,x:wing_depth(y,x)-.031,.028,2))
  # Broad shallow shoulder vane adds volume without rounding away the zigzags.
  xy=[(.105,.643),(.29,.83),(.47,.981),(.41,.845),(.62,.882),(.44,.717),(.25,.655)]
  parent=p[-4]
  p.append(paint('Covert / fitted angular gold shoulder '+str(s),LIGHT,[(s*x,y) for x,y in xy],parent,.008,3))

def tail(p):
 # The fan sweeps backward; black tips remain visible below its five gold points.
 depth=lambda y,x:-.135-1.70*(.38-y)
 black=[(-.064,.392),(-.245,.238),(-.167,.239),(-.212,.086),(-.068,.157),(0,.023),(.068,.157),(.212,.086),(.167,.239),(.245,.238),(.064,.392)]
 gold=[(-.060,.399),(-.261,.273),(-.138,.274),(-.184,.127),(-.055,.199),(0,.059),(.055,.199),(.184,.127),(.138,.274),(.261,.273),(.060,.399)]
 p.append(sheet('Tail / black five-point lightning underfan',BLACK,black,lambda y,x:depth(y,x)-.024,.031,3))
 p.append(sheet('Tail / gold five-point lightning fan',GOLD,gold,depth,.037,3))

@lru_cache(maxsize=1)
def raw_sculpture(species='zapdos'):
 if species not in SPECIES:raise ValueError('unsupported thunder bird')
 body=crease_normals(loft('Body / lean tapering thunder-bird breast',GOLD,curved_profiles(((.204,.028,.036,-.028),(.303,.076,.090,-.020),(.427,.123,.128,-.004),(.564,.150,.130,.008),(.690,.089,.083,.014),(.739,.029,.030,.013)),4),40),45)
 neck=crease_normals(loft('Neck / upright narrow gold throat',GOLD,curved_profiles(((.64,.072,.069,.011),(.78,.065,.066,.016),(.872,.076,.070,.020)),4),32),45)
 head=ellipsoid('Head / compact thunder-bird cranium',GOLD,(0,.929,.018),(.151,.151,.137),40,26);p=[body,neck,head]
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.030,.994),(.117,1.011),(.123,.973),(.074,.947),(.044,.955))]
  socket=paint('Eye socket / sharp fitted black thunder gaze '+str(s),BLACK,xy,head,.009,3);p.append(socket)
  cx=cy=0.
  for x,y in xy:cx+=x;cy+=y
  cx/=len(xy);cy/=len(xy)
  white=paint('Eye / fitted narrow ivory lens '+str(s),IVORY,[(cx+.77*(x-cx),cy+.72*(y-cy)) for x,y in xy],socket,.005,3);p.append(white)
  pupil=paint('Pupil / small dark thunder stare '+str(s),BLACK,oval(s*.083,.978,.007,.015,20),white,.004,2);p.append(pupil)
  # Rooted crown points sweep back as their altitude rises.
 crowns=[ [(-.045,1.017),(-.090,1.114),(-.136,1.327),(-.010,1.149),(.008,1.022)],
          [(-.027,1.022),(-.035,1.173),(-.034,1.405),(.060,1.171),(.045,1.021)],
          [(.010,1.022),(.055,1.156),(.246,1.348),(.139,1.093),(.078,1.015)] ]
 for j,xy in enumerate(crowns):p.append(sheet('Crown / three long lightning points '+str(j),GOLD,xy,lambda y,x:.068-.37*(y-1.02),.037,2))
 for s in (-1,1):
  xy=[(.077,.993),(.18,1.042),(.328,1.093),(.205,.961),(.280,.910),(.151,.930),(.185,.823),(.077,.886)]
  p.append(sheet('Cheek / jagged gold facial ray '+str(s),GOLD,[(s*x,y) for x,y in xy],lambda y,x:.079-.16*abs(x),.038,2))
 for s in (-1,1):
  xy=[(.065,.636),(.146,.610),(.178,.503),(.132,.524),(.153,.407),(.098,.460),(.108,.303),(.055,.371),(.061,.493)]
  p.append(sheet('Breast feather / rooted jagged gold side '+str(s),GOLD,[(s*x,y) for x,y in xy],lambda y,x:.018,.042,3))
 p.append(bill('Bill / long tapered ochre upper lance',OCHRE,[(.108,.036,.887,.931),(.185,.046,.869,.925),(.408,.019,.716,.751),(.579,.0015,.599,.604)]))
 p.append(bill('Bill seam / narrow charcoal join',BLACK,[(.109,.029,.883,.890),(.185,.035,.863,.871),(.405,.014,.713,.720),(.569,.001,.601,.604)]))
 p.append(bill('Bill / long narrow ochre lower lance',OCHRE,[(.109,.029,.863,.883),(.185,.035,.844,.863),(.405,.014,.704,.713),(.569,.0015,.599,.601)]))
 wings(p);start=len(p);feet(p)
 for q in p[start:]:
  if q['part'].startswith(('Leg /','Foot /','Toe /')):q['base_color']=linear(GREY)
  elif q['part'].startswith('Talon /'):q['base_color']=linear(IVORY)
 for s in (-1,1):
  xy=[(.055,.369),(.101,.369),(.143,.293),(.120,.304),(.137,.242),(.104,.258),(.084,.240),(.061,.275)]
  p.append(sheet('Thigh / rooted grey feather spat '+str(s),GREY,[(s*x,y) for x,y in xy],lambda y,x:.010,.042,3))
 # A short jagged throat fringe follows the actual narrow neck.
 xy=[(-.049,.703),(-.039,.809),(0,.828),(.039,.809),(.049,.703),(.030,.723),(.022,.675),(0,.702),(-.020,.675),(-.030,.723)]
 p.append(paint('Throat / fitted jagged feather fringe',LIGHT,xy,neck,.008,3))
 tail(p)
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def sculpture(species='zapdos',normalize=True):
 model=deepcopy(raw_sculpture(species));p=model['primitives']
 if not normalize:return model
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return model

def main():
 from zapdos_glb import export_zapdos
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True)
 (args.output/'zapdos.glb').write_bytes(export_zapdos())
if __name__=='__main__':main()
