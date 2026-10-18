#!/usr/bin/env python3
"""Canonical reclining seals, coherent faces and articulated curved tails and paired fans."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from seel_sculpt import sculpture as seel_sculpture,HEIGHT as SEEL_HEIGHT
from dewgong_sculpt import sculpture as dewgong_sculpture,HEIGHT as DEWGONG_HEIGHT
SPECIES=('seel','dewgong');HEIGHTS={'seel':SEEL_HEIGHT,'dewgong':DEWGONG_HEIGHT}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError(species)
 return (seel_sculpture if species=='seel' else dewgong_sculpture)(normalize=normalize)
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'seel': 'e1b358039ce8dba65f99bfde6d01438da48fa17f8532ea107117e290e7fae852', 'dewgong': '07d7f5f2e844cd65944949d81004e5de86d9493fb99bf220dbc9c552830ae5e9'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 young=species=='seel'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.240,-.100) if young else (0.,.300,-.150)),
  ('head',1,(0.,.275,.315) if young else (0.,.715,.250)),
  ('tail_base',1,(0.,.360,-.450) if young else (0.,.275,-.407)),
  ('flipper_left',0 if young else 1,(-.180,.055,.110) if young else (-.170,.430,.110)),
  ('flipper_right',0 if young else 1,(.180,.055,.110) if young else (.170,.430,.110)),
  ('tail_tip',3,(0.,.740,-.790) if young else (0.,.725,-.618)),
  ('fluke_left',6,(0.,.800,-.855) if young else (0.,.966,-.523)),
  ('fluke_right',6,(0.,.800,-.855) if young else (0.,.966,-.523))]
 if young:raw += [('tongue',2,(0.,.141,.532))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Flipper /'):return rigid(4 if part.endswith('-1') else 5)
 if part.startswith('Flipper fold /'):return rigid(4 if part.split()[-2]=='-1' else 5)
 if part.startswith('Tail fluke /'):return rigid(7 if part.endswith('-1') else 8)
 if species=='seel' and part.startswith(('Tongue /','Tongue crease /')):return rigid(9)
 if not part.startswith('Body /'):return rigid(2)
 a=smoothstep(low+.020,low+.120,y)
 if species=='seel':
  head=smoothstep(.130,.280,z)*smoothstep(.130,.220,y);tail=smoothstep(.300,.580,-z);tip=smoothstep(.660,.820,-z)
 else:
  head=smoothstep(.520,.670,y)*smoothstep(-.030,.090,z);tail=smoothstep(.180,.420,-z);tip=smoothstep(.490,.800,y)
 if head>0.:return [0,1,2,0],[1.-a,a*(1.-head),a*head,0.]
 return [0,1,3,6],[1.-a,a*(1.-tail),a*tail*(1.-tip),a*tail*tip]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid seal clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.012*a,0.),(.014*a,.014*b,0.),(.012*b,.020*a,0.)]
 for side in (-1,1):out += [(0.,side*.045*b,0.)] if species=='seel' else [(.012*a,side*.022*b,-side*.025*a)]
 out += [(-.018*a,.022*b,0.),(.010*b,-.018*a,.025*b),(.010*b,.018*a,-.025*b)]
 if species=='seel':out += [(.025*b,0.,0.)]
 return out

def export_seal(species,model=None):
 if species not in SPECIES:raise ValueError('unknown seal species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed seal neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite seal continuous curved bodies, grounded supports and independent flippers and tail fans skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed support surfaces; blended curved bodies, coherent faces, two tail carriers and independent paired flukes and front flippers; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/seel_sculpt.py;tools/dewgong_sculpt.py','geometry_preservation':'original reclining juvenile and curved adult seal contours; fitted faces, rooted horns and canines; broad front paddles and raised scalloped tail fans',
    'animation_contract':'identity floor root; blended body with fixed low supports; coherent face; two continuous tail carriers and independent paired flukes; independent front flippers and matching folds; Seel tongue flexes from the mouth; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'seal exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('seal exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('seal exceeds existing byte limit')
 return blob


def decode_seal(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_seal(species):raise ValueError('noncanonical seal GLB')
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


def read_seal(path,species):
 with Path(path).open('rb') as f:return decode_seal(f.read(MAX_BYTES+1),species)
def export_seel(model=None):return export_seal('seel',model)
def read_seel(path):return read_seal(path,'seel')
def export_dewgong(model=None):return export_seal('dewgong',model)
def read_dewgong(path):return read_seal(path,'dewgong')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_seal(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
