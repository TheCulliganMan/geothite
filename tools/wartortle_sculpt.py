#!/usr/bin/env python3
"""Original feather-eared turtle with fitted shell plates and a broad cloud tail."""
import argparse,json,math
from pathlib import Path
from sculpt_geometry import ellipsoid,part,crease_normals
from bovine_sculpt import reverse_front,oval,curved_tube
from lugia_sculpt import paddle
from raptor_bird_sculpt import paint,sheet
from wing_insect_sculpt import contour
from skin_glb import COORDINATES
HEIGHT=1.10;SPECIES=('wartortle',)
BLUE=(.46,.56,.81);CLOUD=(.70,.88,.94);LIGHT=(.85,.95,.97)
BROWN=(.49,.35,.23);SEAM=(.29,.24,.19);TAN=(.97,.88,.63)
WHITE=(.98,.98,.92);INK=(.065,.075,.105);IRIS=(.49,.28,.23)

def ribbon(points,width):
 left=[];right=[]
 for j,(x,y) in enumerate(points):
  a=points[max(0,j-1)];b=points[min(len(points)-1,j+1)];dx=b[0]-a[0];dy=b[1]-a[1];length=math.hypot(dx,dy);nx=-dy/length*width;ny=dx/length*width
  left.append((x+nx,y+ny));right.append((x-nx,y-ny))
 return left+list(reversed(right))

def rim():
 ns=64;nt=12;v=[]
 for i in range(ns):
  a=i*math.tau/ns
  for j in range(nt):
   t=j*math.tau/nt;v.append(((.244+.018*math.cos(t))*math.cos(a),.44+(.317+.018*math.cos(t))*math.sin(a),-.035+.018*math.sin(t)))
 f=[]
 for i in range(ns):
  for j in range(nt):f.append((i*nt+j,((i+1)%ns)*nt+j,((i+1)%ns)*nt+(j+1)%nt,i*nt+(j+1)%nt))
 return crease_normals(part('Shell rim / continuous ivory oval edge',WHITE,v,f,True),45)

def ears(p):
 for s in (-1,1):
  ear=paddle('Ear / long pale feather plume '+str(s),CLOUD,[(s*.132,.917,.068),(s*.202,1.065,.040),(s*.280,1.205,.005),(s*.310,1.256,-.007)],[.043,.063,.034,.002],[.033,.034,.026,.002]);p.append(ear)
  p.append(paddle('Ear lobe / rooted pale lower feather '+str(s),CLOUD,[(s*.157,.954,.069),(s*.216,.963,.044),(s*.251,1.016,.025)],[.032,.033,.002],[.027,.027,.002]))
  xy=[(s*x,y) for x,y in ((.170,.991),(.191,1.087),(.275,1.216),(.248,1.101),(.209,1.014))]
  p.append(paint('Ear inset / fitted ivory feather fold '+str(s),LIGHT,xy,ear,.005,3))

def tail(p):
 xy=contour([((-.036,.283),(.031,.253),(.178,.300),(.269,.371)),
             ((.269,.371),(.382,.420),(.479,.556),(.461,.660)),
             ((.461,.660),(.444,.704),(.404,.725),(.386,.718)),
             ((.386,.718),(.390,.730),(.405,.737),(.412,.754)),
             ((.412,.754),(.395,.843),(.315,.879),(.258,.842)),
             ((.258,.842),(.171,.792),(.170,.694),(.200,.602)),
             ((.200,.602),(.231,.503),(.179,.459),(.128,.423)),
             ((.128,.423),(.063,.378),(-.035,.362),(-.036,.283))],7)
 q=sheet('Tail / broad two-curl pale cloud fan',CLOUD,xy,lambda y,x:-.167-.61*(y-.283)+.025*math.sin(5*x)*math.sin(4*(y-.283)),.075,3);p.append(q)
 # Actual fitted closed ribbons trace the two curls on both sides.
 for face in (-1,1):
  parent=q if face==1 else reverse_front(q)
  for j,path in enumerate(([(.242,.578),(.222,.678),(.239,.766),(.286,.812),(.330,.795),(.344,.760)],[(.331,.508),(.390,.561),(.407,.610),(.382,.665),(.343,.672)])):
   xy=ribbon(path,.004)
   patch=paint(f'Tail fold / supported pale spiral {j} {face}',LIGHT,xy if face==1 else [(-x,y) for x,y in xy],parent,.005,3)
   p.append(patch if face==1 else reverse_front(patch))

