#!/usr/bin/env python3
"""Standard Pichu, Cleffa and Igglybuff skins with planted feet and fitted faces."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from baby_fairy_sculpt import sculpture as fairy_sculpture,HEIGHTS as FAIRY_HEIGHTS
from pichu_sculpt import sculpture as pichu_sculpture,HEIGHT as PICHU_HEIGHT
SPECIES=('pichu','cleffa','igglybuff');HEIGHTS={**FAIRY_HEIGHTS,'pichu':PICHU_HEIGHT}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError('unknown baby species')
 return pichu_sculpture(normalize=normalize) if species=='pichu' else fairy_sculpture(species,normalize=normalize)
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=3)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'pichu': '45b03dd0a788ebf6714334c7c25ab472bab589aa9d5b70e2512ad105b7e992be', 'cleffa': '25d7809325a90fc9b89a9451ee3785256001f63b95cfb0a1568e7a3d7bd45725', 'igglybuff': 'de4ed97b7ae946d32303e70a839c3880f24b1a01e0036b72be5aa78bc97010a4'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=3)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.25,0.)),('head',1,(0.,.35,0.))]
 if species=='pichu':
  raw += [('ear_left',2,(-.16,.68,.008)),('ear_right',2,(.16,.68,.008)),('arm_left',1,(-.093,.294,.018)),('arm_right',1,(.093,.294,.018)),('tail',1,(0.,.198,-.085)),('leg_left',0,(-.087,.10,.01)),('leg_right',0,(.087,.10,.01))]
 elif species=='cleffa':
  raw += [('ear_left',2,(-.17,.63,.035)),('ear_right',2,(.17,.63,.035)),('tuft',2,(.02,.65,.06)),('arm_left',2,(-.16,.36,.12)),('arm_right',2,(.16,.36,.12)),('tail',1,(-.07,.22,-.20))]
 else:raw += [('crown',2,(0.,.70,-.02)),('arm_left',2,(-.24,.365,.085)),('arm_right',2,(.24,.365,.085))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
BANKS={s:{name:i for i,(name,parent,pivot) in enumerate(js)} for s,js in JOINTS.items()}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);b=BANKS[species]
 x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Foot /'):return rigid(0)
 if part.startswith('Leg /'):
  a=smoothstep(.06,.17,y);g=.30*smoothstep(.045,.085,y)*(1.-smoothstep(.12,.18,y));owner=b['leg_left' if part.endswith('-1') else 'leg_right']
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Ear /','Ear inset /')):return rigid(b['ear_left' if part.endswith('-1') else 'ear_right'])
 if part.startswith(('Tuft /','Tuft crease /')):return rigid(b['tuft'])
 if part.startswith('Crown /'):return rigid(b['crown'])
 if part.startswith('Arm /'):return rigid(b['arm_left' if part.endswith('-1') else 'arm_right'])
 if part.startswith('Tail /'):return rigid(b['tail'])
 if part.startswith(('Head /','Eye /','Eye glint /','Eye border /','Iris /','Pupil /','Cheek /','Nose /','Mouth /','Tongue /','Forehead /')):return rigid(2)
 a=smoothstep(.31,.39,y) if species=='pichu' else smoothstep(.22,.30,y) if species=='cleffa' else smoothstep(.20,.28,y)
 return [1,2,0,0],[1.-a,a,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid baby clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);d=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  d=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;d*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  if name=='root':angle=(0.,0.,0.)
  elif name=='torso':angle=(0.,.012*a,0.)
  elif name=='head':angle=(.007*a,.020*a,0.)
  elif name.startswith('ear_'):angle=(.012*d,0.,(-1 if name.endswith('left') else 1)*.025*d)
  elif name.startswith('arm_'):angle=(.015*a,.012*d,(-1 if name.endswith('left') else 1)*.035*a)
  elif name=='tail':angle=(.015*d,.035*d,0.)
  elif name in ('tuft','crown'):angle=(.018*d,.018*a,0.)
  else:angle=(0.,(-1 if 'right' in name else 1)*.018*a,0.)
  out.append(angle)
 return out
def export_baby(species,model=None):
 if species not in SPECIES:raise ValueError('unknown baby species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed baby neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite baby Pokémon fitted faces and planted feet skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted feet; coherent fitted faces; articulated ears, arms, curls and tails; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/baby_fairy_sculpt.py and tools/pichu_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; fixed feet; spatial torso/head field; fitted faces share head owner; articulated ears, arms, curls and tails; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'baby exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_baby(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_baby(species):raise ValueError('noncanonical baby GLB')
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


def read_baby(path,species):
 with Path(path).open('rb') as f:return decode_baby(f.read(MAX_BYTES+1),species)
def export_pichu(model=None):return export_baby('pichu',model)
def read_pichu(path):return read_baby(path,'pichu')
def export_cleffa(model=None):return export_baby('cleffa',model)
def read_cleffa(path):return read_baby(path,'cleffa')
def export_igglybuff(model=None):return export_baby('igglybuff',model)
def read_igglybuff(path):return read_baby(path,'igglybuff')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_baby(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
