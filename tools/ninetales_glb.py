#!/usr/bin/env python3
"""Canonical elegant nine-tailed fox with planted paws and nine independently flowing gold-tipped plumes."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from ninetales_sculpt import sculpture,HEIGHT,TAIL_PATHS
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
MAX_BYTES=1024*1024
SOURCE=sculpture();RAW=sculpture(normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
PARTS=tuple(p['part'] for p in SOURCE['primitives'])
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGEST='f435a002995098a49bd288d358210163951d154a7098a145dfeb064d2bd40777'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
def core(p):return p
PATHS=tuple(tuple(core(p) for p in path) for path in TAIL_PATHS)
BASE=[('root',None,(0.,0.,0.)),('body',0,core((0.,.38,-.07))),
 ('neck',1,core((0.,.51,.16))),('head',2,core((0.,.72,.25))),
 ('ear_left',3,core((-.095,.788,.185))),('ear_right',3,core((.095,.788,.185))),
 ('tail_fan',1,core((0.,.36,-.29)))]
for j,path in enumerate(PATHS):BASE.append(('tail_'+str(j),6,path[2]))
for side in (-1,1):
 for row in (0,1):BASE.append(('leg_'+str(side)+'_'+str(row),0,(side*(.090 if row==0 else .123),.21,.155 if row==0 else -.245)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point,path):
 lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.;best=(math.inf,0.)
 for a,b,length in zip(path,path[1:],lengths):
  delta=[b[k]-a[k] for k in range(3)];t=max(0.,min(1.,sum((point[k]-a[k])*delta[k] for k in range(3))/(length*length)))
  best=min(best,(math.dist(point,[a[k]+t*delta[k] for k in range(3)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(part,point):
 x,y,z=[v/SCALE+(LOW if i==1 else 0.) for i,v in enumerate(point)]
 if part.startswith(('Paw /','Toe crease /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Shoulder /','Haunch /')):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Mane /','Mane tuft /')):return [2,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Neck /'):
  a=smoothstep(.39,.49,y);b=smoothstep(.53,.61,y);return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Ear /','Ear inset /')):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=16+(0 if side<0 else 2)+row
  a=smoothstep(.13,.31,y);g=.70*smoothstep(.055,.130,y)*(1.-smoothstep(.270,.370,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Tail /','Tail tip /')):
  j=int(part.split()[-1]);t=path_fraction((x,y,z),PATHS[j]);a=smoothstep(0.,.22,t);b=smoothstep(.12,.40,t)
  return [1,6,7+j,0],[1.-a,a*(1.-b),a*b,0.]
 return [3,0,0,0],[1.,0.,0.,0.]

def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Ninetales clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);body=.002*a;neck=.006*b;head=.012*b;turn=.018*a;ears=.035*b;tail=.035*a;leg=.006*a
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=1. if clip=='attack' else -.65
  body=.007*a*gain;neck=.018*a*gain;head=.040*b*gain;turn=.030*b*gain;ears=.075*b*gain;tail=.090*b*gain;leg=.085*a*gain
 tips=[]
 for j in range(9):
  phase=1.+.055*j;sign=-1. if j%2==0 else 1.
  tips.append((tail*.15*phase,tail*.50*sign,tail*.35*phase*sign))
 return [(0.,0.,0.),(body,0.,0.),(neck,0.,0.),(head,turn,0.),
  (ears*.35,ears*.20,-ears),(ears*.35,-ears*.20,ears),(0.,tail*.3,0.),
  *tips,(leg,0.,0.),(-leg*.75,0.,0.),(-leg*.85,0.,0.),(leg*.65,0.,0.)]

def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def pack_parts(model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part'];key='tips' if name.startswith('Tail tip /') else 'crest' if name.startswith('Crest /') else None
  if key is None:parts.append((source,{'source_order':order}));continue
  if key not in groups:
   label='Tail tip / coherent nine golden fox ends' if key=='tips' else 'Crest / coherent three swept cream fox wisps'
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']};metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];assert joined['base_color']==source['base_color'];start=len(joined['positions'])//3;first=len(joined['indices'])
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGEST='d3c5cac139c338a855ec83edbb1e3df15cbfddb68f7affd4a59f6564301173c3'

def export_ninetales(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='ninetales' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Ninetales anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Ninetales elegant fox, cream mane and nine independently flowing golden-tipped plumes skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'ninetales/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'ninetales','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'ninetales','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'ninetales','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 packed_parts=pack_parts(model)
 if geometry_digest({'primitives':[p for p,metadata in packed_parts]})!=PACKED_REST_DIGEST:raise ValueError('unreviewed packed Ninetales geometry')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'ninetales/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  ranges=metadata.get('source_parts',[{'part':part['part'],'vertex_start':0,'vertex_count':len(part['positions'])//3}])
  for source in ranges:
   for i in range(source['vertex_start']*3,(source['vertex_start']+source['vertex_count'])*3,3):
    point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(source['part'],point);q=quantized_weights(ws)
    ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'ninetales','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'ninetales.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent long fox muzzle, red gaze and swept crest; alert ears and cream mane; planted paws; nine independently flowing plumes with closed golden tip seams; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'ninetales','source_sha256':REST_DIGEST,
    'source_part_ranges':[metadata for p,metadata in packed_parts],'packed_sha256':PACKED_REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/ninetales_sculpt.py','geometry_preservation':'original elegant cream fox, long muzzle, sharp red gaze, full mane and swept crest, four planted paws and nine flowing cream plumes with golden ends',
    'animation_contract':'identity authoring root; four fixed ground paws; blended short limbs; coherent head, muzzle and gaze; independent ears; shared tail-fan carrier and nine continuously blended independent plumes with identical weights across all golden material seams; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Ninetales exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Ninetales exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Ninetales exceeds existing byte limit')
 return blob


def decode_ninetales(blob):
 if len(blob)>MAX_BYTES or blob!=export_ninetales():raise ValueError('noncanonical Ninetales GLB')
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
 return {'name':'ninetales','version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}

def read_ninetales(path):
 with Path(path).open('rb') as f:return decode_ninetales(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_ninetales();(a.out/'ninetales.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
