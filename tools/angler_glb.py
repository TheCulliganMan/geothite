#!/usr/bin/env python3
"""Chinchou and Lanturn swimming fins, flexible connected beacons and coherent lures."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from angler_sculpt import sculpture,SPECIES,HEIGHT
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'chinchou': '0924cdb724dca3b1b63d2893ac161eda23672ff9052be7728c7d84b216c52828', 'lanturn': '50ecc25097080930e9c2f4d69a37ea5388ae765b0c4289cfd4b91e6395944019'}
TRIANGLE_DIGESTS={'chinchou': 'dd167cef8260b1206eebd388e05012e0e2915470f8c65f5693ea1bba6dc6ec96', 'lanturn': '0b6235cbce31a6685f5f1cc0373706d2eb8709f1781cac1d601f5c42b9bae516'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 adult=species=='lanturn';y=.40 if adult else .35;x=.22 if adult else .24
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.39 if adult else .37,0.)),
  ('fin_left',1,(-x,y,.04)),('fin_right',1,(x,y,.04))]
 if adult:
  raw += [('beacon_base',1,(0.,.637,.015)),('beacon_tip',4,(-.035,.95,.125)),
   ('branch_base',4,(0.,.795,.065)),('branch_tip',6,(.18,.92,.06))]
 else:
  raw += [('antenna_left_base',1,(-.06,.556,-.014)),('antenna_left_tip',4,(-.31,.94,.030)),
   ('antenna_right_base',1,(.06,.556,-.014)),('antenna_right_tip',6,(.31,.94,.030))]
 raw += [('pelvic_left',0,(-.145,.153,.02)),('pelvic_right',0,(.145,.153,.02))]
 if adult:raw += [('tail_base',1,(0.,.39,-.29)),('tail_tip',10,(0.,.40,-.50))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
BANKS={s:{name:i for i,(name,parent,pivot) in enumerate(js)} for s,js in JOINTS.items()}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Fin / broad','Fin fold /')):return rigid(2 if part.endswith('-1') else 3)
 if part.startswith('Fin / soft'):return rigid(8 if part.endswith('-1') else 9)
 if part.startswith('Lure /'):
  return rigid((5 if part.endswith('-1') else 7) if species=='chinchou' else (5 if 'large' in part else 7))
 if part.startswith('Stalk /'):
  if species=='chinchou':base=4 if part.endswith('-1') else 6;a=smoothstep(.22,.50,abs(x))
  elif part.startswith('Stalk / single'):base=4;a=smoothstep(.05,.25,-x)
  else:base=6;a=smoothstep(.08,.20,x)
  return [base,base+1,0,0],[1.-a,a,0.,0.]
 if part.startswith('Tail fin /'):return rigid(11)
 if part.startswith('Tail /'):
  a=smoothstep(.36,.50,-z);return [10,11,0,0],[1.-a,a,0.,0.]
 return rigid(1)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid angler clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.020*a,0.),(.030*a,.030*a,.100*a),(.030*a,-.030*a,-.100*a),
  (.008*a,.025*a,-.018*a),(.010*b,.045*b,-.035*b),(.008*a,-.025*a,.018*a),(.010*b,-.045*b,.035*b),
  (0.,.025*a,0.),(0.,-.025*a,0.)]
 if species=='lanturn':out += [(.015*b,.055*a,0.),(.020*b,.100*b,0.)]
 return out
def export_angler(species,model=None):
 if species not in SPECIES:raise ValueError('unknown angler fish species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed angler fish neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite angler fish swimming fins and flexible connected beacons skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'yaw-only grounded pelvic fins; coherent fitted face; pectoral strokes; flexible connected stalks and rigid lures; linked tail lobes; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/angler_sculpt.py','geometry_preservation':'exact source triangle positions and colors; Lanturn mask shares vertices with smooth crease-aware normals',
    'animation_contract':'identity floor root; yaw-only body and pelvic paddles; rigid face; coherent pectoral fins and folds; connected two-stage antennae and beacons with rigid bulb/center owners; linked tail lobes; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'angler fish exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_angler(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_angler(species):raise ValueError('noncanonical angler fish GLB')
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


def read_angler(path,species):
 with Path(path).open('rb') as f:return decode_angler(f.read(MAX_BYTES+1),species)
def export_chinchou(model=None):return export_angler('chinchou',model)
def read_chinchou(path):return read_angler(path,'chinchou')
def export_lanturn(model=None):return export_angler('lanturn',model)
def read_lanturn(path):return read_angler(path,'lanturn')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_angler(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
