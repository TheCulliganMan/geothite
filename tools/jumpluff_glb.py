#!/usr/bin/env python3
"""Canonical Jumpluff skin; preserve all 84 bristles in three render groups."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from wind_plant_sculpt import sculpture as source_sculpture,HEIGHT,jumpluff_seed_point
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('jumpluff',)
MAX_BYTES=512000

def sculpture(species,normalize=True):
 source=source_sculpture(species,normalize=normalize);parts=[]
 for p in source['primitives']:
  component={'part':p['part'],'vertices':len(p['positions'])//3,'indices':len(p['indices'])}
  if ' / fine seed bristle ' in p['part']:
   name=p['part'].split(' / fine seed bristle ')[0]+' / fitted bristle group'
   if not parts or parts[-1]['part']!=name:
    parts.append({'part':name,'positions':[],'normals':[],'indices':[],'base_color':p['base_color'],'components':[]})
   group=parts[-1];offset=len(group['positions'])//3
   if group['base_color']!=p['base_color']:raise ValueError('bristle grouping changes material')
   group['positions'].extend(p['positions']);group['normals'].extend(p['normals'])
   group['indices'].extend(i+offset for i in p['indices']);group['components'].append(component)
  else:parts.append(dict(p,components=[component]))
 return dict(source,primitives=parts)

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

REST_DIGESTS={'jumpluff':'2de06589332b313b7e2264c47f04d319ddb334f09c1caca6632cf7a69b7659fd'}
PARTS={n:tuple(p['part'] for p in sculpture(n)['primitives']) for n in SPECIES}
RAW=sculpture('jumpluff',normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
JOINTS={'jumpluff':tuple((name,parent,pivot(p) if parent is not None else p) for name,parent,p in (
 ('root',None,(0.,0.,0.)),('torso',0,(0.,.25,0.)),
 ('arm_left',1,(-.18,.33,.145)),('arm_right',1,(.18,.33,.145)),
 ('lower_stem_left',1,(-.16,.275,-.012)),('lower_stem_right',1,(.16,.275,-.012)),
 ('lower_seed_left',4,jumpluff_seed_point((-.306,.166,-.014),-1)),('lower_seed_right',5,jumpluff_seed_point((.306,.166,-.014),1)),
 ('crown_stem',1,(0.,.573,-.025)),('crown_stem_tip',8,(0.,.715,-.025)),
 ('crown_seed',9,(0.,.908,-.025)),('bract_left',8,(-.008,.604,-.020)),('bract_right',8,(.008,.604,-.020))))}
CLIPS=(('idle',3.6,91),('attack',1.,65),('hit',1.,65))

def skin_weights(species,part,position):
 x,y,z=position;raw_y=y/SCALE+LOW
 if part.startswith('Foot /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Arm /'):return [2 if x<0 else 3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Seed head /'):
  owner=6 if 'lower -1' in part else 7 if 'lower 1' in part else 10
  return [owner,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Seed stem / lower'):
  # Evaluate the attachment blend in the unsplayed stem's local coordinates.
  raw=jumpluff_seed_point((x/SCALE,raw_y,z/SCALE),-1 if x<0 else 1,inverse=True)
  free=smoothstep(.16,.27,abs(raw[0]))
  return [1,4 if x<0 else 5,0,0],[1.-free,free,0.,0.]
 if part.startswith('Seed stem / tall'):
  attach=smoothstep(.573,.630,raw_y);tip=smoothstep(.670,.790,raw_y)
  return [1,8,9,0],[1.-attach,attach*(1.-tip),attach*tip,0.]
 if part.startswith('Leaf /'):return [11 if x<0 else 12,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Body /'):
  upper=smoothstep(.16,.25,raw_y)
  return [0,1,0,0],[1.-upper,upper,0.,0.]
 return [1,0,0,0],[1.,0.,0.,0.]

def pose_angles(species,clip,t):
 if species!='jumpluff' or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Jumpluff clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*13
 wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
 def lag(phase):return math.sin(math.tau*t-phase)+math.sin(phase)
 if clip=='idle':
  body=.008*breath;arm=.018*wave;lift=.012*breath;seed=.010*lag(.4)
  base=.016*wave;tip=.024*lag(.5);crown=.019*lag(.8)
 else:
  early=envelope(t,((0.,0.),(.18,1.),(.40,.68),(.68,.10),(1.,0.)))
  late=envelope(t,((0.,0.),(.29,1.),(.54,.60),(.79,-.12),(1.,0.)))
  sign=1. if clip=='attack' else -.8
  body=sign*.025*early;arm=sign*.040*early;lift=.020*early;seed=sign*.016*late
  base=sign*.028*early;tip=sign*.040*late;crown=sign*.033*late
 return [(0.,0.,0.),(body,.004*wave,0.),(arm,0.,-.010*breath),(arm,0.,.010*breath),
         (0.,0.,-lift),(0.,0.,lift),(seed,.014*wave,0.),(seed,-.014*wave,0.),
         (base,0.,.008*wave),(tip,0.,.010*lag(.5)),(crown,.012*lag(.7),0.),
         (.013*wave,0.,-.009*breath),(.013*wave,0.,.009*breath)]

def export_jumpluff(model=None):
 species='jumpluff'
 if model is None:model=sculpture(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed wind plant neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite distinct wind plant anatomy and standard skin v1'
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'body brace; lifted lower heads; delayed crown stem and seed heads; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/wind_plant_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'source_components':[p['components'] for p in model['primitives']],
    'animation_contract':'identity root; fixed supporting feet; fitted face; articulated arms seed stems heads and bracts; all bristles stay with their seed head; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('wind plant exceeds existing accessor limit')
 return g.bytes()


def decode_jumpluff(blob,expanded=True):
 if len(blob)>MAX_BYTES or blob!=export_jumpluff():raise ValueError('noncanonical Jumpluff GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 parts=[]
 for p,components in zip(doc['meshes'][0]['primitives'],doc['extras']['source_components']):
  positions=values(p['attributes']['POSITION']);normals=values(p['attributes']['NORMAL']);indices=values(p['indices'])
  color=doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
  if not expanded:
   parts.append({'part':p['extras']['part'],'positions':positions,'normals':normals,'indices':indices,'base_color':color,'components':components})
  else:
   vertex=triangle=0
   for c in components:
    count=c['vertices'];size=c['indices']
    parts.append({'part':c['part'],'positions':positions[vertex*3:(vertex+count)*3],
     'normals':normals[vertex*3:(vertex+count)*3], 'indices':[i-vertex for i in indices[triangle:triangle+size]],'base_color':color})
    vertex+=count;triangle+=size
 return {'name':'jumpluff','version':1,'coordinate_system':COORDINATES,'primitives':parts}

def read_jumpluff(path):
 with Path(path).open('rb') as f:return decode_jumpluff(f.read(MAX_BYTES+1))

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 blob=export_jumpluff();(a.out/'jumpluff.glb').write_bytes(blob)
 print('jumpluff',len(blob),'bytes',len(PARTS['jumpluff']),'render groups',len(JOINTS['jumpluff']),'joints',REST_DIGESTS['jumpluff'])
if __name__=='__main__':main()
