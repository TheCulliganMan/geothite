#!/usr/bin/env python3
"""Canonical Murkrow skin with planted talons, folded wings, feather hat and broom tail."""
import argparse,hashlib,json,math,struct
from pathlib import Path
from murkrow_sculpt import sculpture,HEIGHT
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
REST_DIGEST='1eb3ca435d7fe293a9371188874d4c5e7007e7f2ad87cddd52b1e3202fbbf132'
def pivot(p):return tuple((v-(LOW if i==1 else 0.))*SCALE for i,v in enumerate(p))
BASE=(('root',None,(0.,0.,0.)),('torso',0,(0.,.365,-.016)),
 ('head',1,(0.,.738,.020)),('lower_bill',2,(0.,.750,.142)),
 ('wing_left',1,(-.150,.53,.035)),('wing_right',1,(.150,.53,.035)),
 ('tail_base',1,(0.,.383,-.125)),('broom_fan',6,(0.,.474,-.315)),
 ('crest_center',2,(0.,.938,-.037)),('crest_left',2,(-.080,.938,-.052)),
 ('crest_right',2,(.090,.940,-.067)))
JOINTS=tuple((n,parent,pivot(p) if parent is not None else p) for n,parent,p in BASE)
CLIPS=(('idle',3.2,81),('attack',1.,65),('hit',1.,65))
def skin_weights(part,point):
 x,y,z=(point[0]/SCALE,point[1]/SCALE+LOW,point[2]/SCALE)
 if part.startswith(('Foot /','Toe /','Talon /')):return [0,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Leg /'):
  f=smoothstep(.06,.23,y);return [0,1,0,0],[1.-f,f,0.,0.]
 if part.startswith(('Wing /','Wing feather /')):
  side=int(part.split()[-2] if part.startswith('Wing feather /') else part.split()[-1])
  return [4 if side<0 else 5,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Crest /'):return [8+int(part.split()[-1]),0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Head /','Hat /','Eye /','Pupil /','Nostril /')):return [2,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Bill /'):return [3 if 'lower' in part or 'seam' in part else 2,0,0,0],[1.,0.,0.,0.]
 if part.startswith(('Neck /','Throat /')):
  f=smoothstep(.57,.73,y);return [1,2,0,0],[1.-f,f,0.,0.]
 if part.startswith(('Tail fan /','Tail fold /')):return [7,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Binding /'):return [6,0,0,0],[1.,0.,0.,0.]
 if part.startswith('Tail /'):
  t=(-z-.125)/.190
  a=smoothstep(0.,.40,t);b=smoothstep(.35,1.,t)
  return [1,6,7,0],[1.-a,a*(1.-b),a*b,0.]
 return [1,0,0,0],[1.,0.,0.,0.]
def pose_angles(clip,t):
 if clip not in ('idle','attack','hit') or not 0.<=t<=1.:raise ValueError('invalid Murkrow clip/time')
 if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
 if clip=='idle':
  a=math.sin(math.tau*t);b=math.sin(math.tau*t-.45)+math.sin(.45);strength=1.
 else:
  a=envelope(t,((0.,0.),(.18,1.),(.42,.60),(.72,.06),(1.,0.)))
  b=envelope(t,((0.,0.),(.31,1.),(.55,.55),(.80,-.08),(1.,0.)))
  strength=2.4 if clip=='attack' else -2.
 a*=strength;b*=strength
 return [(0.,0.,0.),(0.,.015*a,0.),(.020*a,.035*a,.008*b),
  (.035*a,0.,0.),(.018*a,.020*b,-.045*a),(.018*a,-.020*b,.045*a),
  (0.,.025*a,0.),(.025*b,.045*b,0.),(.025*b,0.,.014*b),
  (.035*b,0.,-.020*b),(.020*b,0.,.025*b)]
def rotation(clip,t,joint):return quaternion_xyz(*pose_angles(clip,t)[joint])
def export_murkrow(model=None):
 if model is None:model=SOURCE
 if (model.get('name')!='murkrow' or model.get('version')!=1 or model.get('coordinate_system')!=COORDINATES
     or tuple(p.get('part') for p in model.get('primitives',[]))!=PARTS
     or geometry_digest(model)!=REST_DIGEST):raise ValueError('unreviewed Murkrow anatomy')
 g=Glb();g.doc['asset']['generator']='Geothite Murkrow planted witch-crow and articulated broom skin v1'
 nodes=g.doc['nodes'];inverse=[];joints=JOINTS
 for name,parent,pivot in joints:
  origin=joints[parent][2] if parent is not None else (0.,0.,0.)
  nodes.append({'name':'murkrow/'+name,'translation':[a-b for a,b in zip(pivot,origin)]})
  if parent is not None:nodes[parent].setdefault('children',[]).append(len(nodes)-1)
  x,y,z=pivot;inverse += [1.,0.,0.,0.,0.,1.,0.,0.,0.,0.,1.,0.,-x,-y,-z,1.]
 nodes.append({'name':'murkrow','mesh':0,'skin':0})
 g.doc['scenes']=[{'name':'murkrow','nodes':[0,len(joints)]}]
 g.doc['skins']=[{'name':'murkrow','skeleton':0,'joints':list(range(len(joints))), 'inverseBindMatrices':g.accessor(inverse,'MAT4')}]
 shared={};colors={};primitives=[]
 def accessor(values,kind,component=5126,target=None,normalized=False):
  key=(tuple(values),kind,component,target,normalized)
  if key not in shared:shared[key]=g.accessor(values,kind,component,target,normalized=normalized)
  return shared[key]
 for part in model['primitives']:
  color=tuple(part['base_color'])
  if color not in colors:
   colors[color]=len(colors);g.doc['materials'].append({'name':'murkrow/paper_'+str(colors[color]),
    'pbrMetallicRoughness':{'baseColorFactor':list(color),'metallicFactor':0.,'roughnessFactor':1.},'alphaMode':'OPAQUE'})
  ids=[];weights=[]
  for i in range(0,len(part['positions']),3):
   point=struct.unpack('<3f',packed(part['positions'][i:i+3]));js,ws=skin_weights(part['part'],point);q=quantized_weights(ws)
   ids += [j if w else 0 for j,w in zip(js,q)];weights += q
  primitives.append({'attributes':{'POSITION':g.accessor(part['positions'],'VEC3',target=34962,bounds=True),
    'NORMAL':accessor(part['normals'],'VEC3',target=34962),'JOINTS_0':accessor(ids,'VEC4',5121,34962),
    'WEIGHTS_0':accessor(weights,'VEC4',5121,34962,True)},
    'indices':accessor(part['indices'],'SCALAR',5123,34963),'material':colors[color],'mode':4,'extras':{'part':part['part']}})
 g.doc['meshes']=[{'name':'murkrow','primitives':primitives}]
 for clip,duration,count in CLIPS:
  times=g.accessor([i*duration/(count-1) for i in range(count)],'SCALAR',bounds=True);channels=[];samplers=[]
  for joint in range(1,len(joints)):
   rotations=[v for i in range(count) for v in rotation(clip,i/(count-1),joint)]
   channels.append({'sampler':len(samplers),'target':{'node':joint,'path':'rotation'}})
   samplers.append({'input':times,'output':accessor(rotations,'VEC4'),'interpolation':'LINEAR'})
  g.doc['animations'].append({'name':'murkrow.'+clip,'channels':channels,'samplers':samplers,
    'extras':{'loopSuggested':clip=='idle','cueRelative':clip!='idle','purpose':'planted toes; fitted crow gaze and hooked bill; coherent folded wing layers; feather hat and bound broom tail; source battle cues own timing'}})
 g.doc['extras']={'coordinate_system':COORDINATES,'source_name':'murkrow','source_sha256':REST_DIGEST,
    'reference_span':HEIGHT,'body_reference_height':HEIGHT,'full_silhouette_height':HEIGHT,
    'geometry_source':'tools/murkrow_sculpt.py','geometry_preservation':'exact source f32 anatomy, normals, colours and triangle order',
    'animation_contract':'identity root; fixed toes and talons; yaw-only torso; coherent fitted face and bill; rigid folded wing units; three hat feathers; bound articulated broom and fitted creases; rotation-only source-cue clips'}
 if len(g.doc['accessors'])>256:raise ValueError('Murkrow exceeds existing accessor limit')
 return g.bytes()


def decode_murkrow(blob):
 if len(blob)>MAX_BYTES or blob!=export_murkrow():raise ValueError('noncanonical Murkrow GLB')
 length=struct.unpack_from('<I',blob,12)[0];doc=json.loads(blob[20:20+length]);binary=blob[28+length:]
 def values(index):
  a=doc['accessors'][index];view=doc['bufferViews'][a['bufferView']]
  width=3 if a['type']=='VEC3' else 1;code='f' if a['componentType']==5126 else 'H'
  return list(struct.unpack_from('<'+code*(a['count']*width),binary,view.get('byteOffset',0)))
 return {'name':'murkrow','version':1,'coordinate_system':COORDINATES,'primitives':[
  {'part':p['extras']['part'],'positions':values(p['attributes']['POSITION']),
   'normals':values(p['attributes']['NORMAL']),'indices':values(p['indices']),
   'base_color':doc['materials'][p['material']]['pbrMetallicRoughness']['baseColorFactor']}
  for p in doc['meshes'][0]['primitives']]}


def read_murkrow(path):
 with Path(path).open('rb') as f:return decode_murkrow(f.read(MAX_BYTES+1))


def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--out',type=Path,default=Path('crates/crystal-voxel-view/models/battle_species'));a=p.parse_args();a.out.mkdir(parents=True,exist_ok=True)
 b=export_murkrow();(a.out/'murkrow.glb').write_bytes(b);print(len(b),'bytes',REST_DIGEST)
if __name__=='__main__':main()
