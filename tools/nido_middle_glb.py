#!/usr/bin/env python3
"""Original distinct middle poison-pin creatures with per-limb support packing and fitted markings."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import nidorina_sculpt as female_source
import nidorino_sculpt as male_source
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('nidorina','nidorino');HEIGHTS={'nidorina':female_source.HEIGHT,'nidorino':male_source.HEIGHT}
def sculpture(species,normalize=True):
 if species not in SPECIES:raise ValueError('unknown middle poison-pin species')
 return (female_source if species=='nidorina' else male_source).sculpture(normalize)
@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGESTS={'nidorina': 'cfc3978374144332ac6db5a703ee33ac0252cd51ebb33db26e4603f4b48594ae', 'nidorino': '1443837cd4fc174eb9f440a9a329b4533758245eb5ec3099d576fcd26530a505'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species):
 female=species=='nidorina'
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.42,-.04) if female else (0.,.39,-.148)),('head',1,(0.,.69,.10) if female else (0.,.542,.023))]
 raw += [('ear_left',2,(-.14,.98,.03) if female else (-.15,.70,.25)),('ear_right',2,(.14,.98,.03) if female else (.15,.70,.25))]
 if female:
  for side in (-1,1):
   first=len(raw);raw += [('arm_'+('left' if side<0 else 'right'),1,(side*.203,.585,.075)),('forearm_'+('left' if side<0 else 'right'),first,(side*.275,.456,.151)),('wrist_'+('left' if side<0 else 'right'),first+1,(side*.277,.323,.237))]
  raw += [('leg_left',1,(-.212,.238,-.098)),('leg_right',1,(.212,.238,-.098)),('tail_base',1,(0.,.291,-.205)),('tail_tip',13,(0.,.311,-.390))]
 else:
  raw += [('jaw',2,(0.,.352,.289)),('horn',2,(0.,.591,.378))]
  for row in range(2):
   for side in (-1,1):raw.append((('fore_' if row==0 else 'rear_')+('left' if side<0 else 'right'),1,(side*.239,.283,.181 if row==0 else -.406)))
  raw += [('tail_base',1,(0.,.360,-.479)),('tail_tip',11,(0.,.366,-.612))]
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for name,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);x,y,z=(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));female=species=='nidorina'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Foot /','Foot claw /','Claw /')):return rigid(0)
 if part.startswith(('Body /','Breast /','Flank spot /','Back spine /')):return rigid(1)
 if part.startswith(('Head /','Muzzle /','Eye socket /','Eye /','Iris /','Pupil /','Glint /','Nostril /','Fang /','Head spot /')):return rigid(2)
 if part.startswith(('Ear /','Ear inset /','Ear fold /')):return rigid(3 if part.endswith('-1') else 4)
 if part.startswith(('Mouth /','Jaw /')):return rigid(2 if female else 5)
 if part.startswith('Horn /'):return rigid(6)
 if female:
  if part.startswith(('Hand /','Hand claw /')):
   side=int(part.split()[-2] if part.startswith('Hand claw /') else part.split()[-1]);return rigid(7 if side<0 else 10)
  if part.startswith('Arm /'):
   side=int(part.split()[-1]);base=5 if side<0 else 8;a=smoothstep(.34,.46,y);b=smoothstep(.49,.57,y)
   return [base+2,base+1,base,0],[1.-a,a*(1.-b),a*b,0.]
  if part.startswith(('Haunch /','Haunch spot /','Leg /')):
   side=int(part.split()[-2] if part.startswith('Haunch spot /') else part.split()[-1]);owner=11 if side<0 else 12
   if not part.startswith('Leg /'):return rigid(owner)
   a=smoothstep(.055,.135,y);return [0,owner,0,0],[1.-a,a,0.,0.]
 else:
  if part.startswith('Haunch /'):return rigid(1)
  if part.startswith(('Leg /','Leg spot /')):
   side,row=part.split()[-3:-1] if part.startswith('Leg spot /') else part.split()[-2:];owner=7+2*(row=='rear')+(int(side)>0)
   if part.startswith('Leg spot /'):return rigid(owner)
   a=smoothstep(.045,.115,y);b=smoothstep(.28,.32,y);return [0,owner,1,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Tail /'):
  a=smoothstep(.24,.36,-z) if female else smoothstep(.49,.59,-z);b=smoothstep(.40,.50,-z) if female else smoothstep(.63,.70,-z);base=13 if female else 11
  return [1,base,base+1,0],[1.-a,a*(1.-b),a*b,0.]
 raise ValueError('unowned middle poison-pin anatomy: '+part)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid middle poison-pin clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 common=[(0.,0.,0.),(.004*a,.008*b,0.),(.008*b,.014*a,0.),(.012*b,-.016*a,-.020*b),(.012*b,.016*a,.020*b)]
 if species=='nidorina':return common+[(.012*b,-.010*a,-.008*b),(.014*a,-.010*b,0.),(.018*b,.012*a,-.012*b),(.012*b,.010*a,.008*b),(.014*a,.010*b,0.),(.018*b,-.012*a,.012*b),(.014*b,-.010*a,0.),(-.014*b,.010*a,0.),(.018*a,.024*b,0.),(.022*b,.032*a,0.)]
 return common+[(.010*a,0.,0.),(.008*b,.006*a,0.),(.020*b,-.012*a,0.),(-.020*b,.012*a,0.),(-.020*b,-.012*a,0.),(.020*b,.012*a,0.),(.018*a,.024*b,0.),(.022*b,.032*a,0.)]
def pack_parts(species,model):
 parts=[];groups={}
 for order,p in enumerate(model['primitives']):
  name=p['part'];kind=name.split(' /')[0];key=kind if kind in ('Eye socket','Eye','Iris','Pupil','Glint','Nostril','Ear inset','Ear fold','Fang','Back spine','Flank spot','Head spot','Hand claw','Foot claw','Haunch spot','Claw','Leg spot') else None
  leg=None
  if kind in ('Hand claw','Foot claw','Haunch spot'):leg=(name.split()[-2],)
  elif kind in ('Claw','Leg spot'):leg=tuple(name.split()[-3:-1])
  if leg is not None:key=(kind,*leg)
  if key is None:parts.append((p,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete middle poison-pin assembly'+(' '+' '.join(leg) if leg is not None else ''),'positions':[],'normals':[],'indices':[],'base_color':p['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==p['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(p['positions'])//3,'index_start':first,'index_count':len(p['indices'])});joined['positions']+=p['positions'];joined['normals']+=p['normals'];joined['indices'] += [i+start for i in p['indices']]
 return parts
PACKED_REST_DIGESTS={'nidorina': '85c4a8e56049919dc4ca944a15f2711111399da8b3542cf1d4e38fffdd3f9b5e', 'nidorino': '6a11581fdc1fd23d94214c2430bf1c33f8bd92bccc9e94e7abbffdf8f9ccd688'}
def export_nido_middle(species,model=None):
 if species not in SPECIES:raise ValueError('unknown middle poison-pin species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed middle poison-pin neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite original distinct middle poison-pin creatures standard skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed middle poison-pin geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed complete hind paws or four quadruped paws; coherent fitted face and spots; supported independent limbs; articulated pinnae and two tail carriers; blue forearm/wrist chains or purple jaw and horn; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/nidorina_sculpt.py' if species=='nidorina' else 'tools/nidorino_sculpt.py','geometry_preservation':'original hornless blue biped and low purple horned quadruped; notched versus ragged ears; fitted eyes, belly and spots; rooted jaw, horn, spines and tail; complete grounded paws and separate per-limb nails',
    'animation_contract':'identity floor root; fixed complete hind paws and claws or four fixed quadruped paws; coherent fitted face and spots; supported ankle fields; independent ears and tail; articulated blue forearms/wrists or purple jaw/horn; per-limb support assemblies; lossless source-solid packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'middle poison-pin exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('middle poison-pin exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('middle poison-pin exceeds existing byte limit')
 return blob


def decode_nido_middle(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_nido_middle(species):raise ValueError('noncanonical middle poison-pin GLB')
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


def read_nido_middle(path,species):
 with Path(path).open('rb') as f:return decode_nido_middle(f.read(MAX_BYTES+1),species)
def export_nidorina(model=None):return export_nido_middle('nidorina',model)
def export_nidorino(model=None):return export_nido_middle('nidorino',model)
def read_nidorina(path):return read_nido_middle(path,'nidorina')
def read_nidorino(path):return read_nido_middle(path,'nidorino')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--species',choices=SPECIES,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_nido_middle(a.species))
if __name__=='__main__':main()
