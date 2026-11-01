#!/usr/bin/env python3
"""Remoraid fins and Octillery grounded tentacle curls with a hollow articulated siphon."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from jet_fish_sculpt import sculpture,SPECIES,HEIGHT
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'remoraid': '02f2a0f1061633dd1b6772fc886c1ccce95ddc5237d63a8e3ca29a280a6d0514', 'octillery': '11c7a88eb486c7a59ae19980866dcfc9a9d0f25510be9472abfaa419a8a2c36f'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 if species=='remoraid':
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.40,0.)),('tail',1,(0.,.41,-.30)),('dorsal',1,(0.,.56,.16)),('pelvic',1,(0.,.24,-.08)),('pectoral_left',1,(-.16,.32,.13)),('pectoral_right',1,(.16,.32,.13))]
 else:
  raw=[('root',None,(0.,0.,0.)),('mantle',0,(0.,.42,-.03)),('siphon',1,(0.,.679,.24))]
  for i in range(8):
   a=math.tau*i/8+.20;raw.append(('tentacle_'+str(i),0,(math.sin(a)*.40,.17,math.cos(a)*.40)))
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
BANKS={s:{name:i for i,(name,parent,pivot) in enumerate(js)} for s,js in JOINTS.items()}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species)
 x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if species=='remoraid':
  if part.startswith('Dorsal /'):return rigid(3)
  if part.startswith('Pelvic /'):return rigid(4)
  if part.startswith('Pectoral /'):return rigid(5 if part.endswith('-1') else 6)
  if part.startswith('Tail /'):return rigid(2)
  if part.startswith('Body /'):
   a=1.-smoothstep(-.40,-.26,z);return [1,2,0,0],[1.-a,a,0.,0.]
  return rigid(1)
 if part.startswith(('Arm /','Sucker /')):
  i=int(part.split()[-1] if part.startswith('Arm /') else part.split()[-2]);r=math.hypot(x,z)
  h=smoothstep(.25,.45,y)*(1.-smoothstep(.20,.30,r))
  g=smoothstep(.33,.52,r)*smoothstep(.12,.28,y)
  return [0,1,3+i,0],[(1.-h)*(1.-g),h*(1.-g),g,0.]
 if part.startswith('Siphon /'):
  a=smoothstep(.22,.31,z);return [1,2,0,0],[1.-a,a,0.,0.]
 if part.startswith('Body /'):
  a=smoothstep(.25,.45,y);return [0,1,0,0],[1.-a,a,0.,0.]
 return rigid(1)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid jet fish clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);d=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  d=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;d*=strength;out=[]
 for name,parent,pivot in JOINTS[species]:
  if name=='root':angle=(0.,0.,0.)
  elif name=='body':angle=(0.,.035*a,0.)
  elif name=='tail':angle=(0.,.13*d,0.)
  elif name in ('dorsal','pelvic'):angle=(0.,.035*d,0.)
  elif name.startswith('pectoral_'):angle=(.035*d,0.,(-1 if name.endswith('left') else 1)*.10*a)
  elif name=='mantle':angle=(.008*a,.030*a,.006*d)
  elif name=='siphon':angle=(-.035*a,.018*d,0.)
  else:
   i=int(name.split('_')[-1]);phi=math.tau*i/8+.20
   wave=(math.sin(math.tau*t+i*.68)-math.sin(i*.68)) if clip=='idle' else d*(.72+.28*math.cos(i*.68))
   # Tangential bend uncurls the raised tip while the lower support stays fixed.
   angle=(math.cos(phi)*.040*wave,.012*a,-math.sin(phi)*.040*wave)
  out.append(angle)
 return out
def export_jet_fish(species,model=None):
 if species not in SPECIES:raise ValueError('unknown jet fish species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed jet fish neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite jet fish coherent fins and grounded tentacle curls skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent fitted faces; floor-preserving fish yaw and tentacle support; swimming fins and tail; planted tentacle contacts, articulated curls and hollow siphon; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/jet_fish_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; shared spatial mantle/arm fields; coherent fitted gaze; swimming fins and tail; planted tentacle contacts, articulated curls and hollow siphon; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'jet fish exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_jet_fish(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_jet_fish(species):raise ValueError('noncanonical jet fish GLB')
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


def read_jet_fish(path,species):
 with Path(path).open('rb') as f:return decode_jet_fish(f.read(MAX_BYTES+1),species)
def export_remoraid(model=None):return export_jet_fish('remoraid',model)
def read_remoraid(path):return read_jet_fish(path,'remoraid')
def export_octillery(model=None):return export_jet_fish('octillery',model)
def read_octillery(path):return read_jet_fish(path,'octillery')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_jet_fish(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
