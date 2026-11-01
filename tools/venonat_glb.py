#!/usr/bin/env python3
"""Canonical fuzzy insect orb with compound lenses, ivory feelers, articulated hands and planted feet."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from venonat_sculpt import sculpture,HEIGHT,CENTER
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
REST_DIGEST='b1425a1645ef672eb73a014ed63285503c3f184ddecb3f782c466bea7163d4f3'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=[('root',None,(0.,0.,0.)),('body',0,CENTER),
 ('antenna_left',1,(-.095,.805,.105)),('antenna_right',1,(.095,.805,.105)),
 ('arm_left',1,(-.265,.423,.115)),('arm_right',1,(.265,.423,.115)),
 ('hand_left',4,(-.291,.370,.236)),('hand_right',5,(.291,.370,.236)),
 ('leg_left',0,(-.151,.135,.016)),('leg_right',0,(.151,.135,.016))]
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=[v/SCALE+(LOW if i==1 else 0.) for i,v in enumerate(point)]
 if part.startswith(('Foot /','Toe crease /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Body /'):
  a=smoothstep(.13,.31,y);return [0,1,0,0],[1.-a,a,0.,0.]
 if part.startswith(('Antenna /','Antenna tip /')):return [2 if part.endswith('-1') else 3,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Arm /'):
  owner=4 if part.endswith('-1') else 5;a=smoothstep(.268,.302,abs(x));return [1,owner,0,0],[1.-a,a,0.,0.]
 if part.startswith('Hand /'):return [6 if part.endswith('-1') else 7,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Finger /'):return [6 if part.split()[-2]=='-1' else 7,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  owner=8 if part.endswith('-1') else 9;a=smoothstep(.09,.18,y);bend=.65*smoothstep(.07,.12,y)*(1.-smoothstep(.15,.20,y))
  return [0,1,owner,0],[(1.-a)*(1.-bend),a*(1.-bend),bend,0.]
 return [1,0,0,0],[1.,0.,0.,0.]

def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Venonat clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.4)+math.sin(.4);body=.004*a;turn=.012*b;feel=.035*b;reach=.025*a;hand=.040*b;brace=.008*a
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)));b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));gain=1. if clip=='attack' else -.65
  body=.015*a*gain;turn=.018*b*gain;feel=.090*b*gain;reach=.080*a*gain;hand=.100*b*gain;brace=.035*a*gain
 return [(0.,0.,0.),(body,turn,0.),(feel*.2,-feel*.35,-feel),(feel*.2,feel*.35,feel),
  (-reach,reach*.3,-reach*.25),(-reach,-reach*.3,reach*.25),
  (hand*.25,-hand,hand*.3),(hand*.25,hand,-hand*.3),(brace,0.,0.),(-brace*.8,0.,0.)]

def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_venonat(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='venonat' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Venonat anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Venonat fuzzy insect orb, compound lenses, curved feelers, claw hands and planted ivory pads skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'venonat/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'venonat','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'venonat','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'venonat','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'venonat/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'venonat','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'venonat.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'coherent compound gaze and paired fangs; curved feelers; independent arms and three-clawed hands; braced ankles and planted broad feet; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'venonat','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/venonat_sculpt.py','geometry_preservation':'original fuzzy purple insect orb, pink faceted compound lenses and snout, paired ivory fangs and feelers, small claw hands and broad planted ivory feet',
    'animation_contract':'identity floor root; fixed broad feet and toe creases; blended ankles and lower fur body; coherent compound lenses snout fangs and crown; independent feelers arms and claw hands; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Venonat exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Venonat exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Venonat exceeds existing byte limit')
 return blob


def decode_venonat(blob):
 if len(blob)>MAX_BYTES or blob!=export_venonat():raise ValueError('noncanonical Venonat GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'venonat','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_venonat(path):
 with Path(path).open('rb') as f:return decode_venonat(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_venonat();(a.out/'venonat.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
