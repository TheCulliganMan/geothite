#!/usr/bin/env python3
"""Canonical lightning fox with planted paws, sharp ears and independent white needle collar."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from jolteon_sculpt import sculpture,HEIGHT,CORE_DROP
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
REST_DIGEST='3e154fb857ffb4531d84ef1cf35b3ec944ab4d59bbb5b0da316722aa0f6af022'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
def core(p):return (p[0],p[1]-CORE_DROP,p[2])
BASE=[('root',None,(0.,0.,0.)),('body',0,core((0.,.43,-.06))),
 ('neck',1,core((0.,.53,.13))),('head',2,core((0.,.67,.21))),
 ('ear_left',3,core((-.090,.754,.160))),('ear_right',3,core((.090,.754,.160)))]
for side in (-1,1):
 for row in (0,1):BASE.append(('leg_'+str(side)+'_'+str(row),0,(side*(.087 if row==0 else .128),.23-CORE_DROP*smoothstep(.056,.350,.23),.132 if row==0 else -.220)))
BASE.append(('neck_ruff',2,core((0.,.555,.10))))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=[v/SCALE+(LOW if i==1 else 0.) for i,v in enumerate(point)]
 if part.startswith(('Paw /','Toe crease /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Haunch /','Rump quill /','Back quill /')):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Ruff /','Ruff tuft /')):return [10,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Neck /'):
  a=smoothstep(.41,.51,y);b=smoothstep(.51,.58,y);return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Ear /','Ear inset /')):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=6+(0 if side<0 else 2)+row
  a=smoothstep(.13,.34,y);g=.70*smoothstep(.055,.140,y)*(1.-smoothstep(.300,.400,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 return [3,0,0,0],[1.,0.,0.,0.]

def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Jolteon clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);body=.002*a;neck=.006*b;head=.012*b;turn=.018*a;ears=.035*b;ruff=.025*a;leg=.006*a
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=1. if clip=='attack' else -.65
  body=.007*a*gain;neck=.018*a*gain;head=.040*b*gain;turn=.030*b*gain;ears=.075*b*gain;ruff=.060*b*gain;leg=.085*a*gain
 return [(0.,0.,0.),(body,0.,0.),(neck,0.,0.),(head,turn,0.),
  (ears*.35,ears*.20,-ears),(ears*.35,-ears*.20,ears),
  (leg,0.,0.),(-leg*.75,0.,0.),(-leg*.85,0.,0.),(leg*.65,0.,0.),(ruff,0.,0.)]

def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_jolteon(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='jolteon' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Jolteon anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Jolteon lightning fox, white needle collar, long legs and bristled rump skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'jolteon/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'jolteon','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'jolteon','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'jolteon','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'jolteon/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'jolteon','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'jolteon.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'angular violet gaze; sharp alert ears; softly articulated long legs with planted paws; independent needle collar; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'jolteon','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/jolteon_sculpt.py','geometry_preservation':'original lean yellow fox, angular violet gaze, sharp ears with violet insets, white needle collar, four long legs and occupied rump bristles',
    'animation_contract':'identity authoring root; four fixed ground paws; blended long limbs; coherent head, cheeks, muzzle and gaze; independent ears and white needle collar; coherent torso bristles; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Jolteon exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Jolteon exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Jolteon exceeds existing byte limit')
 return blob


def decode_jolteon(blob):
 if len(blob)>MAX_BYTES or blob!=export_jolteon():raise ValueError('noncanonical Jolteon GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'jolteon','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_jolteon(path):
 with Path(path).open('rb') as f:return decode_jolteon(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_jolteon();(a.out/'jolteon.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
