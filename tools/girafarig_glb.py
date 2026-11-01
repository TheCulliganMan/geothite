#!/usr/bin/env python3
"""Canonical Girafarig skin with planted hooves, fitted neck markings and an independent second head."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from hoofed_sculpt import sculpture,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
MAX_BYTES=1024*1024
SOURCE=sculpture('girafarig');RAW=sculpture('girafarig',normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
PARTS=tuple(p['part'] for p in SOURCE['primitives'])
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGEST='9b38583cbb8ceee957133a2774eebd309860117c5f3014922201877716781810'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.49,-.07)),
 ('neck',1,(0.,.61,.174)),('head',2,(0.,1.02,.258)),
 ('ear_left',3,(-.10,1.18,.236)),('ear_right',3,(.10,1.18,.236)),
 ('leg_left_front',0,(-.14,.30,.195)),('leg_left_rear',0,(-.16,.34,-.265)),
 ('leg_right_front',0,(.14,.30,.195)),('leg_right_rear',0,(.16,.34,-.265)),
 ('tail_base',1,(0.,.51,-.285)),('tail_bend',10,(.03,.855,-.512)),('tail_head',11,(.042,.977,-.552)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
TAIL=((0.,.51,-.285),(.008,.67,-.405),(.03,.855,-.512),(.042,.977,-.552))
def path_fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  best=min(best,(math.dist(point,[x+t*d for x,d in zip(a,delta)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 if part.startswith(('Hoof /','Hoof groove /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=6+(0 if side<0 else 2)+row
  a=smoothstep(.08,.53,y);g=.65*smoothstep(.08,.18,y)*(1.-smoothstep(.40,.55,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith('Ear /'):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Head /','Eye /','Mouth /','Muzzle /','Horn /','Horn tip /')):return [3,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Tail head /','Tail eye /','Tail mouth /','Tail tooth /')):return [12,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  t=path_fraction((x,y,z),TAIL);a=smoothstep(0.,.25,t);b=smoothstep(.25,.80,t);c=smoothstep(.65,1.,t)
  return [1,10,11,12],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 if part.startswith(('Neck /','Mane / pink neck','Spot / fitted brown neck','Spot center / fitted dark neck')) or (part.startswith(('Spot / fitted brown shoulder','Spot center / fitted dark shoulder')) and part.endswith('1')):
  a=smoothstep(.48,.65,y);b=smoothstep(.94,1.065,y)
  return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 return [1,0,0,0],[1.,0.,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Girafarig clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.015*a,0.),(0.,.018*a,0.),(.010*a,.020*a,0.),
  (.020*b,0.,-.025*b),(.020*b,0.,.025*b),
  (0.,.018*a,0.),(0.,-.018*a,0.),(0.,-.018*a,0.),(0.,.018*a,0.),
  (0.,.020*a,0.),(.018*b,.025*b,0.),(.020*b,-.030*b,.010*b)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def pack_parts(model):
 parts=[];groups={}
 for order,source in enumerate(model['primitives']):
  name=source['part']
  if not name.startswith(('Spot /','Spot center /')):
   parts.append((source,{'source_order':order}));continue
  key='neck' if 'neck' in name or ('shoulder' in name and name.endswith('1')) else 'shoulder' if 'shoulder' in name else 'hindquarter'
  if name.startswith('Spot center /'):key='center_'+key
  if key not in groups:
   label={'neck':'Spot / fitted brown neck marking group','shoulder':'Spot / fitted brown shoulder marking group 0','hindquarter':'Spot / fitted gold hindquarter marking group','center_neck':'Spot center / fitted dark neck marking group','center_shoulder':'Spot center / fitted dark shoulder marking group 0'}[key]
   joined={'part':label,'positions':[],'normals':[],'indices':[],'base_color':source['base_color']}
   metadata={'source_parts':[]};groups[key]=(joined,metadata);parts.append((joined,metadata))
  joined,metadata=groups[key];start=len(joined['positions'])//3;first=len(joined['indices'])
  assert joined['base_color']==source['base_color']
  metadata['source_parts'].append({'part':name,'order':order,'vertex_start':start,'vertex_count':len(source['positions'])//3,'index_start':first,'index_count':len(source['indices'])})
  joined['positions']+=source['positions'];joined['normals']+=source['normals'];joined['indices'] += [i+start for i in source['indices']]
 return parts
PACKED_REST_DIGEST='6c3c3a187cdce79b3adbe6a17a77a5214cfc55b7c5b0dfed356b2dc46d3bcb4c'

def export_girafarig(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='girafarig' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Girafarig anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Girafarig planted two-headed giraffe skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'girafarig/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'girafarig','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'girafarig','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'girafarig','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 packed_parts=pack_parts(model)
 if geometry_digest({'primitives':[p for p,m in packed_parts]})!=PACKED_REST_DIGEST:raise ValueError('unreviewed packed Girafarig anatomy')
 for part,metadata in packed_parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'girafarig/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'girafarig','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'girafarig.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted cloven hooves; shared body color seam; fitted neck spots and mane; independent rear-facing second head and coherent gaze and grin; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'girafarig','source_sha256':REST_DIGEST,
    'source_part_ranges':[metadata for part,metadata in packed_parts],'packed_sha256':PACKED_REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/hoofed_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed cloven hooves and clefts; yaw-only torso; coherent two-color trunk seam; shared neck/spot/mane field; fitted main face and horns; independent ears and leg middles; articulated tail and rigid second head with rear-facing gaze and grin; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError(f'Girafarig exceeds existing accessor limit: {len(g.doc["accessors"])}')
 return g.bytes()


def decode_girafarig(blob):
 if len(blob)>MAX_BYTES or blob!=export_girafarig():raise ValueError('noncanonical Girafarig GLB')
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
 return {'name':'girafarig','version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(parts,key=lambda pair:pair[0])]}


def read_girafarig(path):
 with Path(path).open('rb') as f:return decode_girafarig(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_girafarig();(a.out/'girafarig.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
