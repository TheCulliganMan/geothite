#!/usr/bin/env python3
"""Canonical single-horn beetle with planted claws and articulated armored limbs."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from heracross_sculpt import sculpture,HEIGHT
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
REST_DIGEST='a33ddd1481ba4cb3435132daf36f3c5770e6b8155e24afc627142a32a82d923d'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('body',0,(0.,.47,-.015)),('thorax',1,(0.,.70,0.)),('head',2,(0.,.85,.020)),
 ('arm_left',2,(-.285,.718,.013)),('forearm_left',4,(-.386,.640,.060)),
 ('arm_right',2,(.285,.718,.013)),('forearm_right',6,(.386,.640,.060)),
 ('antenna_left',3,(-.125,1.065,.078)),('antenna_right',3,(.125,1.065,.078)),
 ('leg_left',1,(-.225,.223,.013)),('leg_right',1,(.225,.223,.013)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 side=part.split()[-2] if part.startswith(('Hand claw /','Gauntlet tooth /')) else part.split()[-1];left=side=='-1'
 if part.startswith(('Foot /','Foot claw /')):return rigid(0)
 if part.startswith(('Haunch /','Thigh thorn /','Shin /')):
  a=smoothstep(.09,.22,y);b=smoothstep(.25,.43,y)
  return [0,10 if left else 11,1,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Upper arm /'):return rigid(4 if left else 6)
 if part.startswith(('Forearm /','Gauntlet tooth /','Hand /','Hand claw /')):return rigid(5 if left else 7)
 if part.startswith(('Antenna /','Antenna bead /')):return rigid(8 if left else 9)
 if part.startswith(('Head /','Horn /','Eye','Pupil /','Glint /','Mouth /')):return rigid(3)
 if part.startswith(('Thorax /','Shoulder /')):return rigid(2)
 return rigid(1)
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Heracross clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.;jab=.012*b
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.;jab=(-.065 if clip=='attack' else .035)*a
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.012*a,0.),(.008*b,.014*a,0.),(jab,.018*a,.008*b),
  (.018*a,.012*b,-.045*b),(-.035*a,.015*b,-.020*b),
  (.018*a,-.012*b,.045*b),(-.035*a,-.015*b,.020*b),
  (.014*b,.018*a,.025*b),(.014*b,-.018*a,-.025*b),
  (0.,.015*a,0.),(0.,-.015*a,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_heracross(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='heracross' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Heracross anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Heracross single-horn beetle, armor and hooked-claw skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'heracross/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'heracross','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'heracross','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'heracross','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'heracross/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'heracross','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'heracross.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted beetle tarsi; fitted golden gaze and coherent forked horn; independent bead antennae; articulated armored forearms with paired ivory claws; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'heracross','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/heracross_sculpt.py','geometry_preservation':'original blue armored beetle, fitted belly plate and golden gaze, continuous flattened forked horn, bead antennae, serrated gauntlets and hooked ivory claws',
    'animation_contract':'identity floor root; fixed tarsi and foot claws with blended coxae and shins; articulated body thorax and head; coherent gaze and forked horn; independent bead antennae; shoulders and armored forearms; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Heracross exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Heracross exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Heracross exceeds existing byte limit')
 return blob


def decode_heracross(blob):
 if len(blob)>MAX_BYTES or blob!=export_heracross():raise ValueError('noncanonical Heracross GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'heracross','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_heracross(path):
 with Path(path).open('rb') as f:return decode_heracross(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_heracross();(a.out/'heracross.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
