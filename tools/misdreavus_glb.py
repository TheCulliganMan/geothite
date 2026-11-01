#!/usr/bin/env python3
"""Canonical Misdreavus skin with coherent spectral gaze, necklace and six swept locks."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from misdreavus_sculpt import sculpture,HEIGHT
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
REST_DIGEST='65520ddfee63c5e0fa3d39643396f88dfe2b9e2fbefd7bc24355cc2491b59935'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
LOCKS=(('crown',(0.,.887,-.055),(-.217,1.105,-.045),(-.248,1.247,-.040)),
 ('left upper',(-.178,.813,-.063),(-.470,.819,-.045),(-.658,.722,.012)),
 ('left lower',(-.197,.700,-.105),(-.398,.511,-.148),(-.552,.520,-.116)),
 ('right upper',(.170,.860,-.080),(.455,.848,-.071),(.420,.717,-.009)),
 ('right lower',(.212,.678,-.109),(.337,.459,-.070),(.488,.436,-.033)),
 ('rear',(0.,.807,-.115),(.231,.696,-.350),(.359,.553,-.306)))
BASE=(('root',None,(0.,0.,0.)),('body',0,(0.,.49,-.010)),('head',1,(0.,.615,.016)))+tuple(
 ('lock_'+name.replace(' ','_'),2,start) for name,start,bend,end in LOCKS)+tuple(
 ('curl_'+name.replace(' ','_'),3+i,bend) for i,(name,start,bend,end) in enumerate(LOCKS))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def lock_fraction(point,path):
 lengths=[math.dist(a,b) for a,b in zip(path,path[1:])];total=sum(lengths);offset=0.;best=(math.inf,0.)
 for a,b,length in zip(path,path[1:],lengths):
  delta=[y-x for x,y in zip(a,b)];t=max(0.,min(1.,sum((p-x)*d for p,x,d in zip(point,a,delta))/(length*length)))
  best=min(best,(math.dist(point,[x+t*d for x,d in zip(a,delta)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(part,point):
 raw=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE);x,y,z=raw
 if part.startswith('Body /'):
  a=smoothstep(.31,.43,y);b=smoothstep(.47,.60,y)
  return [0,1,2,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Hair /','Hair tip /')):
  name=part.removeprefix('Hair / swept volumetric ').removeprefix('Hair tip / tapered pink curl ')
  i=next(i for i,lock in enumerate(LOCKS) if lock[0]==name);_,start,bend,end=LOCKS[i]
  t=lock_fraction(raw,(start,bend,end));a=smoothstep(0.,.50,t);b=smoothstep(.20,1.,t)
  return [2,3+i,9+i,0],[1.-a,a*(1.-b),a*b,0.]
 return [2,0,0,0],[1.,0.,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Misdreavus clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)))
  strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.015*a,0.),(.018*a,.025*a,.006*b)]+[
  (.012*a,.015*b,(-1 if i in (0,1,2) else 1)*.018*a) for i in range(6)]+[
  (.020*b,.018*b,(-1 if i in (0,1,2) else 1)*.025*b) for i in range(6)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def export_misdreavus(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='misdreavus' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Misdreavus anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Misdreavus six swept locks and coherent necklace skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'misdreavus/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'misdreavus','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'misdreavus','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'misdreavus','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'misdreavus/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'misdreavus','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'misdreavus.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'preserved five-lobed hem; coherent spectral gaze and necklace; six delayed swept curls; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'misdreavus','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/misdreavus_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed hem tips; yaw-only body; coherent fitted face, necklace and gem glints; six blended locks and curls preserve color seams; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Misdreavus exceeds existing accessor limit')
 return g.bytes()


def decode_misdreavus(blob):
 if len(blob)>MAX_BYTES or blob!=export_misdreavus():raise ValueError('noncanonical Misdreavus GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'misdreavus','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_misdreavus(path):
 with Path(path).open('rb') as f:return decode_misdreavus(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_misdreavus();(a.out/'misdreavus.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
