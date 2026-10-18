#!/usr/bin/env python3
"""Canonical Celebi skin with coherent fairy gaze, antennae, fingers and wing leaves."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from celebi_sculpt import sculpture,HEIGHT
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
REST_DIGEST='94a8a577208273651ba7294a48345f701b30142a260b5fc2e920b21e252837fb'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.33,-.012)),('head',1,(0.,.52,.03)),
 ('arm_left',1,(-.085,.439,.006)),('arm_right',1,(.085,.439,.006)),
 ('wing_left',1,(-.072,.42,-.078)),('wing_right',1,(.072,.42,-.078)),
 ('antenna_left',2,(-.090,.90,.16)),('antenna_right',2,(.090,.90,.16)),
 ('leg_left',0,(-.075,.17,.006)),('leg_right',0,(.075,.17,.006)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Foot /'):return rigid(0)
 if part.startswith('Leg /'):
  a=smoothstep(.11,.25,y);g=.20*smoothstep(.11,.15,y)*(1.-smoothstep(.20,.255,y))
  return [0,1,9 if part.endswith('-1') else 10,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Arm /','Hand /','Finger /')):
  side=part.split()[-2] if part.startswith('Finger /') else part.split()[-1]
  return rigid(3 if side=='-1' else 4)
 if part.startswith('Wing /'):return rigid(5 if part.endswith('-1') else 6)
 if part.startswith(('Antenna /','Antenna tip /')):return rigid(7 if part.endswith('-1') else 8)
 if part.startswith(('Head /','Head groove /','Eye','Iris /','Pupil /','Glint /','Mouth /')):return rigid(2)
 return rigid(1)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Celebi clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);flutter=math.sin(6.*math.tau*t);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.;flutter=a*strength
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.012*a,0.),(.015*b,.025*a,.010*b),
  (.015*b,.020*a,.060*b),(.015*b,-.020*a,-.060*b),
  (.025*a,.140*flutter,.030*a),(.025*a,-.140*flutter,-.030*a),
  (.012*b,.014*b,.018*b),(.012*b,-.014*b,-.018*b),
  (0.,.010*a,0.),(0.,-.010*a,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_celebi(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='celebi' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Celebi anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Celebi swept onion fairy, antennae and wing-leaf skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'celebi/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'celebi','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'celebi','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'celebi','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'celebi/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'celebi','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'celebi.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'seated fairy feet; coherent fitted gaze and onion folds; independent antennae and leaf-wing flutter; hands and fingers share arm carriers; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'celebi','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/celebi_sculpt.py','geometry_preservation':'original swept onion head, fitted gaze, blue-tipped antennae, rounded wing leaves and three-finger hands',
    'animation_contract':'identity floor root; fixed rounded feet and supported shanks; yawing torso; rigid gaze and folds on head; independent antennae and wing leaves; coherent three-finger hands; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Celebi exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Celebi exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Celebi exceeds existing byte limit')
 return blob


def decode_celebi(blob):
 if len(blob)>MAX_BYTES or blob!=export_celebi():raise ValueError('noncanonical Celebi GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'celebi','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_celebi(path):
 with Path(path).open('rb') as f:return decode_celebi(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_celebi();(a.out/'celebi.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
