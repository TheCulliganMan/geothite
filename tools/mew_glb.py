#!/usr/bin/env python3
"""Canonical Mew with soft feline anatomy and an arc-weighted looping psychic tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from mew_sculpt import sculpture,HEIGHT,TAIL_POINTS,TAIL_STEPS
from bovine_sculpt import smooth
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
REST_DIGEST='36d3fc53223d21ac492d3d93879589a130773ba3d884c34a65f6173b3980d37e'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.36,-.020)),('neck',1,(0.,.59,.035)),('head',2,(0.,.76,.090)),
 ('arm_left',1,(-.090,.526,.005)),('arm_right',1,(.090,.526,.005)),
 ('leg_left',1,(-.113,.279,.005)),('leg_right',1,(.113,.279,.005)),
 ('tail_root',1,TAIL_POINTS[0]),('tail_curve',8,TAIL_POINTS[3]),('tail_tip',9,TAIL_POINTS[6]),
 ('ear_left',3,(-.14,1.025,.035)),('ear_right',3,(.14,1.025,.035)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
TAIL_PATH,_=smooth(TAIL_POINTS,[1.]*len(TAIL_POINTS),TAIL_STEPS)
TAIL_LENGTHS=[0.]
for a,b in zip(TAIL_PATH,TAIL_PATH[1:]):TAIL_LENGTHS.append(TAIL_LENGTHS[-1]+math.dist(a,b))
def tail_progress(point):
 best=float('inf');progress=0.
 for k,(a,b) in enumerate(zip(TAIL_PATH,TAIL_PATH[1:])):
  d=[b[i]-a[i] for i in range(3)];length2=sum(v*v for v in d);t=max(0.,min(1.,sum((point[i]-a[i])*d[i] for i in range(3))/length2))
  distance=sum((point[i]-a[i]-t*d[i])**2 for i in range(3))
  if distance<best:best=distance;progress=(TAIL_LENGTHS[k]+t*math.sqrt(length2))/TAIL_LENGTHS[-1]
 return progress
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Haunch /','Leg /','Foot /','Toe crease /')):
  side=part.split()[-2] if part.startswith('Toe crease /') else part.split()[-1]
  return rigid(6 if side=='-1' else 7)
 if part.startswith(('Arm /','Hand /','Finger /')):
  side=part.split()[-2] if part.startswith('Finger /') else part.split()[-1]
  return rigid(4 if side=='-1' else 5)
 if part.startswith(('Ear /','Ear inset /')):return rigid(11 if part.endswith('-1') else 12)
 if part.startswith('Tail tip /'):return rigid(10)
 if part.startswith('Tail /'):
  t=tail_progress((x,y,z));a=smoothstep(.24,.46,t);b=smoothstep(.66,.86,t)
  return [8,9,10,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Neck /'):return rigid(2)
 if part.startswith(('Head /','Eye','Iris /','Pupil /','Glint /','Mouth /')):return rigid(3)
 return rigid(1)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Mew clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength;kick=-.030*abs(b)
 return [(0.,0.,0.),(0.,.015*a,0.),(.010*a,.015*b,0.),(.018*b,.025*a,.008*b),
  (.018*a,.020*b,.050*b),(.018*a,-.020*b,-.050*b),
  (kick,.020*a,0.),(kick,-.020*a,0.),(0.,.022*b,0.),(0.,-.035*b,0.),(0.,.045*b,0.),
  (.012*b,.015*a,.018*b),(.012*b,-.015*a,-.018*b)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_mew(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='mew' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Mew anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Mew soft psychic cat, long paws and looping tail skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'mew/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'mew','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'mew','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'mew','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'mew/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'mew','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'mew.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'soft fitted feline gaze; coherent ear insets and tiny three-finger hands; lifted long paws; three smoothly weighted looping tail carriers and oval terminal; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'mew','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/mew_sculpt.py','geometry_preservation':'original pear body, soft feline head, fitted gaze, long paws, tiny hands and continuous looping whip with oval tip',
    'animation_contract':'identity authoring root; standard render-only hovering clearance; yawing torso; articulated short neck and fitted gaze; independent ears and tiny hands; lifted long paws; arc-length tail blend and coherent oval terminal; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Mew exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Mew exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Mew exceeds existing byte limit')
 return blob


def decode_mew(blob):
 if len(blob)>MAX_BYTES or blob!=export_mew():raise ValueError('noncanonical Mew GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'mew','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_mew(path):
 with Path(path).open('rb') as f:return decode_mew(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_mew();(a.out/'mew.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
