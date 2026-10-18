#!/usr/bin/env python3
"""Canonical hollow mollusk valves, supported tongue folds and opening armor halves."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from bivalve_sculpt import sculpture,SPECIES,HEIGHT
HEIGHTS=HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'shellder': '9603bdedd0b70757c615251f8ea5ea6b258a33c53c3ce6ab434f2f441edd95e9', 'cloyster': '714eb4a74d131c5a8b303d1fde4af07a30dd0c739fa0741176cee1efbb336d74'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 if species=='shellder':
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.485,.005)),('upper_valve',1,(0.,.600,-.135)),
   ('lower_valve',1,(0.,.300,-.045)),('tongue_base',1,(0.,.370,.095)),('tongue_fold',4,(0.,.255,.245))]
 else:
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.500,-.080)),('valve_left',1,(0.,.500,-.220)),
   ('valve_right',1,(0.,.500,-.220)),('mantle',1,(0.,.500,-.205))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if species=='shellder':
  if part.startswith(('Tongue /','Tongue crease /')):
   anchor=smoothstep(.10,.16,z);fold=smoothstep(.18,.34,z)
   return [1,4,5,0],[1.-anchor,anchor*(1.-fold),anchor*fold,0.]
  if part.startswith(('Jaw /','Jaw lining /')):return rigid(3)
  if part.startswith(('Shell /','Lining /','Hinge /','Horn /','Rear horn /','Shell groove /')):return rigid(2)
  return rigid(1)
 if part.startswith('Shell /'):return rigid(2 if part.endswith('-1') else 3)
 if part.startswith('Outer spike /'):return rigid(2 if part.split()[-2]=='-1' else 3)
 if part.startswith(('Inner hood /','Mantle lining /','Inner horn /')):return rigid(4)
 return rigid(1)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid bivalve clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.pi*t)**2;strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,.04),(1.,0.)));strength=2. if clip=='attack' else -1.3
 if species=='shellder':
  return [(0.,0.,0.),(0.,.012*a*strength,0.),(.022*a*strength,0.,0.),(0.,.018*a*strength,0.),(.012*a*strength,0.,0.),(-.018*a*strength,0.,0.)]
 return [(0.,0.,0.),(0.,.012*a*strength,0.),(0.,-.035*b*strength,0.),(0.,.035*b*strength,0.),(0.,.014*a*strength,0.)]

def export_bivalve(species,model=None):
 if species not in SPECIES:raise ValueError('unknown bivalve species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed bivalve neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite bivalve hollow hinged valves, tongue folds and opening armor halves skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent mollusk faces; independent hollow shell valves; supported tongue fold and mantle; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/bivalve_sculpt.py','geometry_preservation':'original hollow lavender shells, dark mollusk bodies, fitted pale gaze and source-specific tongue or spiked mantle',
    'animation_contract':'identity floor root; coherent dark mollusk faces; Shellder upper hood and lower valve hinge independently with braced lower valve and freely curling tongue fold; Cloyster opens paired armor valves around a separate rooted mantle; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'bivalve exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('bivalve exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('bivalve exceeds existing byte limit')
 return blob


def decode_bivalve(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_bivalve(species):raise ValueError('noncanonical bivalve GLB')
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


def read_bivalve(path,species):
 with Path(path).open('rb') as f:return decode_bivalve(f.read(MAX_BYTES+1),species)
def export_shellder(model=None):return export_bivalve('shellder',model)
def read_shellder(path):return read_bivalve(path,'shellder')
def export_cloyster(model=None):return export_bivalve('cloyster',model)
def read_cloyster(path):return read_bivalve(path,'cloyster')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_bivalve(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
