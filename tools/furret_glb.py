#!/usr/bin/env python3
"""Furret standard skin with a continuous banded body, planted paws and curved tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from furret_sculpt import sculpture as sculpt,HEIGHT,TAIL_KNOTS
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

SPECIES=('furret',)
REST_DIGESTS={'furret':'8fb8689f2753aaa7997768281386e3a57aa2c1370eb3187611fb24df924ee471'}
PARTS={'furret':tuple(p['part'] for p in sculpt()['primitives'])};MAX_BYTES=1024*1024

def sculpture(species):
 if species!='furret':raise ValueError('unsupported banded_ferret')
 return sculpt()
@lru_cache(maxsize=1)
def authoring_transform(species):
 raw=sculpt(normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(point):
 low,span=authoring_transform('furret');return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
JOINTS={'furret':(('root',None,(0.,0.,0.)),('pelvis',0,normalized((0.,.270,-.130))),
 ('chest',1,normalized((0.,.570,.010))),('neck',2,normalized((0.,.870,.125))),('head',3,normalized((0.,.980,.165))),
 ('ear_left',4,normalized((-.140,1.200,.105))),('ear_right',4,normalized((.140,1.200,.105))),
 ('arm_left',2,normalized((-.095,.762,.074))),('arm_right',2,normalized((.095,.762,.074))),
 ('leg_left',1,normalized((-.149,.237,-.085))),('leg_right',1,normalized((.149,.237,-.085))),
 ('tail_base',1,normalized(TAIL_KNOTS[0])),('tail_middle',11,normalized(TAIL_KNOTS[2])),('tail_tip',12,normalized(TAIL_KNOTS[4])))}

CLIPS=(('idle',4.,85),('attack',1.,65),('hit',1.,65))


def skin_weights(species,part,position):
 x,y,z=position
 if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  owner=9 if x<0 else 10;blend=smoothstep(normalized((0.,.070,0.))[1],normalized((0.,.237,0.))[1],y)
  return [0,owner,0,0],[1.-blend,blend,0.,0.]
 if part.startswith(('Arm /','Hand /')):return [7 if x<0 else 8,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Ear /','Ear tip /')):return [5 if part.endswith('-1') else 6,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Eye /','Glint /','Nose /','Mouth /','Face mark /','Back coat /','Forehead /','Tongue /')):
  ranges=[(.15,.27,0,1),(.34,.57,1,2),(.72,.87,2,3),(.94,.98,3,4)]
  for a,b,j,k in ranges:
   aa=normalized((0.,a,0.))[1];bb=normalized((0.,b,0.))[1]
   if y<aa:return [j,0,0,0],[1.,0.,0.,0.]
   if y<=bb:
    blend=smoothstep(aa,bb,y);return [j,k,0,0],[1.-blend,blend,0.,0.]
  return [4,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Tail /','Tail tuft /')):
  low,span=authoring_transform(species);raw=[position[k]*span/HEIGHT+(low if k==1 else 0.) for k in range(3)];best=float('inf');parameter=0.
  for j,(a,b) in enumerate(zip(TAIL_KNOTS,TAIL_KNOTS[1:])):
   delta=[b[k]-a[k] for k in range(3)];length=sum(v*v for v in delta);t=max(0.,min(1.,sum((raw[k]-a[k])*delta[k] for k in range(3))/length));distance=sum((raw[k]-a[k]-delta[k]*t)**2 for k in range(3))
   if distance<best:best=distance;parameter=j+t
  if parameter<2.8:
   blend=smoothstep(.8,2.8,parameter);return [11,12,0,0],[1.-blend,blend,0.,0.]
  blend=smoothstep(2.8,4.8,parameter);return [12,13,0,0],[1.-blend,blend,0.,0.]
 return [4,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='furret' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid ferret clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*14
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t);lag=math.sin(math.tau*t-.45)+math.sin(.45)
 if clip=='idle':body=.003*breath;neck=.007*breath;head=.011*breath;turn=.014*wave;flow=.017*lag;arm=.011*wave;leg=.005*breath
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.68),(.67,.10),(1.,0.)))
  late=envelope(t,((0.,0.),(.28,1.),(.53,.60),(.79,-.08),(1.,0.)));sign=1. if clip=='attack' else -.65
  body=sign*.008*early;neck=sign*.017*early;head=sign*.027*early;turn=sign*.014*late;flow=sign*.028*late;arm=sign*.025*early;leg=sign*.009*early
 return [(0.,0.,0.),(body,0.,0.),(body*.8,0.,0.),(neck,0.,0.),(head,turn,0.),
 (flow*.25,0.,-flow*.12),(flow*.25,0.,flow*.12),
 (arm,0.,-arm*.2),(arm,0.,arm*.2),(leg,0.,0.),(leg,0.,0.),
 (flow*.3,flow*.25,0.),(flow*.6,flow*.35,0.),(flow,flow*.45,0.)]


def export_banded_ferret(species,model=None):
 if species not in SPECIES:raise ValueError('unknown banded_ferret species')
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed banded_ferret neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct banded_ferret anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'early pelvis chest neck and head brace; planted paws; delayed curved tail and ears; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/furret_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; two planted hind paws; coherent fitted face; articulated long body ears forepaws and curved tail; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'banded_ferret exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_banded_ferret(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_banded_ferret(species):raise ValueError('noncanonical banded_ferret GLB')
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


def read_banded_ferret(path,species):
 with Path(path).open('rb') as f:return decode_banded_ferret(f.read(MAX_BYTES+1),species)
def export_furret(model=None):return export_banded_ferret('furret',model)
def read_furret(path):return read_banded_ferret(path,'furret')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species/furret.glb'));a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_furret();a.output.write_bytes(blob);print(len(blob),'bytes',len(PARTS['furret']),'parts',len(JOINTS['furret']),'joints',REST_DIGESTS['furret'])
if __name__=='__main__':main()
