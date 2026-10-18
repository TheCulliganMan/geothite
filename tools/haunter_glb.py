#!/usr/bin/env python3
"""Canonical spiked spectral head and detached hands with six independently curling claws."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from haunter_sculpt import sculpture,HEIGHT,CENTER,palm_center,finger_root
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
MAX_BYTES=1024*1024
SOURCE=sculpture();RAW=sculpture(normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
PARTS=tuple(p['part'] for p in SOURCE['primitives'])
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGEST='de932449c4d842c05d74073ea292ea5b1a81cc4e565110568e11bef78b047f73'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=[('root',None,(0.,0.,0.)),('face_core',0,CENTER),
 ('ear_left',1,(-.22,.79,-.025)),('ear_right',1,(.22,.79,-.025)),
 ('shroud_left',1,(-.30,.52,-.025)),('shroud_right',1,(.30,.52,-.025)),
 ('hem_left',1,(-.16,.25,-.025)),('hem_right',1,(.16,.25,-.025))]
for side in (-1,1):
 hand=len(BASE);BASE.append(('hand_left' if side<0 else 'hand_right',0,palm_center(side)))
 for j in range(3):BASE.append(('finger_'+str(side)+'_'+str(j),hand,finger_root(side,j)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 if part.startswith('Hand /'):return [8 if part.endswith('-1') else 12,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Finger /'):
  side,j=map(int,part.split()[-2:]);return [(9 if side<0 else 13)+j,0,0,0],[1.,0.,0.,0.]
 if not part.startswith('Body /'):return [1,0,0,0],[1.,0.,0.,0.]
 x=point[0]/SCALE;y=point[1]/SCALE+LOW;side=0 if x<0 else 1;w=0.;joint=1
 if y>.72:
  joint=2+side;w=smoothstep(.74,.94,y)*smoothstep(.12,.25,abs(x))
 elif y>.33:
  joint=4+side;w=smoothstep(.29,.46,abs(x))
 elif y<.275:
  joint=6+side;w=(1.-smoothstep(.12,.275,y))*smoothstep(.10,.19,abs(x))
 return [1,joint,0,0],[1.-w,w,0.,0.]

def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Haunter clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4)
  turn=.020*a;spine=.035*b;reach=.040*a;spread=.025*b;curl=.065*b
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)))
  gain=1. if clip=='attack' else -.65;turn=.060*a*gain;spine=.100*b*gain;reach=.160*a*gain;spread=.120*b*gain;curl=.280*b*gain
 poses=[(0.,0.,0.),(0.,turn,0.)]
 for i in range(6):poses.append((0.,spine*(1. if i%2 else -1.)*(1. if i<4 else .7),0.))
 for side in (-1,1):
  poses.append((-reach,side*spread*.7,side*spread))
  for j in range(3):poses.append((curl*(.80+.15*j),0.,side*.08*curl*(j-1)))
 return poses
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_haunter(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='haunter' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Haunter anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Haunter continuous spiked spectral head, fitted grin and detached six-claw hands skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'haunter/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'haunter','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'haunter','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'haunter','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'haunter/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'haunter','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'haunter.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent fitted ghost face and purple-toothed grin; six blended shroud hinges; independent detached hands and six curling claws; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'haunter','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/haunter_sculpt.py','geometry_preservation':'original continuous spiked ghost skull and ragged shroud; white angular gaze, vertical black pupils, pink grin and eight pointed purple teeth; detached three-clawed hands',
    'animation_contract':'identity authoring root; standard render-only hovering clearance; coherent fitted face; six smoothly blended ear/shroud/hem hinges; independent detached hands with three curling digit carriers each; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Haunter exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Haunter exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Haunter exceeds existing byte limit')
 return blob


def decode_haunter(blob):
 if len(blob)>MAX_BYTES or blob!=export_haunter():raise ValueError('noncanonical Haunter GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'haunter','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_haunter(path):
 with Path(path).open('rb') as f:return decode_haunter(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_haunter();(a.out/'haunter.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