def limbs(p):
 for s in (-1,1):
  p.append(paddle('Arm / bent blue-violet turtle forelimb '+str(s),BLUE,[(s*.18,.601,.026),(s*.30,.552,.089),(s*.385,.537,.136)],[.083,.077,.078],[.081,.078,.078]))
  p.append(ellipsoid('Hand / broad blue-violet turtle paw '+str(s),BLUE,(s*.393,.531,.144),(.088,.083,.091),32,22))
  for j,dx in enumerate((-.040,0,.040)):
   p.append(curved_tube(f'Hand claw / three ivory turtle points {s} {j}',WHITE,[(s*.393+dx,.52,.203),(s*.393+dx*.9,.515,.235),(s*.393+dx*.8,.49,.256)],[.017,.013,.002],20))
  p.append(ellipsoid('Haunch / powerful blue-violet turtle thigh '+str(s),BLUE,(s*.171,.184,.041),(.115,.146,.129),36,24))
  p.append(paddle('Leg / rooted short turtle ankle '+str(s),BLUE,[(s*.184,.185,.065),(s*.195,.096,.093),(s*.199,.046,.142)],[.065,.067,.072],[.070,.074,.050]))
  p.append(ellipsoid('Foot / grounded blue-violet turtle paw '+str(s),BLUE,(s*.199,.029,.152),(.090,.029,.100),32,20))
  for j,dx in enumerate((-.051,0,.051)):
   p.append(curved_tube(f'Toe / three blunt blue turtle points {s} {j}',BLUE,[(s*.199+dx*.5,.031,.199),(s*.199+dx,.021,.245),(s*.199+dx,.012,.263)],[.029,.020,.0025],20))

def sculpture(species='wartortle'):
 if species not in SPECIES:raise ValueError('unsupported feather turtle')
 body=ellipsoid('Body / blue-violet mantle beneath turtle plates',BLUE,(0,.44,.016),(.232,.309,.130),44,28)
 shell=ellipsoid('Shell / substantial brown turtle dome',BROWN,(0,.44,-.085),(.265,.337,.155),48,30)
 belly=ellipsoid('Belly / gently domed pale plated breast',TAN,(0,.44,.070),(.226,.300,.118),48,30)
 head=ellipsoid('Head / compact blue-violet turtle cranium',BLUE,(0,.817,.088),(.188,.167,.172),48,30)
 muzzle=ellipsoid('Muzzle / broad low blue-violet turtle snout',BLUE,(0,.746,.209),(.164,.074,.077),44,26);p=[body,shell,belly,rim(),head,muzzle]
 # Belly seams and rear shell seams follow actual stored triangles.
 for j,path in enumerate(([(-.188,.536),(-.100,.512),(0,.503),(.10,.512),(.188,.536)],[(-.153,.346),(-.083,.326),(0,.320),(.083,.326),(.153,.346)],[(0,.699),(0,.590),(0,.480),(0,.370),(0,.190)])):
  p.append(paint('Belly seam / fitted turtle plate join '+str(j),SEAM,ribbon(path,.003),belly,.004,3))
 rear=reverse_front(shell)
 hexagon=[(0,.696),(-.137,.615),(-.158,.428),(0,.304),(.158,.428),(.137,.615)]
 paths=[[a,b] for a,b in zip(hexagon,hexagon[1:]+hexagon[:1])]+[[(0,.304),(0,.132)],[(-.137,.615),(-.180,.643)],[(.137,.615),(.180,.643)],[(-.158,.428),(-.235,.338)],[(.158,.428),(.235,.338)]]
 for j,path in enumerate(paths):
  q=paint('Shell seam / fitted rear turtle scute '+str(j),SEAM,ribbon(path,.003),rear,.004,3);p.append(reverse_front(q))
 for s in (-1,1):
  xy=[(s*x,y) for x,y in ((.025,.862),(.132,.912),(.149,.856),(.110,.791),(.055,.795),(.029,.817))]
  socket=paint('Eye socket / fitted pointed turtle stare '+str(s),INK,xy,head,.008,3);p.append(socket)
  cx=sum(x for x,y in xy)/len(xy);cy=sum(y for x,y in xy)/len(xy)
  eye=paint('Eye / fitted ivory turtle lens '+str(s),WHITE,[(cx+.91*(x-cx),cy+.91*(y-cy)) for x,y in xy],socket,.005,3);p.append(eye)
  iris=paint('Iris / warm brown turtle gaze '+str(s),IRIS,oval(s*.095,.843,.018,.030,24),eye,.004,3);p.append(iris)
  pupil=paint('Pupil / restrained dark turtle center '+str(s),INK,oval(s*.095,.845,.010,.026,24),iris,.003,3);p.append(pupil)
  p.append(paint('Glint / tiny ivory turtle eye '+str(s),WHITE,oval(s*.095-.004,.858,.003,.006,16),pupil,.002,2))
  p.append(curved_tube('Fang / small ivory turtle point '+str(s),WHITE,[(s*.110,.717,.260),(s*.110,.699,.267),(s*.108,.685,.267)],[.014,.012,.002],20))
 mouth=[(-.128,.726),(-.076,.710),(0,.701),(.076,.710),(.128,.726),(.110,.704),(.057,.690),(0,.686),(-.057,.690),(-.110,.704)]
 p.append(paint('Mouth / fitted dark turtle smile',INK,mouth,muzzle,.004,3))
 for s in (-1,1):p.append(paint('Nostril / small fitted turtle dot '+str(s),INK,oval(s*.040,.76,.004,.003,16),muzzle,.003,2))
 ears(p);tail(p);limbs(p)
 lo=min(v for q in p for v in q['positions'][1::3]);hi=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(hi-lo)
 for q in p:
  q['positions']=[round((v-(lo if i%3==1 else 0))*scale,7) for i,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':p}

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args();args.output.mkdir(parents=True,exist_ok=True);(args.output/'wartortle.mesh.json').write_text(json.dumps(sculpture(),separators=(',',':')))
if __name__=='__main__':main()
