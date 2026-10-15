#!/usr/bin/env python3
"""Original rounded Diglett capsule and fitted face emerging through separate soil."""
import argparse,math
from pathlib import Path
from sculpt_geometry import loft,curved_profiles,ellipsoid,part
from raptor_bird_sculpt import paint
from bovine_sculpt import oval
from skin_glb import COORDINATES
HEIGHT=.82
# Verified local normal/front/back RGB555 body (19,11,4), nose/soil (24,6,4).
BROWN=(156/255,90/255,33/255);RED=(198/255,49/255,33/255)
INK=(0.,0.,0.);WHITE=(1.,1.,1.)

def sculpture(*,normalize=True):
 lower=curved_profiles(((.005,.176,.157,0.),(.12,.19,.17,0.),(.26,.19,.17,0.),(.34,.19,.17,0.)),4)
 dome=[]
 for i in range(1,17):
  angle=(math.pi/2-.010)*i/16
  dome.append((.34+.22*math.sin(angle),.19*math.cos(angle),.17*math.cos(angle),0.))
 body=loft('Body / continuous rounded emerging mole capsule',BROWN,lower+dome,32)
 p=[body]
 for side in (-1,1):
  eye=paint('Eye / fitted narrow black mole gaze '+str(side),INK,oval(side*.062,.369,.014,.028,24),body,.006,3);p.append(eye)
  p.append(paint('Eye glint / fitted small white mole highlight '+str(side),WHITE,oval(side*.062-.003,.378,.0035,.0055,16),eye,.002,2))
 p.append(ellipsoid('Nose / rooted broad oval red mole nose',RED,(0.,.258,.180),(.067,.037,.047),28,16))
 p.append(paint('Forehead glint / fitted small smooth cap highlight',WHITE,oval(-.045,.482,.016,.013,20),body,.004,3))
 clods=((.255,0.,.075,.062),(.17,.14,.097,.056),(0.,.205,.081,.063),
        (-.18,.14,.072,.077),(-.245,0.,.086,.057),(-.15,-.15,.095,.062),
        (.015,-.19,.069,.065),(.17,-.135,.083,.070))
 for j,(x,z,rx,rz) in enumerate(clods):
  name='Soil / separate rooted disturbed clod '+str(j)
  q=ellipsoid(name,RED,(x,.045,z),(rx,.045,rz),10,6)
  vertices=[tuple(q['positions'][i:i+3]) for i in range(0,len(q['positions']),3)]
  faces=[tuple(q['indices'][i:i+3]) for i in range(0,len(q['indices']),3)]
  p.append(part(name,RED,vertices,faces,False))
 if normalize:
  low=min(v for q in p for v in q['positions'][1::3]);high=max(v for q in p for v in q['positions'][1::3]);scale=HEIGHT/(high-low)
  for q in p:
   q['positions']=[round((v-(low if j%3==1 else 0.))*scale,7) for j,v in enumerate(q['positions'])];q['normals']=[round(v,7) for v in q['normals']]
 return {'name':'diglett','version':1,'coordinate_system':COORDINATES,'primitives':p}

def body_reference_span():
 body=sculpture()['primitives'][0];return max(body['positions'][1::3])-min(body['positions'][1::3])

def main():
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--output',type=Path,required=True);args=a.parse_args()
 from diglett_glb import export_diglett
 args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_bytes(export_diglett())
if __name__=='__main__':main()
