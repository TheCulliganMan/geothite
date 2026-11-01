#!/usr/bin/env python3
"""Suicune standard skin with a open crest, planted paws, wind mantle and broad streaming ribbons."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from suicune_sculpt import sculpture as sculpt,HEIGHT,RIBBON_PATH
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

SPECIES=('suicune',)
REST_DIGESTS={'suicune':'c3634acaa906645bb52cef6dcc37411a87b9dbd3862b908b3d4235e7c933743e'}
PARTS={'suicune':tuple(p['part'] for p in sculpt()['primitives'])};MAX_BYTES=1024*1024

def sculpture(species):
 if species!='suicune':raise ValueError('unsupported aurora_beast')
 return sculpt()
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpt(normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(point):
 low,span=authoring_transform('suicune');return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={'suicune':(('root',None,(0.,0.,0.)),('torso',0,normalized((0.,.58,-.12))),
 ('neck',1,normalized((0.,.85,.29))),('head',2,normalized((0.,1.12,.45))),
 ('crest',3,normalized((0.,1.23,.51))),
 ('mane_base',1,normalized((0.,1.05,.22))),('mane_middle',5,normalized((0.,1.15,-.25))),('mane_tip',6,normalized((0.,1.13,-.62))),
 ('fore_left',1,normalized((-.155,.52,.23))),('fore_right',1,normalized((.155,.52,.23))),
 ('rear_left',1,normalized((-.155,.52,-.48))),('rear_right',1,normalized((.155,.52,-.48))),
 ('ribbon_left_base',1,normalized((-.12,.64,-.55))),('ribbon_left_bend',12,normalized((-.44,.54,.34))),('ribbon_left_tip',13,normalized((-.66,.67,-.51))),
 ('ribbon_right_base',1,normalized((.12,.64,-.55))),('ribbon_right_bend',15,normalized((.44,.54,.34))),('ribbon_right_tip',16,normalized((.66,.67,-.51))))}

CLIPS=(('idle',4.,85),('attack',1.,65),('hit',1.,65))


def skin_weights(species,part,position):
 if part.startswith(('Foot /','Toe /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Leg /','Leg diamond /')):
  owner=(10 if 'rear' in part or 'hind' in part else 8)+(1 if part.endswith('1') and not part.endswith('-1') else 0)
  flex=smoothstep(normalized((0.,.16,0.))[1],normalized((0.,.50,0.))[1],position[1]);return [0,owner,0,0],[1.-flex,flex,0.,0.]
 if part.startswith('Crest /'):return [4,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Mane /'):
  distance=-position[2];a=-normalized((0.,0.,.12))[2];b=-normalized((0.,0.,-.30))[2];c=-normalized((0.,0.,-.70))[2]
  if distance<b:
   blend=smoothstep(a,b,distance);return [5,6,0,0],[1.-blend,blend,0.,0.]
  blend=smoothstep(b,c,distance);return [6,7,0,0],[1.-blend,blend,0.,0.]
 if part.startswith('Ribbon /'):
  side=-1 if part.endswith('-1') else 1;owner=12 if side<0 else 15
  low,span=authoring_transform(species);raw=[position[k]*span/HEIGHT+(low if k==1 else 0.) for k in range(3)];best=float('inf');parameter=0.
  path=[(side*x,y,z) for x,y,z in RIBBON_PATH]
  for j,(a,b) in enumerate(zip(path,path[1:])):
   delta=[b[k]-a[k] for k in range(3)];length=sum(v*v for v in delta);t=max(0.,min(1.,sum((raw[k]-a[k])*delta[k] for k in range(3))/length));distance=sum((raw[k]-a[k]-delta[k]*t)**2 for k in range(3))
   if distance<best:best=distance;parameter=j+t
  if parameter<5.:
   blend=smoothstep(1.,5.,parameter);return [owner,owner+1,0,0],[1.-blend,blend,0.,0.]
  blend=smoothstep(5.,9.5,parameter);return [owner+1,owner+2,0,0],[1.-blend,blend,0.,0.]
 if part.startswith(('Body /','Body diamond /')):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Neck /','Bib /')):return [2,0,0,0],[1.,0.,0.,0.]
 return [3,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='suicune' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid aurora-beast clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*18
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t);lag=math.sin(math.tau*t-.45)+math.sin(.45)
 if clip=='idle':body=.003*breath;neck=.007*breath;head=.011*breath;turn=.014*wave;flow=.017*lag;leg=.005*breath
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.68),(.67,.10),(1.,0.)))
  late=envelope(t,((0.,0.),(.28,1.),(.53,.60),(.79,-.08),(1.,0.)));sign=1. if clip=='attack' else -.65
  body=sign*.008*early;neck=sign*.017*early;head=sign*.027*early;turn=sign*.014*late;flow=sign*.028*late;leg=sign*.009*early
 return [(0.,0.,0.),(body,0.,0.),(neck,0.,0.),(head,turn,0.),(flow*.15,flow*.1,0.),
 (flow*.35,flow*.2,0.),(flow*.7,flow*.3,0.),(flow,flow*.4,0.),
 (leg,0.,0.),(leg,0.,0.),(-leg*.8,0.,0.),(-leg*.8,0.,0.),
 (flow*.3,-flow*.2,0.),(flow*.6,-flow*.3,0.),(flow,-flow*.4,0.),
 (flow*.3,flow*.2,0.),(flow*.6,flow*.3,0.),(flow,flow*.4,0.)]


def export_aurora_beast(species,model=None):
 if species not in SPECIES:raise ValueError('unknown aurora_beast species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed aurora_beast neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct aurora_beast anatomy and standard skin v1'
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
  # Preserve exact f32 bytes, including signed zero in authored normals.
  key=(packed(values,component),kind,component,target,normalized)
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
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':species,'primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(species,clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':species+'.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early torso neck and head brace; planted paws; delayed wind mantle and ribbon tails; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/suicune_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; four planted paws; coherent head and hollow crest; articulated wind mantle and ribbon tails; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'aurora_beast exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_aurora_beast(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_aurora_beast(species):raise ValueError('noncanonical aurora_beast GLB')
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


def read_aurora_beast(path,species):
 with Path(path).open('rb') as f:return decode_aurora_beast(f.read(MAX_BYTES+1),species)
def export_suicune(model=None):return export_aurora_beast('suicune',model)
def read_suicune(path):return read_aurora_beast(path,'suicune')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/suicune.glb'));a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_suicune();a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS['suicune']),'parts',len(JOINTS['suicune']),'joints',REST_DIGESTS['suicune'])
if __name__=='__main__':main()
