#!/usr/bin/env python3
"""Original striped fire canines with four supported paws and fitted coherent markings."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
import fire_canine_sculpt as source
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=source.SPECIES;HEIGHTS=source.HEIGHTS
@lru_cache(maxsize=2)
def authoring_model(species):return source.sculpture(species)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGESTS={'growlithe': '7b296f21adc1228824b647f6dabb74d2d87403ac0b70947aef11dd6031b3023e', 'arcanine': 'c045e509afaec7694872924d7f56edf85fc0ff5fd260a100a6777861ee265471'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=source.sculpture(species,False);lo=min(v for p in raw['primitives'] for v in p['positions'][1::3]);hi=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return lo,hi-lo
def normalized(species,point):
 lo,span=authoring_transform(species);return tuple((v-(lo if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))
def joints(species):
 big=species=='arcanine';hc=source.head_center(species);mc=source.mane_center(species);tail=source.tail_points(species)
 raw=[('root',None,(0.,0.,0.)),('body',0,(0.,.58 if big else .38,-.13 if big else -.035)),('neck',1,(0.,.58 if big else .38,.19 if big else .14)),('head',2,(0.,hc[1]-.035,hc[2]-.03)),('ear_left',3,source.ear_points(species,-1)[0]),('ear_right',3,source.ear_points(species,1)[0]),('jaw',3,(0.,.945 if big else .650,.56 if big else .415))]
 raw += [(name,1,source.leg_points(species,side,row)[0]) for name,side,row in (('fore_left',-1,0),('fore_right',1,0),('rear_left',-1,1),('rear_right',1,1))]
 raw += [('tail_base',1,tail[0]),('tail_mid',11,tail[2]),('tail_tip',12,tail[-2]),('mane',2,mc),('forelock',3,source.forelock_points(species)[0]),('ruff_left',14,(-.12 if big else -.09,mc[1],mc[2]+.01)),('ruff_right',14,(.12 if big else .09,mc[1],mc[2]+.01))]
 return tuple((n,parent,(0.,0.,0.) if parent is None else normalized(species,p)) for n,parent,p in raw)
JOINTS={s:joints(s) for s in SPECIES};CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(species,part,position):
 lo,span=authoring_transform(species);x,y,z=(v*span/HEIGHTS[species]+(lo if i==1 else 0.) for i,v in enumerate(position));big=species=='arcanine'
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Paw /','Paw crease /','Ankle fur /','Ankle tuft /')):return rigid(0)
 if part.startswith(('Body /','Flank stripe /','Back stripe /')):return rigid(1)
 if part.startswith(('Head /','Muzzle /','Nose /','Lip /','Eye socket /','Eye /','Pupil /','Eye glint /','Forehead stripe /')):return rigid(3)
 if part.startswith(('Ear /','Ear inset /')):return rigid(4 if part.endswith('-1') else 5)
 if part.startswith('Jaw /'):return rigid(6)
 if part.startswith(('Leg /','Leg stripe /')):
  side,row=map(int,part.split()[-2:]);owner=7+row*2+(side>0)
  if part.startswith('Leg stripe /'):return rigid(owner)
  a=smoothstep(.07,.23,y) if big else smoothstep(.06,.18,y);b=smoothstep(.52,.64,y) if big else smoothstep(.35,.45,y)
  return [0,owner,1,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Tail /','Tail tuft /')):
  a=smoothstep(.62,.75,y) if big else smoothstep(.43,.49,y);b=smoothstep(.79,.97,y) if big else smoothstep(.53,.62,y);c=smoothstep(1.04,1.12,y) if big else smoothstep(.66,.73,y)
  return [1,11,12,13],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 if part.startswith('Mane /'):return rigid(14)
 if part.startswith('Mane tuft /'):
  j=int(part.split()[-1]);angle=(j+.5)*math.tau/(10 if big else 7);ux,uy=math.cos(angle),math.sin(angle)
  return rigid(14 if abs(ux)<.30 or uy<-.70 else 16 if ux<0 else 17)
 if part.startswith('Forelock /'):return rigid(15)
 if part.startswith('Neck /'):
  a=smoothstep(.52,.76,y) if big else smoothstep(.38,.58,y);b=smoothstep(.82,.99,y) if big else smoothstep(.65,.76,y)
  return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 raise ValueError('unowned canine anatomy: '+part)
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid canine clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*18
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2. if clip=='attack' else -1.3
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.008*a,0.),(.008*b,.006*a,0.),(.009*b,.014*a,0.),(.012*b,-.012*a,-.018*b),(.012*b,.012*a,.018*b),(.010*a,0.,0.),(.010*b,-.012*a,0.),(-.010*b,.012*a,0.),(-.010*b,-.012*a,0.),(.010*b,.012*a,0.),(.010*b,.016*a,0.),(.012*a,.022*b,0.),(.016*b,.026*a,0.),(.006*b,.008*a,0.),(.012*b,.014*a,0.),(.010*b,-.012*a,-.014*b),(.010*b,.012*a,.014*b)]
def pack_parts(species,model):
 parts=[];groups={}
 for order,p in enumerate(model['primitives']):
  name=p['part'];kind=name.split(' /')[0];key=kind if kind in ('Eye socket','Eye','Pupil','Eye glint','Ear inset','Paw crease','Leg stripe','Ankle fur','Ankle tuft','Flank stripe','Back stripe','Mane tuft','Tail tuft') else None
  # World IK assigns whole runtime primitives to legs; support details stay local.
  leg=None
  if kind in ('Paw crease','Ankle tuft'):leg=tuple(map(int,name.split()[-3:-1]))
  elif kind in ('Leg stripe','Ankle fur'):leg=tuple(map(int,name.split()[-2:]))
  if leg is not None:key=(kind,*leg)
  if key is None:parts.append((p,{'source_order':order}));continue
  if key not in groups:
   joined={'part':kind+' / complete striped-canine assembly'+(' '+' '.join(map(str,leg)) if leg is not None else ''),'positions':[],'normals':[],'indices':[],'base_color':p['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==p['base_color'];start=len(joined['positions'])//3;first=len(joined['indices']);meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(p['positions'])//3,'index_start':first,'index_count':len(p['indices'])});joined['positions']+=p['positions'];joined['normals']+=p['normals'];joined['indices'] += [i+start for i in p['indices']]
 return parts
PACKED_REST_DIGESTS={'growlithe': '93977ea316f368e22fd7a1061f565e63ca5fd01264a654a6b628f2d65961ab99', 'arcanine': 'b8b53169922d6cb2ea77f87da6f84f68bb50279d875021a4bec7b884a9382b7e'}
def export_fire_canine(species,model=None):
 if species not in SPECIES:raise ValueError('unknown fire-canine species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed fire-canine neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite original striped puppy and full-maned guardian standard skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed fire-canine geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'four fixed complete paws; coherent fitted stripes and face; supported independent limbs; articulated ears, jaw, ruff and three tail carriers; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/fire_canine_sculpt.py','geometry_preservation':'original round striped puppy and long-legged guardian; cream ruffs, distinct forelocks and curled plumes; four rooted supports; closed smile and actual fitted eye, ear, flank, back and paw markings',
    'animation_contract':'identity floor root; four fixed complete paws and ankle fur; coherent fitted face, ear insets and tiger markings; supported hip fields; independent ears, jaw, forelock and ruff; three monotone tail carriers; lossless new source-solid packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'fire-canine exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('fire-canine exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('fire-canine exceeds existing byte limit')
 return blob


def decode_fire_canine(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_fire_canine(species):raise ValueError('noncanonical fire-canine GLB')
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


def read_fire_canine(path,species):
 with Path(path).open('rb') as f:return decode_fire_canine(f.read(MAX_BYTES+1),species)
def export_growlithe(model=None):return export_fire_canine('growlithe',model)
def export_arcanine(model=None):return export_fire_canine('arcanine',model)
def read_growlithe(path):return read_fire_canine(path,'growlithe')
def read_arcanine(path):return read_fire_canine(path,'arcanine')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,required=True);p.add_argument('--species',choices=SPECIES,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(export_fire_canine(a.species))
if __name__=='__main__':main()
