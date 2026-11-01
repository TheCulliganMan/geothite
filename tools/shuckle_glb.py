#!/usr/bin/env python3
"""Canonical Shuckle skin with rigid perforated shell, grounded feet and articulated neck."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from shuckle_sculpt import sculpture,HEIGHT,HOLES,FACES,project
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
REST_DIGEST='2a7a21948c474962c403086b8e508fc560f2c001c1448311316a0dbb46bc2206'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
LIMBS=[]
for (face,sx,sy),label in HOLES.items():
 if 'limb' not in label:continue
 root=project(face,sx*.5,sy*.5);n=FACES[face][0]
 inside=(root[0]*.65,root[1],root[2]*.65);elbow=(root[0]+n[0]*.11,.101,root[2]+n[2]*.11);tip=(root[0]+n[0]*.225,.046,root[2]+n[2]*.225)
 LIMBS.append((label,(inside,root,elbow,tip)))
NECK_ROOT=project(4,.5,-.5)
NECK=((NECK_ROOT[0]*.63,.365,NECK_ROOT[2]*.63),NECK_ROOT,(.13,.62,.139),(.092,.785,.173),(.058,.946,.217))
BASE=(('root',None,(0.,0.,0.)),)+tuple(('limb_'+name.replace(' ','_'),0,path[2]) for name,path in LIMBS)+(
 ('neck_base',0,NECK_ROOT),('neck_bend',5,NECK[2]),('head',6,NECK[-1]))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point,path):
 best=(math.inf,0.);lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  distance=math.dist(point,[x+t*d for x,d in zip(a,delta)])
  best=min(best,(distance,(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(part,point):
 raw=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 if part.startswith('Limb /'):
  label=part.removeprefix('Limb / smooth flexible yellow ');i=next(i for i,(name,path) in enumerate(LIMBS) if name==label)
  t=path_fraction(raw,LIMBS[i][1]);f=smoothstep(.30,.50,t)*(1.-smoothstep(.68,.90,t))
  return [0,1+i,0,0],[1.-f,f,0.,0.]
 if part.startswith('Neck /'):
  t=path_fraction(raw,NECK);attach=smoothstep(.24,.40,t);bend=smoothstep(.48,.72,t);head=smoothstep(.82,.98,t)
  return [0,5,6,7],[1.-attach,attach*(1.-bend),attach*bend*(1.-head),attach*bend*head]
 if part.startswith(('Head /','Eye /','Mouth /')):return [7,0,0,0],[1.,0.,0.,0.]
 return [0,0,0,0],[1.,0.,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Shuckle clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)))
  strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.)]+[(0.,(.045 if i%2==0 else -.045)*a,0.) for i in range(4)]+[
  (.020*a,.015*a,0.),(.030*b,-.018*b,.012*b),(-.020*b,.025*b,-.010*b)]
def export_shuckle(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='shuckle' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Shuckle anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Shuckle perforated carapace and soft-body skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'shuckle/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'shuckle','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'shuckle','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'shuckle','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(tuple(values),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'shuckle/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':g.accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'shuckle','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in quaternion_xyz(*pose_angles(clip,i/(count-1))[joint])]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':g.accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'shuckle.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'rigid perforated shell and collars; planted feet; soft limb response between fixed ends; articulated neck and fitted face; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'shuckle','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/shuckle_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; rigid shell collars spots and core; fixed feet; soft limb middles; coherent neck/head/face chain; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Shuckle exceeds existing accessor limit')
 return g.bytes()


def decode_shuckle(blob):
 if len(blob)>MAX_BYTES or blob!=export_shuckle():raise ValueError('noncanonical Shuckle GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'shuckle','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_shuckle(path):
 with Path(path).open('rb') as f:return decode_shuckle(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_shuckle();(a.out/'shuckle.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
