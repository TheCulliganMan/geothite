#!/usr/bin/env python3
"""Check actual published actor rigs against the complete chair mesh.

The pose equations mirror new_bark_actors.rs. Native actor root remains at
(8, 0.04, 16), with no seating correction or gameplay position change.
This geometric check supplements, rather than replaces, native scene review.
"""
from pathlib import Path
import argparse,base64,gzip,hashlib,json,math
import numpy as np
ROOT=Path(__file__).resolve().parent.parent

def load(path):
 value=json.loads(path.read_bytes())
 if value.get('storage')=='geothite-model-text-chunks-v1':value=json.loads(b''.join((path.parent/c['path']).read_bytes()for c in value['chunks']))
 if value.get('storage')=='geothite-model-gzip-v1':
  raw=gzip.decompress(base64.b64decode(value['data'],validate=True));assert len(raw)==value['bytes']and hashlib.sha256(raw).hexdigest()==value['sha256'];value=json.loads(raw)
 return value

def rotation(x=0,y=0,z=0):
 cx,sx=math.cos(x),math.sin(x);cy,sy=math.cos(y),math.sin(y);cz,sz=math.cos(z),math.sin(z)
 return np.array([[cy*cz,-cy*sz,sy],[sx*sy*cz+cx*sz,-sx*sy*sz+cx*cz,-sx*cy],[-cx*sy*cz+sx*sz,cx*sy*sz+sx*cz,cx*cy]])
def target(phase,w,run):
 t=phase%math.tau/math.tau;reach=.28+(.20-.28)*run;stance=2*reach/(1.12+(2.8-1.12)*run)
 if t<stance:return reach*(1-2*t/stance)*w,.12
 swing=(t-stance)/(1-stance);tangent=-2*(1-stance)/stance
 z=-1+tangent*swing+(6-3*tangent)*swing*swing+(2*tangent-4)*swing**3;lift=math.sin(math.pi*swing)**2
 return reach*z*w,.12+(.060*lift+(.170*1.04*lift/(.04+lift)-.060*lift)*run)*w

def posed(rig,library,phase,elapsed,w=0,run=0):
 translations=[np.array(j['translation'],dtype=float)for j in rig['joints']];rots=[np.eye(3)for _ in translations];scales=[np.ones(3)for _ in translations]
 walk_drop=.065+math.cos(phase)**2*.030;run_drop=.068+math.cos(phase)**2*.025
 hip=-.012-(walk_drop+(run_drop-walk_drop)*run-.012)*w*w;translations[0][1]+=hip
 breath=math.sin(elapsed*2.1)*.0025*(1-w*.65);translations[1][1]+=breath
 rots[1]=rotation((.065+.12*run)*w,math.cos(phase)*.045*w,math.sin(phase)*.025*w)
 rots[2]=rotation(-(.038+.07*run)*w+math.sin(elapsed*1.4)*.008*(1-w),-math.cos(phase)*.030*w,-math.sin(phase)*.018*w)
 t=elapsed%4.8;scales[3][1]=1-math.sin(t/.145*math.pi)**2*.94 if t<.145 else 1
 for side,upper,forearm,hand,thigh,shin,shoe in[(-1,4,5,6,10,11,12),(1,7,8,9,13,14,15)]:
  legphase=phase+(0 if side<0 else math.pi);z,y=target(legphase,w,run);down=.66+hip-y;distance=min(max(math.hypot(down,z),.001),.54-.0001)
  knee=math.pi-math.acos(max(-1,min(1,(2*.27*.27-distance*distance)/(2*.27*.27))));angle=math.atan2(-z,down)-math.acos(max(-1,min(1,distance/.54)))
  rots[thigh]=rotation(angle);rots[shin]=rotation(knee);rots[shoe]=rotation(-angle-knee)
  swing=math.cos(legphase)*(.48+.12*run)*w
  rots[upper]=rotation(swing-.055+breath*2,0,side*.035);rots[forearm]=rotation(-.13-.65*run*w-abs(min(swing,0))*.28);rots[hand]=rotation(.045+swing*.08)
 world=[];parts=[]
 for i,j in enumerate(rig['joints']):
  local=np.eye(4);local[:3,:3]=rots[i]@np.diag(scales[i]);local[:3,3]=translations[i]
  world.append(local if j['parent']is None else world[j['parent']]@local)
  for p in j['primitives']:
   vs=np.array(library['geometries'][p['geometry']]['positions']).reshape(-1,3)
   parts.append((j['name'],(vs@world[i][:3,:3].T+world[i][:3,3])*16))
 return parts

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--assets-root',type=Path,default=ROOT);args=parser.parse_args()
 models=args.assets_root/'crates/crystal-voxel-view/models';library=load(models/'johto_characters/shared.geometry.json');chair=load(ROOT/'crates/crystal-voxel-view/models/facility_tables/facility_square_back_chair.mesh.json')
 chair_parts=[]
 for p in chair['primitives']:
  vs=np.array(p['positions']).reshape(-1,3)+np.array([2,0,0]);chair_parts.append((p['name'],vs.min(axis=0),vs.max(axis=0)))
 chair_lows=np.array([p[1]for p in chair_parts]);chair_highs=np.array([p[2]for p in chair_parts])
 checks=0
 for actor in['gentleman','gym_guide','receptionist','scientist','rocket','trainer','trainer_female']:
  file=models/'johto_characters'/(actor+'.rig.json')
  if not file.exists():
   assert actor in['trainer','trainer_female'];continue
  rig=load(file)
  for motion,w,run in ([('idle',0,0)] if actor=='gentleman' else [('idle',0,0),('walk',1,0),('run',1,1)]):
   for sample in range(24):
    phase=sample*math.tau/24;parts=posed(rig,library,phase,sample*4.8/24,w,run)
    for degree in range(0,360,5):
     r=rotation(y=math.radians(degree))
     for label,vs in parts:
      v=vs@r.T+np.array([8,.04,16]);low=v.min(axis=0);high=v.max(axis=0)
      overlap=np.all(np.minimum(high,chair_highs)-np.maximum(low,chair_lows)>1e-5,axis=1)
      assert not overlap.any(),(actor,motion,sample,degree,label,chair_parts[np.flatnonzero(overlap)[0]][0],low,high)
     checks+=1
  print('PASS',actor,'actual pose hierarchy; 24 phases x 72 yaws;', 'idle' if actor=='gentleman' else 'idle/walk/run',flush=True)
 print('PASS',checks,'native-foot actor poses clear all original chair components')
if __name__=='__main__':main()
