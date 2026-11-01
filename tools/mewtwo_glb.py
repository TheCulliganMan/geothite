#!/usr/bin/env python3
"""Canonical genetic psychic cat with planted pads and an arc-weighted heavy tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from mewtwo_sculpt import sculpture,HEIGHT,TAIL_POINTS,TAIL_STEPS
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
REST_DIGEST='1413629f1544e2cffba26a0a692d20dacc0c665123899355d5416f3e3d176ee3'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.86,-.015)),('neck',1,(0.,1.19,.035)),('head',2,(0.,1.36,.105)),
 ('upper_left',1,(-.177,1.115,.035)),('forearm_left',4,(-.293,.925,.100)),('hand_left',5,(-.350,.795,.180)),
 ('upper_right',1,(.177,1.115,.035)),('forearm_right',7,(.293,.925,.100)),('hand_right',8,(.350,.795,.180)),
 ('hip_left',1,(-.19,.62,.025)),('knee_left',10,(-.20,.415,.040)),('hip_right',1,(.19,.62,.025)),('knee_right',12,(.20,.415,.040)),
 ('tail_root',1,TAIL_POINTS[0]),('tail_curve',14,TAIL_POINTS[3]),('tail_tip',15,TAIL_POINTS[6]))
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
 side=part.split()[-2] if part.startswith(('Finger /','Fingertip /','Toe /')) else part.split()[-1]
 left=side=='-1'
 if part.startswith(('Foot /','Toe /')):return rigid(0)
 if part.startswith(('Haunch /','Shin /')):
  high=smoothstep(.10,.25,y);hock=smoothstep(.24,.43,y);upper=smoothstep(.57,.78,y)
  return [0,11 if left else 13,10 if left else 12,1],[1.-high,high*(1.-hock),high*hock*(1.-upper),high*hock*upper]
 if part.startswith('Upper arm /'):return rigid(4 if left else 7)
 if part.startswith('Forearm /'):return rigid(5 if left else 8)
 if part.startswith(('Hand /','Finger /','Fingertip /')):return rigid(6 if left else 9)
 if part.startswith('Tail tip /'):return rigid(16)
 if part.startswith('Tail /'):
  t=tail_progress((x,y,z));a=smoothstep(.24,.46,t);b=smoothstep(.66,.86,t)
  return [14,15,16,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Neck /','Back tube /')):
  a=smoothstep(1.03,1.22,y);b=smoothstep(1.27,1.42,y)
  return [1,2,3,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith(('Head /','Horn /','Eye','Iris /','Pupil /','Glint /','Mouth /','Nostril /')):return rigid(3)
 return rigid(1)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Mewtwo clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.012*a,0.),(.008*a,.012*b,0.),(.014*b,.022*a,.006*b),
  (.015*a,.012*b,-.055*b),(-.025*a,.010*b,0.),(.010*b,0.,-.025*b),
  (.015*a,-.012*b,.055*b),(-.025*a,-.010*b,0.),(.010*b,0.,.025*b),
  (0.,.012*a,0.),(0.,-.014*a,0.),(0.,-.012*a,0.),(0.,.014*a,0.),
  (0.,.018*b,0.),(0.,-.028*b,0.),(0.,.035*b,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_mewtwo(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='mewtwo' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Mewtwo anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Mewtwo genetic psychic cat, digitigrade legs and heavy tail skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'mewtwo/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'mewtwo','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'mewtwo','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'mewtwo','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'mewtwo/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'mewtwo','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'mewtwo.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'stern fitted gaze; articulated shoulders elbows and three-finger hands; planted pads and blended digitigrade legs; three arc-weighted heavy tail carriers; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'mewtwo','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/mewtwo_sculpt.py','geometry_preservation':'pale tapered genetic body, purple pelvis and heavy tail, stern fitted gaze, blunt horns, spinal arch, three-finger hands and digitigrade legs',
    'animation_contract':'identity authoring root and planted toe pads; blended hocks and hips; articulated torso neck and fitted gaze; coherent horns; shoulders elbows and wrists; arc-length heavy tail blend; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Mewtwo exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Mewtwo exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Mewtwo exceeds existing byte limit')
 return blob


def decode_mewtwo(blob):
 if len(blob)>MAX_BYTES or blob!=export_mewtwo():raise ValueError('noncanonical Mewtwo GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'mewtwo','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_mewtwo(path):
 with Path(path).open('rb') as f:return decode_mewtwo(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_mewtwo();(a.out/'mewtwo.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
