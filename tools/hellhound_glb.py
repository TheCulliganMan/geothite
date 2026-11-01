#!/usr/bin/env python3
"""Standard Houndour/Houndoom skins with planted armored paws and articulated canine anatomy."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from hellhound_sculpt import sculpture,HEIGHT,SPECIES
HEIGHTS={s:HEIGHT for s in SPECIES}
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep

def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()

@lru_cache(maxsize=2)
def authoring_model(species):return sculpture(species)
REST_DIGESTS={'houndour': 'f35d0751435b77708a37e8d0c8f5ece071f3de0b65c8fb2586c3775b5d9c1ce6', 'houndoom': '4fe7c0e7bec59d663b355075b40d4ecea6f9ee20620fa2a9da28d1145fb15a80'}
PARTS={species:tuple(p['part'] for p in authoring_model(species)['primitives']) for species in SPECIES}
MAX_BYTES=1024*1024

@lru_cache(maxsize=2)
def authoring_transform(species):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHTS[species]/span for i,v in enumerate(point))

def tail_path(species):
 if species=='houndour':return ((0.,.40,-.28),(.015,.43,-.40),(.040,.49,-.54))
 points=[(0.,.53,-.325),(0.,.52,-.43)]
 for k in range(20):
  a=-math.pi/2+(2.3+math.pi/2)*k/19;points.append((.014*math.sin(a),.76+.25*math.sin(a),-.52-.26*math.cos(a)))
 return tuple(points)
def joints(species):
 big=species=='houndoom';path=tail_path(species)
 raw=[('root',None,(0.,0.,0.)),('torso',0,(0.,.53 if big else .41,-.08)),
 ('neck',1,(0.,.42 if big else .39,.19 if big else .17)),
 ('head',2,(0.,.79 if big else .63,.30 if big else .28))]
 if big:raw.append(('jaw',3,(0.,.771,.36)))
 for side in (-1,1):
  for row in (0,1):raw.append(('leg_'+('left' if side<0 else 'right')+('_front' if row==0 else '_rear'),0,
   (side*(.14 if row==0 else .165),.34 if row==0 else .23,.17 if row==0 else -.18) if big else (side*(.12 if row==0 else .15),.27,.21 if row==0 else -.24)))
 base=len(raw);raw.extend((('tail_base',1,path[0]),('tail_bend',base,path[11] if big else path[1]),('tail_tip',base+1,path[-1])))
 if not big:
  for side in (-1,1):raw.append(('ear_'+('left' if side<0 else 'right'),3,(side*.065,.813,.24)))
 return tuple((name,parent,(0.,0.,0.) if parent is None else normalized(species,pivot)) for name,parent,pivot in raw)
JOINTS={species:joints(species) for species in SPECIES}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  best=min(best,(math.dist(point,[x+t*d for x,d in zip(a,delta)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(species,part,position):
 low,span=authoring_transform(species);raw=tuple(v*span/HEIGHT+(low if i==1 else 0.) for i,v in enumerate(position));x,y,z=raw;big=species=='houndoom'
 if part.startswith(('Paw /','Paw crease /','Claw /','Cuff /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=(5 if big else 4)+(0 if side<0 else 2)+row
  a=smoothstep(.30 if big else .26,.50 if big else .43,y)
  g=.7*smoothstep(.24 if big else .20,.30 if big else .26,y)*(1.-smoothstep(.40 if big else .35,.49 if big else .43,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith('Body /') or part.startswith('Armor / broad'):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Neck /','Chest /')):
  a=smoothstep(.42 if big else .39,.60 if big else .53,y);b=smoothstep(.79 if big else .63,.89 if big else .73,y)
  return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Collar /','Charm /')):return [2,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Ear /'):return [11 if part.endswith('-1') else 12,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail / pointed'):return [11,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  t=path_fraction(raw,tail_path(species));a=smoothstep(0.,.25,t);b=smoothstep(.30,.70,t);c=smoothstep(.70,.95,t);base=9 if big else 8
  return [1,base,base+1,base+2],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 if part.startswith(('Jaw /','Mouth / pink')):return [4,0,0,0],[1.,0.,0.,0.]
 return [3,0,0,0],[1.,0.,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid hellhound clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS[species])
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength;big=species=='houndoom'
 result=[(0.,0.,0.),(0.,.015*a,0.),(0.,.015*a,0.),(.020*a,.030*b,.006*b)]
 if big:result.append((.030*a,0.,0.))
 result.extend((0.,(.018 if i<2 else -.018)*a,0.) for i in range(4))
 result.extend(((0.,.020*a,0.),(.020*b,.025*b,0.),(.015*b,.035*b,0.)))
 if not big:result.extend(((.015*b,0.,-.025*b),(.015*b,0.,.025*b)))
 return result

def pack_parts(model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  if not source['part'].startswith('Claw /'):
   parts.append((source,{'source_order':order}));continue
  key=tuple(source['part'].split()[-3:-1])
  if key not in groups:
   joined={'part':'Claw / pale hooked toe group '+' '.join(key),'positions':[],'normals':[],'indices':[],'base_color':source['base_color']}
   metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':source['part'],'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'houndour': 'f35d0751435b77708a37e8d0c8f5ece071f3de0b65c8fb2586c3775b5d9c1ce6', 'houndoom': '2d2d89e4bd3ce410dfbbeee0d49659d1c6b78e4f9d0e143e14fa02b54bfde878'}

def export_hellhound(species,model=None):
 if species not in SPECIES:raise ValueError('unknown hellhound species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed hellhound neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite bone-armored canine family standard skin v1'
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
 packed_parts=pack_parts(model)
 if geometry_digest({'primitives':[p for p,m in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed hellhound anatomy')
 for part,metadata in packed_parts:
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted paws and ankle bands; fitted dorsal armor; coherent canine faces, skull pendant and horns; segmented tail and spade; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],
    'source_part_ranges':[metadata for part,metadata in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':HEIGHTS[species],'body_reference_height':HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/hellhound_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity floor root; fixed paws, toe clefts, claws and ankle cuffs; rigid dorsal armor on torso; fitted chest shares neck field; coherent skull pendant; rigid horns on head; articulated jaw, leg middles and segmented tail; puppy ears; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'hellhound exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_hellhound(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_hellhound(species):raise ValueError('noncanonical hellhound GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 parts=[]
 for p,extra in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
  positions=values(p['attributes']['POSITION']);normals=values(p['attributes']['NORMAL']);indices=values(p['indices']);color=doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']
  sources=extra.get('source_parts',[{'part':p['extras']['part'],'order':extra.get('source_order'),'vertex_start':0,'vertex_count':len(positions)//3,'index_start':0,'index_count':len(indices)}])
  for src in sources:
   a=src['vertex_start'];b=a+src['vertex_count'];i=src['index_start'];j=i+src['index_count']
   parts.append((src['order'],{'part':src['part'],'positions':positions[3*a:3*b],'normals':normals[3*a:3*b],'indices':[k-a for k in indices[i:j]],'base_color':color}))
 return {'name':species,'version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}


def read_hellhound(path,species):
 with Path(path).open('rb') as f:return decode_hellhound(f.read(MAX_BYTES+1),species)
def export_houndour(model=None):return export_hellhound('houndour',model)
def export_houndoom(model=None):return export_hellhound('houndoom',model)
def read_houndour(path):return read_hellhound(path,'houndour')
def read_houndoom(path):return read_hellhound(path,'houndoom')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--species',choices=SPECIES,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.parent.mkdir(parents=True,exist_ok=True);blob=export_hellhound(a.species);a.output.write_bytes(blob);print(a.species,len(blob),'bytes',len(PARTS[a.species]),'parts',len(JOINTS[a.species]),'joints',REST_DIGESTS[a.species])
if __name__=='__main__':main()
