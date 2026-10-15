#!/usr/bin/env python3
"""Slowpoke standard skin with a coherent sleepy head, planted feet and a trailing tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from slowpoke_sculpt import sculpture as sculpt,HEIGHT,TAIL_POINTS
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

SPECIES=('slowpoke',)
REST_DIGESTS={'slowpoke':'e6b60180c57a253da129803d01cb7856b76599b67a51e6ac3f8d06b4e1e38c64'}
PARTS={'slowpoke':tuple(p['part'] for p in sculpt()['primitives'])};MAX_BYTES=1024*1024

def sculpture(species):
 if species!='slowpoke':raise ValueError('unsupported sleepy_quadruped')
 return sculpt()
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpt(normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(point):
 low,span=authoring_transform('slowpoke');return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={'slowpoke':(('root',None,(0.,0.,0.)),('body',0,normalized((0.,.25,-.10))),
 ('head',1,normalized((0.,.37,.18))),('tail_base',1,normalized(TAIL_POINTS[0])),('tail_tip',3,normalized(TAIL_POINTS[2])),
 ('ear_left',2,normalized((-.216,.605,.221))),('ear_right',2,normalized((.216,.605,.221))),
 ('fore_left',1,normalized((-.219,.23,.225))),('fore_right',1,normalized((.219,.23,.225))),
 ('rear_left',1,normalized((-.219,.23,-.29))),('rear_right',1,normalized((.219,.23,-.29))))}

CLIPS=(('idle',4.,85),('attack',1.,65),('hit',1.,65))


def skin_weights(species,part,position):
 if part.startswith(('Foot /','Toe /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  owner=(9 if 'rear' in part else 7)+(1 if part.endswith('1') and not part.endswith('-1') else 0)
  flex=smoothstep(normalized((0.,.06,0.))[1],normalized((0.,.21,0.))[1],position[1]);return [0,owner,0,0],[1.-flex,flex,0.,0.]
 if part.startswith(('Ear /','Ear inset /')):return [5 if part.endswith('-1') else 6,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail tip /'):return [4,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  low,span=authoring_transform(species);raw=[position[k]*span/HEIGHT+(low if k==1 else 0.) for k in range(3)];best=float('inf');parameter=0.
  for j,(a,b) in enumerate(zip(TAIL_POINTS,TAIL_POINTS[1:])):
   delta=[b[k]-a[k] for k in range(3)];length=sum(v*v for v in delta);t=max(0.,min(1.,sum((raw[k]-a[k])*delta[k] for k in range(3))/length))
   distance=sum((raw[k]-a[k]-delta[k]*t)**2 for k in range(3))
   if distance<best:best=distance;parameter=j+t
  blend=smoothstep(.8,2.3,parameter);return [3,4,0,0],[1.-blend,blend,0.,0.]
 if part.startswith('Body /'):return [1,0,0,0],[1.,0.,0.,0.]
 return [2,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='slowpoke' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid sleepy-quadruped clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*11
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t);lag=math.sin(math.tau*t-.45)+math.sin(.45)
 if clip=='idle':body=.004*breath;head=.010*breath;turn=.018*wave;tail=.018*lag;ear=.011*lag;leg=.006*breath
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.68),(.67,.10),(1.,0.)))
  late=envelope(t,((0.,0.),(.28,1.),(.53,.60),(.79,-.08),(1.,0.)));sign=1. if clip=='attack' else -.65
  body=sign*.010*early;head=sign*.034*early;turn=sign*.014*late;tail=sign*.032*late;ear=sign*.016*late;leg=sign*.009*early
 return [(0.,0.,0.),(body,0.,0.),(head,turn,0.),(tail*.4,tail*.5,0.),(tail,tail*.6,0.),
 (ear,0.,-ear*.2),(ear,0.,ear*.2),(leg,0.,0.),(leg,0.,0.),(-leg*.8,0.,0.),(-leg*.8,0.,0.)]


def export_sleepy_quadruped(species,model=None):
 if species not in SPECIES:raise ValueError('unknown sleepy_quadruped species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed sleepy_quadruped neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct sleepy_quadruped anatomy and standard skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS[species]
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':species+'/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':species,'mesh':0,'skin':0})
 g.doc['scenes']=[{'name':species,'nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':species,'skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(tuple(values),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':species+'/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(species,part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early low body and sleepy head brace; grounded feet; trailing ears and pale-tipped tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/slowpoke_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; four planted feet and claws; coherent sleepy head and muzzle; articulated ears, legs and thick raised tail; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('sleepy_quadruped exceeds existing accessor limit')
 return g.bytes()


def decode_sleepy_quadruped(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_sleepy_quadruped(species):raise ValueError('noncanonical sleepy_quadruped GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_sleepy_quadruped(path,species):
 with Path(path).open('rb') as f:return decode_sleepy_quadruped(f.read(MAX_BYTES+1),species)
def export_slowpoke(model=None):return export_sleepy_quadruped('slowpoke',model)
def read_slowpoke(path):return read_sleepy_quadruped(path,'slowpoke')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/slowpoke.glb'));a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_slowpoke();a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS['slowpoke']),'parts',len(JOINTS['slowpoke']),'joints',REST_DIGESTS['slowpoke'])
if __name__=='__main__':main()
