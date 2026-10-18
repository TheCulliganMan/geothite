#!/usr/bin/env python3
"""Original distinct armored royal creatures with per-limb support packing and fitted markings."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import nidoqueen_sculpt as female_source
import nidoking_sculpt as male_source
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('nidoqueen','nidoking');HEIGHTS={'nidoqueen':female_source.HEIGHT,'nidoking':male_source.HEIGHT}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError('unknown armored royal species')
 return (female_source if species=='nidoqueen' else male_source).sculpture(normalize)
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGESTS={'nidoqueen': 'f9b60413bc6f14f2980c440f61f8d7ce304219d68853bb4c160456b855793307', 'nidoking': '99b3efee252edb4d2e162769e3f8d55f4f958cf55f4ee7befd2b240da698f8cd'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species):
 queen=species=='nidoqueen'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.70,-.04)),('head',1,(0.,1.12,.05)),('ear_left',2,(-.17,1.30,.01)),('ear_right',2,(.17,1.30,.01)),('horn',2,(0.,1.351,.194) if queen else (0.,1.377,.290))]
 if not queen:raw += [('lower_jaw',2,(0.,1.065,.24)),('tongue',6,(0.,.94,.46))]
 for side in (-1,1):
  first=len(raw);suffix='left' if side<0 else 'right'
  points=((side*.309,.866,.008),(side*.450,.683,.058),(side*.444,.511,.230)) if queen else ((side*.309,.880,.082),(side*.475,.695,.135),(side*.495,.491,.310))
  raw += [('arm_'+suffix,1,points[0]),('forearm_'+suffix,first,points[1]),('wrist_'+suffix,first+1,points[2])]
 base=len(raw)
 raw += [('leg_left',1,(-.258,.329,-.092) if queen else (-.267,.333,-.076)),('leg_right',1,(.258,.329,-.092) if queen else (.267,.333,-.076)),('tail_base',1,(0.,.342,-.264) if queen else (0.,.325,-.282)),('tail_mid',base+2,(.04,.20,-.75)),('tail_tip',base+3,(.20,.30,-1.18))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for name,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);x,y,z=(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));queen=species=='nidoqueen'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Foot claw /')):return rigid(0)
 if part.startswith(('Body /','Chest plate /','Belly plate /','Back spine /')):return rigid(1)
 if part.startswith(('Ear /','Ear inset /')):return rigid(3 if part.endswith('-1') else 4)
 if part.startswith('Horn /'):return rigid(5)
 if not queen and (part.startswith('Jaw / open') or part.startswith('Fang / rooted ivory king lower')):return rigid(6)
 if part.startswith('Tongue /'):return rigid(7)
 if part.startswith(('Head /','Muzzle /','Chin /','Mouth /','Jaw /','Eye socket /','Eye /','Iris /','Pupil /','Glint /','Nostril /','Fang /','Cheek barb /')):return rigid(2)
 if part.startswith(('Shoulder /','Arm /','Hand /','Hand claw /')):
  side=int(part.split()[-2] if part.startswith('Hand claw /') else part.split()[-1]);base=(6 if queen else 8)+(3 if side>0 else 0)
  if part.startswith('Shoulder /'):return rigid(base)
  if part.startswith(('Hand /','Hand claw /')):return rigid(base+2)
  a=smoothstep(.54 if queen else .52,.68,y);b=smoothstep(.73,.84,y)
  return [base+2,base+1,base,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Haunch /','Knee /','Leg /')):
  side=int(part.split()[-1]);owner=(12 if queen else 14)+(side>0)
  if not part.startswith('Leg /'):return rigid(owner)
  a=smoothstep(.10,.24,y);return [0,owner,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Tail /','Tail collar /','Tail spine /')):
  base=14 if queen else 16
  a=smoothstep(.34,.48,-z) if queen else smoothstep(.36,.53,-z);b=smoothstep(.65,.86,-z) if queen else smoothstep(.76,.98,-z);c=smoothstep(1.05,1.27,-z) if queen else smoothstep(1.20,1.45,-z)
  return [1,base,base+1,base+2],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 raise ValueError('unowned armored royal anatomy: '+part)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid armored royal clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 # The low, heavy tail and torso yaw around the planted support plane.
 angles=[(0.,0.,0.),(0.,.006*b,0.),(.004*b,.009*a,0.),(.008*b,-.009*a,-.010*b),(.008*b,.009*a,.010*b),(.006*b,.006*a,0.)]
 if species=='nidoking':angles += [(.008*a,0.,0.),(0.,.012*b,0.)]
 for side in (-1,1):angles += [(.006*b,side*.006*a,side*.004*b),(.008*a,side*.006*b,0.),(.010*b,side*.008*a,side*.006*b)]
 angles += [(.004*b,-.006*a,0.),(-.004*b,.006*a,0.),(0.,.009*b,0.),(0.,.012*a,0.),(0.,.014*b,0.)]
 return angles
def pack_parts(species,model):
 parts=[];groups={}
 for order,p in enumerate(model['primitives']):
  name=p['part'];kind=name.split(' /')[0];key=kind if kind in ('Eye socket','Eye','Iris','Pupil','Glint','Nostril','Ear inset','Fang','Cheek barb','Back spine','Tail spine','Chest plate','Belly plate','Hand claw','Foot claw') else None
  leg=None
  if kind in ('Hand claw','Foot claw'):leg=(name.split()[-2] if kind=='Hand claw' or species=='nidoqueen' else name.split()[-1],)
  if leg is not None:key=(kind,*leg)
  if key is None:parts.append((p,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete armored royal assembly'+(' '+' '.join(leg) if leg is not None else ''),'positions':[],'normals':[],'indices':[],'base_color':p['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==p['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(p['positions'])//3,'index_start':first,'index_count':len(p['indices'])});joined['positions']+=p['positions'];joined['normals']+=p['normals'];joined['indices'] += [i+start for i in p['indices']]
 return parts
PACKED_REST_DIGESTS={'nidoqueen': '5b9950cf9bb193673825a3db9214c65d3f2eb037367486e563ec310be35d6132', 'nidoking': '0730055ac57991e822b343db39eb9020d0416bbd179956f8e49ee721f937c346'}
def export_nido_royal(species,model=None):
 if species not in SPECIES:raise ValueError('unknown armored royal species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed armored royal neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite original distinct armored royal creatures standard skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed armored royal geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed complete hind feet and claws; coherent fitted face and armor; supported arm and wrist chains; independent ears and horn; three yawing tail carriers; king lower jaw and tongue; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/nidoqueen_sculpt.py' if species=='nidoqueen' else 'tools/nidoking_sculpt.py','geometry_preservation':'original blue queen with paired chest shields and rounded ears; purple king with open toothed jaw and ragged ears; fitted eyes and segmented armor; substantial spined tails; complete grounded feet and per-limb claws',
    'animation_contract':'identity floor root; fixed complete hind feet and claws; coherent fitted armor and face; supported ankle fields; independent ears, horn, arms and wrists; king jaw and tongue; three yawing tail carriers; per-limb support assemblies; lossless source-solid packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'armored royal exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('armored royal exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('armored royal exceeds existing byte limit')
 return blob


def decode_nido_royal(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_nido_royal(species):raise ValueError('noncanonical armored royal GLB')
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


def read_nido_royal(path,species):
 with Path(path).open('rb') as f:return decode_nido_royal(f.read(MAX_BYTES+1),species)
def export_nidoqueen(model=None):return export_nido_royal('nidoqueen',model)
def export_nidoking(model=None):return export_nido_royal('nidoking',model)
def read_nidoqueen(path):return read_nido_royal(path,'nidoqueen')
def read_nidoking(path):return read_nido_royal(path,'nidoking')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--species',choices=SPECIES,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_nido_royal(a.species))
if __name__=='__main__':main()
