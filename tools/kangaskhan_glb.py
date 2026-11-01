#!/usr/bin/env python3
"""Canonical Kangaskhan parent and seated baby with a real open pouch and supported grips."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from kangaskhan_sculpt import sculpture as source_sculpture,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('kangaskhan',);HEIGHTS={'kangaskhan':HEIGHT}
def sculpture(species='kangaskhan',normalize=True):
 if species!='kangaskhan':raise ValueError(species)
 return source_sculpture(normalize=normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_model(species='kangaskhan'):return sculpture(species)
REST_DIGESTS={'kangaskhan':'476e85961870526d4c0e2867fd693bd68a63cdf2f3862e43a00feca553b83f40'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=1)
def authoring_transform(species='kangaskhan'):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
RAW_JOINTS=(('root',None,(0.,0.,0.)),('body',0,(0.,.55,.01)),('head',1,(0.,.89,.04)),('jaw',2,(0.,.84,.12)),
 ('ear_left',2,(-.158,1.135,.05)),('ear_right',2,(.158,1.135,.05)),('arm_left',1,(-.335,.74,.06)),('wrist_left',6,(-.475,.95,.05)),('arm_right',1,(.335,.74,.06)),('wrist_right',8,(.475,.95,.05)),
 ('leg_left',0,(-.25,.25,.034)),('leg_right',0,(.25,.25,.034)),('tail_base',1,(0.,.25,-.20)),('tail_tip',12,(-.34,.20,-.40)),
 ('baby_body',1,(0.,.44,.34)),('baby_head',14,(0.,.66,.325)),('baby_left_grip',15,(-.061,.647,.348)),('baby_right_grip',15,(.061,.647,.348)),('baby_ear_left',15,(-.06,.775,.325)),('baby_ear_right',15,(.06,.775,.325)))
JOINTS={'kangaskhan':tuple((n,parent,(0.,0.,0.) if parent is None else normalized('kangaskhan',p)) for n,parent,p in RAW_JOINTS)}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 low,span=authoring_transform(species);x,y,z=tuple(v*span/HEIGHT+(low if i==1 else 0.) for i,v in enumerate(position))
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Foot claw /')):return rigid(0)
 if part.startswith(('Body /','Belly /','Pouch /','Back plate /')):return rigid(1)
 if part.startswith(('Jaw /','Tongue /')):return rigid(3)
 if part.startswith('Ear /'):return rigid(4 if part.endswith('-1') else 5)
 if part.startswith(('Shoulder /','Shoulder plate /','Arm /','Fist /','Hand claw /')):
  side=part.split()[-2] if part.startswith('Hand claw /') else part.split()[-1];arm=6 if side=='-1' else 8
  if part.startswith(('Fist /','Hand claw /')):return rigid(arm+1)
  if not part.startswith('Arm /'):return rigid(arm)
  a=smoothstep(.78,.94,y);return [arm,arm+1,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Haunch /','Haunch armor outline /','Haunch plate /')):return rigid(10 if part.endswith('-1') else 11)
 if part.startswith('Ankle /'):
  owner=10 if part.endswith('-1') else 11;a=smoothstep(.08,.24,y);return [owner,0,0,0],[a,1.-a,0.,0.]
 if part.startswith('Tail /'):
  a=smoothstep(.0,.18,-x);b=smoothstep(.20,.50,-x);return [1,12,13,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Baby body /'):return rigid(14)
 if part.startswith('Baby neck /'):
  a=smoothstep(.60,.68,y);return [14,15,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Baby arm /','Baby hand /')):return rigid(16 if part.endswith('-1') else 17)
 if part.startswith('Baby ear /'):return rigid(18 if part.endswith('-1') else 19)
 if part.startswith('Baby '):return rigid(15)
 return rigid(2)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Kangaskhan clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*20
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 out=[(0.,0.,0.),(0.,.009*a,0.),(.010*a,.014*b,0.),(.008*b,0.,0.),(0.,-.012*a,-.016*b),(0.,.012*a,.016*b)]
 for side in (-1,1):out += [(-.020*a,side*.020*b,side*.020*a),(.025*b,side*.016*a,side*.018*b)]
 out += [(0.,-.016*a,0.),(0.,.016*a,0.),(.010*a,.020*b,0.),(.012*b,-.025*a,.008*b),
  (0.,.010*a,0.),(.010*b,.018*a,.006*b),(.006*b,-.010*a,0.),(.006*b,.010*a,0.),(0.,-.015*a,-.018*b),(0.,.015*a,.018*b)]
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=name
  if name.startswith(('Head /','Snout /')):key='head';label='Head / broad tan Kangaskhan skull and upper muzzle'
  elif name.startswith(('Eye /','Fang /')):key='white_head';label='Eye and fang / white Kangaskhan gaze and upper fangs'
  elif name.startswith('Pupil /'):key='red_gaze';label='Pupil / paired red Kangaskhan gaze'
  elif name.startswith(('Eye slit /','Nostril /')):key='dark_gaze';label='Gaze and nostril / fitted dark Kangaskhan facial details'
  elif name.startswith('Back plate /'):key='back';label='Back plate / four rooted Kangaskhan dorsal points'
  elif name.startswith(('Haunch /','Ankle /')):side=name.split()[-1];key='leg_'+side;label='Leg / joined Kangaskhan haunch and ankle '+side
  elif name.startswith(('Hand claw /','Foot claw /')):
   category=name.split(' /')[0];side=name.split()[-2];key=category+side;label=category+' / three coherent white Kangaskhan claws '+side
  elif name.startswith(('Baby body /','Baby neck /')):key='baby_body';label='Baby body / seated Kangaskhan baby body and neck'
  elif name.startswith(('Baby arm /','Baby hand /')):side=name.split()[-1];key='grip_'+side;label='Baby arm / supported Kangaskhan baby arm and rim grip '+side
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])});joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'kangaskhan':'d05bf3560699741a45d687e8c44745f0ba667d37fa551a0c1d7b16ef90cffc94'}

def export_parent(species,model=None):
 if species not in SPECIES:raise ValueError('unknown parent species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed parent neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite parent armored mother, planted feet and independently articulated seated baby skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed parent geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed foot pads and claws; coherent mother and pouch; independent head, ears, jaw, arms and wrists; seated baby has body, head, ear and grip carriers; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/kangaskhan_sculpt.py','geometry_preservation':'all seventy original logical solids; reduced redundant surface sampling, adaptive fitted paint and lossless material packing; true open parent mouth and pouch cavity; fully seated baby',
    'animation_contract':'identity floor root; fixed feet and toe claws; coherent mother body, belly and open pouch; articulated mother head, jaw, ears, arms, wrists and two tail carriers; supported haunches; independent baby body, blended neck, head, ears and grips; matching fitted armor and gaze influences; lossless material packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'parent exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('parent exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('parent exceeds existing byte limit')
 return blob


def decode_parent(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_parent(species):raise ValueError('noncanonical parent GLB')
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


def read_parent(path,species):
 with Path(path).open('rb') as f:return decode_parent(f.read(MAX_BYTES+1),species)

def export_kangaskhan(model=None):return export_parent('kangaskhan',model)
def read_kangaskhan(path):return read_parent(path,'kangaskhan')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True);blob=export_kangaskhan();(a.out/'kangaskhan.glb').write_bytes(blob);print(len(blob),'bytes',len(pack_parts('kangaskhan',authoring_model())),'runtime parts',REST_DIGESTS,PACKED_REST_DIGESTS,flush=True)
if __name__=='__main__':main()
