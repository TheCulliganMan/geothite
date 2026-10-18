#!/usr/bin/env python3
"""Canonical armored rhinoceroses with planted feet, coherent faces and independent hard appendages."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from rhyhorn_sculpt import sculpture as rhyhorn_sculpture,HEIGHT as RHYHORN_HEIGHT
from rhydon_sculpt import sculpture as rhydon_sculpture,HEIGHT as RHYDON_HEIGHT
SPECIES=('rhyhorn','rhydon');HEIGHTS={'rhyhorn':RHYHORN_HEIGHT,'rhydon':RHYDON_HEIGHT}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError(species)
 return (rhyhorn_sculpture if species=='rhyhorn' else rhydon_sculpture)(normalize=normalize)
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'rhyhorn': 'e6ad87610f6a2b99b8e1780fddffabfb97166fdb1c8d50278ca6e765dcbc0188', 'rhydon': 'f65fdcc84a808bd430ba1ce4ed03602ec0301a6bb0905722850774e49f1248c9'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy()
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def joints(species):
 if species=='rhyhorn':
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.425,-.140)),('head',1,(0.,.510,.050)),('tail',1,(0.,.430,-.500))]
  raw += [('leg_'+str(side)+'_'+row,0,(side*.255,.230,z)) for side in (-1,1) for row,z in (('rear',-.370),('front',.130))]
  raw += [('jaw',2,(0.,.335,.210)),('ear_left',2,(-.240,.646,.106)),('ear_right',2,(.240,.646,.106))]
 else:
  raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.550,-.015)),('head',1,(0.,1.180,.050)),('jaw',2,(0.,1.040,.130)),
   ('arm_left',1,(-.286,.942,.033)),('hand_left',4,(-.550,.941,.208)),('arm_right',1,(.286,.942,.033)),('hand_right',6,(.550,.941,.208)),
   ('leg_left',0,(-.280,.200,.080)),('leg_right',0,(.280,.200,.080)),('tail_base',1,(0.,.330,-.229)),('tail_tip',10,(.130,.250,-.829)),
   ('ear_left',2,(-.157,1.327,.055)),('ear_right',2,(.157,1.327,.055)),('drill',2,(0.,1.208,.384))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={s:joints(s) for s in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=1)
def tail_path():
 from rhydon_sculpt import TAIL_POINTS,TAIL_RADII
 from bovine_sculpt import smooth
 return smooth(TAIL_POINTS,TAIL_RADII)[0]
def path_fraction(point,path):
 lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.;best=(math.inf,0.)
 for a,b,length in zip(path,path[1:],lengths):
  delta=[b[k]-a[k] for k in range(3)];t=max(0.,min(1.,sum((point[k]-a[k])*delta[k] for k in range(3))/(length*length)))
  best=min(best,(math.dist(point,[a[k]+t*delta[k] for k in range(3)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(species,part,position):
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHTS[species]+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Foot claw /')) or species=='rhyhorn' and part.startswith('Claw /'):return rigid(0)
 if part.startswith(('Leg /','Leg armor /','Thigh /')):
  if species=='rhyhorn':
   side=int(part.split()[-2]);front=part.split()[-1]=='front';owner=4+(0 if side<0 else 2)+int(front)
  else:owner=8 if part.endswith('-1') else 9
  a=smoothstep(.080,.390,y);b=.01+.64*smoothstep(.080,.160,y)*(1.-smoothstep(.310,.430,y))
  return [owner,0,1,0],[b,(1.-a)*(1.-b),a*(1.-b),0.]
 if species=='rhyhorn':
  if part.startswith(('Body /','Back armor /')):return rigid(1)
  if part.startswith('Tail /'):return rigid(3)
  if part.startswith('Jaw /'):return rigid(8)
  if part.startswith('Ear /'):return rigid(9 if part.endswith('-1') else 10)
  return rigid(2)
 if part.startswith(('Body /','Belly plate /','Collar /','Back armor /')):return rigid(1)
 if part.startswith(('Jaw /','Tongue /')):return rigid(3)
 if part.startswith(('Ear /','Ear inset /')):return rigid(12 if part.endswith('-1') else 13)
 if part.startswith(('Horn /','Horn spiral /')):return rigid(14)
 if part.startswith(('Shoulder /','Arm /','Hand /','Hand claw /')):
  side=int(part.split()[-2] if part.startswith('Hand claw /') else part.split()[-1]);arm=4 if side<0 else 6;hand=arm+1
  if part.startswith(('Hand /','Hand claw /')):return rigid(hand)
  if part.startswith('Shoulder /'):return rigid(arm)
  a=smoothstep(.320,.405,abs(x));b=smoothstep(.480,.550,abs(x))
  return [1,arm,hand,0],[(1.-a)*(1.-b),a*(1.-b),b,0.]
 if part.startswith(('Tail /','Tail armor /')):
  t=path_fraction((x,y,z),tail_path());a=smoothstep(0.,.25,t);b=smoothstep(.50,.82,t)
  return [1,10,11,0],[1.-a,a*(1.-b),a*b,0.]
 return rigid(2)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid rhino clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 if species=='rhyhorn':
  return [(0.,0.,0.),(0.,.012*a,0.),(.012*a,.014*b,0.),(.008*b,.040*a,0.),
   *[(0.,side*.018*a,side*.008*b) for side in (-1,1) for row in range(2)],(.016*b,0.,0.),(0.,-.015*a,-.035*b),(0.,.015*a,.035*b)]
 out=[(0.,0.,0.),(0.,.012*a,0.),(.014*a,.018*b,0.),(.012*b,0.,0.)]
 for side in (-1,1):out += [(-.018*a,side*.030*b,side*.035*a),(.030*b,side*.014*a,side*.016*b)]
 out += [(0.,side*.018*a,side*.008*b) for side in (-1,1)]
 out += [(.010*a,.028*b,0.),(.015*b,-.040*a,.008*b),(0.,-.025*b,-.030*a),(0.,.025*b,.030*a),(0.,.048*b,.036*b)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=None
  if species=='rhydon':
   if name.startswith('Belly plate /'):key='belly';label='Belly plate / four coherent cream Rhydon bands'
   elif name.startswith('Cheek armor /'):key='cheeks';label='Cheek armor / six coherent grey Rhydon skull points'
   elif name.startswith('Back armor /'):key='back';label='Back armor / three coherent grey Rhydon dorsal points'
   elif name.startswith('Hand claw /'):
    side=name.split()[-2];key='hand_'+side;label='Hand claw / three coherent ivory Rhydon claws '+side
   elif name.startswith('Foot claw /'):
    side=name.split()[-2];key='foot_'+side;label='Foot claw / two grounded ivory Rhydon toes '+side
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'rhyhorn': 'e6ad87610f6a2b99b8e1780fddffabfb97166fdb1c8d50278ca6e765dcbc0188', 'rhydon': 'c0131e43f7235c93fc5a645ab0c3e69f63dc8c2f8d8aa36e0134cc89ab22b4f7'}

def export_rhino(species,model=None):
 if species not in SPECIES:raise ValueError('unknown rhino species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed rhino neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite rhino seated armor, planted pads and articulated limbs, jaws and drill skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed rhino geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed broad pads and toe claws; blended braced legs; coherent armored heads; independent jaws ears and tails; Rhydon adds hands and an axial drill; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/rhyhorn_sculpt.py;tools/rhydon_sculpt.py','geometry_preservation':'original lowered armored quadruped and heavy drill biped; red gazes, seated armor, rooted horns, limbs and claws; open adult jaw and fitted tail collars',
    'animation_contract':'identity floor root; fixed foot pads and matching toe claws; blended braced legs; coherent skulls, faces and armor; independent ears, jaws and tails; Rhydon has separate arms, wrists and drill, and matching tail collar influences; lossless material packing retains every logical solid; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'rhino exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('rhino exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('rhino exceeds existing byte limit')
 return blob


def decode_rhino(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_rhino(species):raise ValueError('noncanonical rhino GLB')
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


def read_rhino(path,species):
 with Path(path).open('rb') as f:return decode_rhino(f.read(MAX_BYTES+1),species)
def export_rhyhorn(model=None):return export_rhino('rhyhorn',model)
def read_rhyhorn(path):return read_rhino(path,'rhyhorn')
def export_rhydon(model=None):return export_rhino('rhydon',model)
def read_rhydon(path):return read_rhino(path,'rhydon')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_rhino(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
