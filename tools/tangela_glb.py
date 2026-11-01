#!/usr/bin/env python3
"""Canonical Tangela with rooted individual curls, coherent eye frame and planted red shoes."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from functools import lru_cache
from tangela_sculpt import sculpture as source_sculpture,VINE_PATHS,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
SPECIES=('tangela',);HEIGHTS={'tangela':HEIGHT}
def sculpture(species='tangela',normalize=True):
 if species!='tangela':raise ValueError(species)
 return source_sculpture(normalize=normalize)
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
@lru_cache(maxsize=1)
def authoring_model(species='tangela'):return sculpture(species)
REST_DIGESTS={'tangela':'e8705e62e57b9a088c5df2f38d475e0ec2895b8311a504ef065d2e53f0527064'}
PARTS={s:tuple(p['part'] for p in authoring_model(s)['primitives']) for s in SPECIES}
BODY_REFERENCE_HEIGHTS=HEIGHTS.copy();MAX_BYTES=1024*1024
@lru_cache(maxsize=1)
def authoring_transform(species='tangela'):
 raw=sculpture(species,normalize=False);low=min(v for p in raw['primitives'] for v in p['positions'][1::3]);high=max(v for p in raw['primitives'] for v in p['positions'][1::3]);return low,high-low

def normalized(species,point):
 low,span=authoring_transform(species);return tuple((v-(low if i==1 else 0.))*HEIGHT/span for i,v in enumerate(point))
VINES=tuple(VINE_PATHS)
JOINTS={'tangela':(('root',None,(0.,0.,0.)),('body',0,normalized('tangela',(0.,.48,0.))))+tuple(('vine_'+str(i).zfill(2),1,normalized('tangela',VINE_PATHS[name][0])) for i,name in enumerate(VINES))}
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
@lru_cache(maxsize=37)
def path_segments(name):
 path=VINE_PATHS[name];lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.;result=[]
 for a,b,length in zip(path,path[1:],lengths):
  axis=tuple(b[k]-a[k] for k in range(3));result.append((a,axis,length,offset/total,length/total));offset+=length
 return tuple(result)
def path_fraction(name,point):
 best=(math.inf,0.)
 for a,axis,length,offset,fraction in path_segments(name):
  t=max(0.,min(1.,sum((point[k]-a[k])*axis[k] for k in range(3))/(length*length)));distance=sum((point[k]-a[k]-t*axis[k])**2 for k in range(3));best=min(best,(distance,offset+t*fraction))
 return best[1]
def skin_weights(species,part,position):
 if part.startswith('Shoe /'):return [0,0,0,0],[1.,0.,0.,0.]
 if part not in VINE_PATHS:return [1,0,0,0],[1.,0.,0.,0.]
 low,span=authoring_transform(species);point=tuple(v*span/HEIGHT+(low if i==1 else 0.) for i,v in enumerate(position));t=path_fraction(part,point)
 a=smoothstep(.015,.12,t) if part.startswith('Rear vine /') else smoothstep(.04,.20,t);joint=2+VINES.index(part)
 return [1,joint,0,0],[1.-a,a,0.,0.]
def pose_angles(species,clip,t):
 if species not in SPECIES or clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Tangela clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*39
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=1.8 if clip=='attack' else -1.2
 out=[(0.,0.,0.),(0.,.010*a*strength,0.)]
 for i,name in enumerate(VINES):
  phase=i*2.399963229728653
  if clip=='idle':
   c=math.sin(math.tau*t+phase)-math.sin(phase);d=math.sin(math.tau*t+phase+.7)-math.sin(phase+.7)
  else:c=a*strength;d=b*strength
  out.append((math.sin(phase+.3)*.010*c,math.cos(phase+.5)*.014*d,math.sin(phase+1.)*.010*c))
 return out

def pack_parts(species,model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key=None;label=name
  if name.startswith(('Face /','Pupil /')):key='face';label='Face / coherent dark Tangela eye opening and pupils'
  elif name.startswith('Eye /'):key='eyes';label='Eye / paired rooted white Tangela gaze'
  elif name.startswith(('Front vine /','Front weave /','Rear vine /','Rear weave /')):
   category=name.split(' /')[0];color=tuple(source['base_color']);key=(category,color);label=category+' / individually rooted Tangela strands '+('dark' if category=='Rear vine' and color==tuple(model['primitives'][0]['base_color']) else 'blue')
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};meta={'source_parts':[]};groups[key]=(joined,meta);parts.append((joined,meta))
  joined,meta=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  meta['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])});joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGESTS={'tangela':'9122c50cf61e0829b7d88147233af06cdfc8e5ba7ee98d917c864bc2b918b416'}

def export_vine_tangle(species,model=None):
 if species not in SPECIES:raise ValueError('unknown vine_tangle species')
 if model is None:model=authoring_model(species)
 if (model.get('name')!=species or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS[species]
     or geometry_digest(model)!=REST_DIGESTS[species]):raise ValueError('unreviewed vine_tangle neutral geometry')
 g=Glb();g.doc['asset']['generator']='Geothite vine_tangle rooted individual Tangela curls, coherent eye frame and planted shoes skin v1'
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
 if geometry_digest({'primitives':[p for p,meta in packed_parts]})!=PACKED_REST_DIGESTS[species]:raise ValueError('unreviewed packed vine_tangle geometry')
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
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'fixed red shoes; coherent body, gaze and four eye-frame ropes; thirty-seven independently phased curls with embedded roots; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':species,'source_sha256':REST_DIGESTS[species],'source_part_ranges':[meta for p,meta in packed_parts],'packed_sha256':PACKED_REST_DIGESTS[species],
    'reference_span':BODY_REFERENCE_HEIGHTS[species],'body_reference_height':BODY_REFERENCE_HEIGHTS[species],'full_silhouette_height':HEIGHTS[species],
    'geometry_source':'tools/tangela_sculpt.py','geometry_preservation':'all forty-one original closed vines, fitted face and broad red shoes; reduced redundant path/radial sampling and smooth rope normals; lossless material packing preserves every logical solid',
    'animation_contract':'identity floor root; fixed red shoes; coherent round body, fitted eyes and pupils and four framing ropes; thirty-seven independently phased vine carriers blend from deeply seated roots; lossless material packing; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'vine_tangle exceeds existing accessor limit: {len(g.doc["accessors"])}')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('vine_tangle exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('vine_tangle exceeds existing byte limit')
 return blob


def decode_vine_tangle(blob,species):
 if len(blob)>MAX_BYTES or blob!=export_vine_tangle(species):raise ValueError('noncanonical vine_tangle GLB')
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


def read_vine_tangle(path,species):
 with Path(path).open('rb') as f:return decode_vine_tangle(f.read(MAX_BYTES+1),species)

def export_tangela(model=None):return export_vine_tangle('tangela',model)
def read_tangela(path):return read_vine_tangle(path,'tangela')
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True);blob=export_tangela();(a.out/'tangela.glb').write_bytes(blob);print(len(blob),'bytes',len(pack_parts('tangela',authoring_model())),'runtime parts',REST_DIGESTS,PACKED_REST_DIGESTS,flush=True)
if __name__=='__main__':main()
