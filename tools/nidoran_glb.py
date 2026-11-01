#!/usr/bin/env python3
"""Original distinct poison-pin quadrupeds with per-limb support packing and fitted markings."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import nidoran_sculpt as source
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=source.SPECIES;HEIGHTS=source.HEIGHTS
@lru_cache(maxsize=2)
def authoring_model(species):return source.sculpture(species)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGESTS={'nidoran_f': '17ab9379b5525c666b9c429dcd96f5ce773845daa2836ed30fa3c5896229232d', 'nidoran_m': 'd1a2dbb46f39b1285f5a1aace952caa2c466c8ff40a735e74fb02efd5b043198'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=source.sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species):
 female=species=='nidoran_f'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.25,-.12)),('head',1,(0.,.49,.07) if female else (0.,.35,.08)),('ear_left',2,(-.14,.72,.06) if female else (-.13,.49,.17)),('ear_right',2,(.14,.72,.06) if female else (.13,.49,.17))]
 for row in range(2):
  z=(.108 if row==0 else -.216) if female else (.185 if row==0 else -.306)
  for side in (-1,1):raw.append((('fore_' if row==0 else 'rear_')+('left' if side<0 else 'right'),1,(side*.170,.17,z)))
 tail=[(0.,.277,-.277),(0.,.301,-.401)] if female else [(0.,.215,-.393),(0.,.234,-.460)]
 raw += [('tail_base',1,tail[0]),('tail_tip',9,tail[1])]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for name,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);x,y,z=(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));female=species=='nidoran_f'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Claw /')):return rigid(0)
 if part.startswith(('Body /','Flank spot /','Back spine /','Haunch /','Haunch spot /')):return rigid(1)
 if part.startswith(('Head /','Muzzle /','Horn /','Eye socket /','Eye /','Iris /','Pupil /','Glint /','Nostril /','Mouth /','Incisor /','Cheek barb /')):return rigid(2)
 if part.startswith(('Ear /','Ear inset /','Ear fold /')):return rigid(3 if part.endswith('-1') else 4)
 if part.startswith('Leg /'):
  side=int(part.split()[-2]);row=part.split()[-1]=='rear';owner=5+2*row+(side>0);a=smoothstep(.035,.095,y);b=smoothstep(.13,.18,y)
  return [0,owner,1,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Tail /'):
  a=smoothstep(.30,.38,-z) if female else smoothstep(.40,.45,-z);b=smoothstep(.39,.44,-z) if female else smoothstep(.46,.50,-z)
  return [1,9,10,0],[1.-a,a*(1.-b),a*b,0.]
 raise ValueError('unowned poison-pin anatomy: '+part)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid poison-pin clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*11
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 return [(0.,0.,0.),(.004*a,.010*b,0.),(.012*b,.018*a,0.),(.012*b,-.018*a,-.025*b),(.012*b,.018*a,.025*b),(.024*b,-.012*a,0.),(-.024*b,.012*a,0.),(-.024*b,-.012*a,0.),(.024*b,.012*a,0.),(.018*a,.026*b,0.),(.024*b,.035*a,0.)]
def pack_parts(species,model):
 parts=[];groups={}
 for order,p in enumerate(model['primitives']):
  name=p['part'];kind=name.split(' /')[0];key=kind if kind in ('Eye socket','Eye','Iris','Pupil','Glint','Nostril','Ear inset','Ear fold','Claw','Haunch spot','Flank spot','Cheek barb','Back spine','Incisor') else None
  if kind=='Haunch spot':key=None
  if kind=='Claw':key=('Claw',*name.split()[-3:-1])
  if key is None:parts.append((p,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete poison-pin assembly'+(' '+' '.join(name.split()[-3:-1]) if kind=='Claw' else ''),'positions':[],'normals':[],'indices':[],'base_color':p['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==p['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(p['positions'])//3,'index_start':first,'index_count':len(p['indices'])});joined['positions']+=p['positions'];joined['normals']+=p['normals'];joined['indices'] += [i+start for i in p['indices']]
 return parts
PACKED_REST_DIGESTS={'nidoran_f': '8bd3d7aefc6dd2bb59d701de719be620443ce0c1fc4aaaeb4bcdecc0a9c03802', 'nidoran_m': '5d11703a3c1639cd8682d31e6d0466e48540efb084afadfbc04d29dfde753c81'}
def export_nidoran(species,model=None):
 if species not in SPECIES:raise ValueError('unknown poison-pin species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed poison-pin neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite original distinct small poison-pin quadrupeds standard skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed poison-pin geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'four fixed complete paws and claws; coherent fitted face and spots; supported independent limbs; articulated pinnae and two tail carriers; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/nidoran_sculpt.py','geometry_preservation':'original round pale female and long crouching purple male; wide notched versus upright ragged ears; rooted horn, cheek barbs and back spines; four complete grounded paws; triangle-fitted facial, pinna and flank markings',
    'animation_contract':'identity floor root; four fixed complete paws and eight claws; coherent fitted face, ear insets and body spots; supported ankle fields; independent ears and two monotone tail carriers; lossless source-solid packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'poison-pin exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('poison-pin exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('poison-pin exceeds existing byte limit')
 return blob


def decode_nidoran(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_nidoran(species):raise ValueError('noncanonical poison-pin GLB')
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


def read_nidoran(path,species):
 with Path(path).open('rb') as f:return decode_nidoran(f.read(MAX_BYTES+1),species)
def export_nidoran_f(model=None):return export_nidoran('nidoran_f',model)
def export_nidoran_m(model=None):return export_nidoran('nidoran_m',model)
def read_nidoran_f(path):return read_nidoran(path,'nidoran_f')
def read_nidoran_m(path):return read_nidoran(path,'nidoran_m')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--species',choices=SPECIES,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_nidoran(a.species))
if __name__=='__main__':main()
