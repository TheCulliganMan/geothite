#!/usr/bin/env python3
"""Canonical Stantler skin with planted hooves, coherent antlers and a curled tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from hoofed_sculpt import sculpture,HEIGHT
from skin_glb import COORDINATES,Glb,envelope,packed,quantized_weights,quaternion_xyz,smoothstep
MAX_BYTES=1024*1024
SOURCE=sculpture('stantler');RAW=sculpture('stantler',normalize=False)
LOW=min(v for p in RAW['primitives'] for v in p['positions'][1::3])
SCALE=HEIGHT/(max(v for p in RAW['primitives'] for v in p['positions'][1::3])-LOW)
PARTS=tuple(p['part'] for p in SOURCE['primitives'])
def geometry_digest(model):
 h=hashlib.sha256()
 for p in model['primitives']:
  for key,c in (('positions',5126),('normals',5126),('indices',5125),('base_color',5126)):h.update(packed(p[key],c))
 return h.hexdigest()
REST_DIGEST='6057a007fe3b481aa54c4a074e5b324d38de52370616269aec2962f5b9ae1da3'
def pack_parts(model):
 """Batch flank stripes while retaining each original part and stored range."""
 parts=[];ribs=None
 for order,p in enumerate(model['primitives']):
  if not p['part'].startswith('Rib /'):
   parts.append((p,{'source_order':order}));continue
  if ribs is None:
   ribs=(dict(part='Rib / six fitted cream flank stripes',positions=[],normals=[],indices=[],base_color=p['base_color']),{'source_parts':[]});parts.append(ribs)
  q,meta=ribs;start=len(q['positions'])//3
  meta['source_parts'].append(dict(part=p['part'],order=order,vertex_start=start,vertex_count=len(p['positions'])//3,index_start=len(q['indices']),index_count=len(p['indices'])))
  q['positions']+=p['positions'];q['normals']+=p['normals'];q['indices'] += [i+start for i in p['indices']]
 return parts

def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.55,-.07)),
 ('neck',1,(0.,.61,.20)),('head',2,(0.,.88,.265)),
 ('ear_left',3,(-.10,1.015,.245)),('ear_right',3,(.10,1.015,.245)),
 ('leg_left_front',0,(-.125,.24,.194)),('leg_left_rear',0,(-.135,.32,-.295)),
 ('leg_right_front',0,(.125,.24,.194)),('leg_right_rear',0,(.135,.32,-.295)),
 ('tail_base',1,(0.,.57,-.277)),('tail_curl',10,(0.,.64,-.402)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 if part.startswith(('Hoof /','Hoof groove /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=6+(0 if side<0 else 2)+row
  a=smoothstep(.08,.53,y);g=.65*smoothstep(.08,.18,y)*(1.-smoothstep(.40,.55,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith('Ear /'):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Head /','Eye /','Brow /','Brow spot /','Muzzle /','Muzzle pad /','Nostril /','Antler')):return [3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail / large'):return [11,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  t=(-z-.277)/.125;a=smoothstep(0.,.55,t);b=smoothstep(.4,1.,t)
  return [1,10,11,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Neck /','Chest /')):
  a=smoothstep(.51,.66,y);b=smoothstep(.76,.90,y)
  return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 return [1,0,0,0],[1.,0.,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Stantler clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.015*a,0.),(0.,.018*a,0.),(0.,.020*a,0.),
  (.020*b,0.,-.025*b),(.020*b,0.,.025*b),
  (0.,.018*a,0.),(0.,-.018*a,0.),(0.,-.018*a,0.),(0.,.018*a,0.),
  (0.,.020*a,0.),(.018*b,.030*b,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def export_stantler(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='stantler' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Stantler anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Stantler planted antlered deer skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'stantler/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'stantler','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'stantler','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'stantler','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 parts=pack_parts(model)
 for part,meta in parts:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'stantler/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'stantler','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'stantler.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted cloven hooves; coherent antler loops, branches and orbs; fitted half-lidded gaze; independent ears, leg middles and curled tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'stantler','source_sha256':REST_DIGEST,
    'source_part_ranges':[meta for part,meta in parts],'packed_sha256':geometry_digest({'primitives':[part for part,meta in parts]}),
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/hoofed_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed cloven hooves and clefts; yaw-only torso; shared neck/bib field; coherent head, antler loops, tines and orbs; independent ears, leg middles and two-stage curled tail; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Stantler exceeds existing accessor limit')
 return g.bytes()


def decode_stantler(blob):
 if len(blob)>MAX_BYTES or blob!=export_stantler():raise ValueError('noncanonical Stantler GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 stored=[]
 for p,meta in zip(doc['meshes'][0]['primitives'],doc['extras']['source_part_ranges']):
  pos=values(p['attributes']['POSITION']);normals=values(p['attributes']['NORMAL']);indices=values(p['indices'])
  sources=meta.get('source_parts',[dict(part=p['extras']['part'],order=meta.get('source_order'),vertex_start=0,vertex_count=len(pos)//3,index_start=0,index_count=len(indices))])
  for source in sources:
   a=source['vertex_start'];b=a+source['vertex_count'];i=source['index_start'];j=i+source['index_count']
   stored.append((source['order'],dict(part=source['part'],positions=pos[3*a:3*b],normals=normals[3*a:3*b],indices=[k-a for k in indices[i:j]],base_color=doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor'])))
 return {'name':'stantler','version':1,'coordinate_system':COORDINATES,'primitives':[p for order,p in sorted(stored)]}


def read_stantler(path):
 with Path(path).open('rb') as f:return decode_stantler(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_stantler();(a.out/'stantler.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
