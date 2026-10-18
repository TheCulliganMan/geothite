#!/usr/bin/env python3
"""Canonical fire horses with planted hooves, coherent flame layers and articulated heads and plumes."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from ponyta_sculpt import sculpture as ponyta_sculpture,HEIGHT as PONYTA_HEIGHT
from rapidash_sculpt import sculpture as rapidash_sculpture,HEIGHT as RAPIDASH_HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('ponyta','rapidash');HEIGHTS={'ponyta':PONYTA_HEIGHT,'rapidash':RAPIDASH_HEIGHT}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError(species)
 return (ponyta_sculpture if species=='ponyta' else rapidash_sculpture)(normalize=normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'ponyta': 'e08134f1e01625a819c2a44fab4b227c4658caf579bd33ef1c4b4e52832d342e', 'rapidash': '8e17f3827417a173789716bfc7a5d2af03b236876a0fa606eea76b8bb726d54f'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species):
 young=species=='ponyta'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.675,-.10) if young else (0.,.86,-.12)),
  ('neck',1,(0.,.73,.205) if young else (0.,.935,.26)),('head',2,(0.,1.17,.32) if young else (0.,1.60,.35)),
  ('ear_left',3,(-.098,1.351,.258) if young else (-.102,1.79,.309)),('ear_right',3,(.098,1.351,.258) if young else (.102,1.79,.309)),
  ('tail_stem',1,(0.,.719,-.342) if young else (0.,.919,-.459)),('tail_fire',6,(0.,.80,-.55) if young else (0.,1.00,-.70))]
 for side in (-1,1):
  for row in (0,1):
   pivot=(side*.165,.35,.26 if row==0 else -.25) if young else ((-.24,.70,.45) if side<0 and row==0 else (side*.20,.44,.31 if row==0 else -.32))
   raw.append(('leg_'+str(side)+('_front' if row==0 else '_rear'),0,pivot))
 raw += [('mane_fire',2,(0.,.80,.13) if young else (0.,1.04,.10)),('crown_fire',3,(0.,1.41,.31) if young else (0.,1.83,.23))]
 if not young:raw += [('jaw',3,(0.,1.56,.45))]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def flame_owner(part):
 location=part.split(' / ',1)[1]
 if location.startswith('ankle '):
  words=location.split();side,row=map(int,words[1:3]);return 8+(0 if side<0 else 2)+row
 if location.startswith('mane'):return 12
 if location.startswith('tail'):return 7
 return 13

def skin_weights(species,part,position):
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position));young=species=='ponyta'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Hoof /'):
  side,row=map(int,part.split()[-2:]);return rigid(8 if not young and side==-1 and row==0 else 0)
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=8+(0 if side<0 else 2)+row
  if not young and side==-1 and row==0:
   a=smoothstep(.18,.24,abs(x));return [owner,1,0,0],[a,1.-a,0.,0.]
  a=smoothstep(.10,.62 if young else .80,y);b=.01+.65*smoothstep(.10,.24,y)*(1.-smoothstep(.48 if young else .62,.71 if young else .91,y))
  return [owner,0,1,0],[b,(1.-a)*(1.-b),a*(1.-b),0.]
 if part.startswith(('Body /','Shoulder /')):return rigid(1)
 if part.startswith('Neck /'):
  a=smoothstep(.61 if young else .76,.75 if young else .95,y);b=smoothstep(1.08 if young else 1.48,1.25 if young else 1.68,y)
  return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Ear /','Ear inset /')):return rigid(4 if part.endswith('-1') else 5)
 if part.startswith('Tail stem /'):return rigid(6)
 if part.startswith(('Flame /','Flame glow /')):return rigid(flame_owner(part))
 if part.startswith(('Jaw /','Tongue /')):return rigid(14)
 return rigid(3)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid fire horse clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.009*a,0.),(.009*b,.012*a,0.),(.012*b,.014*a,0.),(0.,-.020*a,-.025*b),(0.,.020*a,.025*b),(.010*b,.025*a,0.),(.018*a,.035*b,0.)]
 for side in (-1,1):
  for row in (0,1):out += [(.010*b if species=='rapidash' and side==-1 and row==0 else 0.,side*.012*a,side*.004*b)]
 out += [(.012*b,.024*a,.008*b),(.018*a,.025*b,.010*b)]
 if species=='rapidash':out += [(.014*b,0.,0.)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part']
  if not name.startswith('Flame glow /'):
   parts.append((source,{'source_order':order}));continue
  location=name.rsplit(' ',1)[0]
  if location not in groups:
   joined={'part':location,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[location]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[location];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'ponyta': '586c58b6e344ad355eed3521ceeb585a241b17255a01a5ae26bb2fb4f3b185ce', 'rapidash': 'c56d7e051ad01b2f418fca26b70baa6f9c682bc2048ad11e790fee0401ec156f'}

def export_fire_horse(species,model=None):
 if species not in SPECIES:raise ValueError('unknown fire_horse species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed fire_horse neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite fire_horse grounded hooves, coherent heads and articulated manes, crowns and tail plumes skin v1'
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
 packed_parts=pack_parts(species,model)
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed fire_horse geometry')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':species+'/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  ranges=metadata.get('source_parts',[{'part':part['part'],'vertex_start':0,'vertex_count':len(part['positions'])//3}])
  for source in ranges:
   for i in range(source['vertex_start']*3,(source['vertex_start']+source['vertex_count'])*3,3):
    point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(species,source['part'],point);q=quantized_weights(ws)
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed supporting hooves and an authored raised adult forehoof; coherent neck and head; independent ears, mane, crown and tail plumes with fitted two-sided cores; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/ponyta_sculpt.py;tools/rapidash_sculpt.py','geometry_preservation':'original slender pony and athletic unicorn anatomy; seven volumetric fire banks with fourteen fitted core layers; reduced interior sampling and adaptive fitted layers retain every closed solid',
    'animation_contract':'identity floor root; fixed supporting hooves; articulated raised adult foreleg; blended neck and braced legs; coherent gaze; independent ears and tail stem; seven fire banks with coherent two-sided cores on their anatomical carriers; independent adult jaw and tongue; lossless core material packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'fire_horse exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('fire_horse exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('fire_horse exceeds existing byte limit')
 return blob


def decode_fire_horse(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_fire_horse(species):raise ValueError('noncanonical fire_horse GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 parts=[]
 for p,extra in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
  positions=values(p['attributes']['POSITION']);normals=values(p['attributes']['NORMAL']);indices=values(p['indices']);color=doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
  sources=extra.get('source_parts',[{'part':p['extras']['part'],'order':extra.get('source_order'),'vertex_start':0,'vertex_count':len(positions)//3,'index_start':0,'index_count':len(indices)}])
  for source in sources:
   a=source['vertex_start'];b=a+source['vertex_count'];i=source['index_start'];j=i+source['index_count']
   parts.append((source['order'],{'part':source['part'],'positions':positions[3*a:3*b],'normals':normals[3*a:3*b],'indices':[k-a for k in indices[i:j]],'base_color':color}))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}


def read_fire_horse(path,species):
 with Path(path).open('rb') as f:return decode_fire_horse(f.read(MAX_BYTES+1),species)
def export_ponyta(model=None):return export_fire_horse('ponyta',model)
def read_ponyta(path):return read_fire_horse(path,'ponyta')
def export_rapidash(model=None):return export_fire_horse('rapidash',model)
def read_rapidash(path):return read_fire_horse(path,'rapidash')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 for species in SPECIES:
  blob=export_fire_horse(species);(a.out/(species+'.glb')).write_bytes(blob);print(species,len(blob),'bytes',len(pack_parts(species,authoring_model(species))),'runtime parts',REST_DIGESTS[species],PACKED_REST_DIGESTS[species],flush=True)
if __name__=='__main__':main()
