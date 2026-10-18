#!/usr/bin/env python3
"""Canonical stag beetle with planted clawed feet and articulated spiked pincers."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from pinsir_sculpt import sculpture,HEIGHT
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
REST_DIGEST='347418889561cb58a6f9b3f05d441ca8f6bbe2a7b8fb6c4df81ec1a24ea01d84'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('abdomen',0,(0.,.43,-.02)),('carapace',1,(0.,.66,-.02)),
 ('pincer_left',2,(-.155,1.015,0.)),('pincer_right',2,(.155,1.015,0.)),
 ('arm_left',2,(-.25,.765,0.)),('forearm_left',5,(-.43,.675,.020)),('hand_left',6,(-.53,.98,.045)),
 ('arm_right',2,(.25,.765,0.)),('forearm_right',8,(.43,.675,.020)),('hand_right',9,(.53,.98,.045)),
 ('leg_left',1,(-.225,.240,.015)),('leg_right',1,(.225,.240,.015)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 def rigid(j):return [j,0,0,0],[1.,0.,0.,0.]
 side=part.split()[-2] if part.startswith(('Hand claw /','Foot claw /','Pincer spike /')) else part.split()[-1];left=side=='-1'
 if part.startswith(('Foot /','Foot claw /')):return rigid(0)
 if part.startswith('Leg /'):
  a=smoothstep(.12,.26,y);b=smoothstep(.30,.43,y)
  return [0,11 if left else 12,1,0],[1.-a,a*(1.-b),a*b,0.]
 if part.startswith('Upper arm /'):return rigid(5 if left else 8)
 if part.startswith('Forearm /'):return rigid(6 if left else 9)
 if part.startswith(('Hand /','Hand claw /')):return rigid(7 if left else 10)
 if part.startswith(('Pincer /','Pincer spike /')):return rigid(3 if left else 4)
 if part.startswith(('Eye','Pupil /','Mouth /','Tooth /')):return rigid(2)
 a=smoothstep(.50,.62,y)
 return [1,2,0,0],[1.-a,a,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Pinsir clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.;pinch=.008*b
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)));strength=2.4 if clip=='attack' else -2.;pinch=(.085 if clip=='attack' else -.030)*b
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.010*a,0.),(.008*a,.014*b,0.),
  (.010*a,.015*b,-pinch),(.010*a,-.015*b,pinch),
  (.015*a,.015*b,-.035*b),(-.025*a,.012*b,.025*b),(.012*b,.010*a,-.020*a),
  (.015*a,-.015*b,.035*b),(-.025*a,-.012*b,-.025*b),(.012*b,-.010*a,.020*a),
  (0.,.015*a,0.),(0.,-.015*a,0.)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])

def export_pinsir(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='pinsir' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Pinsir anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Pinsir stag beetle, barred mouth and spiked-pincer skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'pinsir/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'pinsir','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'pinsir','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'pinsir','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(packed(values,component),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'pinsir/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'pinsir','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'pinsir.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted beetle tarsi; fitted stern gaze and barred aperture; coherent spiked pincers; articulated elbows wrists and three-claw hands; source cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'pinsir','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/pinsir_sculpt.py','geometry_preservation':'original oval brown carapace, fitted narrow gaze and barred mouth, transverse armor seams, curved spiked pincers, thin raised arms and three-claw hands and feet',
    'animation_contract':'identity floor root; fixed tarsi and foot claws with blended shanks; blended abdomen and upper carapace; fitted gaze and barred mouth; coherent articulated spiked pincers; shoulders elbows wrists and clawed hands; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Pinsir exceeds existing accessor limit')
 if sum(len(p['positions'])//3 for p in model['primitives'])>20000 or sum(len(p['indices']) for p in model['primitives'])>120000:raise ValueError('Pinsir exceeds existing geometry limits')
 blob=g.bytes()
 if len(blob)>MAX_BYTES:raise ValueError('Pinsir exceeds existing byte limit')
 return blob


def decode_pinsir(blob):
 if len(blob)>MAX_BYTES or blob!=export_pinsir():raise ValueError('noncanonical Pinsir GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'pinsir','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_pinsir(path):
 with Path(path).open('rb') as f:return decode_pinsir(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_pinsir();(a.out/'pinsir.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
