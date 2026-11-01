#!/usr/bin/env python3
"""Canonical flame fox with planted paws, independent cream collar and a broad flexible plume."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from flareon_sculpt import sculpture,HEIGHT,CORE_DROP,TAIL_POINTS,HEAD_LIFT
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
REST_DIGEST='2bdb14269978d1eaed0163530f5ecd9a966b45b5501db0927542151ec15268b5'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
def core(p):return (p[0],p[1]-CORE_DROP,p[2])
PATH=tuple(core(p) for p in TAIL_POINTS)
BASE=[('root',None,(0.,0.,0.)),('body',0,core((0.,.28,-.06))),
 ('neck',1,core((0.,.39,.12))),('head',2,core((0.,.53+HEAD_LIFT,.20))),
 ('ear_left',3,core((-.104,.673+HEAD_LIFT,.157))),('ear_right',3,core((.104,.673+HEAD_LIFT,.157))),
 ('tail_base',1,PATH[0]),('tail_middle',6,core((.08,.46,-.43))),('tail_tip',7,core((.22,.69,-.53)))]
for side in (-1,1):
 for row in (0,1):BASE.append(('leg_'+str(side)+'_'+str(row),0,(side*(.092 if row==0 else .122),.18-CORE_DROP*smoothstep(.057,.290,.18),.115 if row==0 else -.175)))
BASE.append(('neck_ruff',2,core((0.,.42,.10))))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def path_fraction(point):
 lengths=[math.dist(a,b) for a,b in zip(PATH,PATH[1:])];total=sum(lengths);offset=0.;best=(math.inf,0.)
 for a,b,length in zip(PATH,PATH[1:],lengths):
  delta=[b[k]-a[k] for k in range(3)];t=max(0.,min(1.,sum((point[k]-a[k])*delta[k] for k in range(3))/(length*length)))
  best=min(best,(math.dist(point,[a[k]+t*delta[k] for k in range(3)]),(offset+t*length)/total));offset+=length
 return best[1]
def skin_weights(part,point):
 x,y,z=[v/SCALE+(LOW if i==1 else 0.) for i,v in enumerate(point)]
 if part.startswith(('Paw /','Toe crease /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Body /','Haunch /')):return [1,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Ruff /','Ruff tuft /')):return [13,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Neck /'):
  a=smoothstep(.30,.39,y);b=smoothstep(.41,.47,y);return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Ear /','Ear inset /')):return [4 if part.endswith('-1') else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  side,row=map(int,part.split()[-2:]);owner=9+(0 if side<0 else 2)+row
  a=smoothstep(.105,.235,y);g=.70*smoothstep(.055,.110,y)*(1.-smoothstep(.190,.255,y))
  return [0,1,owner,0],[(1.-a)*(1.-g),a*(1.-g),g,0.]
 if part.startswith(('Tail /','Tail tip /')):
  t=path_fraction((x,y,z));a=smoothstep(0.,.20,t);b=smoothstep(.15,.50,t);c=smoothstep(.45,.73,t)
  return [1,6,7,8],[1.-a,a*(1.-b),a*b*(1.-c),a*b*c]
 return [3,0,0,0],[1.,0.,0.,0.]

def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Flareon clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);body=.002*a;neck=.006*b;head=.012*b;turn=.018*a;ears=.035*b;tail=.035*a;leg=.006*a;ruff=.015*b
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=1. if clip=='attack' else -.65
  body=.007*a*gain;neck=.018*a*gain;head=.040*b*gain;turn=.030*b*gain;ears=.075*b*gain;tail=.090*b*gain;leg=.085*a*gain;ruff=.040*b*gain
 return [(0.,0.,0.),(body,0.,0.),(neck,0.,0.),(head,turn,0.),
  (ears*.35,ears*.20,-ears),(ears*.35,-ears*.20,ears),
  (tail*.20,tail*.35,0.),(tail*.35,tail*.60,0.),(tail*.45,tail,0.),
  (leg,0.,0.),(-leg*.75,0.,0.),(-leg*.85,0.,0.),(leg*.65,0.,0.),(ruff,0.,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_flareon(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='flareon' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Flareon anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Flareon flame fox, full cream collar and forehead tuft, four planted paws and broad plume skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'flareon/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'flareon','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'flareon','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'flareon','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'flareon/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'flareon','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'flareon.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent blue fox gaze; alert ears; planted paws and flexible limbs; independent full collar and broad three-carrier cream plume; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'flareon','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/flareon_sculpt.py','geometry_preservation':'original compact orange flame fox, dark blue gaze, charcoal ear insets, swept cream forehead tuft, voluminous cream collar and broad curled cream plume',
    'animation_contract':'identity authoring root; four fixed ground paws; blended short limbs; coherent head, muzzle and gaze; independent ears; independent cream collar and continuously weighted three-carrier cream plume; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Flareon exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Flareon exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Flareon exceeds existing byte limit')
 return blob


def decode_flareon(blob):
 if len(blob)>MAX_BYTES or blob!=export_flareon():raise ValueError('noncanonical Flareon GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'flareon','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_flareon(path):
 with Path(path).open('rb') as f:return decode_flareon(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_flareon();(a.out/'flareon.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
